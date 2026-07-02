# Zugzwang

A from-scratch terminal chess engine in Rust: full move generation, alpha-beta search with
iterative deepening, and a [UCI](https://www.chessprogramming.org/UCI) interface so it can plug
into any chess GUI (Arena, CuteChess, `xboard`, or a plain terminal).

> *Zugzwang* (German: "compulsion to move") — a position where any legal move weakens it. Fitting
> for an engine that has to find the least-bad move every single ply.

## Why

Most "chess engine" side projects stop at legal move generation. Zugzwang goes further: a real
search algorithm (minimax + alpha-beta pruning + a transposition table), real move ordering
heuristics, and a real protocol implementation, so it can actually sit behind a GUI and play a
game start to finish. That's the difference between a toy and an engine.

## What it does (planned)

- **Board representation** — bitboard-based board state with fast make/unmake move.
- **Move generation** — fully legal move generation for all pieces, including castling, en
  passant, and promotion, validated against perft test positions.
- **Search** — minimax with alpha-beta pruning, iterative deepening, and a transposition table
  keyed by Zobrist hashing.
- **Evaluation** — material + piece-square tables to start, with room to grow (mobility, king
  safety, pawn structure).
- **Move ordering** — MVV-LVA capture ordering, killer moves, and history heuristics to make
  alpha-beta pruning effective.
- **UCI protocol** — a `uci` binary that speaks the Universal Chess Interface over stdin/stdout,
  so any UCI-compatible GUI can drive it.
- **Terminal play** — a minimal CLI mode for playing a game directly in the terminal without a
  GUI.

## Stack

- **Rust** (stable), no async runtime — this is CPU-bound search, not I/O-bound.
- `cargo test` for unit tests (move generation, perft) and integration tests (UCI protocol
  round-trips).
- No external chess libraries — the board, move generator, and search are all original.

## Status

Early scope/planning stage. See [`docs/VISION.md`](docs/VISION.md) for the design and
[`docs/BACKLOG.md`](docs/BACKLOG.md) for the build plan.

## Building

```sh
cargo build --release
cargo test
```

## License

MIT — see [LICENSE](LICENSE).
