use crate::square::Square;
use crate::types::{CastlingRights, Color, PieceKind};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

/// Board state as a flat 8x8 array of optional pieces, plus the rest of a
/// chess position: side to move, castling rights, the en passant target
/// square (if any), and the two FEN move counters.
///
/// A bitboard representation is planned for a later performance pass; the
/// array keeps move generation simple to read and test while it's being
/// built out and validated against perft.
#[derive(Clone, Copy)]
pub struct Board {
    squares: [Option<Piece>; 64],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
}

impl Board {
    pub fn empty() -> Self {
        Board {
            squares: [None; 64],
            side_to_move: Color::White,
            castling: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Builds a board set up for the standard chess starting position.
    pub fn starting_position() -> Self {
        let mut board = Board::empty();
        board.castling = CastlingRights::ALL;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_has_sixteen_pieces_per_side() {
        let board = Board::starting_position();
        let count = |color: Color| {
            (0..64)
                .filter(|&i| {
                    let sq = Square::new((i % 8) as u8, (i / 8) as u8);
                    board.get(sq).map(|p| p.color) == Some(color)
                })
                .count()
        };
        assert_eq!(count(Color::White), 16);
        assert_eq!(count(Color::Black), 16);
    }

    #[test]
    fn starting_position_places_kings_on_e_file() {
        let board = Board::starting_position();
        let white_king = board.get(Square::new(4, 0)).unwrap();
        let black_king = board.get(Square::new(4, 7)).unwrap();
        assert_eq!(white_king.kind, PieceKind::King);
        assert_eq!(black_king.kind, PieceKind::King);
    }
}
