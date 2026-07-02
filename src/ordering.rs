use crate::board::Board;
use crate::eval::piece_value;
use crate::moves::{Move, MoveKind};
use crate::types::PieceKind;

/// Sorts `moves` so alpha-beta explores the most promising ones first.
///
/// Captures sort ahead of quiet moves, ranked by MVV-LVA (most valuable
/// victim, least valuable attacker): a pawn taking a queen sorts before a
/// queen taking a pawn, since the former is far more likely to hold up
/// after the opponent's reply. Quiet moves keep their generation order.
pub fn order_moves(board: &Board, moves: &mut [Move]) {
    moves.sort_by_key(|&mv| std::cmp::Reverse(mvv_lva_score(board, mv)));
}

/// Ranks a capture by victim value scaled above attacker value, so the
/// victim dominates the ordering and the attacker only breaks ties among
/// equal-victim captures. Quiet moves score 0 and sort after every capture.
fn mvv_lva_score(board: &Board, mv: Move) -> i32 {
    let Some(victim) = capture_victim(board, mv) else {
        return 0;
    };
    let attacker = board
        .get(mv.from)
        .expect("move has a piece on its from-square");
    piece_value(victim) * 16 - piece_value(attacker.kind)
}

fn capture_victim(board: &Board, mv: Move) -> Option<PieceKind> {
    if mv.kind == MoveKind::EnPassant {
        return Some(PieceKind::Pawn);
    }
    board.get(mv.to).map(|p| p.kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn captures_sort_before_quiet_moves() {
        // White queen on d1 can capture a pawn on d7 or make a quiet move
        // to d2; the capture must come first regardless of list order.
        let board = Board::from_fen("4k3/3p4/8/8/8/8/8/3QK3 w - - 0 1").unwrap();
        let mut moves = vec![
            Move::new(Square::new(3, 0), Square::new(3, 1)),
            Move::new(Square::new(3, 0), Square::new(3, 6)),
        ];
        order_moves(&board, &mut moves);
        assert_eq!(moves[0].to, Square::new(3, 6));
    }

    #[test]
    fn cheaper_attacker_ranks_first_among_equal_victims() {
        // Both a white pawn on c5 and a white rook on d1 can capture a
        // black pawn on d6; the pawn recapture should sort first since
        // it risks less material if the pawn was actually defended.
        let board = Board::from_fen("4k3/8/3p4/2P5/8/8/8/3RK3 w - - 0 1").unwrap();
        let mut moves = vec![
            Move::new(Square::new(3, 0), Square::new(3, 5)),
            Move::new(Square::new(2, 4), Square::new(3, 5)),
        ];
        order_moves(&board, &mut moves);
        assert_eq!(moves[0].from, Square::new(2, 4));
    }
}
