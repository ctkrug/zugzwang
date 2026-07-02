use crate::board::Board;
use crate::pst;
use crate::square::Square;
use crate::types::{Color, PieceKind};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

/// Centipawn value of a piece kind, independent of color or position.
///
/// Exposed beyond this module so move ordering can rank captures by the
/// same material scale the evaluation uses.
pub fn piece_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => PAWN_VALUE,
        PieceKind::Knight => KNIGHT_VALUE,
        PieceKind::Bishop => BISHOP_VALUE,
        PieceKind::Rook => ROOK_VALUE,
        PieceKind::Queen => QUEEN_VALUE,
        PieceKind::King => 0,
    }
}

/// Material-only evaluation, from White's perspective, in centipawns.
///
/// Piece-square tables, mobility, and king safety are planned additions
/// once the search can make use of a richer signal.
pub fn material_score(board: &Board) -> i32 {
    let mut score = 0;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            if let Some(piece) = board.get(Square::new(file, rank)) {
                let value = piece_value(piece.kind);
                score += if piece.color == Color::White {
                    value
                } else {
                    -value
                };
            }
        }
    }
    score
}

/// Full static evaluation, from White's perspective, in centipawns: raw
/// material plus a piece-square-table bonus for standing on good squares.
pub fn evaluate(board: &Board) -> i32 {
    material_score(board) + pst::score(board)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_material_is_balanced() {
        let board = Board::starting_position();
        assert_eq!(material_score(&board), 0);
    }

    #[test]
    fn starting_position_evaluation_is_balanced() {
        let board = Board::starting_position();
        assert_eq!(evaluate(&board), 0);
    }

    #[test]
    fn evaluate_adds_material_and_pst() {
        // A lone white knight on the rim: material_score is +320, and the
        // knight's PST value on a1 is negative, so the combined
        // evaluation should be strictly less than material alone.
        let mut board = Board::empty();
        board.set(
            Square::new(0, 0),
            Some(crate::board::Piece {
                kind: PieceKind::Knight,
                color: Color::White,
            }),
        );
        assert!(evaluate(&board) < material_score(&board));
    }
}
