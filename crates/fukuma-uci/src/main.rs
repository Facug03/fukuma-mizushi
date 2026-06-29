use std::io::{self, BufRead, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fukuma_core::movegen::legal_moves;
use fukuma_core::position::Position;
use fukuma_core::search::{Limits, Searcher};
use fukuma_core::types::Color;

fn main() {
    let stdin  = io::stdin();
    let stdout = io::stdout();
    let searcher = Arc::new(Mutex::new(Searcher::new()));

    let mut pos = Position::startpos();

    for line in stdin.lock().lines() {
        let line = line.expect("stdin error");
        let line = line.trim();
        if line.is_empty() { continue; }

        let mut tokens = line.splitn(2, ' ');
        match tokens.next() {
            Some("uci") => {
                let mut out = stdout.lock();
                writeln!(out, "id name fukuma-mizushi").unwrap();
                writeln!(out, "id author Kiro [claude-sonnet-4-5] (Amazon)").unwrap();
                writeln!(out, "uciok").unwrap();
            }
            Some("isready")    => println!("readyok"),
            Some("ucinewgame") => {
                pos = Position::startpos();
                searcher.lock().unwrap().tt_clear();
            }
            Some("position") => {
                pos = parse_position(line);
            }
            Some("go") => {
                let rest   = tokens.next().unwrap_or("");
                let limits = parse_go(rest, &pos);
                let s_arc  = Arc::clone(&searcher);
                let mut p  = pos.clone();
                thread::spawn(move || {
                    let mut s   = s_arc.lock().unwrap();
                    let info    = s.search(&mut p, &limits);
                    let best_mv = info.best_move
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "0000".to_string());
                    println!("bestmove {best_mv}");
                });
            }
            Some("stop") => { searcher.lock().unwrap().stop(); }
            Some("quit") | None => break,
            _ => {}
        }
    }
}

// ── Position parsing ──────────────────────────────────────────────────────────

fn parse_position(line: &str) -> Position {
    let rest = line.trim_start_matches("position").trim();

    let (pos_part, moves_part) = match rest.split_once(" moves ") {
        Some((p, m)) => (p, m),
        None         => (rest, ""),
    };

    let mut pos = if pos_part.trim() == "startpos" {
        Position::startpos()
    } else if let Some(fen) = pos_part.trim().strip_prefix("fen ") {
        Position::from_fen(fen).unwrap_or_else(|_| Position::startpos())
    } else {
        Position::startpos()
    };

    for token in moves_part.split_ascii_whitespace() {
        let legal = legal_moves(&pos);
        if let Some(&mv) = legal.iter().find(|m| m.to_string() == token) {
            pos.make_move(mv);
        }
    }
    pos
}

// ── Go / time management ──────────────────────────────────────────────────────

fn parse_go(args: &str, pos: &Position) -> Limits {
    let mut limits     = Limits::default();
    let mut wt: Option<u64> = None;
    let mut bt: Option<u64> = None;
    let mut winc  = 0u64;
    let mut binc  = 0u64;
    let mut movestogo = 30u64;

    let mut toks = args.split_ascii_whitespace();
    while let Some(tok) = toks.next() {
        match tok {
            "depth"     => limits.depth     = toks.next().and_then(|t| t.parse().ok()),
            "movetime"  => limits.move_time = toks.next()
                                .and_then(|t| t.parse::<u64>().ok())
                                .map(Duration::from_millis),
            "wtime"     => wt        = toks.next().and_then(|t| t.parse().ok()),
            "btime"     => bt        = toks.next().and_then(|t| t.parse().ok()),
            "winc"      => winc      = toks.next().and_then(|t| t.parse().ok()).unwrap_or(0),
            "binc"      => binc      = toks.next().and_then(|t| t.parse().ok()).unwrap_or(0),
            "movestogo" => movestogo = toks.next().and_then(|t| t.parse().ok()).unwrap_or(30),
            "infinite"  => limits.depth = Some(64),
            _ => {}
        }
    }

    if limits.depth.is_none() && limits.move_time.is_none() {
        let (our_time, our_inc) = if pos.side_to_move == Color::White {
            (wt.unwrap_or(0), winc)
        } else {
            (bt.unwrap_or(0), binc)
        };
        if our_time > 0 {
            let alloc = (our_time / movestogo + our_inc / 2).max(50);
            limits.move_time = Some(Duration::from_millis(alloc));
        }
    }
    limits
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fukuma_core::search::Limits;

    #[test]
    fn uci_startpos() {
        let pos = parse_position("position startpos");
        assert_eq!(pos.to_fen(), Position::STARTPOS);
    }

    #[test]
    fn uci_startpos_moves() {
        let pos = parse_position("position startpos moves e2e4 e7e5");
        assert_eq!(pos.fullmove_number, 2);
        assert_eq!(pos.side_to_move, Color::White);
    }

    #[test]
    fn uci_fen_position() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let pos = parse_position(&format!("position fen {fen}"));
        assert_eq!(pos.to_fen(), fen);
    }

    #[test]
    fn uci_smoke_bestmove() {
        let mut pos = Position::startpos();
        let legal   = legal_moves(&pos);
        let mut s   = Searcher::new();
        let info    = s.search(&mut pos, &Limits { depth: Some(3), ..Default::default() });
        let best    = info.best_move.unwrap();
        assert!(legal.contains(&best), "best move {best} should be legal");
    }

    #[test]
    fn parse_go_movetime() {
        let pos    = Position::startpos();
        let limits = parse_go("movetime 500", &pos);
        assert_eq!(limits.move_time, Some(Duration::from_millis(500)));
    }

    #[test]
    fn parse_go_depth() {
        let pos    = Position::startpos();
        let limits = parse_go("depth 7", &pos);
        assert_eq!(limits.depth, Some(7));
    }
}
