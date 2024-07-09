use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tetris::turn_timer::turn_timer::{Notifier, TimerStatus, TurnTimer, TurnTimerSubscriber};

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

    let mut turn_timer = TurnTimer::new(3);
    let mut turn_timer_subscriber = TurnTimerSubscriber::new();
    let mut turn_timer_subscriber_1 = TurnTimerSubscriber::new();
    turn_timer.add_subscriber(&mut turn_timer_subscriber);
    turn_timer.add_subscriber(&mut turn_timer_subscriber_1);

    turn_timer.run_timer();

    let (char_sender, char_receiver) = mpsc::channel();
    // set up thread for getting cli input
    thread::spawn(move || {
        let _guard = ScopedRawMode::new();
        loop {
            match turn_timer_subscriber.get_timer_status() {
                TimerStatus::TimerComplete => return,
                TimerStatus::TimerNotComplete => {
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
        if let TimerStatus::TimerComplete = turn_timer_subscriber_1.get_timer_status() {
            break;
        }
        println!("{:?}", recieved);
        thread::sleep(Duration::from_secs(1));
    }
}

// Struct that runs enable_raw_mode on start and disables when it is
// dropped so that it is only active in the scope of the instantiation
struct ScopedRawMode;

impl ScopedRawMode {
    fn new() -> ScopedRawMode {
        enable_raw_mode().expect("Failed to enable raw mode required to display correctly.");
        ScopedRawMode
    }
}

impl Drop for ScopedRawMode {
    fn drop(&mut self) {
        disable_raw_mode()
            .expect("Failed to disable raw mode. Restart terminal to resume normal behaviour.");
    }
}
