use crossterm::{cursor, execute, queue, style, Command};
use std::io;
use std::io::Write;
use std::slice::Iter;
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
#[derive(Debug)]
enum PieceShape {
    Square,
    Bar,
    Z,
    FlippedZ,
    L,
    FlippedL,
    T,
}
impl PieceShape {
    fn shape(&self) -> Vec<Coord> {
        match *self {
            PieceShape::Square => vec![
                Coord { col: 0, row: 0 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 1, row: 0 },
            ],
            PieceShape::Bar => vec![
                Coord { col: 0, row: 0 },
                Coord { col: 1, row: 0 },
                Coord { col: 2, row: 0 },
                Coord { col: 3, row: 0 },
            ],
            PieceShape::Z => vec![
                Coord { col: 0, row: 0 },
                Coord { col: 1, row: 0 },
                Coord { col: 1, row: 1 },
                Coord { col: 2, row: 1 },
            ],
            PieceShape::FlippedZ => vec![
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 0 },
                Coord { col: 1, row: 1 },
                Coord { col: 2, row: 0 },
            ],
            PieceShape::L => vec![
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 2, row: 1 },
                Coord { col: 2, row: 0 },
            ],
            PieceShape::FlippedL => vec![
                Coord { col: 0, row: 0 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 2, row: 1 },
            ],
            PieceShape::T => vec![
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 1, row: 0 },
                Coord { col: 2, row: 1 },
            ],
        }
    }
    pub fn iterator() -> Iter<'static, PieceShape> {
        static PIECE_SHAPES: [PieceShape; 7] = [
            PieceShape::Square,
            PieceShape::Bar,
            PieceShape::Z,
            PieceShape::FlippedZ,
            PieceShape::L,
            PieceShape::FlippedL,
            PieceShape::T,
        ];
        PIECE_SHAPES.iter()
    }
}
#[derive(Debug)]
struct Coord {
    col: u16,
    row: u16,
}
enum Orientation {
    Up,
    Right,
    Down,
    Left,
}
struct TetrisPiece {
    shape: Vec<Coord>,
    centre: Coord,
    orientation: Orientation,
}
impl TetrisPiece {
    fn new(piece_shape: &PieceShape) -> Self {
        Self {
            shape: piece_shape.shape(),
            centre: Coord { col: 4, row: 2 },
            orientation: Orientation::Up,
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
                true => 'o' as u8,
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
    fn draw_piece<W: Write>(writer: &mut W, piece_coordinates: Vec<Coord>) -> std::io::Result<()> {
        for coord in piece_coordinates {
            queue!(
                writer,
                cursor::MoveTo(coord.col + 1, coord.row),
                style::Print("x")
            )?;
        }
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

    #[test]
    fn test_piece_creation_matches_piece() {
        for piece_shape in PieceShape::iterator() {
            let tetris_piece = TetrisPiece::new(piece_shape);

            for i in (0..piece_shape.shape().len()) {
                assert_eq!(tetris_piece.shape[i].col, piece_shape.shape()[i].col);
                assert_eq!(tetris_piece.shape[i].row, piece_shape.shape()[i].row);
            }
            match tetris_piece.orientation {
                Orientation::Up => (),
                _other => assert!(false),
            }
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
        fn MOVE_TO(col: u8, row: u8) -> [u8; 6] {
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
        let mut buf_writer = TestWriter { buffer: Vec::new() };
        CliView::print_board(&mut buf_writer, cli_string);
        assert_eq!(buf_writer.buffer, expected_buffer);
    }

    #[cfg(unix)]
    #[test]
    fn test_cli_view_writes_piece() {
        // Construct expected buffer from commands
        let expected_buffer: Vec<u8> = CommandMapping::MOVE_TO(2, 1)
            .into_iter()
            .chain(['x' as u8])
            .chain(CommandMapping::MOVE_TO(3, 1))
            .chain(['x' as u8])
            .chain(CommandMapping::MOVE_TO(3, 2))
            .chain(['x' as u8])
            .chain(CommandMapping::MOVE_TO(4, 2))
            .chain(['x' as u8])
            .collect();

        let mut buf_writer = TestWriter { buffer: Vec::new() };
        let piece_coords = vec![
            Coord { col: 1, row: 1 },
            Coord { col: 2, row: 1 },
            Coord { col: 2, row: 2 },
            Coord { col: 3, row: 2 },
        ];
        CliView::draw_piece(&mut buf_writer, piece_coords);
        assert_eq!(buf_writer.buffer, expected_buffer);
    }
}
