use std::io::{self, BufRead, Write};
use zugzwang::board::Board;
use zugzwang::movegen::perft;
use zugzwang::play::{
    apply_human_move, engine_reply, game_status, is_threefold_repetition, GameStatus,
};
use zugzwang::uci;
use zugzwang::zobrist;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("uci") => uci::run(),
        Some("perft") => run_perft(&args[2..]),
        Some("play") => run_play(),
        Some(other) => {
            eprintln!("unknown command '{other}'");
            eprintln!("usage: zugzwang [uci|perft <depth> [fen]|play]");
            std::process::exit(1);
        }
        None => {
            let board = Board::starting_position();
            println!("Zugzwang chess engine");
            print!("{board}");
        }
    }
}

/// Runs an interactive terminal game: the human plays White, entering
/// moves in coordinate algebraic notation (e.g. `e2e4`, or `e7e8q` for a
/// promotion); the engine replies as Black. The board is printed after
/// every move, human or engine, and the game ends on checkmate,
/// stalemate, `quit`, or end of input.
fn run_play() {
    let stdin = io::stdin();
    let mut board = Board::starting_position();
    let mut history = vec![zobrist::hash(&board)];
    println!("{board}");

    loop {
        print!("Your move (or 'quit'): ");
        let _ = io::stdout().flush();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        let input = line.trim();
        if input == "quit" {
            break;
        }

        board = match apply_human_move(&board, input) {
            Ok(next) => next,
            Err(message) => {
                println!("{message}");
                continue;
            }
        };
        history.push(zobrist::hash(&board));
        println!("{board}");
        if report_game_over(&board, &history) {
            break;
        }

        let Some((mv, next)) = engine_reply(&board) else {
            break;
        };
        println!("Engine plays {}", mv.to_uci());
        board = next;
        history.push(zobrist::hash(&board));
        println!("{board}");
        if report_game_over(&board, &history) {
            break;
        }
    }
}

/// Prints a message and returns `true` if `board` is a finished game,
/// checking `history` (the Zobrist hash of every position reached so far,
/// including the current one) for a draw by threefold repetition alongside
/// the single-position outcomes `game_status` can determine on its own.
fn report_game_over(board: &Board, history: &[u64]) -> bool {
    match game_status(board) {
        GameStatus::Checkmate => {
            println!("Checkmate.");
            return true;
        }
        GameStatus::Stalemate => {
            println!("Stalemate.");
            return true;
        }
        GameStatus::FiftyMoveDraw => {
            println!("Draw by the fifty-move rule.");
            return true;
        }
        GameStatus::Ongoing => {}
    }
    if is_threefold_repetition(history) {
        println!("Draw by threefold repetition.");
        return true;
    }
    false
}

/// Runs `perft <depth> [fen]`, printing the node count at every depth from
/// 1 up to `depth` for the given position (the starting position if no FEN
/// is given). Printing every depth, not just the deepest, makes it easy to
/// spot exactly where a move generation bug first inflates or deflates the
/// count against known-good reference values.
fn run_perft(args: &[String]) {
    let Some(depth) = args.first().and_then(|s| s.parse::<u32>().ok()) else {
        eprintln!("usage: zugzwang perft <depth> [fen]");
        std::process::exit(1);
    };

    let board = if args.len() > 1 {
        match Board::from_fen(&args[1..].join(" ")) {
            Ok(board) => board,
            Err(err) => {
                eprintln!("invalid FEN: {err}");
                std::process::exit(1);
            }
        }
    } else {
        Board::starting_position()
    };

    for d in 1..=depth {
        println!("perft({d}) = {}", perft(&board, d));
    }
}
