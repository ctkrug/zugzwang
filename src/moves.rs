use crate::square::Square;
use crate::types::PieceKind;

/// A single chess move: origin square, destination square, and an optional
/// promotion piece for pawn moves reaching the back rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            promotion: None,
        }
    }
}
