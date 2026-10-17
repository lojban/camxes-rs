//! Pure CFG overgenerator — Earley recognizer.
//!
//! **Hard rule:** no PEG `&` / `!`, no lookaround, no semantic predicates that
//! inspect unmatched input. Ambiguity is expected; uniqueness is the job of
//! [`crate::cll::constraints`].
//!
//! Normative shapes: CLL §4.1–4.8 phonology/morphology forms only.
//!
//! Earley uses Aycock–Horspool nullable advance (morph CFG has ε-onsets).

pub mod earley;
pub mod forest;
pub mod grammar;

pub use earley::{recognize_starts, recognizes, Grammar, NtId, Symbol};
pub use forest::*;
pub use grammar::*;
