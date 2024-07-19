use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::mpsc;
use std::thread::Scope;
use std::time::Duration;

use crate::models::{Command, TurnEvent};
use crate::turn_timer::turn_timer::{TimerStatus, TurnTimerSubscriberTrait};
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
// TODO: Move the run_user_input_loop fn into a class that implements an interface so
// we don't have to pass in all of these dependencies to this fn.
pub fn timed_user_input<'a, T: CommandCollector, U: TurnTimerSubscriberTrait + Send + 'a>(
    mut turn_timer_subscriber: U,
    command_dispatcher: mpsc::Sender<Command>,
    turn_event_reciever: mpsc::Receiver<TurnEvent>,
    s: &'a Scope<'a, '_>,
) {
    // set up thread for getting cli input

    s.spawn(move || {
        let _guard = ScopedRawMode::new();
        let command_collector = T::new();
        run_user_input_loop::<T, U>(
            &mut turn_timer_subscriber,
            command_dispatcher,
            command_collector,
            turn_event_reciever,
        )
    });
}

/// Runs a loop to collect commands from the user while the turn timer
/// is not yet complete. This has been implemented with dependency injection
/// through the use of generics in order to make testing easier.
///
/// Args:
/// turn_timer_subscriber: a mutable reference to an object that
/// implements the TurnTimerSubscriberTrait and the deived trait Send
/// (so that it can be sent into a thread).
/// command_dispatcher: an mpsc::Sender of type Command, which
/// is used to send the read commands back to the main thread.
/// command_collector: an object that implements the CommandCollector trait. This
/// reference is mutable to make testing easier.
///
/// Edge cases:
/// - Timer never completes: unhandled - user must interrupt program
/// - Get command fails when reading from input
/// - Command is not recognised
/// - Send to main fails
fn run_user_input_loop<'a, T: CommandCollector, U: TurnTimerSubscriberTrait + Send + 'a>(
    turn_timer_subscriber: &mut U,
    command_dispatcher: mpsc::Sender<Command>,
    mut command_collector: T,
    turn_event_reciever: mpsc::Receiver<TurnEvent>,
) {
    loop {
        if let Ok(TurnEvent::EndTurn) = turn_event_reciever.try_recv() {
            return;
        }
        match turn_timer_subscriber.get_timer_status() {
            TimerStatus::TimerComplete => {
                return;
            }
            TimerStatus::TimerNotComplete => match command_collector.get_command() {
                Ok(val) => match val {
                    Some(command) => {
                        if let Err(error) = command_dispatcher.send(command) {
                            log::warn!("{:?}", error.to_string());
                            return;
                        }
                    }
                    None => (),
                },
                Err(e) => {
                    log::warn!("Error encountered reading command {:?}", e);
                    return;
                }
            },
        }
    }
}

pub trait CommandCollector {
    fn new() -> Self;
    fn get_command(&mut self) -> std::io::Result<Option<Command>>;
}

pub struct CliCommandCollector {}
impl CommandCollector for CliCommandCollector {
    fn new() -> Self {
        Self {}
    }
    fn get_command(&mut self) -> std::io::Result<Option<Command>> {
        if poll(Duration::from_millis(2)).expect("Poll of CLI buffer failed.") {
            return match read()? {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Down => Ok(Some(Command::MoveDown)),
                    KeyCode::Left => Ok(Some(Command::MoveLeft)),
                    KeyCode::Right => Ok(Some(Command::MoveRight)),
                    KeyCode::Char('z') => Ok(Some(Command::RotateAnticlockwise)),
                    KeyCode::Char('x') => Ok(Some(Command::RotateClockwise)),
                    KeyCode::Esc => Ok(Some(Command::EndGame)),

                    _other => Ok(None),
                },
                _other => panic!("Unrecognised command!"),
            };
        }
        return Ok(None);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::terminal::is_raw_mode_enabled;

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn test_scoped_raw_mode_controls_raw_mode() {
        assert!(!is_raw_mode_enabled().unwrap());
        {
            let _guard = ScopedRawMode::new();
            assert!(is_raw_mode_enabled().unwrap());
        }
        assert!(!is_raw_mode_enabled().unwrap());
    }
    struct TestTurnTimerSubscriber {
        outputs: Vec<TimerStatus>,
    }
    impl TurnTimerSubscriberTrait for TestTurnTimerSubscriber {
        fn get_timer_status(&mut self) -> TimerStatus {
            self.outputs.pop().unwrap_or(TimerStatus::TimerComplete)
        }
    }

    struct TestCommandCollector {
        outputs: Vec<std::io::Result<Option<Command>>>,
    }
    impl CommandCollector for TestCommandCollector {
        fn new() -> Self {
            Self { outputs: vec![] }
        }
        fn get_command(&mut self) -> std::io::Result<Option<Command>> {
            match self.outputs.pop() {
                Some(val) => val,
                None => Ok(None),
            }
        }
    }
    #[test]
    fn test_loop_does_exit_on_invalid_input() {
        let mut test_turn_timer = TestTurnTimerSubscriber {
            outputs: vec![
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
            ],
        };
        let (command_dispatcher, _command_reciever) = mpsc::channel();
        let (_turn_event_sender, turn_event_reciever) = mpsc::channel::<TurnEvent>();
        let mut command_collector = TestCommandCollector::new();
        command_collector.outputs.push(Ok(Some(Command::MoveDown)));
        command_collector
            .outputs
            .push(Err(std::io::Error::new(std::io::ErrorKind::NotFound, "")));

        run_user_input_loop::<TestCommandCollector, TestTurnTimerSubscriber>(
            &mut test_turn_timer,
            command_dispatcher,
            command_collector,
            turn_event_reciever,
        );
        assert_eq!(
            test_turn_timer.get_timer_status(),
            TimerStatus::TimerNotComplete
        );
    }
    #[test]
    fn test_loop_exits_on_end_turn_event() {
        let mut test_turn_timer = TestTurnTimerSubscriber {
            outputs: vec![
                TimerStatus::TimerComplete,
                TimerStatus::TimerComplete,
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
            ],
        };
        let (command_dispatcher, _command_reciever) = mpsc::channel();
        let (turn_event_sender, turn_event_reciever) = mpsc::channel::<TurnEvent>();
        let mut command_collector = TestCommandCollector::new();
        command_collector.outputs.push(Ok(Some(Command::MoveDown)));
        turn_event_sender
            .send(TurnEvent::EndTurn)
            .expect("Sent end turn event to closed channel.");
        run_user_input_loop::<TestCommandCollector, TestTurnTimerSubscriber>(
            &mut test_turn_timer,
            command_dispatcher,
            command_collector,
            turn_event_reciever,
        );
        assert_eq!(test_turn_timer.outputs.len(), 5);
    }
    #[test]
    fn test_loop_does_not_exit_on_valid_input() {
        let mut test_turn_timer = TestTurnTimerSubscriber {
            outputs: vec![
                TimerStatus::TimerComplete,
                TimerStatus::TimerComplete,
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
                TimerStatus::TimerNotComplete,
            ],
        };
        let (command_dispatcher, _command_reciever) = mpsc::channel();
        let (_turn_event_sender, turn_event_reciever) = mpsc::channel::<TurnEvent>();
        let mut command_collector = TestCommandCollector::new();
        command_collector.outputs.push(Ok(Some(Command::MoveDown)));

        run_user_input_loop::<TestCommandCollector, TestTurnTimerSubscriber>(
            &mut test_turn_timer,
            command_dispatcher,
            command_collector,
            turn_event_reciever,
        );
        assert_eq!(test_turn_timer.outputs.len(), 1);
    }
}
