use crate::board::Board;
use crate::moves::{Move, MoveKind};
use crate::square::Square;
use crate::types::{Color, PieceKind};

const KNIGHT_OFFSETS: [(i32, i32); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

const KING_OFFSETS: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

const BISHOP_DIRS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const ROOK_DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const QUEEN_DIRS: [(i32, i32); 8] = [
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
];

fn in_bounds(file: i32, rank: i32) -> bool {
    (0..8).contains(&file) && (0..8).contains(&rank)
}

/// Generates pseudo-legal moves for the side to move: legal piece movement
/// and capture rules, but without filtering out moves that leave the
/// mover's own king in check. `legal_moves` does that filtering.
pub fn pseudo_legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    pawn_moves(board, &mut moves);
    step_moves(board, &mut moves, PieceKind::Knight, &KNIGHT_OFFSETS);
    step_moves(board, &mut moves, PieceKind::King, &KING_OFFSETS);
    sliding_moves(board, &mut moves, PieceKind::Bishop, &BISHOP_DIRS);
    sliding_moves(board, &mut moves, PieceKind::Rook, &ROOK_DIRS);
    sliding_moves(board, &mut moves, PieceKind::Queen, &QUEEN_DIRS);
    moves
}

/// Generates ray moves (bishop/rook/queen) for every piece of `kind`
/// belonging to the side to move: each direction is walked until it hits
/// the board edge, a friendly piece (stop, don't include), or an enemy
/// piece (include as a capture, then stop).
fn sliding_moves(board: &Board, moves: &mut Vec<Move>, kind: PieceKind, dirs: &[(i32, i32)]) {
    let color = board.side_to_move;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let from = Square::new(file, rank);
            let Some(piece) = board.get(from) else {
                continue;
            };
            if piece.color != color || piece.kind != kind {
                continue;
            }
            for &(df, dr) in dirs {
                let mut f = file as i32 + df;
                let mut r = rank as i32 + dr;
                while in_bounds(f, r) {
                    let to = Square::new(f as u8, r as u8);
                    match board.get(to) {
                        None => moves.push(Move::new(from, to)),
                        Some(occupant) => {
                            if occupant.color != color {
                                moves.push(Move::new(from, to));
                            }
                            break;
                        }
                    }
                    f += df;
                    r += dr;
                }
            }
        }
    }
}

/// Generates single-step moves (knight jumps or king steps) for every piece
/// of `kind` belonging to the side to move: a move onto an empty square or
/// an enemy-occupied one (a capture), never onto a friendly piece.
fn step_moves(board: &Board, moves: &mut Vec<Move>, kind: PieceKind, offsets: &[(i32, i32)]) {
    let color = board.side_to_move;
    for rank in 0..8u8 {
        for file in 0..8u8 {
            let from = Square::new(file, rank);
            let Some(piece) = board.get(from) else {
                continue;
            };
            if piece.color != color || piece.kind != kind {
                continue;
            }
            for &(df, dr) in offsets {
                let f = file as i32 + df;
                let r = rank as i32 + dr;
                if !in_bounds(f, r) {
                    continue;
                }
                let to = Square::new(f as u8, r as u8);
                match board.get(to) {
                    None => moves.push(Move::new(from, to)),
                    Some(occupant) if occupant.color != color => moves.push(Move::new(from, to)),
                    _ => {}
                }
            }
        }
    }
}

/// Pawn pushes (single and double), diagonal captures, and en passant.
/// Promotions are expanded into one move per promotion piece by
/// `push_pawn_move`; check detection filters out any that are illegal.
fn pawn_moves(board: &Board, moves: &mut Vec<Move>) {
    let color = board.side_to_move;
    let (dir, start_rank, promo_rank) = match color {
        Color::White => (1i32, 1u8, 7u8),
        Color::Black => (-1i32, 6u8, 0u8),
    };

    for rank in 0..8u8 {
        for file in 0..8u8 {
            let from = Square::new(file, rank);
            let Some(piece) = board.get(from) else {
                continue;
            };
            if piece.color != color || piece.kind != PieceKind::Pawn {
                continue;
            }

            let one_rank = rank as i32 + dir;
            if in_bounds(file as i32, one_rank) {
                let one_sq = Square::new(file, one_rank as u8);
                if board.get(one_sq).is_none() {
                    push_pawn_move(moves, from, one_sq, promo_rank);
                    if rank == start_rank {
                        let two_sq = Square::new(file, (rank as i32 + dir * 2) as u8);
                        if board.get(two_sq).is_none() {
                            moves.push(Move::new(from, two_sq));
                        }
                    }
                }
            }

            for df in [-1i32, 1] {
                let cf = file as i32 + df;
                if !in_bounds(cf, one_rank) {
                    continue;
                }
                let target = Square::new(cf as u8, one_rank as u8);
                match board.get(target) {
                    Some(occupant) if occupant.color != color => {
                        push_pawn_move(moves, from, target, promo_rank);
                    }
                    None if board.en_passant == Some(target) => {
                        let mut mv = Move::new(from, target);
                        mv.kind = MoveKind::EnPassant;
                        moves.push(mv);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Pushes a pawn move, expanding it into four promotion moves
/// (queen/rook/bishop/knight) if it lands on the back rank.
fn push_pawn_move(moves: &mut Vec<Move>, from: Square, to: Square, promo_rank: u8) {
    if to.rank == promo_rank {
        for &kind in &[
            PieceKind::Queen,
            PieceKind::Rook,
            PieceKind::Bishop,
            PieceKind::Knight,
        ] {
            let mut mv = Move::new(from, to);
            mv.promotion = Some(kind);
            moves.push(mv);
        }
    } else {
        moves.push(Move::new(from, to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Piece;

    #[test]
    fn starting_position_has_twenty_pseudo_legal_moves() {
        // 16 pawn moves (8 single + 8 double pushes) + 4 knight moves;
        // the king and sliding pieces are all boxed in at the start.
        let board = Board::starting_position();
        let moves = pseudo_legal_moves(&board);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn knight_on_empty_board_has_eight_moves() {
        let mut board = Board::empty();
        board.set(
            Square::new(3, 3),
            Some(Piece {
                kind: PieceKind::Knight,
                color: Color::White,
            }),
        );
        let moves = pseudo_legal_moves(&board);
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn king_cannot_capture_its_own_piece() {
        let mut board = Board::empty();
        board.set(
            Square::new(4, 4),
            Some(Piece {
                kind: PieceKind::King,
                color: Color::White,
            }),
        );
        board.set(
            Square::new(4, 5),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        let moves = pseudo_legal_moves(&board);
        let king_moves: Vec<_> = moves
            .iter()
            .filter(|m| m.from == Square::new(4, 4))
            .collect();
        assert_eq!(king_moves.len(), 7);
        assert!(!king_moves.iter().any(|m| m.to == Square::new(4, 5)));
    }

    #[test]
    fn rook_on_empty_board_has_fourteen_moves() {
        let mut board = Board::empty();
        board.set(
            Square::new(3, 3),
            Some(Piece {
                kind: PieceKind::Rook,
                color: Color::White,
            }),
        );
        let moves = pseudo_legal_moves(&board);
        assert_eq!(moves.len(), 14);
    }

    #[test]
    fn bishop_ray_stops_at_a_capture_and_a_blocker() {
        let mut board = Board::empty();
        board.set(
            Square::new(3, 3),
            Some(Piece {
                kind: PieceKind::Bishop,
                color: Color::White,
            }),
        );
        board.set(
            Square::new(5, 5),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            }),
        );
        board.set(
            Square::new(1, 1),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        let moves = pseudo_legal_moves(&board);
        let bishop_moves: Vec<_> = moves
            .iter()
            .filter(|m| m.from == Square::new(3, 3))
            .collect();
        assert!(bishop_moves.iter().any(|m| m.to == Square::new(4, 4)));
        assert!(bishop_moves.iter().any(|m| m.to == Square::new(5, 5)));
        assert!(!bishop_moves.iter().any(|m| m.to == Square::new(6, 6)));
        assert!(bishop_moves.iter().any(|m| m.to == Square::new(2, 2)));
        assert!(!bishop_moves.iter().any(|m| m.to == Square::new(1, 1)));
    }

    #[test]
    fn pawn_on_start_rank_can_push_one_or_two() {
        let board = Board::starting_position();
        let moves = pseudo_legal_moves(&board);
        let e_pawn_moves: Vec<_> = moves
            .iter()
            .filter(|m| m.from == Square::new(4, 1))
            .collect();
        assert_eq!(e_pawn_moves.len(), 2);
        assert!(e_pawn_moves.iter().any(|m| m.to == Square::new(4, 2)));
        assert!(e_pawn_moves.iter().any(|m| m.to == Square::new(4, 3)));
    }

    #[test]
    fn pawn_reaching_back_rank_generates_four_promotions() {
        let mut board = Board::empty();
        board.set(
            Square::new(0, 6),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        let moves = pseudo_legal_moves(&board);
        assert_eq!(moves.len(), 4);
        let promos: Vec<_> = moves.iter().filter_map(|m| m.promotion).collect();
        assert!(promos.contains(&PieceKind::Queen));
        assert!(promos.contains(&PieceKind::Rook));
        assert!(promos.contains(&PieceKind::Bishop));
        assert!(promos.contains(&PieceKind::Knight));
    }

    #[test]
    fn pawn_can_capture_en_passant() {
        let mut board = Board::empty();
        board.set(
            Square::new(4, 4),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::White,
            }),
        );
        board.set(
            Square::new(3, 4),
            Some(Piece {
                kind: PieceKind::Pawn,
                color: Color::Black,
            }),
        );
        board.en_passant = Some(Square::new(3, 5));
        let moves = pseudo_legal_moves(&board);
        assert!(moves
            .iter()
            .any(|m| m.to == Square::new(3, 5) && m.kind == MoveKind::EnPassant));
    }
}
