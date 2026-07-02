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

    /// Applies a move and returns the resulting position.
    ///
    /// Assumes `mv` is at least pseudo-legal for the side to move; callers
    /// filtering for full legality (no self-check) do so by calling this
    /// and then checking whether the moving side's king is attacked.
    pub fn make_move(&self, mv: crate::moves::Move) -> Board {
        use crate::moves::MoveKind;

        let mut board = *self;
        let color = board.side_to_move;
        let piece = board
            .get(mv.from)
            .expect("make_move called with no piece on the from-square");
        let is_pawn_move = piece.kind == PieceKind::Pawn;
        let is_capture = board.get(mv.to).is_some() || mv.kind == MoveKind::EnPassant;

        match mv.kind {
            MoveKind::EnPassant => {
                let captured = Square::new(mv.to.file, mv.from.rank);
                board.set(captured, None);
            }
            MoveKind::CastleKingside => {
                let rank = mv.from.rank;
                board.set(Square::new(5, rank), board.get(Square::new(7, rank)));
                board.set(Square::new(7, rank), None);
            }
            MoveKind::CastleQueenside => {
                let rank = mv.from.rank;
                board.set(Square::new(3, rank), board.get(Square::new(0, rank)));
                board.set(Square::new(0, rank), None);
            }
            MoveKind::Normal => {}
        }

        board.set(mv.from, None);
        let placed = match mv.promotion {
            Some(kind) => Piece { kind, color },
            None => piece,
        };
        board.set(mv.to, Some(placed));

        board.update_castling_rights(mv, piece);

        board.en_passant = if is_pawn_move && mv.from.rank.abs_diff(mv.to.rank) == 2 {
            Some(Square::new(mv.from.file, (mv.from.rank + mv.to.rank) / 2))
        } else {
            None
        };

        board.halfmove_clock = if is_pawn_move || is_capture {
            0
        } else {
            board.halfmove_clock + 1
        };
        if color == Color::Black {
            board.fullmove_number += 1;
        }
        board.side_to_move = color.opposite();
        board
    }

    /// Clears castling rights lost by this move: the king or a rook moving
    /// off its home square, or a rook being captured on its home square.
    fn update_castling_rights(&mut self, mv: crate::moves::Move, moved_piece: Piece) {
        if moved_piece.kind == PieceKind::King {
            match moved_piece.color {
                Color::White => {
                    self.castling.white_kingside = false;
                    self.castling.white_queenside = false;
                }
                Color::Black => {
                    self.castling.black_kingside = false;
                    self.castling.black_queenside = false;
                }
            }
        }
        for sq in [mv.from, mv.to] {
            match (sq.file, sq.rank) {
                (0, 0) => self.castling.white_queenside = false,
                (7, 0) => self.castling.white_kingside = false,
                (0, 7) => self.castling.black_queenside = false,
                (7, 7) => self.castling.black_kingside = false,
                _ => {}
            }
        }
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

    #[test]
    fn make_move_relocates_the_piece_and_flips_side_to_move() {
        let board = Board::starting_position();
        let mv = crate::moves::Move::new(Square::new(4, 1), Square::new(4, 3));
        let next = board.make_move(mv);

        assert!(next.get(Square::new(4, 1)).is_none());
        assert_eq!(next.get(Square::new(4, 3)).unwrap().kind, PieceKind::Pawn);
        assert_eq!(next.side_to_move, Color::Black);
    }

    #[test]
    fn make_move_double_pawn_push_sets_en_passant_target() {
        let board = Board::starting_position();
        let mv = crate::moves::Move::new(Square::new(4, 1), Square::new(4, 3));
        let next = board.make_move(mv);
        assert_eq!(next.en_passant, Some(Square::new(4, 2)));
    }

    #[test]
    fn make_move_king_move_clears_both_castling_rights() {
        let mut board = Board::empty();
        board.castling = CastlingRights::ALL;
        board.set(
            Square::new(4, 0),
            Some(Piece {
                kind: PieceKind::King,
                color: Color::White,
            }),
        );
        let mv = crate::moves::Move::new(Square::new(4, 0), Square::new(4, 1));
        let next = board.make_move(mv);
        assert!(!next.castling.white_kingside);
        assert!(!next.castling.white_queenside);
        assert!(next.castling.black_kingside);
    }

    #[test]
    fn make_move_castle_kingside_relocates_the_rook() {
        let mut board = Board::empty();
        board.castling = CastlingRights::ALL;
        board.set(
            Square::new(4, 0),
            Some(Piece {
                kind: PieceKind::King,
                color: Color::White,
            }),
        );
        board.set(
            Square::new(7, 0),
            Some(Piece {
                kind: PieceKind::Rook,
                color: Color::White,
            }),
        );
        let mut mv = crate::moves::Move::new(Square::new(4, 0), Square::new(6, 0));
        mv.kind = crate::moves::MoveKind::CastleKingside;
        let next = board.make_move(mv);

        assert_eq!(next.get(Square::new(6, 0)).unwrap().kind, PieceKind::King);
        assert_eq!(next.get(Square::new(5, 0)).unwrap().kind, PieceKind::Rook);
        assert!(next.get(Square::new(7, 0)).is_none());
    }

    #[test]
    fn make_move_en_passant_removes_the_captured_pawn() {
        let mut board = Board::empty();
        board.set(
            Square::new(4, 4),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        board.set(
            Square::new(3, 4),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            }),
        );
        board.en_passant = Some(Square::new(3, 5));
        let mut mv = crate::moves::Move::new(Square::new(4, 4), Square::new(3, 5));
        mv.kind = crate::moves::MoveKind::EnPassant;
        let next = board.make_move(mv);

        assert!(next.get(Square::new(3, 4)).is_none());
        assert_eq!(next.get(Square::new(3, 5)).unwrap().kind, PieceKind::Pawn);
    }
}
