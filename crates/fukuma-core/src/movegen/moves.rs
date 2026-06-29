use crate::types::{PieceType, Square};

/// Compact move: 16 bits — from(6) | to(6) | flags(4).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Move(u16);

/// Flag constants (4 bits).
impl Move {
    pub const QUIET: u16 = 0b0000;
    pub const DOUBLE_PUSH: u16 = 0b0001;
    pub const CASTLE_K: u16 = 0b0010;
    pub const CASTLE_Q: u16 = 0b0011;
    pub const CAPTURE: u16 = 0b0100;
    pub const EP_CAPTURE: u16 = 0b0101;
    // Promotions (bit 3 set = promotion, bit 2 = capture+promo)
    pub const PROMO_N: u16 = 0b1000;
    pub const PROMO_B: u16 = 0b1001;
    pub const PROMO_R: u16 = 0b1010;
    pub const PROMO_Q: u16 = 0b1011;
    pub const PROMO_CAP_N: u16 = 0b1100;
    pub const PROMO_CAP_B: u16 = 0b1101;
    pub const PROMO_CAP_R: u16 = 0b1110;
    pub const PROMO_CAP_Q: u16 = 0b1111;

    pub const NULL: Move = Move(0);

    #[inline]
    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        Move((from.as_u8() as u16) | ((to.as_u8() as u16) << 6) | (flags << 12))
    }

    #[inline]
    pub fn from(self) -> Square {
        Square::new((self.0 & 0x3F) as u8)
    }
    #[inline]
    pub fn to(self) -> Square {
        Square::new(((self.0 >> 6) & 0x3F) as u8)
    }
    #[inline]
    pub fn flags(self) -> u16 {
        self.0 >> 12
    }
    #[inline]
    pub fn is_capture(self) -> bool {
        self.flags() & 0b0100 != 0
    }
    #[inline]
    pub fn is_promo(self) -> bool {
        self.flags() & 0b1000 != 0
    }
    #[inline]
    pub fn is_ep(self) -> bool {
        self.flags() == Self::EP_CAPTURE
    }
    #[inline]
    pub fn is_castle(self) -> bool {
        self.flags() == Self::CASTLE_K || self.flags() == Self::CASTLE_Q
    }

    /// Returns the promotion piece type (only valid when `is_promo()`).
    #[inline]
    pub fn promo_piece(self) -> PieceType {
        match self.flags() & 0b0011 {
            0 => PieceType::Knight,
            1 => PieceType::Bishop,
            2 => PieceType::Rook,
            _ => PieceType::Queen,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from();
        let to = self.to();
        let fc = (b'a' + from.file().0) as char;
        let fr = (b'1' + from.rank().0) as char;
        let tc = (b'a' + to.file().0) as char;
        let tr = (b'1' + to.rank().0) as char;
        write!(f, "{fc}{fr}{tc}{tr}")?;
        if self.is_promo() {
            let p = match self.promo_piece() {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                _ => 'q',
            };
            write!(f, "{p}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    #[test]
    fn move_encoding_roundtrip() {
        let m = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);
        assert_eq!(m.from(), Square::E2);
        assert_eq!(m.to(), Square::E4);
        assert_eq!(m.flags(), Move::DOUBLE_PUSH);
        assert!(!m.is_capture());
        assert!(!m.is_promo());
    }

    #[test]
    fn capture_flag() {
        let m = Move::new(Square::D4, Square::E5, Move::CAPTURE);
        assert!(m.is_capture());
    }

    #[test]
    fn promo_piece() {
        let m = Move::new(Square::A7, Square::A8, Move::PROMO_Q);
        assert!(m.is_promo());
        assert_eq!(m.promo_piece(), PieceType::Queen);
        assert_eq!(m.to_string(), "a7a8q");
    }

    #[test]
    fn display_quiet() {
        let m = Move::new(Square::E2, Square::E4, Move::QUIET);
        assert_eq!(m.to_string(), "e2e4");
    }

    #[test]
    fn all_64_squares_encodable() {
        for i in 0u8..64 {
            for j in 0u8..64 {
                let m = Move::new(Square::new(i), Square::new(j), Move::QUIET);
                assert_eq!(m.from(), Square::new(i));
                assert_eq!(m.to(), Square::new(j));
            }
        }
    }
}
