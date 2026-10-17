//! CLL Chapter 21 syntax layer (Cargo feature `grammar`).
//!
//! Pipeline: morphology [`crate::cll::lex::segment_stream`] → lexeme (selma'o /
//! BRIVLA / CMENE) → Ch.19 DERIVED SI/SA/SU erasure → Earley over the Ch.21
//! EBNF compiled to a pure CFG.
//!
//! Normative grammar: `grammar-src/cll-21.md` (extracted to `cll21.ebnf`).
//! Camxes PEG is **not** used. Erasure is **not** in the formal CFG.

pub mod anaphora;
pub mod cfg;
pub mod cmavo;
pub mod earley;
pub mod ebnf;
pub mod erase;
pub mod lexeme;
pub mod parse;

pub use cmavo::{map_selmaho_subtype, normalize_cmavo, CmavoLexicon};
pub use ebnf::{compile_ebnf, CompileStats};
pub use lexeme::{
    lexeme_stream, quote_word_key, split_quote_regions, terminals_only, Lexeme, LexemeResult,
    QuoteRegion, Region, FOREIGN_TERMINAL,
};
pub use parse::{
    cll21_grammar, cll21_stats, lexeme, parse_as, parse_as_with_tree, parse_text, GrammarResult,
};
