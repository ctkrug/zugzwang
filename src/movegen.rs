use crate::board::Board;
use crate::moves::Move;

/// Generates pseudo-legal moves for the side to move.
///
/// This is a scaffold: per-piece movement rules (pawns, knights, sliding
/// pieces, castling, en passant) land in the build phase, validated against
/// perft test positions.
pub fn pseudo_legal_moves(_board: &Board) -> Vec<Move> {
    Vec::new()
}
