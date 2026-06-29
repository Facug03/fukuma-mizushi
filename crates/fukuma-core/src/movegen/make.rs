use crate::position::{CastlingRights, Position};
use crate::types::{Color, Piece, PieceType, Rank, Square};
use super::moves::Move;

/// Everything needed to fully undo a move.
#[derive(Clone, Copy, Debug)]
pub struct UndoState {
    pub moved_kind: PieceType,       // original piece type (pre-promotion)
    pub captured:   Option<Piece>,
    pub castling:   CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove:   u8,
    pub hash:       u64,             // full hash before the move
}

/// Per-square castling-rights mask: AND this when a piece leaves/arrives at that square.
const CASTLING_MASK: [u8; 64] = {
    let mut m = [0b1111u8; 64];
    m[Square::A1.index()] &= !CastlingRights::WQ.0;
    m[Square::H1.index()] &= !CastlingRights::WK.0;
    m[Square::E1.index()] &= !CastlingRights::WK.0 & !CastlingRights::WQ.0;
    m[Square::A8.index()] &= !CastlingRights::BQ.0;
    m[Square::H8.index()] &= !CastlingRights::BK.0;
    m[Square::E8.index()] &= !CastlingRights::BK.0 & !CastlingRights::BQ.0;
    m
};

impl Position {
    pub fn make_move(&mut self, mv: Move) -> UndoState {
        let us   = self.side_to_move;
        let them = us.flip();
        let from = mv.from();
        let to   = mv.to();

        let moving = self.piece_at(from).expect("make_move: empty from-square");
        let undo = UndoState {
            moved_kind: moving.kind,
            captured:   None,
            castling:   self.castling,
            en_passant: self.en_passant,
            halfmove:   self.halfmove_clock,
            hash:       self.hash,
        };

        // XOR out old castling and EP keys before modifying.
        self.hash ^= crate::zobrist::castling_key(self.castling.0);
        if let Some(ep) = self.en_passant { self.hash ^= crate::zobrist::ep_key(ep); }

        self.remove_piece(from);

        let captured = if mv.is_ep() {
            let cap_sq = Square::from_file_rank(to.file(), from.rank());
            let cap    = self.piece_at(cap_sq);
            self.remove_piece(cap_sq);
            cap
        } else if mv.is_capture() {
            let cap = self.piece_at(to);
            self.remove_piece(to);
            cap
        } else {
            None
        };

        let placed = if mv.is_promo() { Piece::new(us, mv.promo_piece()) } else { moving };
        self.put(to, placed);

        if mv.is_castle() {
            let (rf, rt) = castle_rook_squares(us, mv.flags());
            self.remove_piece(rf);
            self.put(rt, Piece::new(us, PieceType::Rook));
        }

        self.castling = CastlingRights(
            self.castling.0 & CASTLING_MASK[from.index()] & CASTLING_MASK[to.index()]
        );

        self.en_passant = if mv.flags() == Move::DOUBLE_PUSH {
            let ep_rank = if us == Color::White { Rank::R3 } else { Rank::R6 };
            Some(Square::from_file_rank(from.file(), ep_rank))
        } else {
            None
        };

        self.halfmove_clock = if mv.is_capture() || moving.kind == PieceType::Pawn {
            0
        } else {
            self.halfmove_clock + 1
        };

        if us == Color::Black { self.fullmove_number += 1; }
        self.side_to_move = them;

        // XOR in new castling, EP, and side-to-move keys.
        self.hash ^= crate::zobrist::castling_key(self.castling.0);
        if let Some(ep) = self.en_passant { self.hash ^= crate::zobrist::ep_key(ep); }
        self.hash ^= crate::zobrist::side_key();

        UndoState { captured, ..undo }
    }

    pub fn unmake_move(&mut self, mv: Move, undo: UndoState) {
        self.side_to_move   = self.side_to_move.flip();
        self.castling       = undo.castling;
        self.en_passant     = undo.en_passant;
        self.halfmove_clock = undo.halfmove;
        if self.side_to_move == Color::Black { self.fullmove_number -= 1; }

        let us   = self.side_to_move;
        let from = mv.from();
        let to   = mv.to();

        self.remove_piece(to);
        self.put(from, Piece::new(us, undo.moved_kind));

        if mv.is_ep() {
            let cap_sq = Square::from_file_rank(to.file(), from.rank());
            if let Some(cap) = undo.captured { self.put(cap_sq, cap); }
        } else if let Some(cap) = undo.captured {
            self.put(to, cap);
        }

        if mv.is_castle() {
            let (rf, rt) = castle_rook_squares(us, mv.flags());
            self.remove_piece(rt);
            self.put(rf, Piece::new(us, PieceType::Rook));
        }
        // Restore hash directly — simpler than reversing all XOR operations.
        self.hash = undo.hash;
    }
}

#[inline]
fn castle_rook_squares(color: Color, flags: u16) -> (Square, Square) {
    match (color, flags) {
        (Color::White, Move::CASTLE_K) => (Square::H1, Square::F1),
        (Color::White, Move::CASTLE_Q) => (Square::A1, Square::D1),
        (Color::Black, Move::CASTLE_K) => (Square::H8, Square::F8),
        _                              => (Square::A8, Square::D8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn make_unmake_restores_startpos() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);
        let undo = pos.make_move(mv);
        pos.unmake_move(mv, undo);
        assert_eq!(pos.to_fen(), Position::STARTPOS);
    }

    #[test]
    fn make_unmake_capture_restores() {
        // Scholar's mate setup: 1.e4 e5 2.Qh5 Nc6 3.Bc4 -- white queen captures f7
        let fen = "r1bqkbnr/pppp1ppp/2n5/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 2 3";
        let mut pos = Position::from_fen(fen).unwrap();
        let mv = Move::new(Square::H5, Square::F7, Move::CAPTURE);
        let undo = pos.make_move(mv);
        pos.unmake_move(mv, undo);
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn make_unmake_castle_kingside_white() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        let mv = Move::new(Square::E1, Square::G1, Move::CASTLE_K);
        let undo = pos.make_move(mv);
        // After castling: king on g1, rook on f1
        assert_eq!(pos.piece_at(Square::G1).map(|p| p.kind), Some(PieceType::King));
        assert_eq!(pos.piece_at(Square::F1).map(|p| p.kind), Some(PieceType::Rook));
        pos.unmake_move(mv, undo);
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn make_unmake_en_passant() {
        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
        let mut pos = Position::from_fen(fen).unwrap();
        let mv = Move::new(Square::E5, Square::D6, Move::EP_CAPTURE);
        let undo = pos.make_move(mv);
        // Black pawn on d5 must be gone
        assert!(pos.piece_at(Square::D5).is_none());
        pos.unmake_move(mv, undo);
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn make_unmake_promotion() {
        let fen = "8/P7/8/8/8/8/8/4K1k1 w - - 0 1";
        let mut pos = Position::from_fen(fen).unwrap();
        let mv = Move::new(Square::A7, Square::A8, Move::PROMO_Q);
        let undo = pos.make_move(mv);
        assert_eq!(pos.piece_at(Square::A8).map(|p| p.kind), Some(PieceType::Queen));
        pos.unmake_move(mv, undo);
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn halfmove_clock_resets_on_pawn_or_capture() {
        let mut pos = Position::startpos();
        let mv = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);
        pos.make_move(mv);
        assert_eq!(pos.halfmove_clock, 0);
    }

    #[test]
    fn fullmove_increments_after_black() {
        let mut pos = Position::startpos();
        pos.make_move(Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH));
        pos.make_move(Move::new(Square::E7, Square::E5, Move::DOUBLE_PUSH));
        assert_eq!(pos.fullmove_number, 2);
    }
}
