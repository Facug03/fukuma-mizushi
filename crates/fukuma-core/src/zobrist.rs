//! Zobrist hashing for positions.
//!
//! Hash is computed incrementally: the Position carries a `hash` field
//! that is updated in make_move/unmake_move.

use crate::types::{Color, PieceType, Square};

// ── Random keys ───────────────────────────────────────────────────────────────

/// XOR-shift PRNG seeded deterministically.
const fn xorshift(mut s: u64) -> u64 {
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    s
}

const fn gen_keys() -> ZobristKeys {
    let mut s = 0xDEAD_BEEF_CAFE_1234u64;
    let mut pieces = [[[0u64; 64]; 6]; 2];
    let mut c = 0usize;
    while c < 2 {
        let mut p = 0usize;
        while p < 6 {
            let mut sq = 0usize;
            while sq < 64 {
                s = xorshift(s);
                pieces[c][p][sq] = s;
                sq += 1;
            }
            p += 1;
        }
        c += 1;
    }

    let mut castling = [0u64; 16];
    let mut i = 0usize;
    while i < 16 {
        s = xorshift(s);
        castling[i] = s;
        i += 1;
    }

    let mut en_passant = [0u64; 8];
    let mut f = 0usize;
    while f < 8 {
        s = xorshift(s);
        en_passant[f] = s;
        f += 1;
    }

    s = xorshift(s);
    ZobristKeys {
        pieces,
        castling,
        en_passant,
        side_to_move: s,
    }
}

struct ZobristKeys {
    pieces: [[[u64; 64]; 6]; 2],
    castling: [u64; 16],
    en_passant: [u64; 8],
    side_to_move: u64,
}

static KEYS: ZobristKeys = gen_keys();

// ── Public helpers ────────────────────────────────────────────────────────────

#[inline]
pub fn piece_key(color: Color, kind: PieceType, sq: Square) -> u64 {
    KEYS.pieces[color as usize][kind as usize][sq.index()]
}

#[inline]
pub fn castling_key(rights: u8) -> u64 {
    KEYS.castling[rights as usize & 0xF]
}

#[inline]
pub fn ep_key(sq: Square) -> u64 {
    KEYS.en_passant[sq.file().0 as usize]
}

#[inline]
pub fn side_key() -> u64 {
    KEYS.side_to_move
}

/// Compute a full hash from scratch (used for verification).
pub fn hash_from_scratch(pos: &crate::position::Position) -> u64 {
    let mut h = 0u64;
    for color in [Color::White, Color::Black] {
        for kind in crate::types::PieceType::ALL {
            for sq in pos.piece_bb(color, kind) {
                h ^= piece_key(color, kind, sq);
            }
        }
    }
    h ^= castling_key(pos.castling.0);
    if let Some(ep) = pos.en_passant {
        h ^= ep_key(ep);
    }
    if pos.side_to_move == Color::Black {
        h ^= side_key();
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::Move;
    use crate::position::Position;

    #[test]
    fn keys_are_unique() {
        // Spot-check: no two piece keys are equal.
        let k1 = piece_key(Color::White, PieceType::Pawn, Square::new(0));
        let k2 = piece_key(Color::White, PieceType::Pawn, Square::new(1));
        let k3 = piece_key(Color::Black, PieceType::Pawn, Square::new(0));
        assert_ne!(k1, k2);
        assert_ne!(k1, k3);
        assert_ne!(k2, k3);
    }

    #[test]
    fn incremental_hash_matches_from_scratch() {
        let mut pos = Position::startpos();
        // Play a few moves and verify incremental == scratch.
        for mv in [
            Move::new(Square::new(12), Square::new(28), Move::DOUBLE_PUSH), // e2e4
            Move::new(Square::new(52), Square::new(36), Move::DOUBLE_PUSH), // e7e5
            Move::new(Square::new(6), Square::new(21), Move::QUIET),        // g1f3
        ] {
            pos.make_move(mv);
            let incremental = pos.hash;
            let scratch = hash_from_scratch(&pos);
            assert_eq!(incremental, scratch, "incremental hash mismatch after {mv}");
        }
    }

    #[test]
    fn unmake_restores_hash() {
        let mut pos = Position::startpos();
        let original_hash = pos.hash;
        let mv = Move::new(Square::new(12), Square::new(28), Move::DOUBLE_PUSH);
        let undo = pos.make_move(mv);
        assert_ne!(pos.hash, original_hash);
        pos.unmake_move(mv, undo);
        assert_eq!(pos.hash, original_hash);
    }
}
