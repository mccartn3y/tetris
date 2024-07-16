use crossterm::{cursor, execute, queue, style, Command};
use std::io;
use std::io::Write;
struct TetrisBoard {
    board: Vec<Vec<bool>>,
}
impl TetrisBoard {
    fn new() -> Self {
        let rows = vec![false; 10];
        Self {
            board: vec![rows; 16],
        }
    }
}
struct CliView;
impl CliView {
    fn generate_board_string_view(tetris_board: TetrisBoard) -> Vec<String> {
        let mut view_lines: Vec<String> = Vec::with_capacity(tetris_board.board.len());
        for line in tetris_board.board {
            let mut line_chars: Vec<u8> = vec!['|' as u8];
            line_chars.extend(line.iter().map(|x| match x {
                true => 'x' as u8,
                false => ' ' as u8,
            }));
            line_chars.push('|' as u8);
            view_lines.push(String::from_utf8(line_chars).expect("Error converting to string."));
        }
        return view_lines;
    }
    fn print_board<W: Write>(writer: &mut W, board_string: Vec<String>) -> std::io::Result<()> {
        queue!(writer, cursor::MoveTo(0, 0),)?;
        for line in &board_string[0..board_string.len() - 1] {
            queue!(writer, style::Print(line), cursor::MoveToNextLine(1),)?;
        }
        queue!(
            writer,
            style::SetAttribute(style::Attribute::Underlined),
            style::Print(&board_string[board_string.len() - 1]),
            style::SetAttribute(style::Attribute::NoUnderline),
        )?;

        writer.flush()?;

        return Ok(());
    }
}
#[cfg(test)]
mod tests {
    use io::Read;

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

    use super::*;
    #[test]
    fn test_cli_view_generates_board() {
        let expected_string = vec![String::from("|          |"); 16];
        let tetris_board = TetrisBoard::new();
        let cli_string = CliView::generate_board_string_view(tetris_board);
        assert_eq!(cli_string, expected_string);
    }

    struct CommandMapping {}
    impl CommandMapping {
        const MOVE_TO_START: [u8; 6] = [27, 91, 49, 59, 49, 72];
        const MOVE_TO_NEXT_LINE: [u8; 4] = [27, 91, 49, 69];
        const SET_UNDERLINED: [u8; 4] = [27, 91, 52, 109];
        const SET_NOT_UNDERLINED: [u8; 5] = [27, 91, 50, 52, 109];
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
        let mut buf_writer = TestWriter { buffer: Vec::new() };
        CliView::print_board(&mut buf_writer, cli_string);
        assert_eq!(buf_writer.buffer, expected_buffer);
    }
}
