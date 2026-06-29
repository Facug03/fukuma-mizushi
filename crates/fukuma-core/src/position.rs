use crate::bitboard::Bitboard;
use crate::types::{Color, File, Piece, PieceType, Rank, Square};

// ── Castling rights ───────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
    pub const NONE: Self = Self(0b0000);
    pub const WK:   Self = Self(0b0001);
    pub const WQ:   Self = Self(0b0010);
    pub const BK:   Self = Self(0b0100);
    pub const BQ:   Self = Self(0b1000);
    pub const ALL:  Self = Self(0b1111);

    #[inline] pub fn has(self, r: Self) -> bool { self.0 & r.0 != 0 }
    #[inline] pub fn remove(self, r: Self) -> Self { Self(self.0 & !r.0) }
}

impl std::ops::BitOrAssign for CastlingRights {
    fn bitor_assign(&mut self, r: Self) { self.0 |= r.0; }
}

// ── Position ──────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Position {
    /// All pieces of each type (both colors).
    pub bb_piece: [Bitboard; 6],
    /// All pieces of each color.
    pub bb_color: [Bitboard; 2],
    pub side_to_move: Color,
    pub castling: CastlingRights,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Position {
    pub const STARTPOS: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub fn startpos() -> Self { Self::from_fen(Self::STARTPOS).unwrap() }

    #[inline]
    pub fn piece_bb(&self, color: Color, kind: PieceType) -> Bitboard {
        self.bb_color[color as usize] & self.bb_piece[kind as usize]
    }

    #[inline]
    pub fn occupancy(&self) -> Bitboard { self.bb_color[0] | self.bb_color[1] }

    #[inline]
    pub fn king_sq(&self, color: Color) -> Square {
        self.piece_bb(color, PieceType::King).lsb()
    }

    pub fn piece_at(&self, sq: Square) -> Option<Piece> {
        let color = if self.bb_color[0].contains(sq) { Color::White }
                    else if self.bb_color[1].contains(sq) { Color::Black }
                    else { return None };
        for kind in PieceType::ALL {
            if self.bb_piece[kind as usize].contains(sq) {
                return Some(Piece::new(color, kind));
            }
        }
        None
    }

    #[inline]
    pub(crate) fn put(&mut self, sq: Square, piece: Piece) {
        let bb = Bitboard::from_sq(sq);
        self.bb_piece[piece.kind as usize] |= bb;
        self.bb_color[piece.color as usize] |= bb;
    }

    #[inline]
    pub(crate) fn remove_piece(&mut self, sq: Square) {
        let mask = !Bitboard::from_sq(sq);
        for b in &mut self.bb_piece { *b &= mask; }
        for b in &mut self.bb_color { *b &= mask; }
    }

    pub fn render_ascii(&self) -> String {
        let mut s = String::with_capacity(128);
        for rank in (0..8u8).rev() {
            s.push((b'1' + rank) as char);
            s.push(' ');
            for file in 0..8u8 {
                let sq = Square::from_file_rank(File(file), Rank(rank));
                s.push(self.piece_at(sq).map(piece_to_char).unwrap_or('.'));
                s.push(' ');
            }
            s.push('\n');
        }
        s.push_str("  a b c d e f g h");
        s
    }
}

// ── FEN ───────────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq)]
pub enum FenError {
    InvalidPiecePlacement,
    InvalidSideToMove,
    InvalidCastling,
    InvalidEnPassant,
    InvalidHalfmove,
    InvalidFullmove,
    TooFewParts,
}

impl std::fmt::Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for FenError {}

impl Position {
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut parts = fen.split_ascii_whitespace();
        let placement = parts.next().ok_or(FenError::TooFewParts)?;
        let stm       = parts.next().ok_or(FenError::TooFewParts)?;
        let castling  = parts.next().ok_or(FenError::TooFewParts)?;
        let ep        = parts.next().ok_or(FenError::TooFewParts)?;
        let halfmove  = parts.next().unwrap_or("0");
        let fullmove  = parts.next().unwrap_or("1");

        let mut pos = Self {
            bb_piece:       [Bitboard::EMPTY; 6],
            bb_color:       [Bitboard::EMPTY; 2],
            side_to_move:   Color::White,
            castling:       CastlingRights::NONE,
            en_passant:     None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        // Piece placement — FEN starts at a8 (sq=56), ranks go down.
        let mut sq = 56i8;
        for ch in placement.chars() {
            match ch {
                '/' => sq -= 16,
                '1'..='8' => sq += (ch as u8 - b'0') as i8,
                _ => {
                    let (color, kind) = piece_from_char(ch)
                        .ok_or(FenError::InvalidPiecePlacement)?;
                    pos.put(Square::new(sq as u8), Piece::new(color, kind));
                    sq += 1;
                }
            }
        }

        pos.side_to_move = match stm {
            "w" => Color::White,
            "b" => Color::Black,
            _   => return Err(FenError::InvalidSideToMove),
        };

        if castling != "-" {
            for ch in castling.chars() {
                pos.castling |= match ch {
                    'K' => CastlingRights::WK,
                    'Q' => CastlingRights::WQ,
                    'k' => CastlingRights::BK,
                    'q' => CastlingRights::BQ,
                    _   => return Err(FenError::InvalidCastling),
                };
            }
        }

        if ep != "-" {
            let b = ep.as_bytes();
            if b.len() < 2 { return Err(FenError::InvalidEnPassant); }
            let file = b[0].checked_sub(b'a').filter(|&f| f < 8).ok_or(FenError::InvalidEnPassant)?;
            let rank = b[1].checked_sub(b'1').filter(|&r| r < 8).ok_or(FenError::InvalidEnPassant)?;
            pos.en_passant = Some(Square::from_file_rank(File(file), Rank(rank)));
        }

        pos.halfmove_clock  = halfmove.parse().map_err(|_| FenError::InvalidHalfmove)?;
        pos.fullmove_number = fullmove.parse().map_err(|_| FenError::InvalidFullmove)?;
        Ok(pos)
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::with_capacity(90);

        for rank in (0..8u8).rev() {
            let mut empty = 0u8;
            for file in 0..8u8 {
                let sq = Square::from_file_rank(File(file), Rank(rank));
                match self.piece_at(sq) {
                    Some(p) => {
                        if empty > 0 { fen.push((b'0' + empty) as char); empty = 0; }
                        fen.push(piece_to_char(p));
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 { fen.push((b'0' + empty) as char); }
            if rank > 0 { fen.push('/'); }
        }

        fen.push(' ');
        fen.push(if self.side_to_move == Color::White { 'w' } else { 'b' });

        fen.push(' ');
        if self.castling == CastlingRights::NONE {
            fen.push('-');
        } else {
            if self.castling.has(CastlingRights::WK) { fen.push('K'); }
            if self.castling.has(CastlingRights::WQ) { fen.push('Q'); }
            if self.castling.has(CastlingRights::BK) { fen.push('k'); }
            if self.castling.has(CastlingRights::BQ) { fen.push('q'); }
        }

        fen.push(' ');
        match self.en_passant {
            Some(sq) => {
                fen.push((b'a' + sq.file().0) as char);
                fen.push((b'1' + sq.rank().0) as char);
            }
            None => fen.push('-'),
        }

        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());
        fen
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn piece_from_char(ch: char) -> Option<(Color, PieceType)> {
    let color = if ch.is_uppercase() { Color::White } else { Color::Black };
    let kind = match ch.to_ascii_lowercase() {
        'p' => PieceType::Pawn,   'n' => PieceType::Knight,
        'b' => PieceType::Bishop, 'r' => PieceType::Rook,
        'q' => PieceType::Queen,  'k' => PieceType::King,
        _   => return None,
    };
    Some((color, kind))
}

pub fn piece_to_char(p: Piece) -> char {
    let ch = match p.kind {
        PieceType::Pawn   => 'p', PieceType::Knight => 'n',
        PieceType::Bishop => 'b', PieceType::Rook   => 'r',
        PieceType::Queen  => 'q', PieceType::King   => 'k',
    };
    if p.color == Color::White { ch.to_ascii_uppercase() } else { ch }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const KIWIPETE: &str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    const EP_FEN: &str =
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

    #[test]
    fn fen_roundtrip_startpos() {
        let pos = Position::startpos();
        assert_eq!(pos.to_fen(), Position::STARTPOS);
    }

    #[test]
    fn fen_roundtrip_kiwipete() {
        let pos = Position::from_fen(KIWIPETE).unwrap();
        assert_eq!(pos.to_fen(), KIWIPETE);
    }

    #[test]
    fn fen_roundtrip_en_passant() {
        let pos = Position::from_fen(EP_FEN).unwrap();
        assert_eq!(pos.to_fen(), EP_FEN);
        assert_eq!(pos.en_passant, Some(Square::E3));
    }

    #[test]
    fn startpos_piece_counts() {
        let pos = Position::startpos();
        assert_eq!(pos.piece_bb(Color::White, PieceType::Pawn).popcount(), 8);
        assert_eq!(pos.piece_bb(Color::Black, PieceType::Pawn).popcount(), 8);
        assert_eq!(pos.occupancy().popcount(), 32);
        assert_eq!(pos.king_sq(Color::White), Square::E1);
        assert_eq!(pos.king_sq(Color::Black), Square::E8);
    }

    #[test]
    fn invalid_stm_errors() {
        assert_eq!(
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1"),
            Err(FenError::InvalidSideToMove)
        );
    }

    #[test]
    fn render_ascii_has_correct_lines() {
        let s = Position::startpos().render_ascii();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 9);
        assert!(lines[0].starts_with('8'));
        assert!(lines[7].starts_with('1'));
        assert_eq!(lines[8], "  a b c d e f g h");
    }
}
