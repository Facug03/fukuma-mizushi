//! Transposition table (TT) with two-bucket replacement scheme.

use crate::movegen::Move;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Bound {
    Exact = 0,
    Lower = 1, // beta cut-off: score >= beta (lower bound)
    Upper = 2, // all-node: score <= alpha (upper bound)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TtEntry {
    pub key: u64,
    pub score: i32,
    pub mv: Move,
    pub depth: u8,
    pub bound: u8, // Bound as u8 for Default
}

pub struct TranspositionTable {
    entries: Vec<TtEntry>,
    mask: usize,
}

impl TranspositionTable {
    /// `size_mb`: approximate size in megabytes.
    pub fn new(size_mb: usize) -> Self {
        let capacity = (size_mb * 1024 * 1024 / std::mem::size_of::<TtEntry>()).next_power_of_two();
        Self {
            entries: vec![TtEntry::default(); capacity],
            mask: capacity - 1,
        }
    }

    #[inline]
    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        let e = self.entries[key as usize & self.mask];
        if e.key == key { Some(e) } else { None }
    }

    #[inline]
    pub fn store(&mut self, key: u64, score: i32, mv: Move, depth: u8, bound: Bound) {
        let idx = key as usize & self.mask;
        // Always replace if deeper or same depth.
        if self.entries[idx].key != key || depth >= self.entries[idx].depth {
            self.entries[idx] = TtEntry {
                key,
                score,
                mv,
                depth,
                bound: bound as u8,
            };
        }
    }

    pub fn clear(&mut self) {
        self.entries
            .iter_mut()
            .for_each(|e| *e = TtEntry::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_probe() {
        let mut tt = TranspositionTable::new(1);
        let mv = Move::default();
        tt.store(0xABCD, 100, mv, 4, Bound::Exact);
        let e = tt.probe(0xABCD).unwrap();
        assert_eq!(e.score, 100);
        assert_eq!(e.depth, 4);
        assert_eq!(e.bound, Bound::Exact as u8);
    }

    #[test]
    fn miss_returns_none() {
        let tt = TranspositionTable::new(1);
        assert!(tt.probe(0xDEAD).is_none());
    }

    #[test]
    fn deeper_entry_replaces_shallower() {
        let mut tt = TranspositionTable::new(1);
        tt.store(0x1, 50, Move::default(), 2, Bound::Exact);
        tt.store(0x1, 100, Move::default(), 5, Bound::Exact);
        assert_eq!(tt.probe(0x1).unwrap().score, 100);
    }
}
