use crate::square::Square;
use crate::types::PieceKind;

/// Distinguishes move mechanics that `Board::make_move` must special-case:
/// en passant removes a pawn that isn't on the destination square, and
/// castling also relocates a rook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    Normal,
    EnPassant,
    CastleKingside,
    CastleQueenside,
}

/// A single chess move: origin square, destination square, an optional
/// promotion piece for pawn moves reaching the back rank, and the move kind
/// needed to apply it correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
    pub kind: MoveKind,
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            promotion: None,
            kind: MoveKind::Normal,
        }
    }

    /// Renders this move in UCI long algebraic notation, e.g. `"e2e4"` or
    /// `"e7e8q"` for a promotion.
    pub fn to_uci(self) -> String {
        let mut s = format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic());
        if let Some(promotion) = self.promotion {
            s.push(match promotion {
                PieceKind::Queen => 'q',
                PieceKind::Rook => 'r',
                PieceKind::Bishop => 'b',
                PieceKind::Knight => 'n',
                PieceKind::Pawn | PieceKind::King => {
                    unreachable!("pawns only promote to queen/rook/bishop/knight")
                }
            });
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_uci_formats_plain_move() {
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        assert_eq!(mv.to_uci(), "e2e4");
    }

    #[test]
    fn to_uci_formats_promotion() {
        let mut mv = Move::new(Square::new(4, 6), Square::new(4, 7));
        mv.promotion = Some(PieceKind::Queen);
        assert_eq!(mv.to_uci(), "e7e8q");
    }

    #[test]
    fn to_uci_formats_all_four_promotion_pieces() {
        for (kind, letter) in [
            (PieceKind::Queen, 'q'),
            (PieceKind::Rook, 'r'),
            (PieceKind::Bishop, 'b'),
            (PieceKind::Knight, 'n'),
        ] {
            let mut mv = Move::new(Square::new(0, 6), Square::new(0, 7));
            mv.promotion = Some(kind);
            assert_eq!(mv.to_uci(), format!("a7a8{letter}"));
        }
    }

    #[test]
    fn to_uci_formats_castling_as_a_plain_coordinate_move() {
        // UCI has no special castling notation: a castle is just the
        // king's own from/to squares, same as any other king move.
        let mut mv = Move::new(Square::new(4, 0), Square::new(6, 0));
        mv.kind = MoveKind::CastleKingside;
        assert_eq!(mv.to_uci(), "e1g1");
    }
}
