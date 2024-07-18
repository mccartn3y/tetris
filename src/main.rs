use log;
use std::cmp;
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
    println!("Game Over! Score: {}", game_runner());
}
fn game_runner() -> u64 {
    let mut tetris_board = TetrisBoard::new();
    let mut cli_writer = CliView::<io::Stdout>::new();
    cli_writer.draw_intro().unwrap();

    let mut score = 0;
    let mut level = 0;
    let mut cleared_rows_count = 0;
    loop {
        let turn_duration = match level {
            val if val < 9 => (1000 * (48 - (5 * level))) / 60,
            _ => cmp::max((1000 * (9 - (level - 9))) / 60, 1000 / 60),
        };
        cli_writer.draw_score(score, level, turn_duration).unwrap();

        let cleared_rows = match run_piece_loop(&mut tetris_board, turn_duration, &mut cli_writer) {
            Ok(cleared_rows) => cleared_rows,
            Err(_) => break,
        };
        cleared_rows_count += cleared_rows;
        score += match cleared_rows {
            1 => 40 * (level + 1),
            2 => 100 * (level + 1),
            3 => 300 * (level + 1),
            4 => 1200 * (level + 1),
            _other => 0,
        };
        if cleared_rows_count >= 10 {
            level += 1;
            cleared_rows_count = 0;
        }
    }
    score
}
fn run_piece_loop(
    tetris_board: &mut TetrisBoard,
    turn_duration: u64,
    cli_writer: &mut CliView<io::Stdout>,
) -> Result<u16, ()> {
    let mut tetris_piece = TetrisPiece::new(tetris::models::PieceShape::random());
    if let PiecePositionValidity::PieceCollision =
        tetris_board.check_is_valid_position(&tetris_piece.coordinates())
    {
        return Err(());
    }
    loop {
        cli_writer
            .draw_piece_and_board(&tetris_piece, &tetris_board)
            .expect("Failed to draw board.");

        let mut turn_timer = TurnTimer::new(turn_duration);
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
                        log::warn!("End turn event sent to closed turn event channel.");
                    };
                    break;
                };
                cli_writer
                    .draw_piece_and_board(&tetris_piece, &tetris_board)
                    .expect("Failed to draw board.");
            }
        });
        if let Some(out_piece) = tetris_piece.move_down(tetris_board) {
            tetris_piece = out_piece;
        } else {
            break;
        }
    }
    Ok(tetris_board.clear_rows())
}
