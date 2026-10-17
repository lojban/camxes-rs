//! PEG engine adapter: camxes-style morphology grammar → [`Morphology`].

use super::tree::words_from_tree;
use super::{Dialect, Morphology, MorphologyError};
use crate::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
use crate::peg::grammar::Peg;

thread_local! {
    // `Peg` keeps its memo table in a `RefCell`, so build one per thread.
    static PARSER: Peg = Peg::new(LOJBAN_MORPHOLOGY_GRAMMAR.0, LOJBAN_MORPHOLOGY_GRAMMAR.1)
        .expect("embedded morphology grammar is valid");
}

fn err(position: usize, message: impl Into<String>) -> MorphologyError {
    MorphologyError {
        engine: "peg",
        position,
        message: message.into(),
    }
}

pub(super) fn analyze(input: &str) -> Result<Morphology, MorphologyError> {
    // The grammar is lowercase-only and needs a pause after the last word to
    // see penultimate stress. ASCII lowercasing keeps byte offsets stable.
    let mut text = input.to_ascii_lowercase();
    text.push(' ');
    let parsed = PARSER.with(|p| p.parse(&text));
    let nodes = match parsed.2.as_ref() {
        Ok(nodes) => nodes,
        Err(e) => return Err(err(e.position.min(input.len()), e.to_string())),
    };
    if parsed.1 < text.len() {
        return Err(err(parsed.1.min(input.len()), "input not fully consumed"));
    }
    let words = words_from_tree(nodes, input).map_err(|(pos, msg)| err(pos, msg))?;
    if words.is_empty() {
        return Err(err(0, "no words"));
    }
    Ok(Morphology {
        words,
        dialect: Dialect::Camxes,
    })
}
