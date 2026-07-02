# Vision

## The problem

Chess engines are one of the most well-trodden domains in computer science, which is exactly
what makes them a good proving ground — the "right answer" is well documented (Shannon, Turing,
decades of chess programming literature) and yet still hard to actually build. Most hobby
attempts stop at legal move generation and never produce something that can play a full game
under a real protocol. Zugzwang's goal is to go all the way: a terminal engine that speaks UCI
well enough to sit behind a real GUI and play a complete, legal game of chess.

## Who it's for

- **Charlie**, as a project that demonstrates real algorithms work (search, pruning, hashing)
  and systems-level Rust (bit manipulation, performance-sensitive code, no hand-holding from a
  garbage collector).
- Anyone browsing the repo who wants to see a from-scratch, dependency-free chess engine as a
  reference for how alpha-beta search and UCI actually fit together in practice.

## The core idea

A UCI-speaking binary with three layers, each replaceable independently:

1. **Board + move generation** — legal chess rules, nothing more.
2. **Search** — decides *which* moves are worth exploring and how deep.
3. **Evaluation** — decides *how good* a position is once search stops.

The UCI protocol layer is a thin adapter on top: it reads `position`/`go`/`stop` commands from
stdin, drives the search, and writes `bestmove` to stdout. Because UCI is a stable, universal
protocol, any GUI (Arena, CuteChess, `en croissant`, etc.) becomes a free frontend.

## Key design decisions

- **No external chess crates.** The board representation, move generator, and search are all
  original code. The point of this project is demonstrating that code, not gluing together
  `pleco` or `shakmaty`.
- **Bitboards over 8x8 arrays for the real board.** The scaffold uses a simple 64-element array
  for readability while the shape of the engine is being built; the build phase replaces it with
  bitboards once move generation needs the performance, since bitwise set operations are what
  make fast legal-move generation possible.
- **Negamax + alpha-beta, not plain minimax.** Negamax is minimax with the sign flip folded into
  the recursion, which keeps the search function symmetric and easier to extend with pruning,
  a transposition table, and move ordering later.
- **Zobrist hashing for the transposition table.** Standard approach: incrementally-updated
  hashes let the table cache position evaluations across transpositions cheaply.
- **UCI, not a custom protocol.** UCI is what every serious GUI already speaks. Building a custom
  protocol would mean building a custom GUI too, which is out of scope.
- **Terminal play mode is secondary.** The engine's primary interface is UCI; the terminal mode
  exists so the engine is directly playable without installing a GUI, not as the main product.

## What "v1 done" looks like

- Full legal move generation (including castling, en passant, promotion) verified against known
  perft results for the standard starting position through at least depth 5.
- Alpha-beta search with iterative deepening and a time budget, backed by a transposition table.
- Material + piece-square-table evaluation.
- A UCI implementation complete enough to play a full game start-to-finish inside a real GUI
  (`uci`, `isready`, `ucinewgame`, `position`, `go`, `stop`, `quit` at minimum).
- A terminal play mode for playing a game directly without a GUI.
- Engine strength isn't the bar for v1 — correctness and protocol completeness are. Playing
  strength is an open-ended thing to keep improving after v1.
