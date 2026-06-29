#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    #[inline]
    pub const fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PieceType {
    Pawn   = 0,
    Knight = 1,
    Bishop = 2,
    Rook   = 3,
    Queen  = 4,
    King   = 5,
}

impl PieceType {
    pub const ALL: [PieceType; 6] = [
        PieceType::Pawn, PieceType::Knight, PieceType::Bishop,
        PieceType::Rook, PieceType::Queen,  PieceType::King,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceType,
}

impl Piece {
    #[inline]
    pub const fn new(color: Color, kind: PieceType) -> Self {
        Self { color, kind }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct File(pub u8);

impl File {
    pub const A: Self = Self(0);
    pub const B: Self = Self(1);
    pub const C: Self = Self(2);
    pub const D: Self = Self(3);
    pub const E: Self = Self(4);
    pub const F: Self = Self(5);
    pub const G: Self = Self(6);
    pub const H: Self = Self(7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rank(pub u8);

impl Rank {
    pub const R1: Self = Self(0);
    pub const R2: Self = Self(1);
    pub const R3: Self = Self(2);
    pub const R4: Self = Self(3);
    pub const R5: Self = Self(4);
    pub const R6: Self = Self(5);
    pub const R7: Self = Self(6);
    pub const R8: Self = Self(7);
}

/// A square on the board. 0 = a1, 63 = h8 (little-endian file mapping).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    #[inline] pub const fn new(index: u8) -> Self { Self(index) }
    #[inline] pub const fn from_file_rank(file: File, rank: Rank) -> Self { Self(rank.0 * 8 + file.0) }
    #[inline] pub const fn file(self) -> File { File(self.0 & 7) }
    #[inline] pub const fn rank(self) -> Rank { Rank(self.0 >> 3) }
    #[inline] pub const fn index(self) -> usize { self.0 as usize }
    #[inline] pub const fn as_u8(self) -> u8 { self.0 }

    pub const A1: Self = Self( 0); pub const B1: Self = Self( 1); pub const C1: Self = Self( 2); pub const D1: Self = Self( 3);
    pub const E1: Self = Self( 4); pub const F1: Self = Self( 5); pub const G1: Self = Self( 6); pub const H1: Self = Self( 7);
    pub const A2: Self = Self( 8); pub const B2: Self = Self( 9); pub const C2: Self = Self(10); pub const D2: Self = Self(11);
    pub const E2: Self = Self(12); pub const F2: Self = Self(13); pub const G2: Self = Self(14); pub const H2: Self = Self(15);
    pub const A3: Self = Self(16); pub const B3: Self = Self(17); pub const C3: Self = Self(18); pub const D3: Self = Self(19);
    pub const E3: Self = Self(20); pub const F3: Self = Self(21); pub const G3: Self = Self(22); pub const H3: Self = Self(23);
    pub const A4: Self = Self(24); pub const B4: Self = Self(25); pub const C4: Self = Self(26); pub const D4: Self = Self(27);
    pub const E4: Self = Self(28); pub const F4: Self = Self(29); pub const G4: Self = Self(30); pub const H4: Self = Self(31);
    pub const A5: Self = Self(32); pub const B5: Self = Self(33); pub const C5: Self = Self(34); pub const D5: Self = Self(35);
    pub const E5: Self = Self(36); pub const F5: Self = Self(37); pub const G5: Self = Self(38); pub const H5: Self = Self(39);
    pub const A6: Self = Self(40); pub const B6: Self = Self(41); pub const C6: Self = Self(42); pub const D6: Self = Self(43);
    pub const E6: Self = Self(44); pub const F6: Self = Self(45); pub const G6: Self = Self(46); pub const H6: Self = Self(47);
    pub const A7: Self = Self(48); pub const B7: Self = Self(49); pub const C7: Self = Self(50); pub const D7: Self = Self(51);
    pub const E7: Self = Self(52); pub const F7: Self = Self(53); pub const G7: Self = Self(54); pub const H7: Self = Self(55);
    pub const A8: Self = Self(56); pub const B8: Self = Self(57); pub const C8: Self = Self(58); pub const D8: Self = Self(59);
    pub const E8: Self = Self(60); pub const F8: Self = Self(61); pub const G8: Self = Self(62); pub const H8: Self = Self(63);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_roundtrip() {
        for i in 0u8..64 {
            let sq = Square::new(i);
            assert_eq!(sq.as_u8(), i);
            assert_eq!(Square::from_file_rank(sq.file(), sq.rank()), sq);
        }
    }

    #[test]
    fn square_constants() {
        assert_eq!(Square::A1.index(), 0);
        assert_eq!(Square::H8.index(), 63);
        assert_eq!(Square::D4, Square::from_file_rank(File::D, Rank::R4));
        assert_eq!(Square::D4.file(), File::D);
        assert_eq!(Square::D4.rank(), Rank::R4);
    }

    #[test]
    fn color_flip() {
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
    }
}
