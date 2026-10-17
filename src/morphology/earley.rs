//! Earley engine adapter.
//!
//! 1. **camxes dialect:** the same [`LOJBAN_MORPHOLOGY_GRAMMAR`] the PEG engine
//!    uses, run on the Earley engine ([`crate::earley`]). When its
//!    PEG-preferred derivation covers the input, the word split and classes
//!    are read off the tree exactly like the PEG adapter does.
//! 2. **CLL dialect:** otherwise tergenzalvi's CLL Chapter 3–4 morphology
//!    (CFG/Earley shapes + constraint filters) classifies each piece; pieces
//!    are split at whitespace and written pauses, and continuous text is
//!    segmented by tergenzalvi's stream segmenter. This accepts the older
//!    dialect that camxes rejects.
//!    Each word of such a result gets a rule-based [`WordDialect`]: obsolete
//!    unless the camxes grammar reads that word alone with the same class.
//! 3. **camxes, complete derivation:** last resort, any camxes-grammar
//!    derivation spanning the whole input.

use super::peg::words_from_tree;
use super::{
    split_cmavo_cluster, Dialect, MorphWord, Morphology, MorphologyError, WordClass, WordDialect,
};
use crate::cll::constraints::ConstrainedResult;
use crate::cll::lex::{morph_parse_stream_piece, segment_continuous};
use crate::cll::shapes::MorphClass;
use crate::earley::EarleyGrammar;
use crate::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
use std::sync::OnceLock;

/// The camxes morphology grammar compiled for the Earley engine.
pub fn camxes_morphology_grammar() -> &'static EarleyGrammar {
    static G: OnceLock<EarleyGrammar> = OnceLock::new();
    G.get_or_init(|| {
        EarleyGrammar::new(LOJBAN_MORPHOLOGY_GRAMMAR.0, LOJBAN_MORPHOLOGY_GRAMMAR.1)
            .expect("embedded morphology grammar compiles")
    })
}

fn err(position: usize, message: impl Into<String>) -> MorphologyError {
    MorphologyError {
        engine: "earley",
        position,
        message: message.into(),
    }
}

pub(super) fn analyze(input: &str) -> Result<Morphology, MorphologyError> {
    let g = camxes_morphology_grammar();
    let mut text = input.to_ascii_lowercase();
    text.push(' ');
    let preferred = g.parse(&text);
    if let Ok(nodes) = preferred.2.as_ref() {
        if preferred.1 == text.len() {
            if let Ok(words) = words_from_tree(nodes, input) {
                if !words.is_empty() {
                    return Ok(Morphology {
                        words,
                        dialect: Dialect::Camxes,
                    });
                }
            }
        }
    }
    let cll = analyze_cll(input);
    if cll.is_ok() {
        return cll;
    }
    let complete = g.parse_complete(&text);
    if let Ok(nodes) = complete.2.as_ref() {
        if let Ok(words) = words_from_tree(nodes, input) {
            if !words.is_empty() {
                return Ok(Morphology {
                    words,
                    dialect: Dialect::CamxesComplete,
                });
            }
        }
    }
    cll
}

/// Choose one word class for an accepted result, in camxes' order of
/// preference (cmevla, cmavo, gismu, fu'ivla, lujvo: camxes' lujvo rule
/// requires `!fuhivla`).
fn class_of(r: &ConstrainedResult) -> Option<WordClass> {
    if !r.accepted {
        return None;
    }
    r.classes
        .iter()
        .filter_map(|c| match c {
            MorphClass::Cmevla => Some(WordClass::Cmevla),
            MorphClass::Cmavo | MorphClass::CmavoCompound => Some(WordClass::Cmavo),
            MorphClass::Gismu => Some(WordClass::Gismu),
            MorphClass::Lujvo => Some(WordClass::Lujvo),
            MorphClass::Fuhivla => Some(WordClass::Fuhivla),
            _ => None,
        })
        .min_by_key(|c| match c {
            WordClass::Cmevla => 0,
            WordClass::Cmavo => 1,
            WordClass::Gismu => 2,
            WordClass::Fuhivla => 3,
            WordClass::Lujvo => 4,
        })
}

/// CLL-dialect analysis with tergenzalvi.
pub(super) fn analyze_cll(input: &str) -> Result<Morphology, MorphologyError> {
    // The CLL (`crate::cll`) orthography normalizer drops unknown characters; reject
    // them here so that e.g. `xa!` is not silently read as `xa`.
    if let Some((pos, ch)) = input.char_indices().find(|&(_, c)| {
        !(c.is_ascii_alphabetic() || c.is_ascii_whitespace() || matches!(c, '\'' | ',' | '.'))
    }) {
        return Err(err(
            pos,
            format!("character {ch:?} is not Lojban orthography"),
        ));
    }
    let mut words = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() || bytes[i] == b'.' {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'.' {
            i += 1;
        }
        let lead_pause = start > 0 && bytes[start - 1] == b'.';
        let trail_pause = i < bytes.len() && bytes[i] == b'.';
        cll_piece(input, start, i, lead_pause, trail_pause, &mut words)?;
    }
    if words.is_empty() {
        return Err(err(0, "no words"));
    }
    for w in &mut words {
        w.word_dialect = word_dialect(&w.text, w.class);
    }
    Ok(Morphology {
        words,
        dialect: Dialect::Cll,
    })
}

fn cll_piece(
    input: &str,
    start: usize,
    end: usize,
    lead_pause: bool,
    trail_pause: bool,
    out: &mut Vec<MorphWord>,
) -> Result<(), MorphologyError> {
    // Spans come from the parser and sit on character boundaries.
    let body = input.get(start..end).unwrap_or_default();
    // Whole piece as one word, using any written pause evidence.
    let mut spelled = String::with_capacity(body.len() + 2);
    if lead_pause {
        spelled.push('.');
    }
    spelled.push_str(body);
    if trail_pause {
        spelled.push('.');
    }
    let mut whole = morph_parse_stream_piece(&spelled);
    if !whole.accepted && spelled.len() != body.len() {
        whole = morph_parse_stream_piece(body);
    }
    if let Some(class) = class_of(&whole) {
        push(input, start, end, class, out);
        return Ok(());
    }
    // Continuous segmentation of the piece.
    let seg = segment_continuous(body);
    let mut cursor = start;
    let mut local = Vec::new();
    for tok in &seg.tokens {
        let Some(class) = class_of(&tok.result) else {
            return Err(err(
                cursor,
                format!("no morphology for `{}` in `{body}`", tok.text),
            ));
        };
        let rel = input[cursor..end]
            .find(tok.text.as_str())
            .ok_or_else(|| err(cursor, format!("token `{}` not found in input", tok.text)))?;
        let s = cursor + rel;
        let e = s + tok.text.len();
        push(input, s, e, class, &mut local);
        cursor = e;
    }
    if local.is_empty() || cursor != end {
        return Err(err(start, format!("no morphology for `{body}`")));
    }
    out.extend(local);
    Ok(())
}

/// Rule-based dialect of one CLL-accepted word: standard when the camxes
/// grammar alone reads it as exactly this one word of this class, obsolete
/// otherwise.
fn word_dialect(word: &str, class: WordClass) -> WordDialect {
    let g = camxes_morphology_grammar();
    let mut text = word.to_ascii_lowercase();
    text.push(' ');
    let r = g.parse(&text);
    let standard = matches!(r.2.as_ref(), Ok(nodes) if r.1 == text.len()
        && matches!(words_from_tree(nodes, word).as_deref(),
            Ok([w]) if w.class == class && w.text.eq_ignore_ascii_case(word)));
    if standard {
        WordDialect::Standard
    } else {
        WordDialect::Obsolete
    }
}

fn push(input: &str, start: usize, end: usize, class: WordClass, out: &mut Vec<MorphWord>) {
    let surface = input.get(start..end).unwrap_or_default();
    let spans = if class == WordClass::Cmavo {
        split_cmavo_cluster(surface)
    } else {
        vec![(0, surface.len())]
    };
    for (s, e) in spans {
        out.push(MorphWord {
            text: input[start + s..start + e].to_string(),
            span: (start + s, start + e),
            class,
            selmaho: None,
            word_dialect: WordDialect::Standard,
        });
    }
}
