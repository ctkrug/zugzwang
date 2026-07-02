use crate::moves::Move;

/// A "butterfly" history table: for every from/to square pair, tracks how
/// often a quiet move between them has caused a beta cutoff, weighted by
/// how deep the cutoff happened (a cutoff found deep in the tree is a
/// stronger signal than one found near the leaves). Used to order quiet
/// moves that aren't captures or killers.
pub struct HistoryTable {
    scores: Vec<Vec<i32>>,
}

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable {
            scores: vec![vec![0; 64]; 64],
        }
    }

    pub fn get(&self, mv: Move) -> i32 {
        self.scores[mv.from.index()][mv.to.index()]
    }

    /// Rewards `mv` for causing a cutoff `depth` plies from a leaf.
    pub fn record(&mut self, mv: Move, depth: u32) {
        self.scores[mv.from.index()][mv.to.index()] += (depth * depth) as i32;
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;

    #[test]
    fn unrecorded_move_has_zero_history() {
        let table = HistoryTable::new();
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        assert_eq!(table.get(mv), 0);
    }

    #[test]
    fn recording_a_cutoff_increases_its_history_score() {
        let mut table = HistoryTable::new();
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        table.record(mv, 3);
        assert_eq!(table.get(mv), 9);
    }

    #[test]
    fn deeper_cutoffs_add_more_weight_than_shallow_ones() {
        let mut table = HistoryTable::new();
        let shallow = Move::new(Square::new(4, 1), Square::new(4, 3));
        let deep = Move::new(Square::new(3, 1), Square::new(3, 3));
        table.record(shallow, 1);
        table.record(deep, 4);
        assert!(table.get(deep) > table.get(shallow));
    }

    #[test]
    fn repeated_cutoffs_accumulate() {
        let mut table = HistoryTable::new();
        let mv = Move::new(Square::new(4, 1), Square::new(4, 3));
        table.record(mv, 2);
        table.record(mv, 2);
        assert_eq!(table.get(mv), 8);
    }
}
