use crate::board::Board;
use crate::movegen::legal_moves;
use crate::moves::Move;
use crate::search::Search;
use std::io::{self, BufRead, Write};
use std::time::Duration;

/// Time budget for a `go` with no time control specified. Real time
/// management (`wtime`/`btime`/`movestogo`) lands separately.
const DEFAULT_MOVE_TIME: Duration = Duration::from_secs(1);

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
            let best = Search::new().find_best_move(&board, DEFAULT_MOVE_TIME);
            let uci_move = best
                .map(|(mv, _)| mv.to_uci())
                .unwrap_or_else(|| "0000".to_string());
            let _ = writeln!(stdout, "bestmove {uci_move}");
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
        board = board.make_move(find_move(&board, uci_move)?);
    }
    Some(board)
}

fn find_move(board: &Board, uci: &str) -> Option<Move> {
    legal_moves(board).into_iter().find(|mv| mv.to_uci() == uci)
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
}
