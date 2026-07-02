use crate::square::Square;
use crate::types::{Color, PieceKind};
use std::fmt;

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

    /// Builds a board set up for the standard chess starting position.
    pub fn starting_position() -> Self {
        let mut board = Board::empty();
        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        for file in 0..8u8 {
            let kind = back_rank[file as usize];
            board.set(
                Square::new(file, 0),
                Some(Piece {
                    kind,
                    color: Color::White,
                }),
            );
            board.set(
                Square::new(file, 1),
                Some(Piece {
                    kind: PieceKind::Pawn,
                    color: Color::White,
                }),
            );
            board.set(
                Square::new(file, 6),
                Some(Piece {
                    kind: PieceKind::Pawn,
                    color: Color::Black,
                }),
            );
            board.set(
                Square::new(file, 7),
                Some(Piece {
                    kind,
                    color: Color::Black,
                }),
            );
        }
        board
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        self.squares[sq.index()]
    }

    pub fn set(&mut self, sq: Square, piece: Option<Piece>) {
        self.squares[sq.index()] = piece;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8u8).rev() {
            for file in 0..8u8 {
                let symbol = match self.get(Square::new(file, rank)) {
                    Some(piece) => piece_symbol(piece),
                    None => '.',
                };
                write!(f, "{symbol} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn piece_symbol(piece: Piece) -> char {
    let c = match piece.kind {
        PieceKind::Pawn => 'p',
        PieceKind::Knight => 'n',
        PieceKind::Bishop => 'b',
        PieceKind::Rook => 'r',
        PieceKind::Queen => 'q',
        PieceKind::King => 'k',
    };
    if piece.color == Color::White {
        c.to_ascii_uppercase()
    } else {
        c
    }
}
