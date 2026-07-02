use zugzwang::board::Board;
use zugzwang::uci;

fn main() {
    let mode = std::env::args().nth(1);
    match mode.as_deref() {
        Some("uci") => uci::run(),
        _ => {
            let board = Board::starting_position();
            println!("Zugzwang chess engine");
            print!("{board}");
        }
    }
}
