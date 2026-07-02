/// How a transposition table entry's score relates to the true value of
/// the position it was stored for, following from why the search stopped:
///
/// - `Exact`: every move was searched and none failed high, so `score` is
///   the position's true value.
/// - `Lower`: a beta cutoff occurred, so the true value is at least
///   `score` (searching further could only find something even better).
/// - `Upper`: every move scored below alpha (a fail-low), so the true
///   value is at most `score`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u32,
    pub score: i32,
    pub bound: Bound,
}

/// A fixed-size, always-replace transposition table keyed by Zobrist hash.
///
/// Table size is rounded down to a power of two so the bucket index is a
/// cheap bitmask instead of a modulo. "Always-replace" means a new store
/// simply overwrites whatever was in its bucket, collision or not — no
/// aging or depth-preferred replacement scheme yet; see BACKLOG.md.
pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize,
}

impl TranspositionTable {
    /// Builds a table sized to roughly `size_mb` megabytes.
    pub fn new(size_mb: usize) -> Self {
        let slot_bytes = std::mem::size_of::<Option<TTEntry>>();
        let capacity = ((size_mb * 1024 * 1024) / slot_bytes)
            .next_power_of_two()
            .max(1);
        TranspositionTable {
            entries: vec![None; capacity],
            mask: capacity - 1,
        }
    }

    fn index(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    /// Looks up `key`, returning `None` on a miss or a hash collision with
    /// a different position (the stored key doesn't match).
    pub fn get(&self, key: u64) -> Option<TTEntry> {
        self.entries[self.index(key)].filter(|entry| entry.key == key)
    }

    pub fn store(&mut self, entry: TTEntry) {
        let index = self.index(entry.key);
        self.entries[index] = Some(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miss_on_an_empty_table() {
        let tt = TranspositionTable::new(1);
        assert!(tt.get(42).is_none());
    }

    #[test]
    fn stores_and_retrieves_an_entry() {
        let mut tt = TranspositionTable::new(1);
        let entry = TTEntry {
            key: 123,
            depth: 4,
            score: 50,
            bound: Bound::Exact,
        };
        tt.store(entry);
        let found = tt.get(123).unwrap();
        assert_eq!(found.score, 50);
        assert_eq!(found.depth, 4);
        assert_eq!(found.bound, Bound::Exact);
    }

    #[test]
    fn a_bucket_collision_does_not_return_the_wrong_position() {
        let mut tt = TranspositionTable::new(1);
        // Two keys that land in the same bucket (same low bits, since the
        // table's mask only looks at those) must not be confused for
        // each other.
        let capacity = tt.mask + 1;
        let key_a = 7u64;
        let key_b = key_a + capacity as u64;
        tt.store(TTEntry {
            key: key_a,
            depth: 1,
            score: 10,
            bound: Bound::Exact,
        });
        assert!(tt.get(key_b).is_none());
    }

    #[test]
    fn storing_again_replaces_the_previous_entry() {
        let mut tt = TranspositionTable::new(1);
        tt.store(TTEntry {
            key: 5,
            depth: 1,
            score: 10,
            bound: Bound::Exact,
        });
        tt.store(TTEntry {
            key: 5,
            depth: 2,
            score: 20,
            bound: Bound::Lower,
        });
        let found = tt.get(5).unwrap();
        assert_eq!(found.depth, 2);
        assert_eq!(found.score, 20);
        assert_eq!(found.bound, Bound::Lower);
    }
}
