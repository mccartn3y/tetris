use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use std::sync::mpsc;
use std::thread;
use tetris::models::{CliView, TetrisBoard, TetrisPiece};
use tetris::turn_timer::turn_timer::{
    Notifier, TimerStatus, TurnTimer, TurnTimerSubscriber, TurnTimerSubscriberTrait,
};
use tetris::ui::{timed_user_input, CliCommandCollector};

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
fn run_piece_loop(tetris_board: &mut TetrisBoard) -> std::io::Result<()> {
    let tetris_piece = TetrisPiece::new(tetris::models::PieceShape::random());
    // if tetris_board.is_collision(tetris_piece.coordinates()) {
    //     break;
    // }

    CliView::draw_piece_and_board(&tetris_piece, &tetris_board).expect("Failed to draw board.");

    let mut turn_timer = TurnTimer::new(3_000);
    let mut turn_timer_subscriber = TurnTimerSubscriber::new();
    let mut turn_timer_subscriber_1 = TurnTimerSubscriber::new();
    turn_timer.add_subscriber(&mut turn_timer_subscriber);
    turn_timer.add_subscriber(&mut turn_timer_subscriber_1);

    turn_timer.run_timer();
    thread::scope(|s| {
        let (command_dispatcher, char_receiver) = mpsc::channel();
        timed_user_input::<CliCommandCollector, TurnTimerSubscriber>(
            turn_timer_subscriber,
            command_dispatcher,
            s,
        );

        for recieved in char_receiver {
            if let TimerStatus::TimerComplete = turn_timer_subscriber_1.get_timer_status() {
                break;
            }
            println!("                             {:?} Recieved!!", recieved);
        }
    });
    println!("Moving to next piece.");
    Ok(())
    // if tetris_board.is_collision(tetris_piece.next_turn_coordinates()) {
    //     break;
    // } else {
    //     tetris_piece.move_down();
    // }
}
