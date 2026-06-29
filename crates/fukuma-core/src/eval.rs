//! Static evaluation: material + piece-square tables with tapered eval.
//!
//! Score is in centipawns from the perspective of the side to move.
//! Positive = advantage for the side to move.

use crate::position::Position;
use crate::types::{Color, PieceType};

// ── Material values (mg, eg) ──────────────────────────────────────────────────

const PIECE_VALUE: [(i32, i32); 6] = [
    (82,  94),   // Pawn
    (337, 281),  // Knight
    (365, 297),  // Bishop
    (477, 512),  // Rook
    (1025, 936), // Queen
    (0,   0),    // King (handled by checkmate logic)
];

// ── Piece-square tables (from White's perspective, a1=index 0) ────────────────
// Source: PeSTO evaluation tables (public domain, widely used).

#[rustfmt::skip]
const PST_MG: [[i32; 64]; 6] = [
    // Pawn MG
    [ 0, 0, 0, 0, 0, 0, 0, 0,
     98,134,61,95,68,126,34,-11,
     -6, 7, 26, 31, 65, 56, 25,-20,
    -14, 13, 6, 21, 23, 12, 17,-23,
    -27, -2, -5, 12, 17, 6, 10,-25,
    -26, -4, -4,-10, 3, 3, 33,-12,
    -35, -1,-20,-23,-15, 24, 38,-22,
      0, 0, 0, 0, 0, 0, 0, 0 ],
    // Knight MG
    [-167,-89,-34,-49, 61,-97,-15,-107,
     -73,-41, 72, 36, 23, 62, 7,-17,
     -47, 60, 37, 65, 84,129, 73, 44,
      -9, 17, 19, 53, 37, 69, 18, 22,
     -13, 4, 16, 13, 28, 19, 21, -8,
     -23, -9, 12, 10, 19, 17, 25,-16,
     -29,-53,-12, -3, -1, 18,-14,-19,
    -105,-21,-58,-33,-17,-28,-19,-23],
    // Bishop MG
    [-29, 4,-82,-37,-25,-42, 7,-8,
     -26, 16,-18,-13, 30, 59, 18,-47,
     -16, 37, 43, 40, 35, 50, 37,-2,
      -4, 5, 19, 50, 37, 37, 7,-2,
      -6, 13, 13, 26, 34, 12, 10, 4,
       0, 15, 15, 15, 14, 27, 18, 10,
       4, 15, 16, 0, 7, 21, 33, 1,
     -33,-3,-14,-21,-13,-12,-39,-21],
    // Rook MG
    [ 32, 42, 32, 51, 63, 9, 31, 43,
      27, 32, 58, 62, 80, 67, 26, 44,
      -5, 19, 26, 36, 17, 45, 61, 16,
     -24,-11, 7, 26, 24, 35,-8,-20,
     -36,-26,-12,-1, 9,-7, 6,-23,
     -45,-25,-16,-17, 3, 0, -5,-33,
     -44,-16,-20, -9,-1, 11,-6,-71,
     -19,-13, 1, 17, 16, 7,-37,-26],
    // Queen MG
    [-28, 0, 29, 12, 59, 44, 43, 45,
     -24,-39, -5, 1,-16, 57, 28, 54,
     -13,-17, 7, 8, 29, 56, 47, 57,
     -27,-27,-16,-16,-1, 17,-2, 1,
      -9,-26, -9,-10,-2,-4, 3,-3,
     -14, 2,-11,-2,-5, 2, 14, 5,
     -35,-8, 11, 2, 8, 15,-3, 1,
      -1,-18,-9, 10,-15,-25,-31,-50],
    // King MG
    [-65, 23, 16,-15,-56,-34, 2, 13,
      29, -1,-20, -7, -8, -4,-38,-29,
      -9, 24, 2,-16,-20, 6, 22,-22,
     -17,-20,-12,-27,-30,-25,-14,-36,
     -49, -1,-27,-39,-46,-44,-33,-51,
     -14,-14,-22,-46,-44,-30,-15,-27,
       1, 7,-8,-64,-43,-16, 9, 8,
     -15, 36, 12,-54, 8,-28, 24, 14],
];

#[rustfmt::skip]
const PST_EG: [[i32; 64]; 6] = [
    // Pawn EG
    [ 0, 0, 0, 0, 0, 0, 0, 0,
    178,173,158,134,147,132,165,187,
     94,100, 85, 67, 56, 53, 82, 84,
     32, 24, 13, 5,-2, 4, 17, 17,
     13, 9,-3,-7,-7,-8, 3,-1,
      4, 7,-6, 1, 0,-5,-1,-8,
     13, 8, 8, 10, 13, 0, 2,-7,
      0, 0, 0, 0, 0, 0, 0, 0],
    // Knight EG
    [-58,-38,-13,-28,-31,-27,-63,-99,
     -25, -8,-25, -2,-9,-25,-24,-52,
     -24,-20, 10, 9,-1,-9,-19,-41,
     -17, 3, 22, 22, 22, 11, 8,-18,
     -18, -6, 16, 25, 16, 17, 4,-18,
     -23, -3, -1, 15, 10,-3,-20,-22,
     -42,-20,-10,-5,-2,-20,-23,-44,
     -29,-51,-23,-15,-22,-18,-50,-64],
    // Bishop EG
    [-14,-21,-11,-8,-7,-9,-17,-24,
      -8, -4, 7,-12,-3,-13,-4,-14,
       2,-8, 0,-1,-2, 6, 0, 4,
      -3, 9, 12, 9, 14, 10, 3, 2,
      -6, 3, 13, 19, 7, 10,-3,-9,
     -12, -3, 8, 10, 13, 3,-7,-15,
     -14,-18,-7,-1, 4,-9,-15,-27,
     -23, -9,-23, -5,-9,-16,-5,-17],
    // Rook EG
    [13, 10, 18, 15, 12,-5,-2,-10,
     10, 20, 35, 22, 28, 16,-4,-6,
     -2, 8,-1, 5,-13,-4,-8,-3,
      0, -5,-1,-6,-8,-2, 5,-10,
     -3, -8,-6,-4,-6,-3,-5,-10,
     -8,-7,-6,-5,-3,-5,-13,-8,
     -9, 7,-3,-3,-2,-3,-4,-5,
      3, -5,-5, 4, 4,-5,-6, 3],
    // Queen EG
    [-9, 22, 22, 27, 27, 19, 10, 20,
     -17, 20, 32, 41, 58, 25, 30, 0,
     -20, 6, 9, 49, 47, 35, 19, 9,
       3, 22, 24, 45, 57, 40, 57, 36,
     -18, 28, 19, 47, 31, 34, 39, 23,
     -16,-27, 15, 6, 9, 17, 10, 5,
     -22,-23,-30,-16,-16,-23,-36,-32,
     -33,-28,-22,-43, -5,-32,-20,-41],
    // King EG
    [-74,-35,-18,-18,-11, 15, 4,-17,
     -12, 17, 14, 17, 17, 38, 23, 11,
      10, 17, 23, 15, 20, 45, 44, 13,
      -8, 22, 24, 27, 26, 33, 26, 3,
     -18, -4, 21, 24, 27, 23, 9,-11,
     -19, -3, 11, 21, 23, 16, 7,-9,
     -27,-11, 4, 13, 14, 4,-5,-17,
     -53,-34,-21,-11,-28,-14,-24,-43],
];

// ── Game phase ────────────────────────────────────────────────────────────────

const PHASE_INC: [i32; 6] = [0, 1, 1, 2, 4, 0];
const MAX_PHASE: i32 = 24;

fn game_phase(pos: &Position) -> i32 {
    let mut phase = 0;
    for kind in PieceType::ALL {
        let count = (pos.bb_piece[kind as usize]).popcount() as i32;
        phase += PHASE_INC[kind as usize] * count;
    }
    phase.min(MAX_PHASE)
}

// ── Evaluation ────────────────────────────────────────────────────────────────

/// Mirrors a square index for Black (so PST stays in White's orientation).
#[inline]
fn mirror(sq: usize) -> usize { sq ^ 56 }

pub fn evaluate(pos: &Position) -> i32 {
    let phase = game_phase(pos);
    let mut mg = [0i32; 2];
    let mut eg = [0i32; 2];

    for color in [Color::White, Color::Black] {
        for kind in PieceType::ALL {
            let (mv, ev) = PIECE_VALUE[kind as usize];
            for sq in pos.piece_bb(color, kind) {
                let idx = if color == Color::White { sq.index() } else { mirror(sq.index()) };
                mg[color as usize] += mv + PST_MG[kind as usize][idx];
                eg[color as usize] += ev + PST_EG[kind as usize][idx];
            }
        }
    }

    let mg_score = mg[0] - mg[1];
    let eg_score = eg[0] - eg[1];
    // Tapered eval: interpolate between MG and EG.
    let score = (mg_score * phase + eg_score * (MAX_PHASE - phase)) / MAX_PHASE;

    // Return from the perspective of the side to move.
    if pos.side_to_move == Color::White { score } else { -score }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn startpos_is_near_zero() {
        let score = evaluate(&Position::startpos());
        // Should be small (PeSTO gives a small advantage for white at startpos).
        assert!(score.abs() < 50, "startpos score {score} should be near zero");
    }

    #[test]
    fn material_advantage_gives_positive_score() {
        // White queen vs nothing extra: white side to move should be positive.
        let fen = "4k3/8/8/8/8/8/8/4KQ2 w - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert!(evaluate(&pos) > 0, "white with a queen should be positive");
    }

    #[test]
    fn symmetry_flipped_position() {
        // Horizontally mirrored position should give opposite sign.
        let w_fen = "4k3/8/8/8/8/8/8/4KR2 w - - 0 1";
        let b_fen = "4kr2/8/8/8/8/8/8/4K3 b - - 0 1";
        let w_score = evaluate(&Position::from_fen(w_fen).unwrap());
        let b_score = evaluate(&Position::from_fen(b_fen).unwrap());
        assert_eq!(w_score, b_score,
            "symmetric positions should give equal scores from side-to-move perspective");
    }

    #[test]
    fn evaluate_side_to_move_perspective() {
        // Same material imbalance: if it's black to move, score should be negative
        // (black is down material relative to white).
        let fen = "4k3/8/8/8/8/8/8/4KQ2 b - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        assert!(evaluate(&pos) < 0, "black to move with white queen ahead should be negative");
    }
}
