# Architecture

A concise map of the codebase for anyone (human or otherwise) picking this project up cold. See
[`docs/VISION.md`](VISION.md) for *why* it's built this way and [`docs/BACKLOG.md`](BACKLOG.md)
for what's left.

## Module map

```
src/
  types.rs      Color, PieceKind, CastlingRights — small shared value types.
  square.rs     Square (file/rank), algebraic <-> index conversions.
  moves.rs      Move (from, to, promotion, MoveKind) and its UCI long-algebraic rendering.
  board.rs      Board: 8x8 array of Option<Piece> + side to move, castling, en passant, clocks.
                FEN parse/serialize (rejecting a placement without exactly one king per side —
                movegen's own king lookup panics otherwise), make_move (pure — returns a new
                Board), Display.
  movegen.rs    pseudo_legal_moves, legal_moves (filters self-check), is_in_check,
                is_square_attacked, perft, find_uci_move.
  eval.rs       material_score + pst::score combined into evaluate (White-relative centipawns),
                plus the shared piece_value table.
  pst.rs        Piece-square tables (one per piece kind), mirrored for Black instead of
                duplicated, rewarding pieces for standing on typically-good squares.
  ordering.rs   order_moves: ranks a move list captures-first (MVV-LVA), then killer moves,
                then history-scored quiets. Tiered scoring (see comments) keeps each signal
                strictly dominant over the ones below it.
  killers.rs    KillerMoves: two per-ply killer-quiet-move slots.
  history.rs    HistoryTable: butterfly from/to cutoff-frequency table.
  zobrist.rs    hash: Zobrist hash of a position (pieces, side to move, castling, en passant),
                recomputed from scratch each call rather than maintained incrementally.
  tt.rs         TranspositionTable: fixed-size, always-replace table of TTEntry (depth, score,
                Exact/Lower/Upper bound) keyed by Zobrist hash.
  search.rs     Search: negamax + alpha-beta, owns a KillerMoves + HistoryTable + Transposition-
                Table for the whole tree. Each node probes the TT before searching (an exact hit
                at sufficient depth returns immediately; a bound hit narrows the window) and
                stores its result after, skipping mate-adjacent scores since those are ply-
                relative. Depth-0 leaves extend into quiescence (capture-only search) instead of
                returning the raw eval, to avoid the horizon effect. find_best_move does
                iterative deepening against a wall-clock budget; find_best_move_to_depth instead
                iterates to an exact depth with no time limit, for a UCI `go depth` request.
  uci.rs        UCI command loop: uci/isready/ucinewgame/position/go/stop/quit. Tracks the
                game as a Board; go honors an explicit `depth` exactly via
                Search::find_best_move_to_depth, otherwise derives a time budget from movetime
                or wtime/btime/movestogo and calls Search::find_best_move.
  play.rs       Pure logic for the terminal play mode (game_status — checkmate/stalemate/
                fifty-move draw/insufficient-material draw off the current Board alone,
                is_threefold_repetition — a third occurrence of the current position's hash in
                a caller-supplied history, apply_human_move, engine_reply) — no I/O, so it's
                unit-tested directly.
  main.rs       CLI entrypoint: no args prints the board, `uci`, `play [fen]` (interactive loop
                built on play.rs, starting from the given position or the standard start,
                tracking each position's Zobrist hash across the game for repetition detection,
                and checking game-over status before the first prompt so an already-finished
                starting position is reported immediately), `perft <depth> [fen]`; any other
                subcommand is a usage error, not silently treated as the no-args default.
```

## Data flow

- **UCI GUI → engine:** stdin line → `uci::run`'s command loop → `position` updates the tracked
  `Board`; `go` builds a `Search`, calls `find_best_move`, writes `bestmove <uci>` to stdout.
- **Terminal play:** `main::run_play` reads a line, calls `play::apply_human_move` (validates via
  `movegen::find_uci_move`), prints the board, checks `play::game_status`, then calls
  `play::engine_reply` (wraps `Search::find_best_move`) for the engine's turn.
- **Search:** `Search::find_best_move` iterative-deepens by calling `root_search` at depth
  1, 2, 3... until a `Duration` budget elapses (checked between root moves, not inside deeper
  plies — a depth already underway is allowed to finish). Each depth calls `negamax`, which
  orders moves via `ordering::order_moves` (using the `Search`'s own killer/history tables),
  recurses with alpha-beta pruning, and on a beta cutoff from a quiet move records it as a
  killer and bumps its history score.
- **Move legality:** `movegen::pseudo_legal_moves` generates piece-rule-legal moves (including
  castling/en passant/promotion); `legal_moves` filters out any that leave the mover's own king
  attacked by calling `Board::make_move` and `is_square_attacked`.

## Running things

```sh
cargo build --release           # optimized binary at target/release/zugzwang
cargo test                      # unit tests (per-module) + tests/cli.rs integration tests
cargo test --release -- --ignored   # includes the slow perft depth-5 test
cargo clippy --all-targets -- -D warnings
cargo fmt

./target/release/zugzwang               # print the starting board and exit
./target/release/zugzwang uci            # UCI mode, for a GUI to drive over stdin/stdout
./target/release/zugzwang play           # interactive terminal game vs. the engine
./target/release/zugzwang perft 5        # move generation node counts, depths 1..5
```

## Known simplifications (see BACKLOG.md for the plan to address them)

- The board is a flat `[Option<Piece>; 64]`, not bitboards — simple and correct, not fast.
- The transposition table has no move-ordering hint (no best-move stored per entry) and no
  replacement scheme beyond always-replace — a deep, valuable entry can be evicted by a shallow
  one that happens to land in the same bucket.
- Zobrist hashing is recomputed from scratch per `hash()` call rather than updated incrementally
  inside `Board::make_move`, so it costs a full board scan instead of a couple of XORs.
- Quiescence only extends captures, not check evasions, so a position where the side to move is
  in check at the search horizon can still misjudge a forced reply.
- UCI `stop` is a recognized no-op: `go` runs synchronously to completion (bounded by its time
  budget) before the next stdin line is read, so there's never an in-flight search to interrupt.
- Terminal play and UCI `moves` both use coordinate algebraic notation (`e2e4`, not SAN like
  `Nf3`) — there's no SAN parser.
- Fifty-move, insufficient-material, and threefold-repetition draw detection
  (`play::game_status` / `play::is_threefold_repetition`) only end a terminal-play game; the
  search itself has no notion of them, so it can't yet steer toward or away from a repetition it
  would otherwise want, and UCI mode (which only tracks the position, not full game-over state)
  doesn't surface any of the three at all.
