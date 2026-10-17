//! Vendored copy of [tergenzalvi](https://github.com/lagleki/tergenzalvi)
//! (commit e07f176, same author; tergenzalvi is MIT OR Apache-2.0 and is
//! included here under the crate's MIT license): CLL Chapters 3–4
//! morphology (CFG/Earley shapes + constraint filters) and the Chapter 21
//! cmavo lexicon / syntax layer. Used by the `earley` feature; vendored so
//! that camxes-rs can be published without a git dependency. Only the
//! `constructions` and `web` modules and the binaries are left out.
//!
//! ## Local changes: camxes extensions (not CLL)
//! `Engine::Earley` must give the camxes word types, so the copy carries
//! changes marked "camxes extension (not CLL)" in the code:
//! - [`mod@cfg`] grammar: camxes cmavo shapes (vowel chains, `y` chains, `i`/`u`
//!   glides, consonant + vowel chains), vowel chains in brivla and names,
//!   `i`/`u` glides in fu'ivla links.
//! - [`constraints`]: slinku'i is the camxes test (`C` + rafsi string); lujvo
//!   are segmented by rafsi form with the camxes `CVVr` hyphen; the
//!   `la`/`lai`/`doi` ban in names (C19) is not applied; fu'ivla with a late
//!   cluster and "a cmavo followed by words is not a brivla" follow camxes.
//! - [`shapes`]: consonant-only fragments are names (camxes `zifcme`).
//! - [`grammar`] lexer: a digit is a `PA` cmavo (camxes `digit`).
//! - [`camxes`]: ports of camxes morphology rules (rafsi strings, syllables,
//!   `y`-hyphen lujvo, comma pauses, the obsolete-form rule) and the
//!   `MorphClass` → camxes word type mapping.
//!
//! Lojban morphology recognizer grounded in CLL Ch. 3–4.
//! Ch. 21 syntax layer in [`grammar`].
//!
//! ## Architecture
//! 1. [`phonology`] — letter classes, diphthongs, consonant pair/triple tables (CLL Ch. 3).
//! 2. [`shapes`] — CV helpers + **shape-layer recognition** ([`shapes::recognize()`]).
//! 3. [`mod@cfg`] — pure CFG overgenerator (Earley); **no lookaround**.
//! 4. [`constraints`] — CLL complement-style filters with section cites ([`constraints::morph_parse`]).
//! 5. [`lex`] — morph token types emitted toward Ch. 21 terminals (`BRIVLA`, `CMENE`, …).
//! 6. [`tables`] — optional external gismu/rafsi data (not invented here).
//!
//! ## Shape layer (working)
//! ```rust
//! use camxes_rs::cll::shapes::{recognize, MorphClass};
//! let r = recognize("creka");
//! assert!(r.accepted);
//! assert!(r.classes.contains(&MorphClass::Gismu));
//! ```
//! See the repository README for the shape and constraint layers.
//!
//! ## Normative boundary
//! - **In:** CLL Chapters 3 and 4.
//! - **Syntax layer:** CLL Chapter 21 EBNF syntax over lexemes.
//! - **Out:** camxes PEG, vlazba peg extras, BPFK folklore beyond CLL.

pub mod camxes;
pub mod cfg;
pub mod constraints;
pub mod lex;
pub mod phonology;
pub mod shapes;
pub mod tables;

pub mod grammar;

/// Public pipeline entry points.
pub mod pipeline {
    use crate::cll::cfg::Forest;
    use crate::cll::constraints::{self, ConstrainedResult, ConstraintId};
    use crate::cll::lex::{Token, WordClass};
    use crate::cll::shapes::{recognize, MorphClass, ShapeResult};

    pub use crate::cll::constraints::ConstrainedResult as Constrained;

    /// Shape-layer morph parse: overgenerate candidate word classes.
    ///
    /// Does **not** apply constraint-layer filters (tosmabru, slinku'i, la-ban, …).
    pub fn shape_parse(input: &str) -> Result<Vec<Token>, MorphError> {
        let r = recognize(input);
        if !r.accepted {
            return Err(MorphError::ShapeRejected);
        }
        let tokens: Vec<Token> = r
            .classes
            .iter()
            .filter_map(|c| c.to_word_class())
            .map(|class| Token::Word {
                class,
                text: r.normalized.clone(),
            })
            .collect();
        if tokens.is_empty() {
            return Ok(vec![]);
        }
        Ok(tokens)
    }

    /// Full morphology parse: shape overgenerate + constraint filters.
    pub fn morph_parse(input: &str) -> ConstrainedResult {
        constraints::morph_parse(input)
    }

    /// Full morphology parse with [`crate::cll::constraints::MorphOptions`].
    pub fn morph_parse_with(
        input: &str,
        opts: crate::cll::constraints::MorphOptions,
    ) -> ConstrainedResult {
        constraints::morph_parse_with(input, opts)
    }

    /// Orthographic / best-effort stream segmentation (§4.1 / §4.9).
    pub fn segment_stream(input: &str) -> crate::cll::lex::StreamResult {
        crate::cll::lex::segment_stream(input)
    }

    /// Apply constraints to an existing shape result.
    pub fn constrain(original: &str, shape: &ShapeResult) -> ConstrainedResult {
        constraints::constrain(original, shape)
    }

    /// Shape recognition → candidate forest (one span, multiple classes).
    pub fn shape_forest(input: &str) -> Forest {
        let r = recognize(input);
        Forest::from_classes(&r.normalized, &r.classes)
    }

    /// Accept at shape layer?
    pub fn shape_accept(input: &str) -> bool {
        recognize(input).accepted
    }

    /// Accept at constraint layer?
    pub fn morph_accept(input: &str) -> bool {
        constraints::morph_parse(input).accepted
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum MorphError {
        ShapeRejected,
        Unimplemented(&'static str),
        /// CLL §4.1: detailed break algorithm not fully debugged in the book.
        Underspecified(&'static str),
        ConstraintRejected {
            rule: &'static str,
            cll: &'static str,
        },
    }

    /// Preferred unique word class after constraints; `None` if zero or many.
    pub fn unique_word_class(input: &str) -> Option<WordClass> {
        let r = constraints::morph_parse(input);
        let word: Vec<WordClass> = r
            .classes
            .into_iter()
            .filter_map(MorphClass::to_word_class)
            .collect();
        if word.len() == 1 {
            Some(word[0])
        } else {
            None
        }
    }

    /// Re-export for callers matching rejected_by lists.
    pub type RejectId = ConstraintId;
}
