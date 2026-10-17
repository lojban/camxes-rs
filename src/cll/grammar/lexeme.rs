//! Morphology words → CLL Ch.21 terminals (BRIVLA / CMENE / selma'o).
//!
//! Also implements the Ch.21 **non-formal** `word` rule at the lexeme layer:
//! `[BAhE] any-word [indicators]` — BAhE and indicators are not CFG terminals
//! mid-stream; they attach to the neighbouring word (DERIVED realization of
//! the non-formal rule). **Magic words / quotes** (ZO, ZOI, LU…LIhU, LOhU…LEhU)
//! outrank this strip: the quoted word(s) are kept even when they are UI/BAhE/…
//! (CLL Ch.6 / Ch.19; DERIVED refinement of non-formal `word`).
//!
//! After that, [`super::erase::apply_erasure`] runs the Ch.19 **DERIVED**
//! metalinguistic SI/SA/SU eraser on the lexeme stream (before Earley). The
//! formal Ch.21 CFG is unchanged regarding erasure.

use super::cmavo::CmavoLexicon;
use crate::cll::lex::stream::{segment_stream, StreamToken};
use crate::cll::shapes::MorphClass;

/// One Ch.21 terminal with provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexeme {
    pub surface: String,
    /// Ch.21 terminal name: `BRIVLA`, `CMENE`, or selma'o (`KOhA`, `LE`, …)
    pub terminal: String,
    pub note: Option<String>,
    /// Indicators attached by the non-formal Ch.21 word rule. They are
    /// preserved for display although they do not add CFG terminals.
    pub attached_indicators: Vec<(String, String)>,
    /// BAhE emphasis words attached to this following word by the same rule.
    pub attached_emphasis: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexemeResult {
    pub lexemes: Vec<Lexeme>,
    pub gaps: Vec<String>,
    pub morph_notes: Vec<String>,
}

/// Ch.21 terminal of a ZOI / LA'O quoted body (CLL Ch.21 `anything`).
///
/// The pre-lex quote step ([`split_quote_regions`]) hands the whole foreign
/// text between the two delimiters to the grammar as this single opaque
/// lexeme, so the body never goes through Lojban morphology and can never be
/// mistaken for the closing delimiter.
pub const FOREIGN_TERMINAL: &str = "FOREIGN";

/// Segment then assign Ch.21 terminals via cmavo lexicon + morph classes.
///
/// ZOI / LA'O quotes are cut out of the text first ([`split_quote_regions`]):
/// the body is opaque foreign text and the closing delimiter must be the same
/// word as the opening one (CLL 19.10, normalized like camxes: case, pauses
/// and commas ignored, `h` ≡ `'`). An unclosed quote yields a `GAP` lexeme,
/// so the syntax layer refuses it.
pub fn lexeme_stream(input: &str) -> LexemeResult {
    let lex = CmavoLexicon::global();
    let mut gaps = Vec::new();
    let mut morph_notes = Vec::new();
    let mut lexemes = Vec::new();

    for region in split_quote_regions(input) {
        match region {
            Region::Lojban(text) => {
                lexemes.extend(lex_lojban_region(text, lex, &mut gaps, &mut morph_notes));
            }
            Region::Quote(quote) => {
                lexemes.extend(lex_quote(&quote, lex, &mut gaps, &mut morph_notes));
            }
        }
    }

    // Ch.21 non-formal word rule: omit BAhE / indicators from CFG terminals;
    // attached indicators remain in Lexeme metadata for the caller.
    let (lexemes, strip_notes) = apply_word_layer(lexemes);
    morph_notes.extend(strip_notes);

    // Ch.19 non-formal erasure (DERIVED): SI/SA/SU on the lexeme stream.
    let (lexemes, erase_notes) = super::erase::apply_erasure(lexemes);
    morph_notes.extend(erase_notes);

    LexemeResult {
        lexemes,
        gaps,
        morph_notes,
    }
}

/// Morphology + selma'o assignment for a quote-free stretch of text.
fn lex_lojban_region(
    text: &str,
    lex: &CmavoLexicon,
    gaps: &mut Vec<String>,
    morph_notes: &mut Vec<String>,
) -> Vec<Lexeme> {
    let seg = segment_stream(text);
    morph_notes.extend(seg.ambiguity.iter().cloned());

    // Each group is lexemes from one stream token, plus whether the following
    // boundary was explicit (space/`.` — do not re-fuse across those).
    let mut groups: Vec<(Vec<Lexeme>, bool)> = Vec::new();
    for tok in &seg.tokens {
        // Each digit is a PA cmavo, so `li 1995` is a number. CLL is silent
        // on digits in Lojban text; Ch. 18 reads numerals digit by digit.
        if !tok.text.is_empty() && tok.text.bytes().all(|b| b.is_ascii_digit()) {
            let piece = tok
                .text
                .chars()
                .map(|d| {
                    plain_lexeme(
                        &d.to_string(),
                        "PA",
                        Some("digit = PA (CLL silent; Ch. 18 digit-by-digit reading)".into()),
                    )
                })
                .collect();
            groups.push((piece, tok.explicit_boundary));
            continue;
        }
        // A token with no letters (`-`, `…` in CLL example text) is
        // punctuation, not a word: skip it without a gap.
        if !tok.text.bytes().any(|b| b.is_ascii_alphabetic()) {
            morph_notes.push(format!("skipped non-word token {:?}", tok.text));
            continue;
        }
        if !tok.result.accepted {
            gaps.push(format!(
                "morph rejected {:?}: rejected_by={:?}",
                tok.text, tok.result.rejected_by
            ));
            continue;
        }
        let mut piece = Vec::new();
        assign_token(tok, lex, &mut piece, gaps, morph_notes);
        groups.push((piece, tok.explicit_boundary));
    }

    // Re-fuse over-split atomic cmavo only inside continuous spans (`co`+`i` →
    // `coi`), never across explicit whitespace/pause boundaries (`do`+`.i` ↛ `doi`).
    let (lexemes, refuse_notes) = refuse_atomic_cmavo_grouped(groups, lex);
    morph_notes.extend(refuse_notes);
    lexemes
}

/// A stretch of input text: ordinary Lojban, or one ZOI / LA'O quote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Region<'a> {
    /// Text lexed by the ordinary morphology pipeline.
    Lojban(&'a str),
    /// `zoi X. … .X` / `la'o X. … .X`.
    Quote(QuoteRegion<'a>),
}

/// A ZOI / LA'O quote found by [`split_quote_regions`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteRegion<'a> {
    /// The quote marker as written (`zoi`, `la'o`, `LA'O`, …).
    pub marker: &'a str,
    /// Opening delimiter word as written (pauses excluded), if any.
    pub open: Option<&'a str>,
    /// Foreign text between the delimiters (trimmed of surrounding pauses).
    pub body: &'a str,
    /// Closing delimiter as written; `None` when the quote is unclosed.
    pub close: Option<&'a str>,
}

/// Normalized form used to compare quote markers and delimiters: lower case,
/// commas dropped, `h` and typographic apostrophes folded into `'` (camxes
/// treats `h` and `'` alike). Pauses are already excluded by tokenization.
pub fn quote_word_key(word: &str) -> String {
    word.chars()
        .filter(|c| *c != ',')
        .map(|c| match c.to_ascii_lowercase() {
            'h' | '\u{2019}' | '\u{2018}' => '\'',
            other => other,
        })
        .collect()
}

fn is_pause_or_space(c: char) -> bool {
    c.is_whitespace() || c == '.'
}

/// Byte spans of pause/space-separated words.
fn raw_words(input: &str) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut start = None;
    for (index, c) in input.char_indices() {
        if is_pause_or_space(c) {
            if let Some(s) = start.take() {
                words.push((s, index));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(s) = start {
        words.push((s, input.len()));
    }
    words
}

/// Pre-lex quote step (CLL 19.10): cut ZOI / LA'O quotes out of the text.
///
/// A quote marker is a pause/space-separated word whose [`quote_word_key`] is
/// `zoi` or `la'o`, unless it is itself quoted by a preceding `zo` or sits
/// inside `lo'u … le'u`. The next word is the delimiter; the quote ends at the
/// **first** later word with the same key, so the body can never contain the
/// delimiter as a separate word.
pub fn split_quote_regions(input: &str) -> Vec<Region<'_>> {
    let words = raw_words(input);
    let mut regions = Vec::new();
    let mut lojban_start = 0;
    let mut index = 0;
    let mut zo_quoted = false;
    let mut in_lohu = false;
    while index < words.len() {
        let (start, end) = words[index];
        let key = quote_word_key(&input[start..end]);
        if zo_quoted {
            zo_quoted = false;
            index += 1;
            continue;
        }
        if in_lohu {
            in_lohu = key != "le'u";
            index += 1;
            continue;
        }
        match key.as_str() {
            "zo" => zo_quoted = true,
            "lo'u" => in_lohu = true,
            "zoi" | "la'o" => {
                if lojban_start < start {
                    regions.push(Region::Lojban(&input[lojban_start..start]));
                }
                let marker = &input[start..end];
                let Some(&(open_start, open_end)) = words.get(index + 1) else {
                    regions.push(Region::Quote(QuoteRegion {
                        marker,
                        open: None,
                        body: "",
                        close: None,
                    }));
                    return regions;
                };
                let delimiter = quote_word_key(&input[open_start..open_end]);
                let close = words[index + 2..]
                    .iter()
                    .position(|&(s, e)| quote_word_key(&input[s..e]) == delimiter)
                    .map(|offset| index + 2 + offset);
                let body_end = close.map_or(input.len(), |c| words[c].0);
                let body = input[open_end..body_end].trim_matches(is_pause_or_space);
                regions.push(Region::Quote(QuoteRegion {
                    marker,
                    open: Some(&input[open_start..open_end]),
                    body,
                    close: close.map(|c| &input[words[c].0..words[c].1]),
                }));
                let Some(close) = close else {
                    return regions;
                };
                lojban_start = words[close].1;
                index = close + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    if lojban_start < input.len()
        && !input[lojban_start..]
            .trim_matches(is_pause_or_space)
            .is_empty()
    {
        regions.push(Region::Lojban(&input[lojban_start..]));
    }
    regions
}

fn plain_lexeme(surface: &str, terminal: &str, note: Option<String>) -> Lexeme {
    Lexeme {
        surface: surface.to_string(),
        terminal: terminal.to_string(),
        note,
        attached_indicators: vec![],
        attached_emphasis: vec![],
    }
}

/// Lexemes for one quote: `ZOI`, delimiter, [`FOREIGN_TERMINAL`] body,
/// delimiter. Problems (no delimiter, non-word delimiter, unclosed quote)
/// become `GAP` lexemes so the syntax layer refuses the text.
fn lex_quote(
    quote: &QuoteRegion<'_>,
    lex: &CmavoLexicon,
    gaps: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> Vec<Lexeme> {
    let marker = if quote_word_key(quote.marker) == "zoi" {
        "zoi"
    } else {
        "la'o"
    };
    let mut out = vec![plain_lexeme(
        marker,
        lex.selmaho(marker).unwrap_or("ZOI"),
        None,
    )];
    let Some(open) = quote.open else {
        gaps.push(format!("{marker} without delimiter word"));
        out.push(plain_lexeme(
            "",
            "GAP",
            Some("ZOI: missing delimiter".into()),
        ));
        return out;
    };
    let mut word_gaps = Vec::new();
    let delimiter = lex_lojban_region(open, lex, &mut word_gaps, notes);
    let delimiter = match (delimiter.as_slice(), word_gaps.is_empty()) {
        ([word], true) if word.terminal != "GAP" && word.terminal != "UNKNOWN_CMAVO" => {
            word.clone()
        }
        _ => {
            gaps.push(format!(
                "{marker} delimiter {open:?} is not a single Lojban word"
            ));
            gaps.extend(word_gaps);
            plain_lexeme(
                open,
                "GAP",
                Some("ZOI: delimiter is not one Lojban word".into()),
            )
        }
    };
    out.push(delimiter.clone());
    match quote.close {
        Some(_) => {
            notes.push(format!(
                "CLL 19.10 {marker}: quote closed by matching delimiter {:?}",
                quote_word_key(open)
            ));
            out.push(plain_lexeme(
                quote.body,
                FOREIGN_TERMINAL,
                Some("foreign text".into()),
            ));
            out.push(delimiter);
        }
        None => {
            gaps.push(format!(
                "{marker} quote never closed by delimiter {:?}",
                quote_word_key(open)
            ));
            out.push(plain_lexeme(
                quote.body,
                "GAP",
                Some("ZOI: unclosed quote".into()),
            ));
        }
    }
    out
}

fn is_indicator_selmaho(sm: &str) -> bool {
    matches!(sm, "UI" | "CAI" | "Y" | "DAhO" | "FUhO" | "FUhE")
}

/// Re-fuse C01 over-splits within continuous (non-explicit) spans only.
fn refuse_atomic_cmavo_grouped(
    groups: Vec<(Vec<Lexeme>, bool)>,
    lexicon: &CmavoLexicon,
) -> (Vec<Lexeme>, Vec<String>) {
    let mut notes = Vec::new();
    let mut out: Vec<Lexeme> = Vec::new();
    // Build runs of lexemes separated by explicit boundaries.
    let mut run: Vec<Lexeme> = Vec::new();
    let flush_run = |run: &mut Vec<Lexeme>, out: &mut Vec<Lexeme>, notes: &mut Vec<String>| {
        if run.is_empty() {
            return;
        }
        let fused = refuse_run(std::mem::take(run), lexicon, notes);
        out.extend(fused);
    };
    for (piece, explicit_boundary) in groups {
        // First token of a stream piece: if previous boundary was explicit,
        // flush before appending. The `explicit_boundary` flag on a token means
        // *this* token started after an explicit break.
        if explicit_boundary {
            flush_run(&mut run, &mut out, &mut notes);
        }
        run.extend(piece);
    }
    flush_run(&mut run, &mut out, &mut notes);
    (out, notes)
}

fn refuse_run(raw: Vec<Lexeme>, lexicon: &CmavoLexicon, notes: &mut Vec<String>) -> Vec<Lexeme> {
    if raw.is_empty() {
        return raw;
    }
    let mut out: Vec<Lexeme> = Vec::with_capacity(raw.len());
    let mut lexeme_index = 0;
    while lexeme_index < raw.len() {
        let mut best_match_end = lexeme_index;
        let mut best_selmaho: Option<String> = None;
        let mut compound_text = String::new();
        let mut lookahead_index = lexeme_index;
        while lookahead_index < raw.len() {
            let terminal = raw[lookahead_index].terminal.as_str();
            if terminal == "BRIVLA"
                || terminal == "CMENE"
                || terminal == "GAP"
                || terminal == "UNKNOWN_CMAVO"
            {
                break;
            }
            compound_text.push_str(&raw[lookahead_index].surface);
            if let Some(sm) = lexicon.selmaho(&compound_text) {
                best_match_end = lookahead_index;
                best_selmaho = Some(sm.to_string());
            }
            if compound_text.chars().count() > 12 {
                break;
            }
            lookahead_index += 1;
        }
        if let Some(sm) = best_selmaho {
            if best_match_end > lexeme_index {
                let fused: String = raw[lexeme_index..=best_match_end]
                    .iter()
                    .map(|lexeme| lexeme.surface.as_str())
                    .collect();
                notes.push(format!(
                    "DERIVED: re-fuse over-split cmavo {:?} → {} ({sm})",
                    raw[lexeme_index..=best_match_end]
                        .iter()
                        .map(|lexeme| lexeme.surface.as_str())
                        .collect::<Vec<_>>()
                        .join("+"),
                    fused
                ));
                out.push(Lexeme {
                    surface: fused,
                    terminal: sm,
                    note: Some("lexicon-refuse".into()),
                    attached_indicators: vec![],
                    attached_emphasis: vec![],
                });
                lexeme_index = best_match_end + 1;
                continue;
            }
        }
        out.push(raw[lexeme_index].clone());
        lexeme_index += 1;
    }
    out
}

/// Attach BAhE and post-word indicators per Ch.21 non-formal `word` /
/// `indicators`, with **magic-word / quotation priority** (CLL Ch.6 / Ch.19).
///
/// Leading indicators (before any content word) are kept so the formal `text`
/// prefix `(indicators & free …)` still matches.
///
/// Magic scopes outrank the non-formal word-layer (DERIVED refinement):
/// - **ZO**: the next word is quoted as-is (UI, BAhE, ZOI, SI, another ZO, …).
/// - **ZOI**: delimiter + body + close-delimiter are opaque (no strip).
/// - **LU … LIhU**: inner is `text` — word-layer runs recursively with a fresh
///   `seen_content` so leading UI of the quote is kept and does not attach to LU;
///   mid-stream indicators inside still strip (e.g. editorial `sa'a` on `sei`).
/// - **LOhU … LEhU**: inner is `any-word ...` — fully opaque (UI/BAhE kept).
fn apply_word_layer(raw: Vec<Lexeme>) -> (Vec<Lexeme>, Vec<String>) {
    let mut out = Vec::with_capacity(raw.len());
    let mut notes = Vec::new();
    let mut pending_emphasis = Vec::new();
    let mut seen_content = false;
    let mut lexeme_index = 0;
    while lexeme_index < raw.len() {
        let term = raw[lexeme_index].terminal.as_str();

        // --- Magic / quote scopes (outrank word-layer stripping) ---
        if term == "ZO" {
            push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
            seen_content = true;
            lexeme_index += 1;
            if lexeme_index < raw.len() {
                notes.push(format!(
                    "DERIVED Ch.6/19 magic: ZO quotes next word {}:{} (word-layer bypass)",
                    raw[lexeme_index].terminal, raw[lexeme_index].surface
                ));
                out.push(raw[lexeme_index].clone());
                lexeme_index += 1;
            } else {
                notes.push("DERIVED Ch.6/19 magic: ZO without following word".into());
            }
            continue;
        }

        if term == "ZOI" {
            // ZOI any-word anything any-word: the pre-lex quote step already
            // produced exactly delimiter + FOREIGN body + delimiter (or a GAP).
            push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
            seen_content = true;
            let quote_end = (lexeme_index + 4).min(raw.len());
            notes.push(format!(
                "DERIVED Ch.6/19 magic: ZOI opaque region {:?} (word-layer bypass)",
                raw[lexeme_index + 1..quote_end]
                    .iter()
                    .map(|lexeme| lexeme.surface.as_str())
                    .collect::<Vec<_>>()
            ));
            out.extend(raw[lexeme_index + 1..quote_end].iter().cloned());
            lexeme_index = quote_end;
            continue;
        }

        if term == "LU" {
            // Inner is Ch.21 `text`: word-layer still applies inside, but must
            // not treat UI/BAhE as attaching to LU itself (reset seen_content).
            notes.push(format!(
                "DERIVED Ch.6/19 magic: LU…LIhU inner text (word-layer reset); open {:?}",
                raw[lexeme_index].surface
            ));
            push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
            lexeme_index += 1;
            let (inner, close_index, inner_notes) = take_balanced(&raw, lexeme_index, "LU", "LIhU");
            notes.extend(inner_notes);
            let (inner_out, strip_notes) = apply_word_layer(inner);
            notes.extend(strip_notes);
            out.extend(inner_out);
            if close_index < raw.len() && raw[close_index].terminal == "LIhU" {
                out.push(raw[close_index].clone());
                lexeme_index = close_index + 1;
                notes.push("DERIVED Ch.6/19 magic: closed LU…LIhU".into());
            } else {
                lexeme_index = close_index;
                notes.push("DERIVED Ch.6/19 magic: unclosed LU…LIhU (GAP partial)".into());
            }
            seen_content = true;
            continue;
        }

        if term == "LOhU" {
            // Inner is `any-word ...`: every word (incl. UI/BAhE) must survive.
            notes.push(format!(
                "DERIVED Ch.6/19 magic: LOhU…LEhU opaque (word-layer bypass); open {:?}",
                raw[lexeme_index].surface
            ));
            push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
            seen_content = true;
            lexeme_index += 1;
            lexeme_index =
                emit_opaque_until(&raw, lexeme_index, "LOhU", "LEhU", &mut out, &mut notes);
            continue;
        }

        // --- Ordinary Ch.21 word-layer ---

        // BAhE emphasizes the following word — never a CFG terminal.
        if term == "BAhE" {
            notes.push(format!(
                "DERIVED Ch.21 word-layer: attach BAhE {:?} to following word",
                raw[lexeme_index].surface
            ));
            pending_emphasis.push((
                raw[lexeme_index].surface.clone(),
                raw[lexeme_index].terminal.clone(),
            ));
            lexeme_index += 1;
            continue;
        }

        // Mid-stream indicators (+ optional NAI) attach to the previous word.
        if seen_content && is_indicator_selmaho(term) {
            let mut lookahead_index = lexeme_index + 1;
            // indicator = (UI|CAI) [NAI] | Y | DAhO | FUhO ; FUhE starts a run
            if (term == "UI" || term == "CAI")
                && lookahead_index < raw.len()
                && raw[lookahead_index].terminal == "NAI"
            {
                lookahead_index += 1;
            }
            notes.push(format!(
                "DERIVED Ch.21 word-layer: attach indicator(s) {:?} to preceding word",
                raw[lexeme_index..lookahead_index]
                    .iter()
                    .map(|lexeme| lexeme.surface.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
            if let Some(previous) = out.last_mut() {
                previous.attached_indicators.extend(
                    raw[lexeme_index..lookahead_index]
                        .iter()
                        .map(|indicator| (indicator.surface.clone(), indicator.terminal.clone())),
                );
            }
            lexeme_index = lookahead_index;
            continue;
        }

        if !is_indicator_selmaho(term) && term != "NAI" {
            // NAI can be content (negation) elsewhere; only skip when glued above.
            seen_content = true;
        } else if !seen_content {
            // leading indicators: keep for text-prefix
        }

        push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
        lexeme_index += 1;
    }
    (out, notes)
}

fn push_with_emphasis(out: &mut Vec<Lexeme>, lexeme: &Lexeme, pending: &mut Vec<(String, String)>) {
    let mut lexeme = lexeme.clone();
    lexeme.attached_emphasis = std::mem::take(pending);
    out.push(lexeme);
}

/// Slice a balanced open…close region (exclusive of the open already consumed;
/// returns inner lexemes, index of closer or end, and notes). Nesting-aware.
fn take_balanced(
    raw: &[Lexeme],
    mut lexeme_index: usize,
    open: &str,
    close: &str,
) -> (Vec<Lexeme>, usize, Vec<String>) {
    let mut notes = Vec::new();
    let mut inner = Vec::new();
    let mut depth: usize = 1;
    while lexeme_index < raw.len() {
        let term = raw[lexeme_index].terminal.as_str();
        if term == open {
            depth += 1;
            inner.push(raw[lexeme_index].clone());
            lexeme_index += 1;
        } else if term == close {
            depth -= 1;
            if depth == 0 {
                return (inner, lexeme_index, notes);
            }
            inner.push(raw[lexeme_index].clone());
            lexeme_index += 1;
        } else {
            inner.push(raw[lexeme_index].clone());
            lexeme_index += 1;
        }
    }
    notes.push(format!(
        "DERIVED Ch.6/19 magic: unclosed {open}…{close} while taking balanced region"
    ));
    (inner, lexeme_index, notes)
}

/// Emit lexemes until the matching closer, nesting same-open/close pairs.
/// Magic ZO/ZOI inside the region are still emitted verbatim (already opaque).
fn emit_opaque_until(
    raw: &[Lexeme],
    mut lexeme_index: usize,
    open: &str,
    close: &str,
    out: &mut Vec<Lexeme>,
    notes: &mut Vec<String>,
) -> usize {
    let mut depth: usize = 1;
    while lexeme_index < raw.len() {
        let term = raw[lexeme_index].terminal.as_str();
        if term == open {
            depth += 1;
        } else if term == close {
            depth -= 1;
            if depth == 0 {
                out.push(raw[lexeme_index].clone());
                lexeme_index += 1;
                notes.push(format!(
                    "DERIVED Ch.6/19 magic: closed {open}…{close} opaque region"
                ));
                return lexeme_index;
            }
        }
        out.push(raw[lexeme_index].clone());
        lexeme_index += 1;
    }
    notes.push(format!(
        "DERIVED Ch.6/19 magic: unclosed {open}…{close} opaque region (GAP partial)"
    ));
    lexeme_index
}

fn assign_token(
    tok: &StreamToken,
    lexicon: &CmavoLexicon,
    out: &mut Vec<Lexeme>,
    gaps: &mut Vec<String>,
    notes: &mut Vec<String>,
) {
    let classes = &tok.result.classes;
    let text = tok.text.as_str();

    // Prefer cmevla if classified as such (names are consonant-final).
    if classes.contains(&MorphClass::Cmevla)
        && !classes.contains(&MorphClass::Cmavo)
        && !classes.contains(&MorphClass::CmavoCompound)
    {
        out.push(Lexeme {
            surface: text.to_string(),
            terminal: "CMENE".into(),
            note: None,
            attached_indicators: vec![],
            attached_emphasis: vec![],
        });
        return;
    }

    // Brivla family → BRIVLA
    let is_brivla = classes.iter().any(|c| {
        matches!(
            c,
            MorphClass::Gismu | MorphClass::Lujvo | MorphClass::Fuhivla
        )
    });
    if is_brivla
        && !classes.contains(&MorphClass::Cmavo)
        && !classes.contains(&MorphClass::CmavoCompound)
    {
        out.push(Lexeme {
            surface: text.to_string(),
            terminal: "BRIVLA".into(),
            note: None,
            attached_indicators: vec![],
            attached_emphasis: vec![],
        });
        return;
    }

    // Cmavo / compound cmavo
    if classes.contains(&MorphClass::Cmavo) || classes.contains(&MorphClass::CmavoCompound) {
        if let Some(sm) = lexicon.selmaho(text) {
            out.push(Lexeme {
                surface: text.to_string(),
                terminal: sm.to_string(),
                note: lexicon
                    .raw_selmaho(text)
                    .map(|r| format!("raw_selmaho={r}")),
                attached_indicators: vec![],
                attached_emphasis: vec![],
            });
            return;
        }
        // Try compound split (DERIVED when morphology left a fused form)
        if let Some(parts) = lexicon.split_compound(text) {
            notes.push(format!(
                "DERIVED: split compound cmavo {text:?} → {} pieces",
                parts.len()
            ));
            for (surf, sm) in parts {
                out.push(Lexeme {
                    surface: surf,
                    terminal: sm,
                    note: Some("compound-split".into()),
                    attached_indicators: vec![],
                    attached_emphasis: vec![],
                });
            }
            return;
        }
        gaps.push(format!(
            "cmavo {text:?} not in lexicon (Ch.20/jbovlaste); cannot assign selma'o"
        ));
        out.push(Lexeme {
            surface: text.to_string(),
            terminal: "UNKNOWN_CMAVO".into(),
            note: Some("GAP: missing selma'o".into()),
            attached_indicators: vec![],
            attached_emphasis: vec![],
        });
        return;
    }

    // Ambiguous cmavo+brivla etc.: prefer lexicon if form is known cmavo
    if let Some(sm) = lexicon.selmaho(text) {
        out.push(Lexeme {
            surface: text.to_string(),
            terminal: sm.to_string(),
            note: Some(format!("ambiguous morph classes {classes:?}; lexicon wins")),
            attached_indicators: vec![],
            attached_emphasis: vec![],
        });
        return;
    }
    if is_brivla {
        out.push(Lexeme {
            surface: text.to_string(),
            terminal: "BRIVLA".into(),
            note: Some(format!("ambiguous morph classes {classes:?}; BRIVLA")),
            attached_indicators: vec![],
            attached_emphasis: vec![],
        });
        return;
    }

    gaps.push(format!(
        "no Ch.21 terminal for {text:?} classes={classes:?}"
    ));
    out.push(Lexeme {
        surface: text.to_string(),
        terminal: "GAP".into(),
        note: Some("unassigned".into()),
        attached_indicators: vec![],
        attached_emphasis: vec![],
    });
}

/// Round-trip helpers for tests.
pub fn terminals_only(input: &str) -> Vec<String> {
    lexeme_stream(input)
        .lexemes
        .into_iter()
        .map(|l| l.terminal)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_known_cmavo() {
        let t = terminals_only("mi le cu");
        assert_eq!(t, vec!["KOhA", "LE", "CU"]);
    }

    #[test]
    fn brivla_and_cmavo() {
        let t = terminals_only("mi viska le gerku");
        assert_eq!(t, vec!["KOhA", "BRIVLA", "LE", "BRIVLA"]);
    }

    #[test]
    fn word_layer_strips_midstream_ui() {
        let t = terminals_only("la djan. .iu klama");
        assert_eq!(t, vec!["LA", "CMENE", "BRIVLA"], "{t:?}");
    }

    #[test]
    fn word_layer_strips_bahe() {
        let result = lexeme_stream("mi viska ba'e la djordj.");
        let t = result
            .lexemes
            .iter()
            .map(|lexeme| lexeme.terminal.as_str())
            .collect::<Vec<_>>();
        assert_eq!(t, vec!["KOhA", "BRIVLA", "LA", "CMENE"], "{t:?}");
        assert_eq!(
            result.lexemes[2].attached_emphasis,
            vec![("ba'e".into(), "BAhE".into())]
        );
    }

    #[test]
    fn leading_ui_kept_for_text_prefix() {
        let t = terminals_only(".ui mi klama");
        assert_eq!(t[0], "UI");
        assert!(t.contains(&"KOhA".into()));
    }

    #[test]
    fn refuse_atomic_coi_and_ui() {
        assert_eq!(terminals_only("coi"), vec!["COI"]);
        assert_eq!(terminals_only("ui"), vec!["UI"]);
        assert_eq!(terminals_only("iu"), vec!["UI"]);
    }

    #[test]
    fn refuse_does_not_cross_pause_into_doi() {
        // `do .ije` must stay KOhA + I + JA, not fuse do+i → DOI
        let t = terminals_only("mi cinba do .ije do cinba mi");
        assert_eq!(t[2], "KOhA", "{t:?}");
        assert!(t.contains(&"I".into()) || t.contains(&"JA".into()), "{t:?}");
    }

    #[test]
    fn ex_17_19_yhy_letteral_lexemes() {
        // CLL §17.2: `.y'y` is BY; `.y'y.bu` = BY + BU; vowel letterals = V + BU
        let t = terminals_only(".y'y.bu .abu ny. vo zy. .ibu vo");
        assert_eq!(
            t,
            vec!["BY", "BU", "A", "BU", "BY", "PA", "BY", "I", "BU", "PA"],
            "{t:?}"
        );
    }

    #[test]
    fn zo_quotes_ui_outranks_word_layer() {
        // CLL ex-6-93: mi cusku zo .ai — `.ai` is UI but ZO captures it.
        let t = terminals_only("mi cusku zo .ai");
        assert_eq!(t, vec!["KOhA", "BRIVLA", "ZO", "UI"], "{t:?}");
    }

    #[test]
    fn zo_quotes_bahe() {
        let t = terminals_only("zo ba'e");
        assert_eq!(t, vec!["ZO", "BAhE"], "{t:?}");
    }

    #[test]
    fn zo_quotes_zoi_cmavo() {
        let t = terminals_only("zo zoi");
        assert_eq!(t, vec!["ZO", "ZOI"], "{t:?}");
    }

    #[test]
    fn zo_quotes_another_zo() {
        let t = terminals_only("zo zo");
        assert_eq!(t, vec!["ZO", "ZO"], "{t:?}");
    }

    #[test]
    fn lu_inner_ui_not_stripped() {
        // Leading UI of quoted text must not attach to LU (fresh seen_content).
        let t = terminals_only("lu .ui mi li'u");
        assert_eq!(t, vec!["LU", "UI", "KOhA", "LIhU"], "{t:?}");
    }

    #[test]
    fn lu_inner_midstream_ui_still_strips() {
        // Editorial sa'a on sei inside LU is mid-stream UI of inner text → strip.
        let t = terminals_only("lu mi klama seisa'a la djan. cusku li'u");
        assert_eq!(
            t,
            vec!["LU", "KOhA", "BRIVLA", "SEI", "LA", "CMENE", "BRIVLA", "LIhU"],
            "{t:?}"
        );
    }

    #[test]
    fn lohu_inner_ui_not_stripped() {
        let t = terminals_only("lo'u .ui mi le'u");
        assert_eq!(t, vec!["LOhU", "UI", "KOhA", "LEhU"], "{t:?}");
    }

    #[test]
    fn lu_nested_preserves_inner_quote() {
        let t = terminals_only("lu lu .ui mi li'u li'u");
        assert_eq!(t, vec!["LU", "LU", "UI", "KOhA", "LIhU", "LIhU"], "{t:?}");
    }

    #[test]
    fn zoi_body_ui_not_stripped() {
        // Body may look like UI; it is opaque foreign text, never stripped.
        let t = terminals_only("zoi gy. .ui .gy.");
        assert_eq!(t, vec!["ZOI", "BY", "FOREIGN", "BY"], "{t:?}");
    }

    fn quote(input: &str) -> QuoteRegion<'_> {
        split_quote_regions(input)
            .into_iter()
            .find_map(|region| match region {
                Region::Quote(quote) => Some(quote),
                Region::Lojban(_) => None,
            })
            .expect("quote region")
    }

    #[test]
    fn zoi_closes_on_matching_delimiter_only() {
        let q = quote("mi cusku zoi gy. hello .dy world .gy. cu glico");
        assert_eq!(q.body, "hello .dy world");
        assert_eq!(q.close, Some("gy"));
        let t = terminals_only("zoi gy. hello world .gy cu glico");
        assert_eq!(
            t,
            vec!["ZOI", "BY", "FOREIGN", "BY", "CU", "BRIVLA"],
            "{t:?}"
        );
    }

    #[test]
    fn zoi_delimiter_normalized_like_camxes() {
        // Case, commas and `h`/`'` spelling do not matter.
        assert_eq!(quote("zoi GY. hello .gy").close, Some("gy"));
        assert_eq!(quote("la'o .djan. John .DJAN.").close, Some("DJAN"));
        assert_eq!(quote("zoi xu'u. a b .xuhu").close, Some("xuhu"));
    }

    #[test]
    fn zoi_body_ends_at_first_delimiter() {
        // The body can never contain the delimiter as a separate word.
        let q = quote("zoi gy. mi .gy. klama .gy");
        assert_eq!(q.body, "mi");
        let t = terminals_only("zoi gy. mi .gy. klama .gy");
        assert_eq!(t[..4], ["ZOI", "BY", "FOREIGN", "BY"], "{t:?}");
        assert_eq!(t[4], "BRIVLA", "{t:?}");
    }

    #[test]
    fn zoi_unclosed_or_mismatched_is_gap() {
        for input in ["zoi gy. hello .dy glico", "zoi gy. hello", "zoi"] {
            let t = terminals_only(input);
            assert!(t.iter().any(|t| t == "GAP"), "{input}: {t:?}");
        }
    }

    #[test]
    fn la_o_quotes_names() {
        let t = terminals_only("la'o dy. New York .dy");
        assert_eq!(t, vec!["ZOI", "BY", "FOREIGN", "BY"], "{t:?}");
        assert_eq!(
            lexeme_stream("la'o dy. New York .dy").lexemes[2].surface,
            "New York"
        );
    }

    #[test]
    fn zo_and_lohu_protect_quote_markers() {
        assert!(split_quote_regions("zo zoi cmavo")
            .iter()
            .all(|r| matches!(r, Region::Lojban(_))));
        assert!(split_quote_regions("lo'u zoi gy le'u")
            .iter()
            .all(|r| matches!(r, Region::Lojban(_))));
    }

    #[test]
    fn outside_ui_still_stripped_after_zo_quote() {
        // Only the one word after ZO is protected; following mid-stream UI strips.
        let t = terminals_only("mi cusku zo .ai .ui");
        assert_eq!(t, vec!["KOhA", "BRIVLA", "ZO", "UI"], "{t:?}");
    }
}
