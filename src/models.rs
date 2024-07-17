use rand::seq::IteratorRandom;
use std::slice::Iter;
pub struct TetrisBoard {
    pub board: Vec<Vec<bool>>,
}
impl TetrisBoard {
    pub fn new() -> Self {
        let rows = vec![false; 10];
        Self {
            board: vec![rows; 16],
        }
    }
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
pub struct Coord {
    pub col: u16,
    pub row: u16,
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
}
