use crate::models::{Coord, TetrisBoard, TetrisPiece};
use crossterm::terminal;
use crossterm::{cursor, execute, queue, style};
use std::io;
use std::io::Write;

pub struct CliView<W: Write> {
    writer: W,
}
impl CliView<io::Stdout> {
    pub fn new() -> Self {
        let mut writer = io::stdout();
        execute!(writer, terminal::EnterAlternateScreen).unwrap();
        Self { writer }
    }
}
impl<W: Write> Drop for CliView<W> {
    fn drop(&mut self) {
        execute!(self.writer, terminal::LeaveAlternateScreen).expect(
            "Failed to exit alternate screen. Restart terminal to resume normal behaviour.",
        );
    }
}
impl<W: Write> CliView<W> {
    fn generate_board_string_view(tetris_board: &TetrisBoard) -> Vec<String> {
        let mut view_lines: Vec<String> = Vec::with_capacity(tetris_board.board.len());
        for line in &tetris_board.board {
            let mut line_chars: Vec<u8> = vec!['|' as u8];
            line_chars.extend(line.iter().map(|x| match x {
                true => 'o' as u8,
                false => ' ' as u8,
            }));
            line_chars.push('|' as u8);
            view_lines.push(String::from_utf8(line_chars).expect("Error converting to string."));
        }
        return view_lines;
    }
    fn draw_board(&mut self, board_string: Vec<String>) -> std::io::Result<()> {
        queue!(self.writer, cursor::MoveTo(0, 0),)?;
        for line in &board_string[0..board_string.len() - 1] {
            queue!(self.writer, style::Print(line), cursor::MoveToNextLine(1),)?;
        }
        queue!(
            self.writer,
            style::SetAttribute(style::Attribute::Underlined),
            style::Print(&board_string[board_string.len() - 1]),
            style::SetAttribute(style::Attribute::NoUnderline),
        )?;

        self.writer.flush()?;

        return Ok(());
    }
    fn draw_piece(&mut self, piece_coordinates: Vec<Coord>) -> std::io::Result<()> {
        for coord in piece_coordinates {
            queue!(
                self.writer,
                cursor::MoveTo((coord.col + 1) as u16, coord.row as u16),
                style::Print("x")
            )?;
        }
        self.writer.flush()?;
        return Ok(());
    }
    pub fn draw_piece_and_board(
        &mut self,
        piece: &TetrisPiece,
        board: &TetrisBoard,
    ) -> std::io::Result<()> {
        let board_string = Self::generate_board_string_view(board);
        Self::draw_board(self, board_string)?;
        Self::draw_piece(self, piece.coordinates())?;
        Ok(())
    }
    pub fn draw_score(
        &mut self,
        score: u64,
        level: u64,
        time_per_turn: u64,
    ) -> std::io::Result<()> {
        queue!(
            self.writer,
            cursor::MoveTo(20, 13),
            style::Print(format!("Score: {}", score)),
            cursor::MoveTo(20, 14),
            style::Print(format!("Current Level: {}", level)),
            cursor::MoveTo(20, 15),
            style::Print(format!("Time per turn: {} ms", time_per_turn)),
        )?;
        self.writer.flush()?;
        return Ok(());
    }
    pub fn draw_intro(&mut self) -> std::io::Result<()> {
        let tetris_art = vec![
            String::from("##### ##### ##### ###   #####   ### "),
            String::from("  #   #       #   #  #    #    # "),
            String::from("  #   ####    #   ##      #     ##"),
            String::from("  #   #       #   # #     #       #"),
            String::from("  #   ####    #   #  #  #####   ##"),
        ];
        for i in 0..tetris_art.len() {
            queue!(
                self.writer,
                cursor::MoveTo(20, i as u16),
                style::Print(&tetris_art[i]),
            )?;
        }
        queue!(
            self.writer,
            cursor::MoveTo(20, 6),
            style::Print(
                "Use the arrows to move, 'x' to rotate clockwise and 'z' to rotate anticlockise. Hit Esc to quit."
            ),
        )?;
        self.writer.flush()?;
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestWriter {
        buffer: Vec<u8>,
    }
    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn test_cli_view_generates_board() {
        let expected_string = vec![String::from("|          |"); 16];
        let tetris_board = TetrisBoard::new();
        let cli_string = CliView::<TestWriter>::generate_board_string_view(&tetris_board);
        assert_eq!(cli_string, expected_string);
    }

    struct CommandMapping {}
    impl CommandMapping {
        const MOVE_TO_START: [u8; 6] = [27, 91, 49, 59, 49, 72];
        const MOVE_TO_NEXT_LINE: [u8; 4] = [27, 91, 49, 69];
        const SET_UNDERLINED: [u8; 4] = [27, 91, 52, 109];
        const SET_NOT_UNDERLINED: [u8; 5] = [27, 91, 50, 52, 109];
        fn move_to(col: u8, row: u8) -> [u8; 6] {
            return [
                Self::MOVE_TO_START[0],
                Self::MOVE_TO_START[1],
                Self::MOVE_TO_START[2] + row,
                Self::MOVE_TO_START[3],
                Self::MOVE_TO_START[4] + col,
                Self::MOVE_TO_START[5],
            ];
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_cli_view_writes_board() {
        let board_row = "|          |";
        let board_row_bytes: [u8; 12] = board_row.as_bytes().try_into().unwrap();

        // Construct expected buffer from commands
        let expected_buffer: Vec<u8> = CommandMapping::MOVE_TO_START
            .into_iter()
            .chain(board_row_bytes)
            .chain(CommandMapping::MOVE_TO_NEXT_LINE)
            .chain(CommandMapping::SET_UNDERLINED)
            .chain(board_row_bytes)
            .chain(CommandMapping::SET_NOT_UNDERLINED)
            .collect();

        let cli_string = vec![String::from(board_row); 2];
        let mut test_viewer = CliView {
            writer: TestWriter { buffer: Vec::new() },
        };
        test_viewer
            .draw_board(cli_string)
            .expect("Writing to test writer failed.");
        assert_eq!(test_viewer.writer.buffer, expected_buffer);
    }

    #[cfg(unix)]
    #[test]
    fn test_cli_view_writes_piece() {
        // Construct expected buffer from commands
        let expected_buffer: Vec<u8> = CommandMapping::move_to(2, 1)
            .into_iter()
            .chain(['x' as u8])
            .chain(CommandMapping::move_to(3, 1))
            .chain(['x' as u8])
            .chain(CommandMapping::move_to(3, 2))
            .chain(['x' as u8])
            .chain(CommandMapping::move_to(4, 2))
            .chain(['x' as u8])
            .collect();

        let mut test_viewer = CliView {
            writer: TestWriter { buffer: Vec::new() },
        };
        let piece_coords = vec![
            Coord { col: 1, row: 1 },
            Coord { col: 2, row: 1 },
            Coord { col: 2, row: 2 },
            Coord { col: 3, row: 2 },
        ];
        test_viewer
            .draw_piece(piece_coords)
            .expect("Writing to test writer failed.");
        assert_eq!(test_viewer.writer.buffer, expected_buffer);
    }
}
