use crate::board::Board;
use crate::movegen::{find_uci_move, is_in_check, legal_moves};
use crate::moves::Move;
use crate::search::Search;
use std::time::Duration;

/// Time budget for each engine reply in terminal play mode. Short enough
/// to keep the game moving in an interactive terminal session.
pub const ENGINE_MOVE_TIME: Duration = Duration::from_millis(500);

/// Whether the game at `board` is still ongoing or has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Checkmate,
    Stalemate,
    /// Drawn under the fifty-move rule: 100 consecutive halfmoves (50 full
    /// moves by each side) with no pawn move and no capture.
    FiftyMoveDraw,
}

/// Classifies `board` by whether the side to move has a legal move, and if
/// not, whether that's because they're in check (checkmate) or not
/// (stalemate). A side with a legal move can still be in a position that's
/// drawn under the fifty-move rule, checked against `halfmove_clock`
/// (already tracked on `Board` for FEN round-tripping).
pub fn game_status(board: &Board) -> GameStatus {
    if legal_moves(board).is_empty() {
        return if is_in_check(board) {
            GameStatus::Checkmate
        } else {
            GameStatus::Stalemate
        };
    }
    if board.halfmove_clock >= 100 {
        return GameStatus::FiftyMoveDraw;
    }
    GameStatus::Ongoing
}

/// Applies a human move given in coordinate algebraic notation (e.g.
/// `"e2e4"`, or `"e7e8q"` for a promotion — the same notation
/// `Move::to_uci` produces), rejecting anything that isn't a legal move
/// in `board` rather than trusting free-form input.
pub fn apply_human_move(board: &Board, input: &str) -> Result<Board, String> {
    find_uci_move(board, input.trim())
        .map(|mv| board.make_move(mv))
        .ok_or_else(|| format!("illegal move: {input}"))
}

/// Has the engine choose and play a reply within `ENGINE_MOVE_TIME`.
/// Returns `None` if the side to move has no legal move (checkmate or
/// stalemate should already have ended the game before this is called).
pub fn engine_reply(board: &Board) -> Option<(Move, Board)> {
    Search::new()
        .find_best_move(board, ENGINE_MOVE_TIME)
        .map(|(mv, _)| (mv, board.make_move(mv)))
}

/// Whether the most recent position in `history` (the Zobrist hash of every
/// position reached so far in the game, including the current one) has now
/// occurred a third time — a draw by threefold repetition. `Board` itself
/// has no memory of prior positions, so callers that play out a full game
/// (like terminal play) must accumulate this history themselves.
pub fn is_threefold_repetition(history: &[u64]) -> bool {
    match history.last() {
        Some(&current) => history.iter().filter(|&&h| h == current).count() >= 3,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_status_is_ongoing_at_the_start() {
        assert_eq!(
            game_status(&Board::starting_position()),
            GameStatus::Ongoing
        );
    }

    #[test]
    fn game_status_detects_checkmate() {
        let board = Board::from_fen("k7/Q7/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(game_status(&board), GameStatus::Checkmate);
    }

    #[test]
    fn game_status_detects_a_fifty_move_draw() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 100 60").unwrap();
        assert_eq!(game_status(&board), GameStatus::FiftyMoveDraw);
    }

    #[test]
    fn game_status_checkmate_takes_priority_over_the_fifty_move_clock() {
        // A halfmove clock past 100 doesn't matter if the side to move has
        // actually been checkmated: the game already ended on that move.
        let board = Board::from_fen("k7/Q7/1K6/8/8/8/8/8 b - - 100 60").unwrap();
        assert_eq!(game_status(&board), GameStatus::Checkmate);
    }

    #[test]
    fn game_status_detects_stalemate() {
        let board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(game_status(&board), GameStatus::Stalemate);
    }

    #[test]
    fn apply_human_move_plays_a_legal_move() {
        let board = Board::starting_position();
        let next = apply_human_move(&board, "e2e4").unwrap();
        assert_eq!(
            next.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn apply_human_move_rejects_an_illegal_move() {
        let board = Board::starting_position();
        assert!(apply_human_move(&board, "e2e5").is_err());
    }

    #[test]
    fn apply_human_move_promotes_a_pawn_when_asked() {
        let board = Board::from_fen("k7/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let next = apply_human_move(&board, "e7e8q").unwrap();
        assert_eq!(
            next.get(crate::square::Square::new(4, 7)).unwrap().kind,
            crate::types::PieceKind::Queen
        );
    }

    #[test]
    fn apply_human_move_rejects_a_promotion_missing_its_piece_letter() {
        // The move itself (e7e8) is legal shaped, but a pawn reaching the
        // back rank must promote — the bare from/to pair with no
        // promotion letter doesn't match any of the four generated moves.
        let board = Board::from_fen("k7/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert!(apply_human_move(&board, "e7e8").is_err());
    }

    #[test]
    fn engine_reply_plays_a_legal_move() {
        let board = Board::starting_position();
        let (mv, next) = engine_reply(&board).unwrap();
        assert!(legal_moves(&Board::starting_position()).contains(&mv));
        assert_eq!(next.side_to_move, crate::types::Color::Black);
    }

    #[test]
    fn engine_reply_returns_none_with_no_legal_moves() {
        let board = Board::from_fen("k7/Q7/1K6/8/8/8/8/8 b - - 0 1").unwrap();
        assert!(engine_reply(&board).is_none());
    }

    #[test]
    fn is_threefold_repetition_is_false_with_no_history() {
        assert!(!is_threefold_repetition(&[]));
    }

    #[test]
    fn is_threefold_repetition_is_false_below_three_occurrences() {
        assert!(!is_threefold_repetition(&[1, 2, 1]));
    }

    #[test]
    fn is_threefold_repetition_is_true_on_the_third_occurrence() {
        assert!(is_threefold_repetition(&[1, 2, 1, 3, 1]));
    }

    #[test]
    fn is_threefold_repetition_only_counts_the_current_position() {
        // Position `2` has occurred three times, but the game is currently
        // at position `1` (the last entry), which has only occurred once.
        assert!(!is_threefold_repetition(&[2, 2, 2, 1]));
    }
}
