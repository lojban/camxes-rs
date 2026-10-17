//! Shared by both engine adapters: the word sequence of a
//! `LOJBAN_MORPHOLOGY_GRAMMAR` parse tree.

use super::{split_cmavo_cluster, MorphWord, WordClass, WordDialect};
use crate::peg::parsing::ParseNode;

/// Read the word sequence off a `LOJBAN_MORPHOLOGY_GRAMMAR` parse tree (from
/// either engine). `input` is the original text (same byte offsets).
pub(super) fn words_from_tree(
    nodes: &[ParseNode],
    input: &str,
) -> Result<Vec<MorphWord>, (usize, String)> {
    let mut words = Vec::new();
    collect(nodes, input, &mut words)?;
    Ok(words)
}

fn collect(
    nodes: &[ParseNode],
    input: &str,
    out: &mut Vec<MorphWord>,
) -> Result<(), (usize, String)> {
    for node in nodes {
        let ParseNode::NonTerminal { name, children, .. } = node else {
            continue;
        };
        if name != "jbovla" {
            collect(children, input, out)?;
            continue;
        }
        let Some(ParseNode::NonTerminal {
            name: kind, span, ..
        }) = children
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
        let Some(surface) = input.get(start..end) else {
            return Err((start, format!("bad span {start}..{end}")));
        };
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
        text: input.get(start..end).unwrap_or_default().to_string(),
        span: (start, end),
        class,
        selmaho: None,
        word_dialect: WordDialect::Standard,
    }
}
