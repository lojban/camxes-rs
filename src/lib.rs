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
//! // Re-export Loglan grammar for external use
//! pub mod examples {
//!     // We need to make the example module public temporarily to access the grammar.
//!     // A better approach might be to move the grammar definition to a more central, public location.
//!     #![allow(dead_code)] // Allow unused items within the included file
//!     #![allow(unused_variables)] // Allow unused variables within the included file
//!     include!("examples/loglan.rs");
//!     pub use self::LOGLAN_GRAMMAR;
//! }
//!

pub mod grammars;
pub mod peg;