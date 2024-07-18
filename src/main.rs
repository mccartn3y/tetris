use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::sync::mpsc;
use std::thread;

use tetris::models::{PiecePositionValidity, TetrisBoard, TetrisPiece, TurnEvent};
use tetris::turn_timer::turn_timer::{
    Notifier, TimerStatus, TurnTimer, TurnTimerSubscriber, TurnTimerSubscriberTrait,
};
use tetris::ui::{timed_user_input, CliCommandCollector};
use tetris::views::CliView;

fn main() {
    game_runner();
}
fn game_runner() {
    // Steps:
    // - Create board
    // Loop:
    //  - Create piece with random shape
    //  - add piece to board, if error then break
    //  - print board and piece
    //  Loop while no collision:
    //      - start timer
    //      Loop till timer ends or user send down command:
    //          - user can translate or rotate piece
    //          - print board and piece
    //      - check if moving down would collide with piece stack
    //  - fix piece on board
    // - print Game Over!
    let mut tetris_board = TetrisBoard::new();
    let mut writer = io::stdout();
    execute!(writer, EnterAlternateScreen).unwrap();

    loop {
        if let Err(_) = run_piece_loop(&mut tetris_board) {
            break;
        }
    }
    execute!(writer, LeaveAlternateScreen).unwrap();
    println!("Game Over!");
}
fn run_piece_loop(tetris_board: &mut TetrisBoard) -> Result<(), ()> {
    let mut tetris_piece = TetrisPiece::new(tetris::models::PieceShape::random());
    if let PiecePositionValidity::PieceCollision =
        tetris_board.check_is_valid_position(&tetris_piece.coordinates())
    {
        return Err(());
    }
    loop {
        CliView::draw_piece_and_board(&tetris_piece, &tetris_board).expect("Failed to draw board.");

        let mut turn_timer = TurnTimer::new(3_000);
        let mut turn_timer_subscriber = TurnTimerSubscriber::new();
        let mut turn_timer_subscriber_1 = TurnTimerSubscriber::new();
        turn_timer.add_subscriber(&mut turn_timer_subscriber);
        turn_timer.add_subscriber(&mut turn_timer_subscriber_1);

        turn_timer.run_timer();
        thread::scope(|s| {
            let (command_dispatcher, command_reciever) = mpsc::channel();
            let (turn_event_sender, turn_event_reciever) = mpsc::channel::<TurnEvent>();
            timed_user_input::<CliCommandCollector, TurnTimerSubscriber>(
                turn_timer_subscriber,
                command_dispatcher,
                turn_event_reciever,
                s,
            );

            for recieved in command_reciever {
                if let TimerStatus::TimerComplete = turn_timer_subscriber_1.get_timer_status() {
                    break;
                }
                if let Some(TurnEvent::EndTurn) = tetris_piece.move_peice(&tetris_board, recieved) {
                    if let Err(_) = turn_event_sender.send(TurnEvent::EndTurn) {
                        eprintln!("End turn event sent to closed turn event channel.")
                    };
                    break;
                };
                CliView::draw_piece_and_board(&tetris_piece, &tetris_board)
                    .expect("Failed to draw board.");
            }
        });
        if let Some(out_piece) = tetris_piece.move_down(tetris_board) {
            tetris_piece = out_piece;
        } else {
            break;
        }
    }
    println!("Moving to next piece.");
    Ok(())
}
