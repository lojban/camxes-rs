//! PEG engine adapter: camxes-style morphology grammar → [`Morphology`].

use super::{split_cmavo_cluster, Dialect, Morphology, MorphologyError, MorphWord, WordClass, WordDialect};
use crate::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
use crate::peg::grammar::Peg;
use crate::peg::parsing::ParseNode;

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

/// Read the word sequence off a `LOJBAN_MORPHOLOGY_GRAMMAR` parse tree (from
/// either engine). `input` is the original text (same byte offsets).
pub(super) fn words_from_tree(nodes: &[ParseNode], input: &str) -> Result<Vec<MorphWord>, (usize, String)> {
    let mut words = Vec::new();
    collect(nodes, input, &mut words)?;
    Ok(words)
}

fn collect(nodes: &[ParseNode], input: &str, out: &mut Vec<MorphWord>) -> Result<(), (usize, String)> {
    for node in nodes {
        let ParseNode::NonTerminal { name, children, .. } = node else {
            continue;
        };
        if name != "jbovla" {
            collect(children, input, out)?;
            continue;
        }
        let Some(ParseNode::NonTerminal { name: kind, span, .. }) = children
            .iter()
            .find(|c| matches!(c, ParseNode::NonTerminal { .. }))
        else {
            continue;
        };
        let (start, end) = (span.0, span.1.min(input.len()));
        let class = match kind.as_str() {
            "pause_0" => continue,
            "cmevla" => WordClass::Cmevla,
            "cmavo" => WordClass::Cmavo,
            "gismu" => WordClass::Gismu,
            "lujvo_core" => WordClass::Lujvo,
            "fuhivla" => WordClass::Fuhivla,
            other => return Err((start, format!("unexpected jbovla child `{other}`"))),
        };
        let surface = &input[start..end];
        if class == WordClass::Cmavo {
            // The grammar already yields one simple cmavo per node, but keep
            // the split for safety with trailing pause characters.
            for (s, e) in split_cmavo_cluster(surface) {
                out.push(word(input, start + s, start + e, class));
            }
        } else {
            out.push(word(input, start, end, class));
        }
    }
    Ok(())
}

fn word(input: &str, start: usize, end: usize, class: WordClass) -> MorphWord {
    MorphWord {
        text: input[start..end].to_string(),
        span: (start, end),
        class,
        selmaho: None,
        word_dialect: WordDialect::Standard,
    }
}
