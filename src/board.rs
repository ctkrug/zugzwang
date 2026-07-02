use crate::square::Square;
use crate::types::{Color, PieceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

/// Board state as a flat 8x8 array of optional pieces.
///
/// A bitboard representation is planned for the build phase once move
/// generation needs the extra performance; the array keeps this scaffold
/// simple to read and test.
pub struct Board {
    squares: [Option<Piece>; 64],
    pub side_to_move: Color,
}

impl Board {
    pub fn empty() -> Self {
        Board {
            squares: [None; 64],
            side_to_move: Color::White,
        }
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        self.squares[sq.index()]
    }

    pub fn set(&mut self, sq: Square, piece: Option<Piece>) {
        self.squares[sq.index()] = piece;
    }
}
