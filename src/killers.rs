use crate::moves::Move;

/// Two killer-quiet-move slots per search ply.
///
/// A "killer" is a quiet move that caused a beta cutoff at a given ply in
/// one branch of the tree; trying it first in sibling branches at the same
/// ply is cheap and, since the position is similar, often cuts off again.
/// Keeping the two most recent killers (rather than just one) catches the
/// common case where a node has two independently strong quiet replies.
pub struct KillerMoves {
    slots: Vec<[Option<Move>; 2]>,
}

impl KillerMoves {
    pub fn new() -> Self {
        KillerMoves { slots: Vec::new() }
    }

    /// The killer moves recorded for `ply`, most recent first.
    pub fn get(&self, ply: u32) -> [Option<Move>; 2] {
        self.slots
            .get(ply as usize)
            .copied()
            .unwrap_or([None, None])
    }

    /// Records `mv` as a killer at `ply`, promoting it to the front slot.
    /// A move already stored at this ply is not duplicated into both slots.
    pub fn store(&mut self, ply: u32, mv: Move) {
        let ply = ply as usize;
        if self.slots.len() <= ply {
            self.slots.resize(ply + 1, [None, None]);
        }
        let slot = &mut self.slots[ply];
        if slot[0] == Some(mv) {
            return;
        }
        slot[1] = slot[0];
        slot[0] = Some(mv);
    }
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn unrecorded_ply_has_no_killers() {
        let killers = KillerMoves::new();
        assert_eq!(killers.get(3), [None, None]);
    }

    #[test]
    fn stored_move_becomes_the_primary_killer() {
        let mut killers = KillerMoves::new();
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        killers.store(2, mv);
        assert_eq!(killers.get(2), [Some(mv), None]);
    }

    #[test]
    fn a_second_distinct_move_pushes_the_first_into_the_backup_slot() {
        let mut killers = KillerMoves::new();
        let first = Move::new(Square::new(4, 1), Square::new(4, 3));
        let second = Move::new(Square::new(3, 1), Square::new(3, 3));
        killers.store(2, first);
        killers.store(2, second);
        assert_eq!(killers.get(2), [Some(second), Some(first)]);
    }

    #[test]
    fn storing_the_same_move_again_does_not_duplicate_it() {
        let mut killers = KillerMoves::new();
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        killers.store(2, mv);
        killers.store(2, mv);
        assert_eq!(killers.get(2), [Some(mv), None]);
    }
}
