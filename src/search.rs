use crate::board::Board;
use crate::eval::material_score;

/// Depth-limited negamax skeleton.
///
/// Alpha-beta pruning, move ordering, and a transposition table land once
/// move generation is complete — negamax gives the search loop its final
/// shape now so move generation can be wired in incrementally.
pub fn negamax(board: &Board, depth: u32) -> i32 {
    if depth == 0 {
        return material_score(board);
    }
    material_score(board)
}
