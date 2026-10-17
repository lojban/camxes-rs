//! Phonology tables from CLL Chapter 3.
//!
//! Builds the alphabet and pair predicates consumed by CFG terminals and by
//! constraint C20. No lookaround; finite sets only.

pub mod clusters;
pub mod diphthongs;
pub mod letters;

pub use clusters::*;
pub use diphthongs::*;
pub use letters::*;
