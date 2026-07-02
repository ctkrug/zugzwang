# Backlog

Epic/story breakdown for the build phase. High-level on purpose — each story gets refined into
concrete commits when it's actually built.

## Epic 1: Board & Move Generation

The foundation everything else depends on: a correct board and fully legal move generation.

- [ ] Replace the array-based board with a bitboard representation for performance.
- [x] Implement FEN parsing and serialization (`Board::from_fen` / `Board::to_fen`).
- [x] Implement pseudo-legal move generation for all six piece types, including castling,
      en passant, and promotion.
- [x] Implement check detection and legal-move filtering (pins, moving into check, castling
      through/out of check).
- [x] Validate move generation against a perft test suite for known positions through at
      least depth 5.

## Epic 2: Search & Evaluation

Turns legal moves into a decision: what's the best move in this position, right now.

- [x] Implement alpha-beta pruning on top of the existing negamax skeleton.
- [x] Add iterative deepening with a configurable time budget.
- [x] Implement Zobrist hashing and a transposition table keyed by position hash.
- [x] Add move ordering (MVV-LVA for captures, killer moves, history heuristic) so alpha-beta
      pruning is effective.
- [x] Extend evaluation with piece-square tables and add quiescence search to avoid the
      horizon effect on tactical positions.

## Epic 3: UCI Protocol & Terminal Play

Makes the engine usable — by a GUI over UCI, or directly in a terminal.

- [x] Wire the `position` and `go` UCI commands to the search, including `bestmove` output.
- [x] Support `ucinewgame`, `stop`, and time-control parameters (`wtime`/`btime`/`movestogo`).
- [x] Add a terminal play mode with algebraic move input and board rendering after each move.
- [x] Add a `perft` debug command for validating move generation from an arbitrary FEN.
- [ ] Play a full game end-to-end against a real UCI GUI (e.g. CuteChess) and fix any protocol
      gaps found in the process.
