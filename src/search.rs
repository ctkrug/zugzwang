use crate::board::Board;
use crate::eval::evaluate;
use crate::history::HistoryTable;
use crate::killers::KillerMoves;
use crate::movegen::{is_in_check, legal_moves};
use crate::moves::Move;
use crate::ordering::{is_capture, order_moves};
use crate::tt::{Bound, TTEntry, TranspositionTable};
use crate::types::Color;
use crate::zobrist;
use std::time::{Duration, Instant};

/// Score magnitude assigned to a checkmate found at the root (ply 0).
/// Mates found deeper in the tree score `MATE_SCORE - ply`, so the search
/// always prefers a shorter forced mate over a longer one.
pub const MATE_SCORE: i32 = 1_000_000;

/// Default transposition table size for a fresh `Search`.
const DEFAULT_TT_SIZE_MB: usize = 16;

/// Mate scores are ply-relative (they decay the deeper a mate is found),
/// so caching one at the ply it was found at and reusing it at a
/// different ply would misreport the distance to mate. Simplest fix:
/// don't cache scores anywhere near mate magnitude at all.
const MATE_CACHE_MARGIN: i32 = 1_000;

/// Holds the state a single search shares across its whole tree.
///
/// A fresh `Search` should be created per root search rather than reused
/// across unrelated positions: the killer table is keyed by ply and tuned
/// to the tree currently being explored.
pub struct Search {
    killers: KillerMoves,
    history: HistoryTable,
    tt: TranspositionTable,
}

impl Search {
    pub fn new() -> Self {
        Search {
            killers: KillerMoves::new(),
            history: HistoryTable::new(),
            tt: TranspositionTable::new(DEFAULT_TT_SIZE_MB),
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
        mut beta: i32,
    ) -> i32 {
        let mut moves = legal_moves(board);
        if moves.is_empty() {
            return if is_in_check(board) {
                -(MATE_SCORE - ply as i32)
            } else {
                0
            };
        }

        let key = zobrist::hash(board);
        if let Some(entry) = self.tt.get(key) {
            if entry.depth >= depth {
                match entry.bound {
                    Bound::Exact => return entry.score,
                    Bound::Lower if entry.score > alpha => alpha = entry.score,
                    Bound::Upper if entry.score < beta => beta = entry.score,
                    _ => {}
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }
        order_moves(board, &mut moves, self.killers.get(ply), &self.history);

        let original_alpha = alpha;
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
                    self.history.record(mv, depth);
                }
                break;
            }
        }

        if best.abs() <= MATE_SCORE - MATE_CACHE_MARGIN {
            let bound = if best <= original_alpha {
                Bound::Upper
            } else if best >= beta {
                Bound::Lower
            } else {
                Bound::Exact
            };
            self.tt.store(TTEntry {
                key,
                depth,
                score: best,
                bound,
            });
        }
        best
    }

    /// Quiescence search: extends a leaf with capture-only search until the
    /// position is "quiet" (no captures left, or the best capture doesn't
    /// help), so `negamax`'s depth cutoff doesn't stop mid-exchange and
    /// misread a position where a piece is about to be recaptured (the
    /// horizon effect).
    ///
    /// Uses the static evaluation as a lower bound ("stand pat"): a side
    /// can always choose not to capture, so if just standing there already
    /// beats `beta`, or beats `alpha`, that's folded in before trying any
    /// capture.
    fn quiescence(&mut self, board: &Board, mut alpha: i32, beta: i32) -> i32 {
        let stand_pat = perspective_score(board);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut captures = legal_moves(board);
        captures.retain(|&mv| is_capture(board, mv));
        order_moves(board, &mut captures, [None, None], &self.history);

        for mv in captures {
            let next = board.make_move(mv);
            let score = -self.quiescence(&next, -beta, -alpha);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    /// Iterative deepening from the root: searches depth 1, then 2, and so
    /// on, until `budget` elapses, returning the best move and score found
    /// by the deepest depth that finished. Each completed depth also warms
    /// up the killer and history tables the next, deeper depth reuses.
    ///
    /// The time check happens between root moves, not inside deeper plies,
    /// so a depth already in progress when the budget expires is allowed to
    /// finish rather than being cut off mid-move.
    ///
    /// Depth 1 always runs to completion regardless of `budget`: it's cheap
    /// (one shallow search per legal move), and this guarantees a legal
    /// move is always returned — even under a near-zero or zero budget —
    /// instead of `None`, which a UCI GUI reads as `bestmove 0000` and
    /// treats as a forfeit.
    pub fn find_best_move(&mut self, board: &Board, budget: Duration) -> Option<(Move, i32)> {
        let deadline = Instant::now() + budget;
        let no_deadline = Instant::now() + Duration::from_secs(3600);
        let mut best = self.root_search(board, 1, no_deadline)?;
        let mut depth = 2;
        while Instant::now() < deadline {
            match self.root_search(board, depth, deadline) {
                Some(result) => best = result,
                None => break,
            }
            depth += 1;
        }
        Some(best)
    }

    /// Searches to exactly `depth` with no time limit, for a UCI `go depth`
    /// request: unlike `find_best_move`, which treats depth as a target to
    /// iterate toward within a time budget, this honors the requested depth
    /// precisely regardless of how long it takes.
    pub fn find_best_move_to_depth(&mut self, board: &Board, depth: u32) -> Option<(Move, i32)> {
        let no_deadline = Instant::now() + Duration::from_secs(3600);
        let mut best = self.root_search(board, 1, no_deadline)?;
        for d in 2..=depth {
            best = self.root_search(board, d, no_deadline)?;
        }
        Some(best)
    }

    /// Searches every root move to `depth` and returns the best one found,
    /// or `None` if `deadline` was already reached before finishing.
    fn root_search(&mut self, board: &Board, depth: u32, deadline: Instant) -> Option<(Move, i32)> {
        let mut moves = legal_moves(board);
        if moves.is_empty() {
            return None;
        }
        order_moves(board, &mut moves, self.killers.get(0), &self.history);

        let mut alpha = -MATE_SCORE;
        let beta = MATE_SCORE;
        let mut best_move = moves[0];
        let mut best_score = i32::MIN + 1;
        for mv in moves {
            if Instant::now() >= deadline {
                return None;
            }
            let next = board.make_move(mv);
            let score = -self.negamax(&next, depth - 1, 1, -beta, -alpha);
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            if best_score > alpha {
                alpha = best_score;
            }
        }
        Some((best_move, best_score))
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

/// Static evaluation, sign-flipped so it reads as "how good is this
/// position for the side to move" rather than always from White's
/// perspective.
fn perspective_score(board: &Board) -> i32 {
    match board.side_to_move {
        Color::White => evaluate(board),
        Color::Black => -evaluate(board),
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

    #[test]
    fn quiescence_sees_past_a_hanging_queen_to_the_recapture() {
        // Black to move, right after White played Qxe5, capturing a rook:
        // a white queen now sits on e5, a black pawn on d6 can recapture
        // it with dxe5. A plain material snapshot of this exact position
        // says Black is down a queen for nothing (the horizon effect); a
        // quiescence-aware search instead plays out dxe5 and finds Black
        // is actually the one ahead, having traded a rook for a queen.
        let board = Board::from_fen("k7/8/3p4/4Q3/8/8/8/K7 b - - 0 1").unwrap();
        let naive = -crate::eval::evaluate(&board);
        let quiesced = Search::new().negamax(&board, 0, 0, -MATE_SCORE, MATE_SCORE);
        assert!(
            naive < -500,
            "expected the naive snapshot to look bad, got {naive}"
        );
        assert!(
            quiesced > 0,
            "expected quiescence to find Black ahead after dxe5, got {quiesced}"
        );
    }

    #[test]
    fn negamax_populates_the_transposition_table() {
        let board = Board::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").unwrap();
        let mut search = Search::new();
        let score = search.negamax(&board, 2, 0, -MATE_SCORE, MATE_SCORE);
        let key = crate::zobrist::hash(&board);
        let entry = search.tt.get(key).expect("root position should be cached");
        assert_eq!(entry.depth, 2);
        assert_eq!(entry.score, score);
    }

    #[test]
    fn find_best_move_finds_a_mate_in_one() {
        // White queen on h7 has several mating slides along the 7th rank
        // and 8th rank (e.g. a7# or g8#). A mate found at ply 1 dominates
        // every other move regardless of how deep the iterative search
        // gets to run within its budget, so the resulting score is
        // depth-independent even though the exact mating move isn't.
        let board = Board::from_fen("k7/7Q/1K6/8/8/8/8/8 w - - 0 1").unwrap();
        let mut search = Search::new();
        let (mv, score) = search
            .find_best_move(&board, Duration::from_millis(200))
            .unwrap();
        assert_eq!(score, MATE_SCORE - 1);
        let after = board.make_move(mv);
        assert!(is_in_check(&after) && legal_moves(&after).is_empty());
    }

    #[test]
    fn find_best_move_avoids_a_queen_for_pawn_losing_trade() {
        // White queen can capture a pawn on d7, but a black king on d8
        // defends it: Qxd7 Kxd7 loses a queen for a pawn. Move ordering
        // tries captures first, so a search that only looked one ply deep
        // would walk straight into this; the engine must look past the
        // immediate capture to the recapture and prefer something else.
        let board = Board::from_fen("3k4/3p4/8/8/8/8/8/3QK3 w - - 0 1").unwrap();
        let (mv, score) = Search::new()
            .find_best_move(&board, Duration::from_millis(200))
            .unwrap();
        assert_ne!(mv.to_uci(), "d1d7");
        assert!(score > -400, "expected no material-down score, got {score}");
    }

    #[test]
    fn find_best_move_to_depth_treats_a_zero_depth_as_depth_one() {
        // `go depth 0` isn't meaningful chess input, but the engine must
        // still hand back a legal move rather than None (which uci::run
        // renders as the forfeit-signaling "bestmove 0000").
        let board = Board::starting_position();
        let (mv, _) = Search::new().find_best_move_to_depth(&board, 0).unwrap();
        assert!(legal_moves(&board).contains(&mv));
    }

    #[test]
    fn find_best_move_to_depth_searches_the_exact_requested_depth() {
        // Depth 1 is enough to spot a hanging pawn, so a depth-1 request
        // must return the capture even though a deeper search might find
        // something else first in move ordering.
        let board = Board::from_fen("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1").unwrap();
        let (mv, score) = Search::new().find_best_move_to_depth(&board, 1).unwrap();
        assert_eq!(mv.to_uci(), "e4d5");
        assert!(score > 0);
    }

    #[test]
    fn find_best_move_returns_none_with_no_legal_moves() {
        let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(Search::new()
            .find_best_move(&board, Duration::from_millis(50))
            .is_none());
    }

    #[test]
    fn find_best_move_still_returns_a_legal_move_with_a_zero_time_budget() {
        // A UCI GUI can send `go movetime 0` (or a wtime/btime so low the
        // clamped budget rounds down); the engine must still answer with a
        // legal move rather than `None`, which uci::run renders as
        // `bestmove 0000` — read by most GUIs as a forfeit.
        let board = Board::starting_position();
        let (mv, _) = Search::new()
            .find_best_move(&board, Duration::ZERO)
            .expect("a zero budget must still return a legal move");
        assert!(legal_moves(&board).contains(&mv));
    }
}
