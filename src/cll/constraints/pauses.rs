//! Pause obligations — CLL §4.9 (also §3.3).
//!
//! §3.3: "Technically, the period is an optional reminder to the reader of a
//! mandatory pause that is dictated by the rules of the language; because these
//! rules are unambiguous, a missing period can be inferred from otherwise
//! correct text."
//!
//! Isolation policy: for a single-word call we **infer** obligatory pauses
//! (notes) rather than hard-failing V-initial forms without `.`, matching §3.3.
//! C-final cmevla and Cy without a following pause are still rejected when the
//! orthography cannot show the pause (no trailing `.`).

use super::{Constraint, ConstraintId, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::{is_c, is_v, is_y};
use crate::cll::shapes::MorphClass;
use std::collections::BTreeSet;

pub struct PauseRules;

impl Constraint for PauseRules {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C07-C11,C06,C09,C24",
            cll: "§4.9; §3.3; §4.2 (.y.)",
            status: ConstraintStatus::Stated,
        }
    }

    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject> {
        let mut classes = BTreeSet::new();
        match cand.proposed_class {
            crate::cll::lex::WordClass::Cmavo => {
                classes.insert(MorphClass::Cmavo);
            }
            crate::cll::lex::WordClass::Gismu => {
                classes.insert(MorphClass::Gismu);
            }
            crate::cll::lex::WordClass::Lujvo => {
                classes.insert(MorphClass::Lujvo);
            }
            crate::cll::lex::WordClass::Fuhivla => {
                classes.insert(MorphClass::Fuhivla);
            }
            crate::cll::lex::WordClass::Cmevla => {
                classes.insert(MorphClass::Cmevla);
            }
        }
        match check_orthography(&cand.text, &classes) {
            Ok(_) => Ok(()),
            Err((ids, detail)) => Err(Reject {
                id: ids.first().map(|i| i.as_str()).unwrap_or("C09"),
                cll: "§4.9",
                detail,
            }),
        }
    }
}

pub fn check_orthography(
    norm: &str,
    classes: &BTreeSet<MorphClass>,
) -> Result<Vec<String>, (Vec<ConstraintId>, String)> {
    let mut notes = Vec::new();

    let is_single_content_word = classes.iter().any(|c| {
        matches!(
            c,
            MorphClass::Gismu | MorphClass::Lujvo | MorphClass::Fuhivla | MorphClass::Cmevla
        )
    }) && !classes
        .iter()
        .any(|c| matches!(c, MorphClass::Cmavo | MorphClass::CmavoCompound));

    if is_single_content_word {
        let body = norm.trim_matches('.');
        if body.contains('.') {
            return Err((
                vec![ConstraintId::C24],
                "C24 §4.9.1: pause mid-word is illegal".into(),
            ));
        }
    }

    // Also catch mid-word pause on cmavo compounds written as one token with
    // an internal `.` that is NOT a pause-surrounded compound (§4.2 allows
    // written compounds with `.` between members — those keep the dots).
    // C24 only fires for content-word classes above.

    if classes.contains(&MorphClass::Cmevla) {
        if let Some(last) = norm.bytes().rev().find(|&c| is_c(c) || is_v(c) || is_y(c)) {
            if is_c(last) && !norm.ends_with('.') {
                return Err((
                    vec![ConstraintId::C09],
                    "C09 §4.9.2: consonant-final word requires following pause (write trailing `.`)".into(),
                ));
            }
        }
        notes.push("C09: cmevla trailing pause present or not required in this orthography".into());
    }

    if norm == ".y." {
        notes.push("C06: .y. surrounded by pauses".into());
    } else if norm.trim_matches('.') == "y" && norm != ".y." {
        return Err((
            vec![ConstraintId::C06],
            "C06 §4.2: .y. requires pauses before and after".into(),
        ));
    }

    let core = norm.trim_matches('.');
    if core.len() == 2
        && is_c(core.as_bytes()[0])
        && is_y(core.as_bytes()[1])
        && !norm.ends_with('.')
    {
        return Err((
            vec![ConstraintId::C07],
            "C07 §4.9.6: Cy requires following pause (unless Cy follows)".into(),
        ));
    }

    if let Some(first) = norm.bytes().find(|&c| is_c(c) || is_v(c) || is_y(c)) {
        if is_v(first) && norm.starts_with('.') {
            notes.push("C08: V-initial written with preceding pause".into());
        } else if is_v(first) && !norm.starts_with('.') {
            // §3.3: missing period can be inferred.
            notes.push(
                "C08 §3.3/§4.9.3: V-initial — preceding pause obligatory; `.` omitted but inferred"
                    .into(),
            );
        }
    }

    // C-final non-cmevla (should not happen after C03) — still note.
    if let Some(last) = core.bytes().rev().find(|&c| is_c(c) || is_v(c) || is_y(c)) {
        if is_c(last) && !norm.ends_with('.') && !classes.contains(&MorphClass::Cmevla) {
            notes.push(
                "C09 §3.3: C-final requires surrounding pauses — `.` omitted but inferred where class allows"
                    .into(),
            );
        }
    }

    if !norm.contains('.') {
        notes.push(
            "pause: no explicit `.`; obligatory pauses inferred per §3.3 where applicable".into(),
        );
    }

    Ok(notes)
}

/// Whether a stream boundary must contain a pause per §4.9 given left/right words.
pub fn obligatory_pause_between(left: &str, right: &str) -> Option<&'static str> {
    let left_core = left.trim_matches('.');
    let right_core = right.trim_matches('.');
    if left_core.is_empty() || right_core.is_empty() {
        return None;
    }
    let left_last = left_core
        .bytes()
        .rev()
        .find(|&byte| is_c(byte) || is_v(byte) || is_y(byte));
    let right_first = right_core
        .bytes()
        .find(|&byte| is_c(byte) || is_v(byte) || is_y(byte));
    if let (Some(left_byte), Some(right_byte)) = (left_last, right_first) {
        if is_c(left_byte) {
            return Some("C09 §4.9.2: pause after C-final");
        }
        if is_v(right_byte) {
            return Some("C08 §4.9.3: pause before V-initial");
        }
        if left_core.len() == 2 && is_c(left_core.as_bytes()[0]) && is_y(left_core.as_bytes()[1]) {
            let right_starts_with_cy = right_core.len() >= 2
                && is_c(right_core.as_bytes()[0])
                && is_y(right_core.as_bytes()[1]);
            if !right_starts_with_cy {
                return Some("C07 §4.9.6: pause after Cy");
            }
        }
    }
    None
}

/// CLL §4.9.4: cmevla must be preceded by a pause unless previous word is
/// la / lai / la'i / doi.
pub fn la_family_cmavo(word: &str) -> bool {
    matches!(word.trim_matches('.'), "la" | "lai" | "la'i" | "doi")
}

/// Whether a cmevla may follow `left` in continuous text (no written pause).
///
/// - Stream start (`left` empty) → yes (implicit pause).
/// - After C-final word → yes (pause already obligatory by §4.9.2).
/// - After la/lai/la'i/doi → yes (§4.9.4 exception).
/// - Otherwise after V-final → no without an explicit pause.
pub fn cmevla_allowed_after(left: &str) -> bool {
    let left_core = left.trim_matches('.');
    if left_core.is_empty() {
        return true;
    }
    if la_family_cmavo(left_core) {
        return true;
    }
    let last_letter = left_core
        .bytes()
        .rev()
        .find(|&byte| is_c(byte) || is_v(byte) || is_y(byte));
    matches!(last_letter, Some(byte) if is_c(byte))
}
