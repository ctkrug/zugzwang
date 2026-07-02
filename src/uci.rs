use crate::board::Board;
use crate::movegen::find_uci_move;
use crate::search::Search;
use crate::types::Color;
use std::io::{self, BufRead, Write};
use std::time::Duration;

/// Time budget for a `go` with no time control at all (no `movetime` and no
/// `wtime`/`btime` for the side to move).
const DEFAULT_MOVE_TIME: Duration = Duration::from_secs(1);

/// Floor and ceiling on the budget derived from `wtime`/`btime`, so neither
/// a near-flagging clock nor a huge one drives the search to an extreme.
const MIN_MOVE_TIME_MS: u64 = 50;
const MAX_MOVE_TIME_MS: u64 = 5_000;

/// Plies-to-go assumed when `movestogo` isn't given, i.e. sudden-death time
/// control: spend a conservative slice of the clock on each move rather
/// than trying to budget for the entire rest of the game.
const DEFAULT_MOVES_TO_GO: u64 = 30;

/// Runs the UCI command loop over stdin/stdout.
///
/// Handles the engine identification handshake (`uci`/`isready`), `quit`,
/// `position`, and `go`, tracking the current game as a `Board` and
/// running the search to answer `go` with a `bestmove`.
pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut board = Board::starting_position();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line == "uci" {
            let _ = writeln!(stdout, "id name Zugzwang");
            let _ = writeln!(stdout, "id author Charlie Krug");
            let _ = writeln!(stdout, "uciok");
        } else if line == "isready" {
            let _ = writeln!(stdout, "readyok");
        } else if line == "ucinewgame" {
            board = Board::starting_position();
        } else if let Some(args) = line.strip_prefix("position") {
            if let Some(new_board) = parse_position(args.trim()) {
                board = new_board;
            }
        } else if line == "go" || line.starts_with("go ") {
            let args = line.strip_prefix("go").unwrap_or("").trim();
            let best = match go_depth(args) {
                Some(depth) => Search::new().find_best_move_to_depth(&board, depth),
                None => {
                    let budget = move_time(args, board.side_to_move);
                    Search::new().find_best_move(&board, budget)
                }
            };
            let uci_move = best
                .map(|(mv, _)| mv.to_uci())
                .unwrap_or_else(|| "0000".to_string());
            let _ = writeln!(stdout, "bestmove {uci_move}");
        } else if line == "stop" {
            // `go` runs synchronously to completion before the next line is
            // read, so there is never a search in flight to interrupt.
        } else if line == "quit" {
            break;
        }
        let _ = stdout.flush();
    }
}

/// Parses a UCI `position` command's arguments: `startpos` or `fen <fen>`,
/// optionally followed by `moves <uci> <uci> ...` to replay onto it.
/// Returns `None` if the setup keyword, FEN, or any move is malformed.
fn parse_position(args: &str) -> Option<Board> {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    let moves_at = tokens.iter().position(|&t| t == "moves");
    let (setup, moves) = match moves_at {
        Some(i) => (&tokens[..i], &tokens[i + 1..]),
        None => (&tokens[..], &[][..]),
    };

    let mut board = match *setup.first()? {
        "startpos" => Board::starting_position(),
        "fen" => Board::from_fen(&setup[1..].join(" ")).ok()?,
        _ => return None,
    };

    for &uci_move in moves {
        board = board.make_move(find_uci_move(&board, uci_move)?);
    }
    Some(board)
}

/// Reads the numeric value following `key` in a whitespace-tokenized `go`
/// argument list, e.g. `token_value("wtime 30000", "wtime") == Some(30000)`.
fn token_value(args: &str, key: &str) -> Option<u64> {
    let tokens: Vec<&str> = args.split_whitespace().collect();
    tokens
        .iter()
        .position(|&t| t == key)
        .and_then(|i| tokens.get(i + 1))
        .and_then(|v| v.parse().ok())
}

/// Parses a `go depth <n>` request, so it can be honored exactly instead of
/// falling back to a time-boxed search: a GUI or analysis tool asking for a
/// specific depth expects that depth, not whatever iterative deepening
/// happens to reach within `DEFAULT_MOVE_TIME`.
fn go_depth(args: &str) -> Option<u32> {
    token_value(args, "depth").map(|d| d as u32)
}

/// Derives a search time budget from a `go` command's arguments.
///
/// `movetime <ms>` is honored directly. Otherwise, the side to move's own
/// clock (`wtime`/`btime`) is divided by `movestogo` (or
/// `DEFAULT_MOVES_TO_GO` if absent) and clamped to a sane range. With
/// neither present, falls back to `DEFAULT_MOVE_TIME`.
fn move_time(args: &str, side_to_move: Color) -> Duration {
    let value_after = |key: &str| token_value(args, key);

    if let Some(movetime) = value_after("movetime") {
        return Duration::from_millis(movetime);
    }

    let own_time = match side_to_move {
        Color::White => value_after("wtime"),
        Color::Black => value_after("btime"),
    };
    let Some(remaining_ms) = own_time else {
        return DEFAULT_MOVE_TIME;
    };

    let moves_to_go = value_after("movestogo")
        .unwrap_or(DEFAULT_MOVES_TO_GO)
        .max(1);
    let budget_ms = (remaining_ms / moves_to_go).clamp(MIN_MOVE_TIME_MS, MAX_MOVE_TIME_MS);
    Duration::from_millis(budget_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_position_startpos_with_no_moves() {
        let board = parse_position("startpos").unwrap();
        assert_eq!(board.to_fen(), Board::starting_position().to_fen());
    }

    #[test]
    fn parse_position_startpos_replays_moves() {
        let board = parse_position("startpos moves e2e4 e7e5").unwrap();
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"
        );
    }

    #[test]
    fn parse_position_fen_with_no_moves() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let board = parse_position(&format!("fen {fen}")).unwrap();
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn parse_position_fen_replays_moves() {
        let fen = "4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1";
        let board = parse_position(&format!("fen {fen} moves e4d5")).unwrap();
        assert!(board.get(crate::square::Square::new(3, 4)).is_some());
    }

    #[test]
    fn parse_position_rejects_unknown_keyword() {
        assert!(parse_position("nonsense").is_none());
    }

    #[test]
    fn parse_position_rejects_illegal_move() {
        assert!(parse_position("startpos moves e2e5").is_none());
    }

    #[test]
    fn go_depth_reads_an_explicit_depth() {
        assert_eq!(go_depth("depth 6"), Some(6));
        assert_eq!(go_depth("wtime 30000 depth 4"), Some(4));
    }

    #[test]
    fn go_depth_is_none_without_a_depth_token() {
        assert_eq!(go_depth("movetime 250"), None);
        assert_eq!(go_depth(""), None);
    }

    #[test]
    fn move_time_honors_explicit_movetime() {
        let budget = move_time("movetime 250 wtime 60000", Color::White);
        assert_eq!(budget, Duration::from_millis(250));
    }

    #[test]
    fn move_time_divides_own_clock_by_movestogo() {
        let budget = move_time("wtime 30000 btime 30000 movestogo 10", Color::White);
        assert_eq!(budget, Duration::from_millis(3_000));
    }

    #[test]
    fn move_time_reads_the_clock_for_the_side_to_move() {
        let budget = move_time("wtime 100000 btime 1000 movestogo 20", Color::Black);
        assert_eq!(
            budget,
            Duration::from_millis(MIN_MOVE_TIME_MS.max(1000 / 20))
        );
    }

    #[test]
    fn move_time_falls_back_to_default_with_no_time_control() {
        assert_eq!(move_time("", Color::White), DEFAULT_MOVE_TIME);
        assert_eq!(move_time("infinite", Color::White), DEFAULT_MOVE_TIME);
    }

    #[test]
    fn move_time_clamps_a_huge_budget() {
        let budget = move_time("wtime 100000000 movestogo 1", Color::White);
        assert_eq!(budget, Duration::from_millis(MAX_MOVE_TIME_MS));
    }
}
