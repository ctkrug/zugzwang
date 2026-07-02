use crate::board::Board;
use crate::square::Square;
use crate::types::{Color, PieceKind};
use std::sync::OnceLock;

const PIECE_KINDS: usize = 6;
const COLORS: usize = 2;
const SQUARES: usize = 64;

/// Random keys XORed together to build a Zobrist hash: one per
/// piece/color/square combination, plus one for side-to-move, four for
/// castling rights, and eight for the en passant file. XORing in (or back
/// out) the key for whatever changed is what makes Zobrist hashing cheap
/// to maintain incrementally — this module only computes it from scratch
/// each time, which is simpler and still correct, just not as fast.
struct Keys {
    piece_square: [[[u64; SQUARES]; PIECE_KINDS]; COLORS],
    side_to_move: u64,
    castling: [u64; 4],
    en_passant_file: [u64; 8],
}

/// A small, fixed-seed PRNG (SplitMix64) used only to fill the key tables.
/// Determinism matters more than randomness quality here: the same seed
/// must produce the same keys on every run, or a position hashed by one
/// run of the engine wouldn't match the same position hashed by another.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(|| {
        let mut state = 0x2545F4914F6CDD1Du64;
        let mut next = || splitmix64(&mut state);
        Keys {
            piece_square: std::array::from_fn(|_| {
                std::array::from_fn(|_| std::array::from_fn(|_| next()))
            }),
            side_to_move: next(),
            castling: std::array::from_fn(|_| next()),
            en_passant_file: std::array::from_fn(|_| next()),
        }
    })
}

fn color_index(color: Color) -> usize {
    match color {
        Color::White => 0,
        Color::Black => 1,
    }
}

fn kind_index(kind: PieceKind) -> usize {
    match kind {
        PieceKind::Pawn => 0,
        PieceKind::Knight => 1,
        PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    }
}

/// Computes a Zobrist hash of `board`'s position: pieces, side to move,
/// castling rights, and en passant target. Two boards with the same hash
/// are (almost certainly) the same position; used to key the
/// transposition table.
pub fn hash(board: &Board) -> u64 {
    let keys = keys();
    let mut h = 0u64;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let sq = Square::new(file, rank);
            if let Some(piece) = board.get(sq) {
                h ^=
                    keys.piece_square[color_index(piece.color)][kind_index(piece.kind)][sq.index()];
            }
        }
    }
    if board.side_to_move == Color::Black {
        h ^= keys.side_to_move;
    }
    if board.castling.white_kingside {
        h ^= keys.castling[0];
    }
    if board.castling.white_queenside {
        h ^= keys.castling[1];
    }
    if board.castling.black_kingside {
        h ^= keys.castling[2];
    }
    if board.castling.black_queenside {
        h ^= keys.castling[3];
    }
    if let Some(ep) = board.en_passant {
        h ^= keys.en_passant_file[ep.file as usize];
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_positions_hash_the_same() {
        let a = Board::starting_position();
        let b = Board::starting_position();
        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn different_positions_hash_differently() {
        let start = Board::starting_position();
        let after_e4 = start.make_move(crate::moves::Move::new(
            Square::new(4, 1),
            Square::new(4, 3),
        ));
        assert_ne!(hash(&start), hash(&after_e4));
    }

    #[test]
    fn transposed_move_orders_hash_the_same() {
        // 1.Nf3 Nf6 2.Nc3 Nc6 and 1.Nc3 Nc6 2.Nf3 Nf6 reach the identical
        // position by different move orders — exactly the case a
        // transposition table exists to exploit, so their hashes (and not
        // just their FENs) must match.
        use crate::movegen::find_uci_move;
        let play = |moves: &[&str]| {
            let mut board = Board::starting_position();
            for &uci in moves {
                board = board.make_move(find_uci_move(&board, uci).unwrap());
            }
            board
        };
        let a = play(&["g1f3", "g8f6", "b1c3", "b8c6"]);
        let b = play(&["b1c3", "b8c6", "g1f3", "g8f6"]);
        assert_eq!(a.to_fen(), b.to_fen());
        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn side_to_move_affects_the_hash() {
        let white = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let black = Board::from_fen("4k3/8/8/8/8/8/8/4K3 b - - 0 1").unwrap();
        assert_ne!(hash(&white), hash(&black));
    }

    #[test]
    fn castling_rights_affect_the_hash() {
        let with_rights = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let without_rights = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();
        assert_ne!(hash(&with_rights), hash(&without_rights));
    }

    #[test]
    fn en_passant_target_affects_the_hash() {
        let with_ep =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3")
                .unwrap();
        let without_ep =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3").unwrap();
        assert_ne!(hash(&with_ep), hash(&without_ep));
    }
}
