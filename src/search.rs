use crate::board::Board;
use crate::eval::material_score;
use crate::killers::KillerMoves;
use crate::movegen::{is_in_check, legal_moves};
use crate::ordering::{is_capture, order_moves};
use crate::types::Color;

/// Score magnitude assigned to a checkmate found at the root (ply 0).
/// Mates found deeper in the tree score `MATE_SCORE - ply`, so the search
/// always prefers a shorter forced mate over a longer one.
pub const MATE_SCORE: i32 = 1_000_000;

/// Holds the state a single search shares across its whole tree.
///
/// A fresh `Search` should be created per root search rather than reused
/// across unrelated positions: the killer table is keyed by ply and tuned
/// to the tree currently being explored.
pub struct Search {
    killers: KillerMoves,
}

impl Search {
    pub fn new() -> Self {
        Search {
            killers: KillerMoves::new(),
        }
    }

    /// Negamax search with alpha-beta pruning.
    ///
    /// Returns the score of `board` from the perspective of the side to
    /// move, in centipawns. `ply` is the distance from the root, used to
    /// make mate scores decay with depth so shorter forced mates are
    /// preferred over longer ones.
    pub fn negamax(
        &mut self,
        board: &Board,
        depth: u32,
        ply: u32,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        let mut moves = legal_moves(board);
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
        order_moves(board, &mut moves, self.killers.get(ply));

        let mut best = i32::MIN + 1;
        for mv in moves {
            let next = board.make_move(mv);
            let score = -self.negamax(&next, depth - 1, ply + 1, -beta, -alpha);
            if score > best {
                best = score;
            }
            if best > alpha {
                alpha = best;
            }
            if alpha >= beta {
                if !is_capture(board, mv) {
                    self.killers.store(ply, mv);
                }
                break;
            }
        }
        best
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

/// Material score, sign-flipped so it reads as "how good is this position
/// for the side to move" rather than always from White's perspective.
fn perspective_score(board: &Board) -> i32 {
    match board.side_to_move {
        Color::White => material_score(board),
        Color::Black => -material_score(board),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negamax_scores_checkmate_as_a_loss_for_the_mated_side() {
        // Black king boxed in on a8, white queen delivers mate on a7
        // covered by the white king on b6; black has no legal reply.
        let board = Board::from_fen("k7/Q7/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        let score = Search::new().negamax(&board, 4, 0, -MATE_SCORE, MATE_SCORE);
        assert_eq!(score, -(MATE_SCORE));
    }

    #[test]
    fn negamax_scores_stalemate_as_a_draw() {
        // Black king on h8 has no legal move (g8/g7/h7 all covered by the
        // white king on g6 and queen on f7) and is not itself in check.
        let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        let score = Search::new().negamax(&board, 4, 0, -MATE_SCORE, MATE_SCORE);
        assert_eq!(score, 0);
    }

    #[test]
    fn negamax_prefers_a_free_pawn_capture() {
        // White to move: a pawn on e4 can capture a hanging pawn on d5
        // with nothing recapturing, so the position should score as
        // favorable for White at even a shallow depth.
        let board = Board::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").unwrap();
        let score = Search::new().negamax(&board, 1, 0, -MATE_SCORE, MATE_SCORE);
        assert!(score > 0);
    }
}
