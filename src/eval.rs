use crate::board::Board;
use crate::square::Square;
use crate::types::{Color, PieceKind};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

fn piece_value(kind: PieceKind) -> i32 {
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
