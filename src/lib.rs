//! A Parsing Expression Grammar (PEG) parser generator
//!
//! # Example
//! ```rust
//! use camxes_rs::peg::grammar::Peg;
//!
//! let grammar = r#"
//! start <- 'hello' 'world'
//! "#;
//!
//! let parser = Peg::new("start", grammar).unwrap();
//! let result = parser.parse("hello world");
//! ```
//!
//! # Modules
//! * [`peg`] — the PEG parser generator.
//! * [`earley`] — an Earley engine that runs the same PEG grammar text.
//! * [`grammars`] — embedded grammars: Loglan, Lojban syntax (camxes) and
//!   Lojban morphology.
//! * [`syntax`] — whole-text Lojban syntax check with either engine.
//! * [`morphology`] — engine-independent Lojban morphology (word split,
//!   classes, selma'o, standard/obsolete dialect).
//! * `cll` (feature `earley`) — vendored tergenzalvi CLL morphology.

#[cfg(feature = "earley")]
pub mod cll;
pub mod earley;
pub mod grammars;
pub mod morphology;
pub mod peg;
pub mod syntax;
