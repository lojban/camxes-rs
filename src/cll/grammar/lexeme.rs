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

/// Segment then assign Ch.21 terminals via cmavo lexicon + morph classes.
pub fn lexeme_stream(input: &str) -> LexemeResult {
    let seg = segment_stream(input);
    let lex = CmavoLexicon::global();
    let mut gaps = Vec::new();
    let mut morph_notes = seg.ambiguity.clone();

    // Each group is lexemes from one stream token, plus whether the following
    // boundary was explicit (space/`.` — do not re-fuse across those).
    let mut groups: Vec<(Vec<Lexeme>, bool)> = Vec::new();
    for tok in &seg.tokens {
        if !tok.result.accepted {
            gaps.push(format!(
                "morph rejected {:?}: rejected_by={:?}",
                tok.text, tok.result.rejected_by
            ));
            continue;
        }
        let mut piece = Vec::new();
        assign_token(tok, lex, &mut piece, &mut gaps, &mut morph_notes);
        groups.push((piece, tok.explicit_boundary));
    }

    // Re-fuse over-split atomic cmavo only inside continuous spans (`co`+`i` →
    // `coi`), never across explicit whitespace/pause boundaries (`do`+`.i` ↛ `doi`).
    let (lexemes, refuse_notes) = refuse_atomic_cmavo_grouped(groups, lex);
    morph_notes.extend(refuse_notes);

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
            // ZOI any-word anything any-word — keep delim + body + close intact.
            push_with_emphasis(&mut out, &raw[lexeme_index], &mut pending_emphasis);
            seen_content = true;
            lexeme_index += 1;
            if lexeme_index >= raw.len() {
                notes.push("DERIVED Ch.6/19 magic: ZOI without delimiter".into());
                continue;
            }
            let delim = raw[lexeme_index].surface.clone();
            notes.push(format!(
                "DERIVED Ch.6/19 magic: ZOI opaque region delim={delim:?} (word-layer bypass)"
            ));
            out.push(raw[lexeme_index].clone()); // open delimiter
            lexeme_index += 1;
            while lexeme_index < raw.len() && raw[lexeme_index].surface != delim {
                out.push(raw[lexeme_index].clone());
                lexeme_index += 1;
            }
            if lexeme_index < raw.len() && raw[lexeme_index].surface == delim {
                out.push(raw[lexeme_index].clone()); // close delimiter
                lexeme_index += 1;
            } else {
                notes.push(format!(
                    "DERIVED Ch.6/19 magic: ZOI unclosed delim {delim:?} (GAP partial)"
                ));
            }
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
        // Delimiter `gy` / body may include UI; word-layer must not eat them.
        let t = terminals_only("zoi gy. .ui .gy.");
        assert_eq!(t, vec!["ZOI", "BY", "UI", "BY"], "{t:?}");
    }

    #[test]
    fn outside_ui_still_stripped_after_zo_quote() {
        // Only the one word after ZO is protected; following mid-stream UI strips.
        let t = terminals_only("mi cusku zo .ai .ui");
        assert_eq!(t, vec!["KOhA", "BRIVLA", "ZO", "UI"], "{t:?}");
    }
}
