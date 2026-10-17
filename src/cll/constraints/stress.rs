//! Stress constraints — CLL §3.9, §4.3, §4.9.5.
//!
//! - Brivla: primary stress on penultimate **counted** syllable (§3.9 / §4.3).
//!   Syllables with `y` or syllabic consonants are not counted.
//! - When no stress capitals/marks: penultimate stress is **assumed** (CLL
//!   default), so isolated brivla pass.
//! - Cmavo may stress any syllable; primary stress on the syllable just before
//!   a brivla requires a pause (§4.9.5 / §3.9) — enforced for multiword phrases
//!   and for continuous DP adjacency when stress is **capital-marked**.
//! - Commas are syllable-break marks (§3.3), not word breaks.
//! - Unmarked continuous text does **not** invent default cmavo stress; only
//!   marked final stress triggers the §4.9.5 pause obligation.

use super::{Constraint, ConstraintId, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::{is_c, is_v, is_y};
use crate::cll::shapes::MorphClass;
use std::collections::BTreeSet;

pub struct StressRules;

impl Constraint for StressRules {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C04,C11",
            cll: "§3.9; §4.3.3; §4.9.5",
            status: ConstraintStatus::Stated,
        }
    }
    fn check(&self, _cand: &CandidateSpan) -> Result<(), Reject> {
        Ok(())
    }
}

#[derive(Debug)]
pub enum StressVerdict {
    Ok(Vec<String>),
    Skip(String),
    Fail(ConstraintId, String),
}

pub fn check_original(original: &str, norm: &str, classes: &BTreeSet<MorphClass>) -> StressVerdict {
    // Multiword: delegate to phrase checker.
    if original.split_whitespace().count() > 1 {
        return check_phrase(original);
    }

    // Single orthographic token may still contain `.` (cmavo compound / pause).
    // Treat comma-only forms as single words.
    let has_capital = original
        .chars()
        .any(|character| character.is_ascii_uppercase());
    let mut notes = Vec::new();

    let is_brivla = classes.iter().any(|class| {
        matches!(
            class,
            MorphClass::Gismu | MorphClass::Lujvo | MorphClass::Fuhivla
        )
    });
    let is_cmevla = classes.contains(&MorphClass::Cmevla);

    if !has_capital {
        if is_brivla {
            notes.push(
                "C04 §3.9: no stress marks — penultimate counted stress assumed (CLL default)"
                    .into(),
            );
            return StressVerdict::Ok(notes);
        }
        notes.push("C04: no stress capitals; nothing to check".into());
        return StressVerdict::Ok(notes);
    }

    notes.push("C04: stress capitals present".into());

    if is_brivla {
        let stressed = capital_nucleus_indices(original);
        let counted = counted_nucleus_indices(norm);
        if counted.len() >= 2 {
            let penult = counted[counted.len() - 2];
            if !stressed.is_empty() && !stressed.contains(&penult) {
                return StressVerdict::Fail(
                    ConstraintId::C04,
                    format!(
                        "C04 §3.9/§4.3: brivla stress must be penultimate counted syllable (penult {penult}, stressed {stressed:?})"
                    ),
                );
            }
            notes.push(format!(
                "C04: brivla penultimate counted syllable OK ({penult})"
            ));
        } else if counted.len() == 1 {
            notes.push("C04: monosyllable brivla — stress on its only counted syllable".into());
        }
    } else if is_cmevla {
        notes.push(
            "C04: cmevla may stress any syllable; capitals record non-penultimate (§4.8)".into(),
        );
    } else {
        notes.push("C04: non-brivla/cmevla with capitals — recorded".into());
    }

    StressVerdict::Ok(notes)
}

/// Multiword / phrase stress + §4.9.5 pause-before-brivla.
///
/// Splits on whitespace and `.` so dotted phrases without spaces (e.g.
/// `e'U.BRI,di`) are checked the same way as spaced ones. A written `.`
/// between words satisfies the pause; bare whitespace does **not**.
pub fn check_phrase(original: &str) -> StressVerdict {
    use crate::cll::lex::stream::segment_stream;

    let mut notes = Vec::new();
    let stream = segment_stream(original);
    for ambiguity in &stream.ambiguity {
        notes.push(format!("C01/stream: {ambiguity}"));
    }

    let pieces = phrase_atoms(original);
    for win in pieces.windows(2) {
        let (left, pause_after_left) = &win[0];
        let (right, _) = &win[1];
        let left_stress_last = final_syllable_stressed(left);
        let right_is_brivla = {
            let r = crate::cll::constraints::morph_parse(right.trim_start_matches('.'));
            r.classes.iter().any(|c| {
                matches!(
                    c,
                    MorphClass::Gismu | MorphClass::Lujvo | MorphClass::Fuhivla
                )
            })
        };
        if left_stress_last && right_is_brivla && !*pause_after_left {
            // §4.9.5 / §3.9: primary stress on final counted syllable before
            // brivla needs a pause. Example: e'U.BRI,di has the pause;
            // "e'U BRI,di" / "le RE nanmu" without `.` → Fail C11.
            return StressVerdict::Fail(
                ConstraintId::C11,
                format!(
                    "C11 §4.9.5: stress on final syllable of `{left}` before brivla `{right}` requires pause"
                ),
            );
        }
        if left_stress_last && right_is_brivla && *pause_after_left {
            notes.push(format!(
                "C11 §4.9.5: pause after final-stressed `{left}` before brivla `{right}` OK"
            ));
        }
    }

    // Validate each atom as an isolated word (reattach pause orthography).
    for (piece, pause_after) in &pieces {
        let parse_text = if *pause_after {
            format!("{piece}.")
        } else {
            (*piece).to_string()
        };
        let r = crate::cll::lex::stream::morph_parse_stream_piece(&parse_text);
        match check_original(piece, &r.normalized, &r.classes) {
            StressVerdict::Ok(mut n) => notes.append(&mut n),
            StressVerdict::Fail(id, d) => return StressVerdict::Fail(id, d),
            StressVerdict::Skip(s) => notes.push(s),
        }
    }
    notes.push("C04/C11: phrase stress check complete".into());
    StressVerdict::Ok(notes)
}

/// True when capitals mark primary stress on the word's **last counted**
/// syllable (CLL §4.9.5 / §3.9). Unmarked words return false — default /
/// assumed stresses do not trigger the pause obligation.
pub fn final_syllable_stressed(word: &str) -> bool {
    if !word.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }
    let norm: String = word
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|c| c.is_ascii_alphabetic() || *c == '\'' || *c == ',' || *c == '.')
        .collect();
    let counted = counted_nucleus_indices(&norm);
    let stressed = capital_nucleus_indices(word);
    match counted.last() {
        Some(&last) => stressed.contains(&last),
        None => false,
    }
}

/// Split on whitespace and `.` into (word, pause_after) atoms.
fn phrase_atoms(original: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for ws_piece in original.split_whitespace() {
        if ws_piece.is_empty() {
            continue;
        }
        let parts: Vec<&str> = ws_piece.split('.').collect();
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            let pause_after =
                i + 1 < parts.len() || (i + 1 == parts.len() && ws_piece.ends_with('.'));
            out.push(((*part).to_string(), pause_after));
        }
    }
    out
}

fn counted_nucleus_indices(norm: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let chars: Vec<u8> = norm.bytes().filter(|&c| c != b'.' && c != b',').collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if is_y(c) {
            i += 1;
            continue;
        }
        // Syllabic consonants not counted — we don't mark them specially in
        // orthography beyond being consonants; CLL counts only vowel nuclei here.
        if is_v(c) {
            out.push(i);
            if i + 1 < chars.len() && is_v(chars[i + 1]) {
                // diphthong = one nucleus
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn capital_nucleus_indices(original: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut idx = 0usize;
    for ch in original.chars() {
        if ch == '.' || ch == ',' || ch == ' ' {
            continue;
        }
        if ch == '\'' {
            idx += 1;
            continue;
        }
        if !ch.is_ascii_alphabetic() {
            continue;
        }
        let lower = ch.to_ascii_lowercase() as u8;
        if ch.is_ascii_uppercase() && is_v(lower) {
            out.push(idx);
        }
        idx += 1;
    }
    let _ = is_c;
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::constraints::morph_parse;

    #[test]
    fn dikyjvo_capitals_ok() {
        let r = morph_parse("DI,ky,jvo");
        assert!(r.accepted, "{r:?}");
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn ninmu_comma_ok() {
        let r = morph_parse("nin,mu");
        assert!(r.accepted, "{r:?}");
        assert!(r.classes.contains(&MorphClass::Gismu), "{r:?}");
    }

    #[test]
    fn final_syllable_stressed_marked_cmavo() {
        assert!(final_syllable_stressed("RE"));
        assert!(final_syllable_stressed("e'U"));
        assert!(!final_syllable_stressed("E'u")); // stress on first of two
        assert!(!final_syllable_stressed("re")); // unmarked
                                                 // Monosyllable with capital: the only counted syllable is final → true.
        assert!(final_syllable_stressed("LE"));
        assert!(!final_syllable_stressed("nanmu"));
        assert!(!final_syllable_stressed("NANmu")); // stress on penult, not final
    }

    #[test]
    fn c11_fail_space_without_pause() {
        match check_phrase("le RE nanmu") {
            StressVerdict::Fail(ConstraintId::C11, d) => {
                assert!(d.contains("§4.9.5"), "{d}");
            }
            other => panic!("expected C11 Fail, got {other:?}"),
        }
        match check_phrase("e'U BRI,di") {
            StressVerdict::Fail(ConstraintId::C11, _) => {}
            other => panic!("expected C11 Fail, got {other:?}"),
        }
    }

    #[test]
    fn c11_ok_with_explicit_pause() {
        match check_phrase("le RE. NANmu") {
            StressVerdict::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
        match check_phrase("e'U.BRI,di") {
            StressVerdict::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn c11_ok_stress_not_final() {
        // Stress on LE (first cmavo) + unstressed re before nanmu — OK.
        match check_phrase("LE re NANmu") {
            StressVerdict::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
        // Unstressed cmavo before brivla — OK (§3.9 alternatives).
        match check_phrase("le re NANmu") {
            StressVerdict::Ok(_) => {}
            other => panic!("expected Ok, got {other:?}"),
        }
    }
}
