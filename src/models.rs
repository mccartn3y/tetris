use rand::seq::IteratorRandom;
use std::slice::Iter;

pub struct TetrisBoard {
    pub board: Vec<Vec<bool>>,
}
impl TetrisBoard {
    const NUM_ROWS: usize = 16;
    const NUM_COLS: usize = 10;

    pub fn new() -> Self {
        let row = vec![false; Self::NUM_COLS];
        Self {
            board: vec![row; Self::NUM_ROWS],
        }
    }
    fn check_coordinates_on_board(&self, coordinates: &Vec<Coord>) -> bool {
        for coord in coordinates {
            if coord.row < 0
                || coord.row >= Self::NUM_ROWS as i16
                || coord.col < 0
                || coord.col >= Self::NUM_COLS as i16
            {
                return false;
            }
        }
        return true;
    }
    fn check_is_valid_position(&self, coordinates: &Vec<Coord>) -> PiecePositionValidity {
        if !self.check_coordinates_on_board(coordinates) {
            return PiecePositionValidity::OffOfBoard;
        }
        for coord in coordinates {
            if self.board[coord.row as usize][coord.col as usize] {
                return PiecePositionValidity::PieceCollision;
            }
        }
        return PiecePositionValidity::Valid;
    }

    fn fix_piece_in_place(&mut self, piece: TetrisPiece) {
        for coord in piece.coordinates() {
            self.board[coord.row as usize][coord.col as usize] = true;
        }
    }
}
#[derive(Debug, PartialEq)]
enum PiecePositionValidity {
    Valid,
    OffOfBoard,
    PieceCollision,
}
#[derive(Debug)]
pub enum PieceShape {
    Square,
    Bar,
    Z,
    FlippedZ,
    L,
    FlippedL,
    T,
}
impl PieceShape {
    pub fn shape(&self) -> Vec<Coord> {
        match *self {
            PieceShape::Square => vec![
                Coord { col: 0, row: 0 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 1, row: 0 },
            ],
            PieceShape::Bar => vec![
                Coord { col: -1, row: 0 },
                Coord { col: 0, row: 0 },
                Coord { col: 1, row: 0 },
                Coord { col: 2, row: 0 },
            ],
            PieceShape::Z => vec![
                Coord { col: -1, row: 0 },
                Coord { col: 0, row: 0 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
            ],
            PieceShape::FlippedZ => vec![
                Coord { col: -1, row: 1 },
                Coord { col: 0, row: 0 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 0 },
            ],
            PieceShape::L => vec![
                Coord { col: -1, row: 1 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
                Coord { col: 1, row: 0 },
            ],
            PieceShape::FlippedL => vec![
                Coord { col: -1, row: 0 },
                Coord { col: -1, row: 1 },
                Coord { col: 0, row: 1 },
                Coord { col: 1, row: 1 },
            ],
            PieceShape::T => vec![
                Coord { col: -1, row: 1 },
                Coord { col: 0, row: 1 },
                Coord { col: 0, row: 0 },
                Coord { col: 1, row: 1 },
            ],
        }
    }
    pub fn iterator() -> Iter<'static, Self> {
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
    pub fn random() -> &'static Self {
        Self::iterator().choose(&mut rand::thread_rng()).unwrap()
    }
}
#[derive(Debug)]
pub enum MoveCommand {
    Left,
    Down,
    Right,
    Clockwise,
    Anticlockwise,
}
#[derive(PartialEq, Debug, Clone)]
pub struct Coord {
    pub col: i16,
    pub row: i16,
}
enum Orientation {
    Up,
    Right,
    Down,
    Left,
}
pub struct TetrisPiece {
    shape: Vec<Coord>,
    centre: Coord,
    orientation: Orientation,
}
impl TetrisPiece {
    pub fn new(piece_shape: &PieceShape) -> Self {
        Self {
            shape: piece_shape.shape(),
            centre: Coord { col: 4, row: 2 },
            orientation: Orientation::Up,
        }
    }
    pub fn coordinates(&self) -> Vec<Coord> {
        self.calc_coordinates_with_centre(None)
    }
    fn calc_coordinates_with_centre(&self, new_centre: Option<&Coord>) -> Vec<Coord> {
        let new_centre = match new_centre {
            None => &self.centre,
            Some(centre) => centre,
        };
        let mut coordinates = Vec::with_capacity(self.shape.len());
        for coords in &self.shape {
            coordinates.push(Coord {
                col: coords.col + new_centre.col,
                row: coords.row + new_centre.row,
            });
        }
        return coordinates;
    }
    pub fn move_peice(&mut self, board: &TetrisBoard, direction: MoveCommand) -> Option<TurnEvent> {
        match direction {
            MoveCommand::Right => {
                let new_centre = Coord {
                    col: self.centre.col + 1,
                    row: self.centre.row,
                };
                let new_coordinates = self.calc_coordinates_with_centre(Some(&new_centre));
                if let PiecePositionValidity::Valid =
                    board.check_is_valid_position(&new_coordinates)
                {
                    self.centre.col += 1;
                }
                return None;
            }
            MoveCommand::Left => {
                let new_centre = Coord {
                    col: self.centre.col - 1,
                    row: self.centre.row,
                };
                let new_coordinates = self.calc_coordinates_with_centre(Some(&new_centre));
                if let PiecePositionValidity::Valid =
                    board.check_is_valid_position(&new_coordinates)
                {
                    self.centre.col -= 1;
                }
                return None;
            }
            MoveCommand::Down => Some(TurnEvent::EndTurn),
            _other => return None,
        }
    }
    pub fn move_down(mut self, board: &mut TetrisBoard) -> Option<Self> {
        let new_centre = Coord {
            row: self.centre.row + 1,
            ..self.centre
        };
        match board.check_is_valid_position(&self.calc_coordinates_with_centre(Some(&new_centre))) {
            PiecePositionValidity::Valid => {
                self.centre = new_centre;
                return Some(self);
            }
            _other => {
                board.fix_piece_in_place(self);
                return None;
            }
        }
    }
}
pub enum TurnEvent {
    EndTurn,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_piece_creation_matches_piece() {
        for piece_shape in PieceShape::iterator() {
            let tetris_piece = TetrisPiece::new(piece_shape);

            for i in 0..piece_shape.shape().len() {
                assert_eq!(tetris_piece.shape[i].col, piece_shape.shape()[i].col);
                assert_eq!(tetris_piece.shape[i].row, piece_shape.shape()[i].row);
            }
            match tetris_piece.orientation {
                Orientation::Up => (),
                _other => assert!(false),
            }
        }
    }

    #[test]
    fn test_piece_coordinates_generated() {
        for piece_shape in PieceShape::iterator() {
            let tetris_piece = TetrisPiece::new(piece_shape);
            let piece_coordinates = tetris_piece.coordinates();
            for i in 0..piece_shape.shape().len() {
                assert_eq!(
                    piece_coordinates[i].col,
                    piece_shape.shape()[i].col + tetris_piece.centre.col
                );
                assert_eq!(
                    piece_coordinates[i].row,
                    piece_shape.shape()[i].row + tetris_piece.centre.row
                );
            }
        }
    }
    #[test]
    fn test_piece_position_validity_returns_off_board() {
        let tetris_board = TetrisBoard::new();
        let off_board_coords = vec![
            vec![Coord { row: 0, col: -1 }],
            vec![Coord { row: -1, col: 0 }],
            vec![Coord { row: 0, col: 20 }],
            vec![Coord { row: 20, col: 0 }],
        ];
        for coord in off_board_coords {
            assert_eq!(
                tetris_board.check_is_valid_position(&coord),
                PiecePositionValidity::OffOfBoard
            );
        }
    }

    #[test]
    fn test_piece_position_validity_returns_valid() {
        let tetris_board = TetrisBoard::new();
        assert_eq!(
            tetris_board.check_is_valid_position(&vec![Coord { row: 0, col: 0 }]),
            PiecePositionValidity::Valid
        );
    }
    #[test]
    fn test_piece_position_validity_returns_collision() {
        let mut tetris_board = TetrisBoard::new();
        tetris_board.board[0][0] = true;
        assert_eq!(
            tetris_board.check_is_valid_position(&vec![Coord { row: 0, col: 0 }]),
            PiecePositionValidity::PieceCollision
        );
    }

    #[test]
    fn test_piece_cannot_go_off_board() {
        let tetris_board = TetrisBoard::new();
        let mut tetris_piece = TetrisPiece::new(&PieceShape::Bar);
        for _ in 0..20 {
            tetris_piece.move_peice(&tetris_board, MoveCommand::Right);
            for coord in tetris_piece.coordinates() {
                assert!(coord.col < TetrisBoard::NUM_COLS as i16);
            }
        }
        for _ in 0..20 {
            tetris_piece.move_peice(&tetris_board, MoveCommand::Left);
            for coord in tetris_piece.coordinates() {
                assert!(coord.col >= 0);
            }
        }
    }

    #[test]
    fn test_piece_moves_down_if_no_collision() {
        let mut tetris_board = TetrisBoard::new();
        let mut tetris_piece = TetrisPiece::new(&PieceShape::Bar);
        let start_centre = tetris_piece.centre.clone();
        let expected_centre = Coord {
            row: start_centre.row + 1,
            ..start_centre
        };
        tetris_piece = tetris_piece.move_down(&mut tetris_board).unwrap();
        assert_eq!(tetris_piece.centre, expected_centre);
    }

    #[test]
    fn test_fix_in_place_updates_board() {
        let mut tetris_board = TetrisBoard::new();
        let tetris_piece = TetrisPiece::new(&PieceShape::Bar);
        let piece_centre = tetris_piece.centre.clone();
        tetris_board.fix_piece_in_place(tetris_piece);
        assert!(tetris_board.board[piece_centre.row as usize][piece_centre.col as usize])
    }

    #[test]
    fn piece_is_fixed_if_down_is_collision() {
        let mut tetris_board = TetrisBoard::new();
        let mut tetris_piece = TetrisPiece::new(&PieceShape::Bar);
        tetris_piece.centre = Coord { row: 0, col: 2 };
        tetris_board.board[tetris_piece.centre.row as usize + 1]
            [tetris_piece.centre.col as usize] = true;
        tetris_piece.move_down(&mut tetris_board);
        for i in 2..4 {
            assert!(tetris_board.board[0][i as usize]);
        }
    }
    #[test]
    fn piece_is_fixed_if_down_is_end_of_board() {
        let mut tetris_board = TetrisBoard::new();
        let mut tetris_piece = TetrisPiece::new(&PieceShape::Bar);
        tetris_piece.centre = Coord { row: 15, col: 2 };
        tetris_piece.move_down(&mut tetris_board);
        for i in 2..4 {
            assert!(tetris_board.board[15][i as usize]);
        }
    }
}
