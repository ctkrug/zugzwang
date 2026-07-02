# Zugzwang

**▶ Live demo: [apps.charliekrug.com/zugzwang](https://apps.charliekrug.com/zugzwang/)**

*A chess engine you can actually read.*

[![CI](https://github.com/ctkrug/zugzwang/actions/workflows/ci.yml/badge.svg)](https://github.com/ctkrug/zugzwang/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A from-scratch terminal chess engine in Rust: full move generation, alpha-beta search with
iterative deepening, and a [UCI](https://www.chessprogramming.org/UCI) interface so it can plug
into any chess GUI (Arena, CuteChess, `xboard`, or a plain terminal).

> *Zugzwang* (German: "compulsion to move") is a position where any legal move weakens it. Fitting
> for an engine that has to find the least-bad move every single ply.

## Why

Most "chess engine" side projects stop at legal move generation. Zugzwang goes further: a real
search algorithm (minimax plus alpha-beta pruning plus a transposition table), real move ordering
heuristics, and a real protocol implementation, so it can actually sit behind a GUI and play a
game start to finish. That is the difference between a toy and an engine. It also stays small
enough (about 4,000 lines) to read end to end, which is the point: it is a reference for how these
pieces fit together, not a black box.

## What it does

- **Move generation:** fully legal move generation for all pieces, including castling, en
  passant, and promotion, validated against perft test positions through depth 5 (starting
  position and the "Kiwipete" position).
- **Search:** negamax with alpha-beta pruning, iterative deepening against a configurable time
  budget, quiescence search to avoid the horizon effect, and a transposition table keyed by
  Zobrist hashing.
- **Move ordering:** MVV-LVA capture ordering, non-capturing promotions, killer moves, and a
  history heuristic, so alpha-beta pruning cuts off effectively.
- **Evaluation:** material plus piece-square tables, rewarding pieces for standing on
  typically-good squares, not just for existing.
- **UCI protocol:** `uci` mode speaks the Universal Chess Interface (`uci`, `isready`,
  `ucinewgame`, `position`, `go` with `depth`/`movetime`/`wtime`/`btime`/`movestogo`, `stop`,
  `quit`) over stdin/stdout, so any UCI-compatible GUI can drive it.
- **Terminal play:** `play` mode for playing a full game directly in the terminal, no GUI
  required, with checkmate, stalemate, fifty-move, insufficient-material, and threefold-repetition
  detection.
- **Perft debug command:** `perft <depth> [fen]` prints node counts per depth for validating move
  generation against known-good positions.

See [`docs/BACKLOG.md`](docs/BACKLOG.md) for what's not built yet (mainly a bitboard board
representation for performance, and playing a full game against a real UCI GUI end to end).

## What it looks like

A UCI session, exactly as the engine answers it:

```text
$ zugzwang uci
uci
id name Zugzwang
id author Charlie Krug
uciok
isready
readyok
position startpos moves e2e4 e7e5
go depth 4
bestmove b1c3
```

Or a game in the terminal, no GUI required:

```text
$ zugzwang play
r n b q k b n r
p p p p p p p p
. . . . . . . .
. . . . . . . .
. . . . . . . .
. . . . . . . .
P P P P P P P P
R N B Q K B N R

Your move (or 'quit'): e2e4
Engine plays b8c6
```

## Stack

- **Rust** (stable), no async runtime: this is CPU-bound search, not I/O-bound.
- `cargo test` for unit tests (move generation, perft) and integration tests (UCI protocol
  round-trips).
- No external chess libraries: the board, move generator, and search are all original.

## Status

Core search and protocol are working end-to-end. See [`docs/VISION.md`](docs/VISION.md) for the
design, [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the module map, and
[`docs/BACKLOG.md`](docs/BACKLOG.md) for the build plan.

## Building and running

```sh
cargo build --release
cargo test

./target/release/zugzwang            # print the starting position and exit
./target/release/zugzwang uci        # UCI mode, for a chess GUI to drive over stdin/stdout
./target/release/zugzwang play       # play a game against the engine in the terminal
./target/release/zugzwang play <fen> # play from a specific position instead of move one
./target/release/zugzwang perft 5    # move generation node counts, depths 1..5
```

Terminal play and UCI's `position ... moves ...` both take moves in coordinate algebraic
notation (`e2e4`, `e7e8q` for a promotion) rather than SAN (`Nf3`).

## License

MIT, see [LICENSE](LICENSE).

---

More of Charlie's projects → [apps.charliekrug.com](https://apps.charliekrug.com)
</content>
