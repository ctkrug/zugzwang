use crate::board::Board;
use crate::eval::piece_value;
use crate::history::HistoryTable;
use crate::moves::{Move, MoveKind};
use crate::types::PieceKind;

/// Score tiers, from highest to lowest priority. Each tier occupies a
/// disjoint range far wider than any score that can occur within it, so a
/// move's tier always dominates its in-tier score when moves are compared.
const CAPTURE_TIER: i32 = 3_000_000;
const PROMOTION_TIER: i32 = 2_500_000;
const KILLER_TIER: i32 = 2_000_000;
const HISTORY_TIER: i32 = 1_000_000;

/// Sorts `moves` so alpha-beta explores the most promising ones first.
///
/// Ordering, highest priority first:
/// 1. Captures, ranked by MVV-LVA (most valuable victim, least valuable
///    attacker): a pawn taking a queen sorts before a queen taking a pawn,
///    since the former is far more likely to hold up after the opponent's
///    reply.
/// 2. Non-capturing promotions, ranked by the promoted piece's value — a
///    queening push is as tactically loud as a capture and worth trying
///    before quieter moves, even when it doesn't take anything.
/// 3. `killers`, quiet moves that caused a beta cutoff at this ply in a
///    sibling branch.
/// 4. Remaining quiet moves, ranked by `history` — how often that from/to
///    pair has caused a cutoff anywhere in the tree so far.
pub fn order_moves(
    board: &Board,
    moves: &mut [Move],
    killers: [Option<Move>; 2],
    history: &HistoryTable,
) {
    moves.sort_by_key(|&mv| std::cmp::Reverse(order_score(board, mv, killers, history)));
}

fn order_score(board: &Board, mv: Move, killers: [Option<Move>; 2], history: &HistoryTable) -> i32 {
    if let Some(victim) = capture_victim(board, mv) {
        let attacker = board
            .get(mv.from)
            .expect("move has a piece on its from-square");
        return CAPTURE_TIER + piece_value(victim) * 16 - piece_value(attacker.kind);
    }
    if let Some(promotion) = mv.promotion {
        return PROMOTION_TIER + piece_value(promotion);
    }
    if killers[0] == Some(mv) {
        return KILLER_TIER + 1;
    }
    if killers[1] == Some(mv) {
        return KILLER_TIER;
    }
    HISTORY_TIER + history.get(mv)
}

fn capture_victim(board: &Board, mv: Move) -> Option<PieceKind> {
    if mv.kind == MoveKind::EnPassant {
        return Some(PieceKind::Pawn);
    }
    board.get(mv.to).map(|p| p.kind)
}

/// Whether `mv` captures a piece (including en passant), for callers such
/// as the search that only want to record killer moves for quiet moves.
pub fn is_capture(board: &Board, mv: Move) -> bool {
    capture_victim(board, mv).is_some()
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
        order_moves(&board, &mut moves, [None, None], &HistoryTable::new());
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
        order_moves(&board, &mut moves, [None, None], &HistoryTable::new());
        assert_eq!(moves[0].from, Square::new(2, 4));
    }

    #[test]
    fn killer_move_sorts_before_other_quiet_moves() {
        let board = Board::starting_position();
        let killer = Move::new(Square::new(6, 0), Square::new(5, 2));
        let mut moves = vec![
            Move::new(Square::new(4, 1), Square::new(4, 2)),
            killer,
            Move::new(Square::new(3, 1), Square::new(3, 2)),
        ];
        order_moves(
            &board,
            &mut moves,
            [Some(killer), None],
            &HistoryTable::new(),
        );
        assert_eq!(moves[0], killer);
    }

    #[test]
    fn capture_still_outranks_a_killer_move() {
        // White queen on d1 can capture a pawn on d7; a knight hop is
        // marked as the killer at this node but must still sort second.
        let board = Board::from_fen("4k3/3p4/8/8/8/8/8/3QK1N1 w - - 0 1").unwrap();
        let capture = Move::new(Square::new(3, 0), Square::new(3, 6));
        let killer = Move::new(Square::new(6, 0), Square::new(5, 2));
        let mut moves = vec![killer, capture];
        order_moves(
            &board,
            &mut moves,
            [Some(killer), None],
            &HistoryTable::new(),
        );
        assert_eq!(moves[0], capture);
    }

    #[test]
    fn higher_history_score_ranks_a_quiet_move_first() {
        let board = Board::starting_position();
        let strong = Move::new(Square::new(4, 1), Square::new(4, 2));
        let weak = Move::new(Square::new(3, 1), Square::new(3, 2));
        let mut history = HistoryTable::new();
        history.record(strong, 4);
        let mut moves = vec![weak, strong];
        order_moves(&board, &mut moves, [None, None], &history);
        assert_eq!(moves[0], strong);
    }

    #[test]
    fn non_capturing_promotion_sorts_before_a_killer_and_other_quiets() {
        // A pawn on e7 pushing to e8=Q doesn't capture anything, but it's
        // still far more urgent to search than an ordinary quiet move.
        let board = Board::from_fen("7k/4P3/8/8/8/8/8/K5N1 w - - 0 1").unwrap();
        let mut promotion = Move::new(Square::new(4, 6), Square::new(4, 7));
        promotion.promotion = Some(PieceKind::Queen);
        let killer = Move::new(Square::new(6, 0), Square::new(5, 2));
        let mut moves = vec![killer, promotion];
        order_moves(
            &board,
            &mut moves,
            [Some(killer), None],
            &HistoryTable::new(),
        );
        assert_eq!(moves[0], promotion);
    }

    #[test]
    fn a_capture_still_outranks_a_non_capturing_promotion() {
        // Even a modest capture (a rook taking a pawn) outranks a
        // non-capturing promotion: captures are unconditionally the
        // highest tier, promotions the next one down.
        let board = Board::from_fen("7k/3pP3/8/8/8/8/8/3R3K w - - 0 1").unwrap();
        let capture = Move::new(Square::new(3, 0), Square::new(3, 6));
        let mut promotion = Move::new(Square::new(4, 6), Square::new(4, 7));
        promotion.promotion = Some(PieceKind::Queen);
        let mut moves = vec![promotion, capture];
        order_moves(&board, &mut moves, [None, None], &HistoryTable::new());
        assert_eq!(moves[0], capture);
    }
}
