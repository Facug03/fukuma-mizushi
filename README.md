# fukuma-mizushi

An AI-assisted chess engine written in Rust, competing against [Muryōkūsho](https://github.com/Facug03/muryokusho) — a hand-crafted engine by the same author.

Both engines play each other to see who wins: human intuition or AI-assisted development.

## Features

- Bitboard-based board representation
- Complete legal move generation (castling, en passant, promotions)
- Negamax with alpha-beta pruning and iterative deepening
- Quiescence search
- Transposition table (Zobrist hashing)
- Move ordering: TT move → MVV-LVA captures → killer moves → history heuristic
- Static evaluation: material + PeSTO piece-square tables with tapered eval (MG/EG interpolation)
- UCI protocol — compatible with any chess GUI or `lichess-bot`
- Time management: `movetime`, `depth`, and clock-based (`wtime`/`btime`)

## Crates

| Crate | Description |
|---|---|
| `fukuma-core` | Core library: board, movegen, search, evaluation |
| `fukuma-uci` | UCI binary (`fukuma-mizushi`) |

## Build

```bash
cargo build --release
```

Binary: `target/release/fukuma-mizushi`

## UCI smoke test

```bash
echo -e "uci\nisready\nquit" | ./target/release/fukuma-mizushi
```

Expected output:
```
id name fukuma-mizushi
id author Kiro [claude-sonnet-4-5] (Amazon)
uciok
readyok
```

## Run a match

Requires [`cutechess-cli`](https://cutechess.com/) and a release build of the opponent.

```bash
./scripts/match.sh ../muryokusho/target/release/muryokusho 200
```

Results are saved as PGN files in `results/`.

## Lichess deployment

See [`docs/lichess-deployment.md`](docs/lichess-deployment.md).

## Authorship

Every AI-assisted commit includes the trailer:

```
Assisted-by: Kiro [claude-sonnet-4-5] (Amazon)
```

Filter AI commits:

```bash
git log --grep="Assisted-by: Kiro"
```

## License

Dual-licensed under [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE).
