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

    Assisted-by: Kiro [<model-id>] (Amazon)

Where `<model-id>` is the specific model that generated the code, for example:

    Assisted-by: Kiro [claude-sonnet-4-5] (Amazon)

This allows filtering commits by model in git:

    git log --grep="claude-sonnet-4-5"

The model id is the identifier used by Amazon Kiro at the time of generation.
Human-authored commits do NOT include this trailer.

## Project structure

- `crates/fukuma-core` — core library (board, movegen, search, evaluation). Public API, usable as a dependency.
- `crates/fukuma-uci` — UCI binary (`fukuma-mizushi`), compatible with chess GUIs and `lichess-bot`.
- `crates/fukuma-bot` — Lichess bot integration (future).
