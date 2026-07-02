/// Color of a chess piece or the side to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// The six distinct chess piece kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Which castling moves each side still has the right to attempt.
///
/// A right is lost permanently once the king or the relevant rook moves (or
/// that rook is captured) — it does not track whether castling is legal
/// *right now* (e.g. blocked by check), only whether it is still possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub const NONE: CastlingRights = CastlingRights {
        white_kingside: false,
        white_queenside: false,
        black_kingside: false,
        black_queenside: false,
    };

    pub const ALL: CastlingRights = CastlingRights {
        white_kingside: true,
        white_queenside: true,
        black_kingside: true,
        black_queenside: true,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_toggles_color() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }
}
