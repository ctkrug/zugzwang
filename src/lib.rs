//! Zugzwang: a from-scratch chess engine.
//!
//! `board` and `moves`/`movegen` model the game; `search` and `eval` decide
//! what to play; `uci` exposes the engine to any UCI-compatible GUI.

pub mod board;
pub mod eval;
pub mod history;
pub mod killers;
pub mod movegen;
pub mod moves;
pub mod ordering;
pub mod play;
pub mod pst;
pub mod search;
pub mod square;
pub mod types;
pub mod uci;
pub mod zobrist;
