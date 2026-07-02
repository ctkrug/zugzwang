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

    /// Builds a square from a flat board-array index (a1 = 0, h8 = 63).
    pub fn from_index(index: usize) -> Self {
        Square::new((index % 8) as u8, (index / 8) as u8)
    }

    /// Parses standard algebraic square notation, e.g. `"e4"`.
    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(format!("invalid square '{s}'"));
        }
        let (file, rank) = (bytes[0], bytes[1]);
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return Err(format!("invalid square '{s}'"));
        }
        Ok(Square::new(file - b'a', rank - b'1'))
    }

    /// Renders this square as standard algebraic notation, e.g. `"e4"`.
    pub fn to_algebraic(self) -> String {
        format!(
            "{}{}",
            (b'a' + self.file) as char,
            (b'1' + self.rank) as char
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_round_trip() {
        for file in 0..8u8 {
            for rank in 0..8u8 {
                let sq = Square::new(file, rank);
                assert_eq!(Square::from_algebraic(&sq.to_algebraic()), Ok(sq));
            }
        }
    }

    #[test]
    fn from_algebraic_rejects_out_of_range_input() {
        assert!(Square::from_algebraic("i1").is_err());
        assert!(Square::from_algebraic("a9").is_err());
        assert!(Square::from_algebraic("a").is_err());
    }

    #[test]
    fn from_index_matches_index() {
        for i in 0..64 {
            assert_eq!(Square::from_index(i).index(), i);
        }
    }
}
