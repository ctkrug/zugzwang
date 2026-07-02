---
title: "Building a chess engine you can actually read"
published: false
tags: rust, chess, algorithms, programming
---

I wanted to understand how a chess engine really works, so I built one from scratch in Rust. Not
a competitor to Stockfish, which is tens of thousands of lines of hand-tuned bitboard code, but
something at the other end of the trade-off: small enough to read end to end. It is called
Zugzwang, it speaks UCI so any chess GUI can play it, and the whole thing is about 4,000 lines.

Live page: https://apps.charliekrug.com/zugzwang/
Source: https://github.com/ctkrug/zugzwang

Two problems from the build were more interesting than I expected, so here they are.

## The horizon effect hides a queening pawn

The core of the search is negamax with alpha-beta pruning. It looks a fixed number of plies ahead
and then calls a static evaluation to score the leaf. The problem with stopping at a fixed depth
is the horizon effect: if the last move searched is the capture of a queen, and the recapture
happens one ply past the horizon, the engine cheerfully reports that it is up a queen.

The standard fix is quiescence search. Instead of returning the static eval at a leaf, you keep
searching, but only "loud" moves, until the position is quiet. My first version extended captures
only, which is the textbook definition. It passed the obvious tests. Then I gave it a position
with a pawn on the seventh rank and no captures anywhere on the board.

The engine misjudged it. A pawn one push away from becoming a queen is worth roughly a pawn to a
static evaluation, because the pawn is still a pawn until it actually promotes. Quiescence stopped
right before the promotion, since a promotion push captures nothing, and handed back a score that
priced a soon-to-be queen as a pawn. That is the same horizon effect, just triggered by promotion
instead of recapture.

The fix was to treat a non-capturing promotion as loud too:

```rust
let mut tactical = legal_moves(board);
tactical.retain(|&mv| is_capture(board, mv) || mv.promotion.is_some());
```

One line, but it took writing the failing test to see that "loud" means more than "capture."

## You cannot cache a mate score like any other score

The second one bit me in the transposition table. The table caches the evaluation of a position
keyed by its Zobrist hash, so when the search reaches the same position by a different move order,
it reuses the result instead of re-searching. Huge speedup, and mostly it just works.

Mate scores broke it. I score a checkmate as a large constant minus the ply it was found at, so
that a mate in three is preferred over a mate in five (shorter mates score higher). That relative
encoding is the problem: the same mate found at a different distance from the root has a different
score. Cache a mate score at one ply and reuse it at another, and you misreport the distance to
mate, which can make the engine walk in circles instead of finishing the game.

The clean fix is to adjust mate scores by the current ply on the way in and out of the table. I
took the simpler route for a v1: do not cache scores anywhere near mate magnitude at all.

```rust
if best.abs() <= MATE_SCORE - MATE_CACHE_MARGIN {
    // safe to store; ordinary positional score
}
```

It leaves a little speed on the table near forced mates, which is exactly where you can afford it,
and it keeps the table honest everywhere else.

## What I would do differently

Two things. The board is a flat 64-square array, which keeps move generation readable but slow;
bitboards are the obvious rewrite once speed is the point. And the Zobrist hash is recomputed from
scratch on every call rather than updated incrementally as moves are made, which is a full board
scan where a couple of XORs would do. Both are deliberate v1 simplifications, and both are on the
backlog.

The most useful habit through all of it was writing the failing test first. Every bug above looked
fine until a specific position proved it was not. The engine is at 168 tests now, and the perft
move-generation counts match the known references through depth 5 (4,865,609 nodes), which is the
only reason I trust anything the search says on top of it.

If you want to read how alpha-beta, quiescence, Zobrist hashing, and UCI actually fit together,
the source is meant to be read: https://github.com/ctkrug/zugzwang
</content>
