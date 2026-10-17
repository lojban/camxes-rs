//! Candidate parse forest / span types (engine-agnostic).
//!
//! Earley recognition yields a set of matching shape nonterminals; constraints
//! later consume [`CandidateSpan`] values.

use crate::cll::lex::WordClass;
use crate::cll::shapes::MorphClass;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateSpan {
    pub start: usize,
    pub end: usize,
    pub proposed_class: WordClass,
    /// Raw slice into the (possibly pause-normalized) input.
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct Forest {
    pub candidates: Vec<CandidateSpan>,
}

impl Forest {
    pub fn empty() -> Self {
        Self {
            candidates: Vec::new(),
        }
    }

    pub fn from_classes(text: &str, classes: &BTreeSet<MorphClass>) -> Self {
        let candidates = classes
            .iter()
            .filter_map(|c| {
                c.to_word_class().map(|wc| CandidateSpan {
                    start: 0,
                    end: text.len(),
                    proposed_class: wc,
                    text: text.to_string(),
                })
            })
            .collect();
        Self { candidates }
    }
}
