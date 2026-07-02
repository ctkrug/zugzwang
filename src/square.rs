/// A single square on the 8x8 board, addressed by file (a-h) and rank (1-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Square {
    pub file: u8, // 0..=7 for a..=h
    pub rank: u8, // 0..=7 for 1..=8
}

impl Square {
    pub fn new(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8 && rank < 8);
        Square { file, rank }
    }

    /// Index into a flat 64-element board array (a1 = 0, h8 = 63).
    pub fn index(self) -> usize {
        (self.rank as usize) * 8 + self.file as usize
    }
}
