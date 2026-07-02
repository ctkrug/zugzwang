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

    /// Parses a FEN (Forsyth-Edwards Notation) position string.
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let mut fields = fen.split_whitespace();
        let placement = fields.next().ok_or("FEN is missing piece placement")?;
        let side = fields.next().unwrap_or("w");
        let castling_str = fields.next().unwrap_or("-");
        let en_passant_str = fields.next().unwrap_or("-");
        let halfmove_str = fields.next().unwrap_or("0");
        let fullmove_str = fields.next().unwrap_or("1");

        let mut board = Board::empty();
        let ranks: Vec<&str> = placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(format!(
                "FEN piece placement must have 8 ranks, got {}",
                ranks.len()
            ));
        }
        for (i, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - i as u8;
            let mut file = 0u8;
            for c in rank_str.chars() {
                if let Some(skip) = c.to_digit(10) {
                    file += skip as u8;
                } else {
                    if file >= 8 {
                        return Err(format!("FEN rank '{rank_str}' has too many squares"));
                    }
                    let color = if c.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let kind = match c.to_ascii_lowercase() {
                        'p' => PieceKind::Pawn,
                        'n' => PieceKind::Knight,
                        'b' => PieceKind::Bishop,
                        'r' => PieceKind::Rook,
                        'q' => PieceKind::Queen,
                        'k' => PieceKind::King,
                        _ => return Err(format!("invalid FEN piece character '{c}'")),
                    };
                    board.set(Square::new(file, rank), Some(Piece { kind, color }));
                    file += 1;
                }
            }
            if file != 8 {
                return Err(format!(
                    "FEN rank '{rank_str}' must cover exactly 8 squares, got {file}"
                ));
            }
        }

        for color in [Color::White, Color::Black] {
            let kings = (0..64)
                .filter(|&i| {
                    let sq = Square::new((i % 8) as u8, (i / 8) as u8);
                    board.get(sq) == Some(Piece { kind: PieceKind::King, color })
                })
                .count();
            if kings != 1 {
                return Err(format!(
                    "FEN must have exactly one {color:?} king, got {kings}"
                ));
            }
        }

        board.side_to_move = match side {
            "w" => Color::White,
            "b" => Color::Black,
            other => return Err(format!("invalid FEN side to move '{other}'")),
        };

        board.castling = CastlingRights {
            white_kingside: castling_str.contains('K'),
            white_queenside: castling_str.contains('Q'),
            black_kingside: castling_str.contains('k'),
            black_queenside: castling_str.contains('q'),
        };

        board.en_passant = if en_passant_str == "-" {
            None
        } else {
            Some(Square::from_algebraic(en_passant_str)?)
        };

        board.halfmove_clock = halfmove_str
            .parse()
            .map_err(|_| format!("invalid FEN halfmove clock '{halfmove_str}'"))?;
        board.fullmove_number = fullmove_str
            .parse()
            .map_err(|_| format!("invalid FEN fullmove number '{fullmove_str}'"))?;

        Ok(board)
    }

    /// Renders this position as a FEN (Forsyth-Edwards Notation) string.
    pub fn to_fen(&self) -> String {
        let mut placement = String::new();
        for i in 0..8u8 {
            let rank = 7 - i;
            let mut empty_run = 0u8;
            for file in 0..8u8 {
                match self.get(Square::new(file, rank)) {
                    Some(piece) => {
                        if empty_run > 0 {
                            placement.push_str(&empty_run.to_string());
                            empty_run = 0;
                        }
                        placement.push(piece_symbol(piece));
                    }
                    None => empty_run += 1,
                }
            }
            if empty_run > 0 {
                placement.push_str(&empty_run.to_string());
            }
            if rank != 0 {
                placement.push('/');
            }
        }

        let side = match self.side_to_move {
            Color::White => "w",
            Color::Black => "b",
        };

        let mut castling = String::new();
        if self.castling.white_kingside {
            castling.push('K');
        }
        if self.castling.white_queenside {
            castling.push('Q');
        }
        if self.castling.black_kingside {
            castling.push('k');
        }
        if self.castling.black_queenside {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }

        let en_passant = self
            .en_passant
            .map(|sq| sq.to_algebraic())
            .unwrap_or_else(|| "-".to_string());

        format!(
            "{placement} {side} {castling} {en_passant} {} {}",
            self.halfmove_clock, self.fullmove_number
        )
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
    fn display_renders_ranks_top_down_with_dots_for_empty_squares() {
        let board = Board::starting_position();
        let rendered = board.to_string();
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 8);
        // Rank 8 (Black's back rank) is printed first.
        assert_eq!(lines[0], "r n b q k b n r ");
        assert_eq!(lines[2], ". . . . . . . . ");
        assert_eq!(lines[7], "R N B Q K B N R ");
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
    fn make_move_resets_the_halfmove_clock_on_a_pawn_move() {
        let mut board = Board::from_fen("4k3/8/8/8/8/8/4P3/4K3 w - - 12 20").unwrap();
        board.halfmove_clock = 12;
        let mv = crate::moves::Move::new(Square::new(4, 1), Square::new(4, 2));
        assert_eq!(board.make_move(mv).halfmove_clock, 0);
    }

    #[test]
    fn make_move_resets_the_halfmove_clock_on_a_capture() {
        let board = Board::from_fen("4k3/8/8/8/8/4n3/4R3/4K3 w - - 12 20").unwrap();
        let mv = crate::moves::Move::new(Square::new(4, 1), Square::new(4, 2));
        assert_eq!(board.make_move(mv).halfmove_clock, 0);
    }

    #[test]
    fn make_move_increments_the_halfmove_clock_on_a_quiet_non_pawn_move() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4KR2 w - - 12 20").unwrap();
        let mv = crate::moves::Move::new(Square::new(5, 0), Square::new(5, 3));
        assert_eq!(board.make_move(mv).halfmove_clock, 13);
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
    fn from_fen_parses_the_starting_position() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.castling, CastlingRights::ALL);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 1);
        assert_eq!(board.get(Square::new(4, 0)).unwrap().kind, PieceKind::King);
    }

    #[test]
    fn from_fen_parses_castling_rights_and_en_passant() {
        let board = Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3")
            .unwrap();
        assert_eq!(board.en_passant, Some(Square::new(3, 5)));
        assert_eq!(board.castling, CastlingRights::ALL);
    }

    #[test]
    fn from_fen_rejects_malformed_placement() {
        assert!(Board::from_fen("not-a-fen").is_err());
        assert!(Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP w KQkq - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_a_rank_that_is_too_short() {
        // "6" only covers six squares; a valid rank must sum to exactly 8.
        assert!(Board::from_fen("6/8/8/8/8/8/8/8 w - - 0 1").is_err());
        assert!(Board::from_fen("pppppp/8/8/8/8/8/8/8 w - - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_a_rank_that_is_too_long() {
        // A single "9" skip code overshoots the 8 squares in a rank.
        assert!(Board::from_fen("9/8/8/8/8/8/8/8 w - - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_an_invalid_piece_character() {
        assert!(
            Board::from_fen("xnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err()
        );
    }

    #[test]
    fn from_fen_rejects_a_position_with_no_white_king() {
        assert!(Board::from_fen("4k3/8/8/8/8/8/8/8 w - - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_a_position_with_two_black_kings() {
        assert!(Board::from_fen("3kk3/8/8/8/8/8/8/4K3 w - - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_an_invalid_side_to_move() {
        assert!(Board::from_fen("8/8/8/8/8/8/8/8 x - - 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_an_invalid_en_passant_square() {
        assert!(Board::from_fen("8/8/8/8/8/8/8/8 w - z9 0 1").is_err());
    }

    #[test]
    fn from_fen_rejects_a_non_numeric_halfmove_clock() {
        assert!(Board::from_fen("8/8/8/8/8/8/8/8 w - - abc 1").is_err());
    }

    #[test]
    fn from_fen_rejects_a_non_numeric_fullmove_number() {
        assert!(Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 abc").is_err());
    }

    #[test]
    fn from_fen_fills_in_defaults_for_missing_trailing_fields() {
        // Only piece placement is strictly required; everything else falls
        // back to a sane default (as some non-standard FEN sources omit
        // the move counters).
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3").unwrap();
        assert_eq!(board.side_to_move, Color::White);
        assert_eq!(board.castling, CastlingRights::NONE);
        assert_eq!(board.en_passant, None);
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 1);
    }

    #[test]
    fn to_fen_round_trips_the_starting_position() {
        let board = Board::starting_position();
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn fen_round_trip_preserves_arbitrary_positions() {
        let fens = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "8/8/8/4k3/8/8/8/4K2R w K - 5 39",
        ];
        for fen in fens {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(board.to_fen(), fen);
        }
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
