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
}

/// Classifies `board` by whether the side to move has a legal move, and if
/// not, whether that's because they're in check (checkmate) or not
/// (stalemate).
pub fn game_status(board: &Board) -> GameStatus {
    if !legal_moves(board).is_empty() {
        return GameStatus::Ongoing;
    }
    if is_in_check(board) {
        GameStatus::Checkmate
    } else {
        GameStatus::Stalemate
    }
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
}
