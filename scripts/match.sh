#!/usr/bin/env bash
# Usage: ./scripts/match.sh <path-to-opponent-binary> [rounds]
# Runs a match between fukuma-mizushi and an opponent engine using cutechess-cli.
# Requires: cutechess-cli in PATH, and a release build of fukuma-mizushi.
#
# Example:
#   cargo build --release
#   ./scripts/match.sh ../muryokusho/target/release/muryokusho 200

set -euo pipefail

OPPONENT="${1:?Usage: $0 <opponent-binary> [rounds]}"
ROUNDS="${2:-100}"
FUKUMA="./target/release/fukuma-mizushi"

if ! command -v cutechess-cli &>/dev/null; then
    echo "error: cutechess-cli not found. Install from https://cutechess.com/" >&2
    exit 1
fi

if [[ ! -x "$FUKUMA" ]]; then
    echo "error: $FUKUMA not found. Run 'cargo build --release' first." >&2
    exit 1
fi

cutechess-cli \
    -engine cmd="$FUKUMA"   name=fukuma-mizushi \
    -engine cmd="$OPPONENT" name=opponent \
    -each proto=uci tc=10+0.1 \
    -rounds "$ROUNDS" \
    -concurrency 2 \
    -resign movecount=5 score=900 \
    -draw movenumber=40 movecount=8 score=10 \
    -pgnout results/match-"$(date +%Y%m%d-%H%M%S)".pgn
