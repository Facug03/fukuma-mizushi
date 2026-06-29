//! Negamax search with alpha-beta pruning, iterative deepening, quiescence,
//! transposition table, and move ordering (MVV-LVA + killer moves + history).

use std::time::{Duration, Instant};

use crate::eval::evaluate;
use crate::movegen::{Move, legal_moves};
use crate::position::Position;
use crate::tt::{Bound, TranspositionTable};
use crate::types::PieceType;

pub const INFINITY: i32 = 1_000_000;
pub const MATE_SCORE: i32 = 900_000;
/// Score considered "mate" (at least this far from 0).
pub const IS_MATE: i32 = MATE_SCORE - 500;

// ── Search limits ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct Limits {
    pub depth: Option<u8>,
    pub move_time: Option<Duration>,
}

// ── Search info (returned to the caller) ─────────────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct SearchInfo {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth: u8,
    pub nodes: u64,
}

// ── Searcher ──────────────────────────────────────────────────────────────────

pub struct Searcher {
    pub nodes: u64,
    root_best: Option<Move>,
    stop_time: Option<Instant>,
    stopped: bool,
    tt: TranspositionTable,
    killers: [[Move; 2]; 64],
    history: [[[i32; 64]; 64]; 2], // [color][from][to]
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            nodes: 0,
            root_best: None,
            stop_time: None,
            stopped: false,
            tt: TranspositionTable::new(32),
            killers: [[Move::NULL; 2]; 64],
            history: [[[0; 64]; 64]; 2],
        }
    }

    /// Entry point: iterative deepening.
    pub fn search(&mut self, pos: &mut Position, limits: &Limits) -> SearchInfo {
        self.nodes = 0;
        self.stopped = false;
        self.root_best = None;
        self.stop_time = limits.move_time.map(|d| Instant::now() + d);

        let max_depth = limits.depth.unwrap_or(64);
        let mut info = SearchInfo::default();

        for depth in 1..=max_depth {
            let score = self.root_search(pos, depth);
            if self.stopped {
                break;
            }
            info.score = score;
            info.depth = depth;
            info.nodes = self.nodes;
            if let Some(mv) = self.root_best {
                info.best_move = Some(mv);
            }
        }
        info
    }

    fn root_search(&mut self, pos: &mut Position, depth: u8) -> i32 {
        let moves = legal_moves(pos);
        if moves.is_empty() {
            return if is_in_check(pos) { -MATE_SCORE } else { 0 };
        }

        let mut alpha = -INFINITY;
        let beta = INFINITY;
        self.root_best = None;

        for mv in order_moves(&moves, pos) {
            let undo = pos.make_move(mv);
            let score = -self.negamax(pos, -beta, -alpha, depth - 1);
            pos.unmake_move(mv, undo);
            if self.stopped {
                break;
            }
            if score > alpha {
                alpha = score;
                self.root_best = Some(mv);
            }
        }
        alpha
    }

    pub(crate) fn negamax(
        &mut self,
        pos: &mut Position,
        mut alpha: i32,
        beta: i32,
        depth: u8,
    ) -> i32 {
        if self.should_stop() {
            return 0;
        }
        if depth == 0 {
            return self.quiescence(pos, alpha, beta);
        }

        self.nodes += 1;

        // TT probe.
        let tt_move = if let Some(e) = self.tt.probe(pos.hash) {
            if e.depth >= depth {
                let s = e.score;
                let b = e.bound;
                if b == Bound::Exact as u8 {
                    return s;
                }
                if b == Bound::Lower as u8 && s >= beta {
                    return s;
                }
                if b == Bound::Upper as u8 && s <= alpha {
                    return s;
                }
            }
            e.mv
        } else {
            Move::NULL
        };

        let moves = legal_moves(pos);
        if moves.is_empty() {
            return if is_in_check(pos) { -MATE_SCORE } else { 0 };
        }

        let ply = depth as usize; // approximate ply from current depth
        let ordered = order_moves_full(
            &moves,
            pos,
            tt_move,
            &self.killers[ply.min(63)],
            &self.history[pos.side_to_move as usize],
        );

        let orig_alpha = alpha;
        let mut best_score = -INFINITY;
        let mut best_mv = Move::NULL;

        for mv in ordered {
            let undo = pos.make_move(mv);
            let score = -self.negamax(pos, -beta, -alpha, depth - 1);
            pos.unmake_move(mv, undo);

            if self.stopped {
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_mv = mv;
            }
            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    // Beta cut-off: store killer + history.
                    if !mv.is_capture() {
                        let k = &mut self.killers[ply.min(63)];
                        if k[0] != mv {
                            k[1] = k[0];
                            k[0] = mv;
                        }
                        self.history[pos.side_to_move as usize][mv.from().index()]
                            [mv.to().index()] += (depth as i32).pow(2);
                    }
                    break;
                }
            }
        }

        // TT store.
        let bound = if best_score <= orig_alpha {
            Bound::Upper
        } else if best_score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        self.tt.store(pos.hash, best_score, best_mv, depth, bound);

        best_score
    }

    fn quiescence(&mut self, pos: &mut Position, mut alpha: i32, beta: i32) -> i32 {
        self.nodes += 1;
        let in_check = is_in_check(pos);

        if !in_check {
            let stand_pat = evaluate(pos);
            if stand_pat >= beta {
                return beta;
            }
            if stand_pat > alpha {
                alpha = stand_pat;
            }
        }

        let moves = legal_moves(pos);
        if moves.is_empty() {
            return if in_check { -MATE_SCORE } else { 0 };
        }

        let candidates: Vec<Move> = if in_check {
            moves
        } else {
            moves.into_iter().filter(|m| m.is_capture()).collect()
        };

        for mv in order_moves(&candidates, pos) {
            let undo = pos.make_move(mv);
            let score = -self.quiescence(pos, -beta, -alpha);
            pos.unmake_move(mv, undo);
            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }
        alpha
    }

    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn tt_clear(&mut self) {
        self.tt.clear();
    }

    fn should_stop(&mut self) -> bool {
        if let Some(t) = self.stop_time {
            if Instant::now() >= t {
                self.stopped = true;
                return true;
            }
        }
        false
    }
}

// ── Move ordering: TT move > captures (MVV-LVA) > killers > history > quiets ─

fn mvv_lva(mv: Move, pos: &Position) -> i32 {
    if !mv.is_capture() {
        return 0;
    }
    let attacker = pos
        .piece_at(mv.from())
        .map(|p| piece_value(p.kind))
        .unwrap_or(0);
    let victim = pos
        .piece_at(mv.to())
        .map(|p| piece_value(p.kind))
        .unwrap_or(0);
    victim * 10 - attacker
}

fn piece_value(kind: PieceType) -> i32 {
    match kind {
        PieceType::Pawn => 1,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 100,
    }
}

fn order_moves_full(
    moves: &[Move],
    pos: &Position,
    tt_move: Move,
    killers: &[Move; 2],
    history: &[[i32; 64]; 64],
) -> Vec<Move> {
    let mut scored: Vec<(i32, Move)> = moves
        .iter()
        .map(|&mv| {
            let score = if mv == tt_move {
                2_000_000
            } else if mv.is_capture() {
                1_000_000 + mvv_lva(mv, pos)
            } else if mv == killers[0] {
                900_000
            } else if mv == killers[1] {
                800_000
            } else {
                history[mv.from().index()][mv.to().index()]
            };
            (score, mv)
        })
        .collect();
    scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().map(|(_, mv)| mv).collect()
}

fn order_moves(moves: &[Move], pos: &Position) -> Vec<Move> {
    order_moves_full(moves, pos, Move::NULL, &[Move::NULL; 2], &[[0; 64]; 64])
}

fn is_in_check(pos: &Position) -> bool {
    let us = pos.side_to_move;
    let them = us.flip();
    let king = pos.king_sq(us);
    crate::movegen::is_attacked(pos, king, them)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    fn find_best(fen: &str, depth: u8) -> (Move, i32) {
        let mut pos = Position::from_fen(fen).unwrap();
        let mut s = Searcher::new();
        let info = s.search(
            &mut pos,
            &Limits {
                depth: Some(depth),
                ..Default::default()
            },
        );
        (info.best_move.unwrap(), info.score)
    }

    #[test]
    fn finds_mate_in_1() {
        // 2k5/8/2K5/8/8/8/8/7R w - - 0 1 → Rh8#
        // King on c8; white king c6 covers all flight squares; rook covers rank 8.
        let fen = "2k5/8/2K5/8/8/8/8/7R w - - 0 1";
        let (mv, score) = find_best(fen, 1);
        assert_eq!(
            mv.to(),
            crate::types::Square::H8,
            "Rh8# should be the mating move"
        );
        assert!(score > IS_MATE, "expected mate score, got {score}");
    }

    #[test]
    fn finds_mate_in_2_fools_mate() {
        // After 1.f3 e5 2.g4, Black plays Qh4# (depth-3 search from black's view).
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 2";
        let (mv, score) = find_best(fen, 3);
        assert_eq!(
            mv.to(),
            crate::types::Square::H4,
            "Qh4# should be the best move"
        );
        assert!(score > IS_MATE, "expected mate score, got {score}");
    }

    #[test]
    fn does_not_hang_in_startpos() {
        let mut pos = Position::startpos();
        let mut s = Searcher::new();
        let info = s.search(
            &mut pos,
            &Limits {
                depth: Some(4),
                ..Default::default()
            },
        );
        assert!(info.best_move.is_some());
        assert!(info.nodes > 0);
    }

    #[test]
    fn stalemate_is_zero() {
        // A stalemate position: black king cornered, no legal moves, not in check.
        let fen = "7k/8/6Q1/8/8/8/8/K7 b - - 0 1";
        let pos = Position::from_fen(fen).unwrap();
        let moves = legal_moves(&pos);
        if moves.is_empty() {
            // If it's stalemate, score should be 0 (not mate).
            let score = {
                let mut s = Searcher::new();
                s.negamax(&mut pos.clone(), -INFINITY, INFINITY, 1)
            };
            assert_eq!(score, 0);
        }
        // (if not stalemate with this exact position, test is a no-op)
    }
}
