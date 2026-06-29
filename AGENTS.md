# AGENTS.md

## Competition: fukuma-mizushi vs Muryōkūsho

**fukuma-mizushi** is an AI-assisted chess engine written in Rust.
**Muryōkūsho** is its opponent — a chess engine written by the repository owner without AI assistance, also in Rust.
The goal of both projects is to compete against each other.

## This repository

- Engine: **fukuma-mizushi**
- Assisted by: Kiro (Amazon)
- Language: Rust

## Commit authorship convention

Every commit made with AI assistance includes the following trailer:

    Assisted-by: Kiro (Amazon)

This makes it clear which commits were AI-generated vs human-written.

## Project structure

- `crates/fukuma-core` — core library (board, movegen, search, evaluation). Public API, usable as a dependency.
- `crates/fukuma-uci` — UCI binary (`fukuma-mizushi`), compatible with chess GUIs and `lichess-bot`.
- `crates/fukuma-bot` — Lichess bot integration (future).
