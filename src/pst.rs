use crate::board::Board;
use crate::square::Square;
use crate::types::{Color, PieceKind};

/// Piece-square tables (Tomasz Michniewski's widely used "simplified
/// evaluation function" values, in centipawns), each written as commonly
/// published: row 0 is rank 8 (a8..h8), row 7 is rank 1 (a1..h1). They
/// reward pieces for standing on squares that are typically good for them
/// — knights toward the center, a king tucked in a corner behind pawns —
/// on top of the flat material count.
#[rustfmt::skip]
const PAWN: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

fn table_for(kind: PieceKind) -> &'static [i32; 64] {
    match kind {
        PieceKind::Pawn => &PAWN,
        PieceKind::Knight => &KNIGHT,
        PieceKind::Bishop => &BISHOP,
        PieceKind::Rook => &ROOK,
        PieceKind::Queen => &QUEEN,
        PieceKind::King => &KING,
    }
}

/// Looks up `sq` in `table`, which is laid out rank-8-first. White reads
/// the table top-down as published; Black reads it bottom-up, which
/// mirrors it vertically — the standard trick for reusing one
/// White-oriented table for the side that starts on the opposite edge.
fn lookup(table: &[i32; 64], sq: Square, color: Color) -> i32 {
    let row = match color {
        Color::White => 7 - sq.rank,
        Color::Black => sq.rank,
    };
    table[row as usize * 8 + sq.file as usize]
}

/// Piece-square-table evaluation, from White's perspective, in centipawns.
/// Added to material score to reward pieces for standing on good squares,
/// not just for existing.
pub fn score(board: &Board) -> i32 {
    let mut score = 0;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let sq = Square::new(file, rank);
            if let Some(piece) = board.get(sq) {
                let value = lookup(table_for(piece.kind), sq, piece.color);
                score += if piece.color == Color::White {
                    value
                } else {
                    -value
                };
            }
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_pst_score_is_symmetric() {
        assert_eq!(score(&Board::starting_position()), 0);
    }

    #[test]
    fn knight_is_rewarded_for_centralizing() {
        let mut board = Board::empty();
        let rim = Square::new(0, 0);
        let center = Square::new(3, 3);
        let knight = crate::board::Piece {
            kind: PieceKind::Knight,
            color: Color::White,
        };
        board.set(rim, Some(knight));
        let rim_score = score(&board);
        board.set(rim, None);
        board.set(center, Some(knight));
        let center_score = score(&board);
        assert!(center_score > rim_score);
    }

    #[test]
    fn king_is_rewarded_for_a_castled_corner_over_the_open_center() {
        // The KING table rewards early-game safety (tucked behind pawns on
        // the back rank) over centralization — the opposite priority from
        // every other piece — since an exposed king in the middle of an
        // open board is a liability, not an asset.
        let mut board = Board::empty();
        let castled = Square::new(6, 0);
        let center = Square::new(3, 3);
        let king = crate::board::Piece {
            kind: PieceKind::King,
            color: Color::White,
        };
        board.set(castled, Some(king));
        let castled_score = score(&board);
        board.set(castled, None);
        board.set(center, Some(king));
        let center_score = score(&board);
        assert!(castled_score > center_score);
    }

    #[test]
    fn white_and_black_pawn_advancement_are_mirrored() {
        // A white pawn one step from promoting and a black pawn one step
        // from promoting should be rewarded identically in magnitude.
        let mut white_board = Board::empty();
        white_board.set(
            Square::new(4, 6),
            Some(crate::board::Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        let mut black_board = Board::empty();
        black_board.set(
            Square::new(4, 1),
            Some(crate::board::Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            }),
        );
        assert_eq!(score(&white_board), -score(&black_board));
    }
}
