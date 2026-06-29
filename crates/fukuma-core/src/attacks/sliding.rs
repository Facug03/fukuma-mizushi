//! Sliding piece attacks via magic bitboards.
//!
//! Magic numbers are found at first use (OnceLock). Correctness is guaranteed
//! by construction: the magic finder rejects any magic with collisions.
//!
//! `slow_*_attacks` are ray-based reference implementations used only in tests.

use crate::bitboard::Bitboard;
use crate::types::Square;
use std::sync::OnceLock;

// ── Reference implementations (ray-based) ────────────────────────────────────

pub fn slow_rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rays(sq, occ, &[(0i8, 1i8), (0, -1), (1, 0), (-1, 0)])
}

pub fn slow_bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rays(sq, occ, &[(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)])
}

fn rays(sq: Square, occ: Bitboard, dirs: &[(i8, i8)]) -> Bitboard {
    let r = sq.rank().0 as i8;
    let f = sq.file().0 as i8;
    let mut att = 0u64;
    for &(dr, df) in dirs {
        let (mut rr, mut ff) = (r + dr, f + df);
        while (0..8).contains(&rr) && (0..8).contains(&ff) {
            let bit = 1u64 << (rr * 8 + ff);
            att |= bit;
            if occ.0 & bit != 0 { break; }
            rr += dr; ff += df;
        }
    }
    Bitboard(att)
}

// ── Mask computation ──────────────────────────────────────────────────────────

fn rook_mask(sq: Square) -> u64 {
    let r = sq.rank().0;
    let f = sq.file().0;
    let mut mask = 0u64;
    for ff in 1..7u8 { if ff != f { mask |= 1u64 << (r * 8 + ff); } }
    for rr in 1..7u8 { if rr != r { mask |= 1u64 << (rr * 8 + f); } }
    mask
}

fn bishop_mask(sq: Square) -> u64 {
    let r = sq.rank().0 as i8;
    let f = sq.file().0 as i8;
    let mut mask = 0u64;
    for &(dr, df) in &[(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)] {
        let (mut rr, mut ff) = (r + dr, f + df);
        while rr > 0 && rr < 7 && ff > 0 && ff < 7 {
            mask |= 1u64 << (rr * 8 + ff);
            rr += dr; ff += df;
        }
    }
    mask
}

// ── Magic finder ──────────────────────────────────────────────────────────────

fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

fn find_magic(sq: Square, mask: u64, is_rook: bool, rng: &mut u64) -> u64 {
    let bits  = mask.count_ones() as usize;
    let size  = 1usize << bits;
    let shift = (64 - bits) as u32;

    // Enumerate all occupancy subsets and their attacks (Carry-Rippler).
    let mut occs = vec![0u64; size];
    let mut atts = vec![0u64; size];
    let mut occ = 0u64;
    for i in 0..size {
        occs[i] = occ;
        atts[i] = if is_rook {
            slow_rook_attacks(sq, Bitboard(occ)).0
        } else {
            slow_bishop_attacks(sq, Bitboard(occ)).0
        };
        occ = occ.wrapping_sub(mask) & mask;
    }

    loop {
        // Sparse magic candidate.
        let magic = xorshift(rng) & xorshift(rng) & xorshift(rng);
        if (mask.wrapping_mul(magic) >> 56).count_ones() < 6 { continue; }

        let mut used = vec![u64::MAX; size];
        let mut ok = true;
        for i in 0..size {
            let idx = (occs[i].wrapping_mul(magic) >> shift) as usize;
            if used[idx] == u64::MAX {
                used[idx] = atts[i];
            } else if used[idx] != atts[i] {
                ok = false;
                break;
            }
        }
        if ok { return magic; }
    }
}

// ── Magic table ───────────────────────────────────────────────────────────────

struct MagicEntry {
    mask:   u64,
    magic:  u64,
    shift:  u32,
    offset: usize,
}

struct MagicTable {
    entries: Vec<MagicEntry>,
    attacks: Vec<Bitboard>,
}

fn build_table(is_rook: bool) -> MagicTable {
    let mut entries = Vec::with_capacity(64);
    let mut attacks: Vec<Bitboard> = Vec::new();
    let mut rng = 0x123456789ABCDEF0u64;

    for i in 0u8..64 {
        let sq    = Square::new(i);
        let mask  = if is_rook { rook_mask(sq) } else { bishop_mask(sq) };
        let bits  = mask.count_ones() as usize;
        let size  = 1usize << bits;
        let shift = (64 - bits) as u32;
        let magic = find_magic(sq, mask, is_rook, &mut rng);

        let offset = attacks.len();
        attacks.resize(offset + size, Bitboard::EMPTY);

        // Populate table for this magic.
        let mut occ = 0u64;
        loop {
            let idx = (occ.wrapping_mul(magic) >> shift) as usize;
            attacks[offset + idx] = if is_rook {
                slow_rook_attacks(sq, Bitboard(occ))
            } else {
                slow_bishop_attacks(sq, Bitboard(occ))
            };
            occ = occ.wrapping_sub(mask) & mask;
            if occ == 0 { break; }
        }

        entries.push(MagicEntry { mask, magic, shift, offset });
    }

    MagicTable { entries, attacks }
}

static ROOK_TABLE:   OnceLock<MagicTable> = OnceLock::new();
static BISHOP_TABLE: OnceLock<MagicTable> = OnceLock::new();

fn rook_table()   -> &'static MagicTable { ROOK_TABLE.get_or_init(|| build_table(true))  }
fn bishop_table() -> &'static MagicTable { BISHOP_TABLE.get_or_init(|| build_table(false)) }

// ── Public API ────────────────────────────────────────────────────────────────

#[inline]
pub fn rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let t = rook_table();
    let e = &t.entries[sq.index()];
    let idx = ((occ.0 & e.mask).wrapping_mul(e.magic) >> e.shift) as usize;
    t.attacks[e.offset + idx]
}

#[inline]
pub fn bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let t = bishop_table();
    let e = &t.entries[sq.index()];
    let idx = ((occ.0 & e.mask).wrapping_mul(e.magic) >> e.shift) as usize;
    t.attacks[e.offset + idx]
}

#[inline]
pub fn queen_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn verify_all(is_rook: bool) {
        for i in 0u8..64 {
            let sq   = Square::new(i);
            let mask = if is_rook { rook_mask(sq) } else { bishop_mask(sq) };
            let mut occ = 0u64;
            loop {
                let expected = if is_rook {
                    slow_rook_attacks(sq, Bitboard(occ))
                } else {
                    slow_bishop_attacks(sq, Bitboard(occ))
                };
                let got = if is_rook {
                    rook_attacks(sq, Bitboard(occ))
                } else {
                    bishop_attacks(sq, Bitboard(occ))
                };
                assert_eq!(got, expected,
                    "{} sq={sq:?} occ={occ:#018x}",
                    if is_rook { "rook" } else { "bishop" });
                occ = occ.wrapping_sub(mask) & mask;
                if occ == 0 { break; }
            }
        }
    }

    #[test]
    fn rook_magic_matches_slow_all_squares() { verify_all(true); }

    #[test]
    fn bishop_magic_matches_slow_all_squares() { verify_all(false); }

    #[test]
    fn rook_a1_empty_board() {
        let att = rook_attacks(Square::A1, Bitboard::EMPTY);
        assert!(att.contains(Square::H1));
        assert!(att.contains(Square::A8));
        assert!(!att.contains(Square::A1));
    }

    #[test]
    fn rook_blocked() {
        let occ = Bitboard::from_sq(Square::A4);
        let att = rook_attacks(Square::A1, occ);
        assert!(att.contains(Square::A4));   // blocker included
        assert!(!att.contains(Square::A5));  // behind blocker: excluded
    }

    #[test]
    fn bishop_d4_diagonals() {
        let att = bishop_attacks(Square::D4, Bitboard::EMPTY);
        assert!(att.contains(Square::H8));
        assert!(att.contains(Square::A1));
        assert!(att.contains(Square::G7));
    }

    #[test]
    fn queen_combines_rook_and_bishop() {
        let occ = Bitboard::EMPTY;
        assert_eq!(
            queen_attacks(Square::D4, occ),
            rook_attacks(Square::D4, occ) | bishop_attacks(Square::D4, occ)
        );
    }
}
