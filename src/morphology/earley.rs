//! Earley engine adapter: tergenzalvi's CLL Chapter 3–4 morphology (vendored
//! as [`crate::cll`]): CFG/Earley shapes, constraint filters and stream
//! segmentation.
//!
//! 1. The text is split at whitespace and written pauses (`.`); a comma is
//!    a syllable break inside a word (CLL §3.3), not a word boundary.
//! 2. Each piece is first read as one word ([`morph_parse_stream_piece`]),
//!    otherwise segmented by the stream segmenter ([`segment_continuous`]).
//! 3. Each word gets the camxes word type ([`camxes::word_class`], a mapping
//!    only) and a rule-based [`WordDialect`] ([`camxes::is_obsolete_form`]).
//!
//! Nothing here changes what the CLL pipeline accepts: `crate::cll` is the
//! same code as tergenzalvi.

use super::camxes;
use super::{
    split_cmavo_cluster, Dialect, MorphWord, Morphology, MorphologyError, WordClass, WordDialect,
};
use crate::cll::constraints::ConstrainedResult;
use crate::cll::lex::{morph_parse_stream_piece, segment_continuous, WordClass as CllClass};

fn err(position: usize, message: impl Into<String>) -> MorphologyError {
    MorphologyError {
        engine: "earley",
        position,
        message: message.into(),
    }
}

/// tergenzalvi `MorphClass` → camxes word type (via [`camxes::word_class`]).
fn class_of(r: &ConstrainedResult, text: &str) -> Option<WordClass> {
    Some(match camxes::word_class(r, text)? {
        CllClass::Cmavo => WordClass::Cmavo,
        CllClass::Gismu => WordClass::Gismu,
        CllClass::Lujvo => WordClass::Lujvo,
        CllClass::Fuhivla => WordClass::Fuhivla,
        CllClass::Cmevla => WordClass::Cmevla,
    })
}

pub(super) fn analyze(input: &str) -> Result<Morphology, MorphologyError> {
    // The CLL orthography normalizer drops unknown characters; reject them
    // here so that e.g. `xa!` is not silently read as `xa`.
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
        piece(input, start, i, lead_pause, trail_pause, &mut words)?;
    }
    if words.is_empty() {
        return Err(err(0, "no words"));
    }
    for w in &mut words {
        if camxes::is_obsolete_form(&w.text, w.class == WordClass::Cmevla) {
            w.word_dialect = WordDialect::Obsolete;
        }
    }
    Ok(Morphology {
        words,
        dialect: Dialect::Cll,
    })
}

fn piece(
    input: &str,
    start: usize,
    end: usize,
    lead_pause: bool,
    trail_pause: bool,
    out: &mut Vec<MorphWord>,
) -> Result<(), MorphologyError> {
    // Byte offsets here sit on ASCII characters (checked in `analyze`).
    let body = &input[start..end];
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
    if let Some(class) = class_of(&whole, body) {
        push(input, start, end, class, out);
        return Ok(());
    }
    // Continuous segmentation of the piece.
    let seg = segment_continuous(body);
    let mut cursor = start;
    let mut local = Vec::new();
    for tok in &seg.tokens {
        let Some(class) = class_of(&tok.result, &tok.text) else {
            return Err(err(
                cursor,
                format!("no morphology for `{}` in `{body}`", tok.text),
            ));
        };
        let rel = input[cursor..end]
            .to_ascii_lowercase()
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

fn push(input: &str, start: usize, end: usize, class: WordClass, out: &mut Vec<MorphWord>) {
    let surface = &input[start..end];
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
