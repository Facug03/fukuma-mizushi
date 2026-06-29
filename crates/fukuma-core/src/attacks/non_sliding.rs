use crate::bitboard::Bitboard;
use crate::types::{Color, Square};

// ── Precomputed attack tables ─────────────────────────────────────────────────

const fn knight_attacks_for(sq: u8) -> u64 {
    let b = 1u64 << sq;
    let not_ab = !0x0303_0303_0303_0303u64; // not file A or B
    let not_gh = !0xC0C0_C0C0_C0C0_C0C0u64; // not file G or H
    let not_a  = !Bitboard::FILE_A.0;
    let not_h  = !Bitboard::FILE_H.0;

    let mut att = 0u64;
    att |= (b & not_ab) >> 10; // -2 file, -1 rank
    att |= (b & not_gh) >>  6; // +2 file, -1 rank
    att |= (b & not_ab) <<  6; // -2 file, +1 rank
    att |= (b & not_gh) << 10; // +2 file, +1 rank
    att |= (b & not_a)  >> 17; // -1 file, -2 rank
    att |= (b & not_h)  >> 15; // +1 file, -2 rank
    att |= (b & not_a)  << 15; // -1 file, +2 rank
    att |= (b & not_h)  << 17; // +1 file, +2 rank
    att
}

const fn king_attacks_for(sq: u8) -> u64 {
    let b = 1u64 << sq;
    let not_a = !Bitboard::FILE_A.0;
    let not_h = !Bitboard::FILE_H.0;

    let mut att = 0u64;
    att |= (b & not_a) >> 1;
    att |= (b & not_h) << 1;
    att |=  b >> 8;
    att |=  b << 8;
    att |= (b & not_a) >> 9;
    att |= (b & not_h) << 9;
    att |= (b & not_a) << 7;
    att |= (b & not_h) >> 7;
    att
}

const fn pawn_attacks_for(sq: u8, white: bool) -> u64 {
    let b = 1u64 << sq;
    let not_a = !Bitboard::FILE_A.0;
    let not_h = !Bitboard::FILE_H.0;
    if white {
        ((b & not_a) << 7) | ((b & not_h) << 9)
    } else {
        ((b & not_a) >> 9) | ((b & not_h) >> 7)
    }
}

const fn build_table_knight() -> [u64; 64] {
    let mut t = [0u64; 64];
    let mut i = 0usize;
    while i < 64 { t[i] = knight_attacks_for(i as u8); i += 1; }
    t
}

const fn build_table_king() -> [u64; 64] {
    let mut t = [0u64; 64];
    let mut i = 0usize;
    while i < 64 { t[i] = king_attacks_for(i as u8); i += 1; }
    t
}

const KNIGHT_ATTACKS: [u64; 64] = build_table_knight();
const KING_ATTACKS:   [u64; 64] = build_table_king();

const PAWN_ATTACKS: [[u64; 64]; 2] = {
    let mut t = [[0u64; 64]; 2];
    let mut i = 0usize;
    while i < 64 {
        t[0][i] = pawn_attacks_for(i as u8, true);
        t[1][i] = pawn_attacks_for(i as u8, false);
        i += 1;
    }
    t
};

// ── Public API ────────────────────────────────────────────────────────────────

#[inline]
pub fn knight_attacks(sq: Square) -> Bitboard {
    Bitboard(KNIGHT_ATTACKS[sq.index()])
}

#[inline]
pub fn king_attacks(sq: Square) -> Bitboard {
    Bitboard(KING_ATTACKS[sq.index()])
}

#[inline]
pub fn pawn_attacks(sq: Square, color: Color) -> Bitboard {
    Bitboard(PAWN_ATTACKS[color as usize][sq.index()])
}

/// All squares a pawn on `sq` can advance to (1 or 2 squares), given `occ`.
#[inline]
pub fn pawn_pushes(sq: Square, color: Color, occ: Bitboard) -> Bitboard {
    let bb = Bitboard::from_sq(sq);
    let single = (if color == Color::White { bb.north() } else { bb.south() }) & !occ;
    let start_rank = if color == Color::White {
        crate::bitboard::Bitboard(0x0000_0000_0000_FF00) // rank 2
    } else {
        crate::bitboard::Bitboard(0x00FF_0000_0000_0000) // rank 7
    };
    let double = if !(bb & start_rank).is_empty() {
        (if color == Color::White { single.north() } else { single.south() }) & !occ
    } else {
        Bitboard::EMPTY
    };
    single | double
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    #[test]
    fn knight_d4_attacks() {
        let att = knight_attacks(Square::D4);
        // D4 = file 3 (d), rank 3 (4th). Expected squares: b3, b5, c2, c6, e2, e6, f3, f5
        for sq in [Square::B3, Square::B5, Square::C2, Square::C6,
                   Square::E2, Square::E6, Square::F3, Square::F5] {
            assert!(att.contains(sq), "knight d4 should attack {sq:?}");
        }
        assert_eq!(att.popcount(), 8);
    }

    #[test]
    fn knight_a1_attacks() {
        let att = knight_attacks(Square::A1);
        assert!(att.contains(Square::B3));
        assert!(att.contains(Square::C2));
        assert_eq!(att.popcount(), 2);
    }

    #[test]
    fn knight_h8_attacks() {
        let att = knight_attacks(Square::H8);
        assert!(att.contains(Square::F7));
        assert!(att.contains(Square::G6));
        assert_eq!(att.popcount(), 2);
    }

    #[test]
    fn king_e4_attacks() {
        let att = king_attacks(Square::E4);
        assert_eq!(att.popcount(), 8);
        for sq in [Square::D3, Square::D4, Square::D5,
                   Square::E3,             Square::E5,
                   Square::F3, Square::F4, Square::F5] {
            assert!(att.contains(sq));
        }
    }

    #[test]
    fn king_corner_a1() {
        assert_eq!(king_attacks(Square::A1).popcount(), 3);
    }

    #[test]
    fn king_corner_h8() {
        assert_eq!(king_attacks(Square::H8).popcount(), 3);
    }

    #[test]
    fn pawn_attacks_white_e4() {
        let att = pawn_attacks(Square::E4, Color::White);
        assert!(att.contains(Square::D5));
        assert!(att.contains(Square::F5));
        assert_eq!(att.popcount(), 2);
    }

    #[test]
    fn pawn_attacks_black_d5() {
        let att = pawn_attacks(Square::D5, Color::Black);
        assert!(att.contains(Square::C4));
        assert!(att.contains(Square::E4));
        assert_eq!(att.popcount(), 2);
    }

    #[test]
    fn pawn_no_wrap_a_file() {
        // White pawn on a2 attacks only b3 (not h3 via wrap)
        let att = pawn_attacks(Square::A2, Color::White);
        assert!(!att.contains(Square::H3));
        assert!(att.contains(Square::B3));
        assert_eq!(att.popcount(), 1);
    }

    #[test]
    fn pawn_no_wrap_h_file() {
        let att = pawn_attacks(Square::H5, Color::Black);
        assert!(!att.contains(Square::A4));
        assert!(att.contains(Square::G4));
        assert_eq!(att.popcount(), 1);
    }

    #[test]
    fn pawn_pushes_double_from_start() {
        let occ = Bitboard::EMPTY;
        let pushes = pawn_pushes(Square::E2, Color::White, occ);
        assert!(pushes.contains(Square::E3));
        assert!(pushes.contains(Square::E4));
        assert_eq!(pushes.popcount(), 2);
    }

    #[test]
    fn pawn_push_blocked() {
        let occ = Bitboard::from_sq(Square::E3);
        let pushes = pawn_pushes(Square::E2, Color::White, occ);
        assert_eq!(pushes, Bitboard::EMPTY);
    }
}
