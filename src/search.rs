use crate::board::Board;

/// Depth-limited negamax skeleton.
///
/// Alpha-beta pruning, move ordering, and a transposition table land once
/// move generation is complete — negamax gives the search loop its final
/// shape now so evaluation and move generation can be wired in incrementally.
pub fn negamax(_board: &Board, depth: u32) -> i32 {
    if depth == 0 {
        return 0;
    }
    0
}
