//! A Parsing Expression Grammar (PEG) parser generator
//!
//! # Example
//! ```rust
//! # #[cfg(feature = "peg")] {
//! use camxes_rs::peg::grammar::Peg;
//!
//! let grammar = r#"
//! start <- 'hello' 'world'
//! "#;
//!
//! let parser = Peg::new("start", grammar).unwrap();
//! let result = parser.parse("hello world");
//! # }
//! ```
//!
//! # Cargo features
//! * `peg` (default) — the PEG engine (`peg::grammar::Peg`) and
//!   `Engine::Peg`.
//! * `earley` — the Earley engine (`earley`), `Engine::Earley` and the
//!   vendored CLL morphology (`cll`).
//!
//! At least one engine must be enabled. Shared by both: the grammar texts
//! ([`grammars`]), the parse-tree types ([`peg::parsing`]), the grammar AST
//! ([`peg::rule::Rule`]) and the stack guard ([`peg::stack`]).
//!
//! # Modules
//! * [`peg`] — the PEG parser generator (engine: feature `peg`).
//! * `earley` (feature `earley`) — an Earley engine that runs the same PEG
//!   grammar text.
//! * [`grammars`] — embedded grammars: Lojban syntax (camxes) and
//!   Lojban morphology.
//! * [`syntax`] — whole-text Lojban syntax check with either engine.
//! * [`morphology`] — engine-independent Lojban morphology (word split,
//!   classes, selma'o, standard/obsolete dialect).
//! * `cll` (feature `earley`) — vendored tergenzalvi CLL morphology and
//!   Chapter 21 grammar.

#[cfg(not(any(feature = "peg", feature = "earley")))]
compile_error!(
    "camxes-rs needs at least one engine: enable the `peg` feature (default) and/or `earley`"
);

#[cfg(feature = "earley")]
pub mod cll;
#[cfg(feature = "earley")]
pub mod earley;
pub mod grammars;
pub mod morphology;
pub mod peg;
pub mod syntax;

/// Compiles and runs the README's code examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
