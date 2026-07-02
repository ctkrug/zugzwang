use std::io::{self, BufRead, Write};

/// Runs the UCI command loop over stdin/stdout.
///
/// Handles the engine identification handshake (`uci`/`isready`) and `quit`
/// now; `position`/`go` land once search and move generation are wired to
/// the board.
pub fn run() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        match line.trim() {
            "uci" => {
                let _ = writeln!(stdout, "id name Zugzwang");
                let _ = writeln!(stdout, "id author Charlie Krug");
                let _ = writeln!(stdout, "uciok");
            }
            "isready" => {
                let _ = writeln!(stdout, "readyok");
            }
            "quit" => break,
            _ => {}
        }
        let _ = stdout.flush();
    }
}
