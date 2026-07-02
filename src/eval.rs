use crate::board::Board;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_material_is_balanced() {
        let board = Board::starting_position();
        assert_eq!(material_score(&board), 0);
    }
}
