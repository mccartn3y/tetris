use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    // This code will create a UI for the tetris game.
    // The complexity of a tetis UI comes from the
    // fact that you have a limited amount of time to
    // make changes to the state of the piece before
    // it moves. This means that we need to have some way
    // of collecting and processing moves for a set time
    // period, after which we shall cease to collect inputs
    // and make some sort of change.
    //
    // At the moment it looks like the best way to do this is
    // leveraging channels. (This will be my first bit of
    // concurrent programming, YaY!). We shall set up two channels,
    // one that collects inputs from the cli and the other which
    // runs a timer for the given period of time. When there is a
    // message from the first channel (i.e. a movement), we shall
    // process it; when there is a message fromt the second channel
    // we shall move on.

    let (timeout_sender, timeout_receiver) = mpsc::channel();
    let (timeout_sender_1, timeout_receiver_1) = mpsc::channel();
    let (char_sender, char_receiver) = mpsc::channel();

    let timeout_duration = 10;
    // set up timer to accept input for
    thread::spawn(move || {
        // TODO: Convert this to use a shared variable
        thread::sleep(Duration::from_secs(timeout_duration));
        timeout_sender
            .send(true)
            .expect("Failed to send timer finished signal to input logger.");
        timeout_sender_1
            .send(true)
            .expect("Failed to send timer finished signal to main thread.");
        println!("Timer complete!");
    });
    // set up thread for getting cli input
    thread::spawn(move || {
        enable_raw_mode().expect("Failed to enable raw mode required to display correctly.");
        let _guard = ThreadGuard(Some(|| {
            // Custom code to run when the thread ends
            disable_raw_mode()
                .expect("Failed to disable raw mode. Restart terminal to resume normal behaviour.");
        }));
        loop {
            match timeout_receiver.try_recv() {
                Ok(_) => return,
                Err(_) => {
                    if poll(Duration::from_millis(100)).expect("Poll of CLI buffer failed.") {
                        match read().expect("Read of CLI buffer failed.") {
                            Event::Key(key_event) => match key_event.code {
                                KeyCode::Char(_) => {
                                    if let Err(_) = char_sender.send(key_event) {
                                        println!("Failed to send key to main thread.");
                                    }
                                }
                                _other => return,
                            },
                            _other => return,
                        }
                    }
                }
            }
        }
    });
    for recieved in char_receiver {
        if let Ok(_) = timeout_receiver_1.try_recv() {
            break;
        }
        println!("{:?}", recieved);
        thread::sleep(Duration::from_secs(1));
    }
}

struct ThreadGuard<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Drop for ThreadGuard<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            f();
        }
    }
}
