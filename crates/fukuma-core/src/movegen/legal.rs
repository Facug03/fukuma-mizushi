//! Legal move generation.
//!
//! Strategy: generate pseudo-legal moves, then filter by legality (king not in
//! check after the move).  For king moves we also pre-filter attacked squares.

use super::moves::Move;
use crate::attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, queen_attacks, rook_attacks,
};
use crate::bitboard::Bitboard;
use crate::position::{CastlingRights, Position};
use crate::types::{Color, PieceType, Rank, Square};

// ── Public entry point ────────────────────────────────────────────────────────

/// Returns all legal moves for the side to move.
pub fn legal_moves(pos: &Position) -> Vec<Move> {
    let mut moves = Vec::with_capacity(40);
    gen_all(pos, &mut moves);
    moves
}

/// Returns the number of leaf nodes at depth `depth` (for testing movegen).
pub fn perft(pos: &mut Position, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = legal_moves(pos);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut nodes = 0u64;
    for mv in moves {
        let undo = pos.make_move(mv);
        nodes += perft(pos, depth - 1);
        pos.unmake_move(mv, undo);
    }
    nodes
}

// ── Internal generation ───────────────────────────────────────────────────────

fn gen_all(pos: &Position, out: &mut Vec<Move>) {
    let us = pos.side_to_move;
    let them = us.flip();
    let occ = pos.occupancy();
    let ours = pos.bb_color[us as usize];
    let theirs = pos.bb_color[them as usize];

    gen_pawns(pos, us, occ, theirs, out);
    gen_piece(pos, us, PieceType::Knight, occ, ours, theirs, out);
    gen_piece(pos, us, PieceType::Bishop, occ, ours, theirs, out);
    gen_piece(pos, us, PieceType::Rook, occ, ours, theirs, out);
    gen_piece(pos, us, PieceType::Queen, occ, ours, theirs, out);
    gen_king(pos, us, occ, ours, theirs, out);
    gen_castling(pos, us, occ, out);

    // Filter: keep only moves that leave king not in check.
    out.retain(|&mv| {
        let mut p = pos.clone();
        let undo = p.make_move(mv);
        let king = p.king_sq(us);
        let legal = !is_attacked(&p, king, them);
        p.unmake_move(mv, undo);
        legal
    });
}

fn gen_pawns(pos: &Position, us: Color, occ: Bitboard, theirs: Bitboard, out: &mut Vec<Move>) {
    let pawns = pos.piece_bb(us, PieceType::Pawn);
    let promo_rank = if us == Color::White {
        Rank::R8
    } else {
        Rank::R1
    };

    for sq in pawns {
        // Pushes
        let push1 = (if us == Color::White {
            Bitboard::from_sq(sq).north()
        } else {
            Bitboard::from_sq(sq).south()
        }) & !occ;
        let on_start = if us == Color::White {
            sq.rank() == Rank::R2
        } else {
            sq.rank() == Rank::R7
        };
        let push2 = if on_start {
            (if us == Color::White {
                push1.north()
            } else {
                push1.south()
            }) & !occ
        } else {
            Bitboard::EMPTY
        };

        for to in push1 {
            add_pawn_move(sq, to, to.rank() == promo_rank, false, out);
        }
        for to in push2 {
            out.push(Move::new(sq, to, Move::DOUBLE_PUSH));
        }

        // Captures
        let atk = pawn_attacks(sq, us) & theirs;
        for to in atk {
            add_pawn_move(sq, to, to.rank() == promo_rank, true, out);
        }

        // En passant
        if let Some(ep) = pos.en_passant {
            if pawn_attacks(sq, us).contains(ep) {
                out.push(Move::new(sq, ep, Move::EP_CAPTURE));
            }
        }
    }
}

fn add_pawn_move(from: Square, to: Square, promo: bool, capture: bool, out: &mut Vec<Move>) {
    if promo {
        let bases = if capture {
            [
                Move::PROMO_CAP_N,
                Move::PROMO_CAP_B,
                Move::PROMO_CAP_R,
                Move::PROMO_CAP_Q,
            ]
        } else {
            [Move::PROMO_N, Move::PROMO_B, Move::PROMO_R, Move::PROMO_Q]
        };
        for f in bases {
            out.push(Move::new(from, to, f));
        }
    } else {
        out.push(Move::new(
            from,
            to,
            if capture { Move::CAPTURE } else { Move::QUIET },
        ));
    }
}

fn gen_piece(
    pos: &Position,
    us: Color,
    kind: PieceType,
    occ: Bitboard,
    ours: Bitboard,
    theirs: Bitboard,
    out: &mut Vec<Move>,
) {
    for sq in pos.piece_bb(us, kind) {
        let att = piece_attacks(kind, sq, occ) & !ours;
        for to in att & !theirs {
            out.push(Move::new(sq, to, Move::QUIET));
        }
        for to in att & theirs {
            out.push(Move::new(sq, to, Move::CAPTURE));
        }
    }
}

fn gen_king(
    pos: &Position,
    us: Color,
    _occ: Bitboard,
    ours: Bitboard,
    theirs: Bitboard,
    out: &mut Vec<Move>,
) {
    let sq = pos.king_sq(us);
    let att = king_attacks(sq) & !ours;
    for to in att & !theirs {
        out.push(Move::new(sq, to, Move::QUIET));
    }
    for to in att & theirs {
        out.push(Move::new(sq, to, Move::CAPTURE));
    }
}

fn gen_castling(pos: &Position, us: Color, occ: Bitboard, out: &mut Vec<Move>) {
    let them = us.flip();
    let (king_sq, ks_rights, qs_rights, ks_between, qs_between, qs_no_attack, ks_to, qs_to) =
        if us == Color::White {
            (
                Square::E1,
                CastlingRights::WK,
                CastlingRights::WQ,
                sq_bb(Square::F1) | sq_bb(Square::G1),
                sq_bb(Square::B1) | sq_bb(Square::C1) | sq_bb(Square::D1),
                sq_bb(Square::C1) | sq_bb(Square::D1),
                Square::G1,
                Square::C1,
            )
        } else {
            (
                Square::E8,
                CastlingRights::BK,
                CastlingRights::BQ,
                sq_bb(Square::F8) | sq_bb(Square::G8),
                sq_bb(Square::B8) | sq_bb(Square::C8) | sq_bb(Square::D8),
                sq_bb(Square::C8) | sq_bb(Square::D8),
                Square::G8,
                Square::C8,
            )
        };

    if pos.castling.has(ks_rights)
        && (occ & ks_between).is_empty()
        && !is_attacked(pos, king_sq, them)
        && !squares_attacked(pos, ks_between, them)
    {
        out.push(Move::new(king_sq, ks_to, Move::CASTLE_K));
    }

    if pos.castling.has(qs_rights)
        && (occ & qs_between).is_empty()
        && !is_attacked(pos, king_sq, them)
        && !squares_attacked(pos, qs_no_attack, them)
    {
        out.push(Move::new(king_sq, qs_to, Move::CASTLE_Q));
    }
}

// ── Attack helpers ────────────────────────────────────────────────────────────

pub fn is_attacked(pos: &Position, sq: Square, by: Color) -> bool {
    let occ = pos.occupancy();
    if !(knight_attacks(sq) & pos.piece_bb(by, PieceType::Knight)).is_empty() {
        return true;
    }
    if !(king_attacks(sq) & pos.piece_bb(by, PieceType::King)).is_empty() {
        return true;
    }
    if !(pawn_attacks(sq, by.flip()) & pos.piece_bb(by, PieceType::Pawn)).is_empty() {
        return true;
    }
    if !(rook_attacks(sq, occ)
        & (pos.piece_bb(by, PieceType::Rook) | pos.piece_bb(by, PieceType::Queen)))
    .is_empty()
    {
        return true;
    }
    if !(bishop_attacks(sq, occ)
        & (pos.piece_bb(by, PieceType::Bishop) | pos.piece_bb(by, PieceType::Queen)))
    .is_empty()
    {
        return true;
    }
    false
}

fn squares_attacked(pos: &Position, sqs: Bitboard, by: Color) -> bool {
    sqs.into_iter().any(|sq| is_attacked(pos, sq, by))
}

fn sq_bb(sq: Square) -> Bitboard {
    Bitboard::from_sq(sq)
}

fn piece_attacks(kind: PieceType, sq: Square, occ: Bitboard) -> Bitboard {
    match kind {
        PieceType::Knight => knight_attacks(sq),
        PieceType::Bishop => bishop_attacks(sq, occ),
        PieceType::Rook => rook_attacks(sq, occ),
        PieceType::Queen => queen_attacks(sq, occ),
        _ => Bitboard::EMPTY,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    fn perft_pos(fen: &str, depth: u8) -> u64 {
        perft(&mut Position::from_fen(fen).unwrap(), depth)
    }

    // Standard perft reference values: https://www.chessprogramming.org/Perft_Results
    #[test]
    fn perft_startpos() {
        assert_eq!(perft_pos(Position::STARTPOS, 1), 20);
        assert_eq!(perft_pos(Position::STARTPOS, 2), 400);
        assert_eq!(perft_pos(Position::STARTPOS, 3), 8_902);
        assert_eq!(perft_pos(Position::STARTPOS, 4), 197_281);
    }

    #[test]
    fn perft_kiwipete_depth1() {
        // Position 2: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        assert_eq!(perft_pos(fen, 1), 48);
        assert_eq!(perft_pos(fen, 2), 2_039);
    }

    #[test]
    fn perft_position3() {
        // Position 3: 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        assert_eq!(perft_pos(fen, 1), 14);
        assert_eq!(perft_pos(fen, 2), 191);
        assert_eq!(perft_pos(fen, 3), 2_812);
    }

    #[test]
    fn perft_position5_depth1() {
        // CPW Position 5: rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        assert_eq!(perft_pos(fen, 1), 44);
    }

    #[test]
    fn startpos_legal_count_is_20() {
        let pos = Position::startpos();
        assert_eq!(legal_moves(&pos).len(), 20);
    }

    #[test]
    fn no_moves_in_checkmate() {
        // Fool's mate — black is checkmated after Qh4#
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        assert_eq!(legal_moves(&Position::from_fen(fen).unwrap()).len(), 0);
    }
}
