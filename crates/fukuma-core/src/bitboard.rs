use crate::types::Square;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY:  Self = Self(0);
    pub const FULL:   Self = Self(u64::MAX);
    pub const FILE_A: Self = Self(0x0101_0101_0101_0101);
    pub const FILE_H: Self = Self(0x8080_8080_8080_8080);
    pub const RANK_1: Self = Self(0x0000_0000_0000_00FF);
    pub const RANK_8: Self = Self(0xFF00_0000_0000_0000);

    #[inline] pub const fn from_sq(sq: Square) -> Self { Self(1u64 << sq.as_u8()) }
    #[inline] pub const fn set(self, sq: Square) -> Self { Self(self.0 | (1u64 << sq.as_u8())) }
    #[inline] pub const fn clear(self, sq: Square) -> Self { Self(self.0 & !(1u64 << sq.as_u8())) }
    #[inline] pub const fn contains(self, sq: Square) -> bool { (self.0 >> sq.as_u8()) & 1 == 1 }
    #[inline] pub const fn is_empty(self) -> bool { self.0 == 0 }
    #[inline] pub const fn popcount(self) -> u32 { self.0.count_ones() }
    #[inline] pub fn lsb(self) -> Square { Square::new(self.0.trailing_zeros() as u8) }
    #[inline] pub fn pop_lsb(&mut self) -> Square {
        let sq = self.lsb();
        self.0 &= self.0 - 1;
        sq
    }

    #[inline] pub const fn north(self) -> Self { Self(self.0 << 8) }
    #[inline] pub const fn south(self) -> Self { Self(self.0 >> 8) }
    #[inline] pub const fn east(self)  -> Self { Self((self.0 & !Self::FILE_H.0) << 1) }
    #[inline] pub const fn west(self)  -> Self { Self((self.0 & !Self::FILE_A.0) >> 1) }
    #[inline] pub const fn north_east(self) -> Self { Self((self.0 & !Self::FILE_H.0) << 9) }
    #[inline] pub const fn north_west(self) -> Self { Self((self.0 & !Self::FILE_A.0) << 7) }
    #[inline] pub const fn south_east(self) -> Self { Self((self.0 & !Self::FILE_H.0) >> 7) }
    #[inline] pub const fn south_west(self) -> Self { Self((self.0 & !Self::FILE_A.0) >> 9) }
}

impl Iterator for Bitboard {
    type Item = Square;
    fn next(&mut self) -> Option<Square> {
        if self.is_empty() { None } else { Some(self.pop_lsb()) }
    }
}

impl BitAnd  for Bitboard { type Output = Self; fn bitand (self, r: Self) -> Self { Self(self.0 &  r.0) } }
impl BitOr   for Bitboard { type Output = Self; fn bitor  (self, r: Self) -> Self { Self(self.0 |  r.0) } }
impl BitXor  for Bitboard { type Output = Self; fn bitxor (self, r: Self) -> Self { Self(self.0 ^  r.0) } }
impl Not     for Bitboard { type Output = Self; fn not    (self)           -> Self { Self(!self.0)       } }
impl BitAndAssign for Bitboard { fn bitand_assign(&mut self, r: Self) { self.0 &= r.0; } }
impl BitOrAssign  for Bitboard { fn bitor_assign (&mut self, r: Self) { self.0 |= r.0; } }
impl BitXorAssign for Bitboard { fn bitxor_assign(&mut self, r: Self) { self.0 ^= r.0; } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    #[test]
    fn set_contains_clear() {
        let bb = Bitboard::EMPTY.set(Square::D4);
        assert!(bb.contains(Square::D4));
        assert!(!bb.contains(Square::E4));
        assert_eq!(bb.clear(Square::D4), Bitboard::EMPTY);
    }

    #[test]
    fn popcount_and_iteration() {
        let squares = [Square::A1, Square::D4, Square::H8];
        let bb = squares.iter().fold(Bitboard::EMPTY, |b, &s| b.set(s));
        assert_eq!(bb.popcount(), 3);
        let collected: Vec<Square> = bb.collect();
        assert_eq!(collected, squares);
    }

    #[test]
    fn no_wrap_at_edges() {
        assert_eq!(Bitboard::FILE_H.east(),  Bitboard::EMPTY);
        assert_eq!(Bitboard::FILE_A.west(),  Bitboard::EMPTY);
        assert_eq!(Bitboard::RANK_8.north(), Bitboard::EMPTY);
        assert_eq!(Bitboard::RANK_1.south(), Bitboard::EMPTY);
    }

    #[test]
    fn north_moves_square_up() {
        let a1 = Bitboard::from_sq(Square::A1);
        assert_eq!(a1.north(), Bitboard::from_sq(Square::A2));
    }

    #[test]
    fn diagonal_shifts() {
        let d4 = Bitboard::from_sq(Square::D4);
        assert_eq!(d4.north_east(), Bitboard::from_sq(Square::E5));
        assert_eq!(d4.south_west(), Bitboard::from_sq(Square::C3));
    }

    #[test]
    fn bitwise_ops() {
        let a = Bitboard::from_sq(Square::A1);
        let b = Bitboard::from_sq(Square::H8);
        assert_eq!((a | b).popcount(), 2);
        assert_eq!((a & b), Bitboard::EMPTY);
        assert_eq!((a ^ b).popcount(), 2);
        assert!(!(a).contains(Square::A1) == false);
    }
}
