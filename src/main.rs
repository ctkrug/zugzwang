use zugzwang::board::Board;
use zugzwang::movegen::perft;
use zugzwang::uci;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("uci") => uci::run(),
        Some("perft") => run_perft(&args[2..]),
        _ => {
            let board = Board::starting_position();
            println!("Zugzwang chess engine");
            print!("{board}");
        }
    }
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
