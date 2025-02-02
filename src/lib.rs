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

pub mod peg;
