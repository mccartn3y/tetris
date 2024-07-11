use crossterm::event::{poll, read, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::mpsc;

use crate::turn_timer::turn_timer::{TimerStatus, TurnTimerSubscriber};

use std::thread;
use std::time::Duration;
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

pub fn timed_user_input_thread(
    mut turn_timer_subscriber: TurnTimerSubscriber,
    char_sender: mpsc::Sender<KeyEvent>,
) {
    // set up thread for getting cli input
    thread::spawn(move || {
        let _guard = ScopedRawMode::new();
        loop {
            match turn_timer_subscriber.get_timer_status() {
                TimerStatus::TimerComplete => return,
                TimerStatus::TimerNotComplete => read_cli_input(&char_sender),
            }
        }
    });
}

fn read_cli_input(char_sender: &mpsc::Sender<KeyEvent>) {
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
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::terminal::is_raw_mode_enabled;

    #[test]
    fn test_scoped_raw_mode_controls_raw_mode() {
        assert!(!is_raw_mode_enabled().unwrap());
        {
            let _guard = ScopedRawMode::new();
            assert!(is_raw_mode_enabled().unwrap());
        }
        assert!(!is_raw_mode_enabled().unwrap());
    }
}
