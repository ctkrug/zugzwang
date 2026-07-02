use crate::board::Board;
use crate::eval::material_score;
use crate::movegen::{is_in_check, legal_moves};
use crate::types::Color;

/// Score magnitude assigned to a checkmate found at the root (ply 0).
/// Mates found deeper in the tree score `MATE_SCORE - ply`, so the search
/// always prefers a shorter forced mate over a longer one.
pub const MATE_SCORE: i32 = 1_000_000;

/// Negamax search with alpha-beta pruning.
///
/// Returns the score of `board` from the perspective of the side to move,
/// in centipawns. `ply` is the distance from the root, used to make mate
/// scores decay with depth so shorter forced mates are preferred over
/// longer ones.
pub fn negamax(board: &Board, depth: u32, ply: u32, mut alpha: i32, beta: i32) -> i32 {
    let moves = legal_moves(board);
    if moves.is_empty() {
        return if is_in_check(board) {
            -(MATE_SCORE - ply as i32)
        } else {
            0
        };
    }
    if depth == 0 {
        return perspective_score(board);
    }

    let mut best = i32::MIN + 1;
    for mv in moves {
        let next = board.make_move(mv);
        let score = -negamax(&next, depth - 1, ply + 1, -beta, -alpha);
        if score > best {
            best = score;
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Material score, sign-flipped so it reads as "how good is this position
/// for the side to move" rather than always from White's perspective.
fn perspective_score(board: &Board) -> i32 {
    match board.side_to_move {
        Color::White => material_score(board),
        Color::Black => -material_score(board),
    }
}
