//! slinku'i test — CLL §4.7.3.
//!
//! CLL states exactly:
//! > a fu'ivla with a CV cmavo joined to the front of it must not have the
//! > form of a lujvo (the so-called "slinku'i test", not discussed further
//! > in this book)
//!
//! camxes-rs uses the camxes reading (local change, not in tergenzalvi):
//! reject a candidate fu'ivla `F` when `F` minus its first consonant starts
//! with a camxes rafsi string ([`crate::cll::camxes::starts_with_rafsi_string`]),
//! i.e. when `CV ++ F` would start with CVC rafsi + rafsi string. The named
//! form `slinku'i` fails (`pa` + `slinku'i` → `pas+lin+ku'i`).

use super::{Constraint, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::is_c;
use crate::cll::shapes::MorphClass;
use std::collections::BTreeSet;

pub struct Slinkui;

impl Constraint for Slinkui {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C17",
            cll: "§4.7.3",
            status: ConstraintStatus::Stated,
        }
    }
    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject> {
        if cand.proposed_class != crate::cll::lex::WordClass::Fuhivla {
            return Ok(());
        }
        if let Some(cv) = failing_cv_prefix(&cand.text) {
            return Err(Reject {
                id: "C17",
                cll: "§4.7.3",
                detail: format!("C17 §4.7.3 slinku'i: CV `{cv}` + form yields lujvo-shaped string"),
            });
        }
        Ok(())
    }
}

/// Returns `(notes, rejected_fuhivla)`.
pub fn note_for(norm: &str, classes: &BTreeSet<MorphClass>) -> (Vec<String>, bool) {
    let mut notes = Vec::new();
    if !classes.contains(&MorphClass::Fuhivla) {
        return (notes, false);
    }
    let body = norm.trim_matches('.');
    match failing_cv_prefix(body) {
        Some(cv) => {
            notes.push(format!(
                "C17 §4.7.3 slinku'i: `{cv}` + `{body}` has the form of a lujvo — fu'ivla rejected"
            ));
            (notes, true)
        }
        None => {
            notes.push(
                "C17 §4.7.3: no CV prefix yields a lujvo-form (slinku'i characterization from CLL)"
                    .into(),
            );
            (notes, false)
        }
    }
}

// CLL §4.7.3: exists CV such that CV++F has lujvo form.
thread_local! {
    static SLINKUI_CACHE: std::cell::RefCell<std::collections::HashMap<String, Option<String>>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

pub fn failing_cv_prefix(fuhivla: &str) -> Option<String> {
    let body = fuhivla.trim_matches('.').to_string();
    if let Some(hit) = SLINKUI_CACHE.with(|c| c.borrow().get(&body).cloned()) {
        return hit;
    }
    let result = failing_cv_prefix_uncached(&body);
    SLINKUI_CACHE.with(|c| c.borrow_mut().insert(body, result.clone()));
    result
}

fn failing_cv_prefix_uncached(body: &str) -> Option<String> {
    // camxes reading of the test (camxes `!(zunsna rafsi_string)`): a CV
    // cmavo `Ca` runs into F as a lujvo exactly when F without its first
    // consonant starts with a rafsi string (`pa` + `slinku'i` → `pas-lin-ku'i`).
    // A vowel-initial F starts after a pause, so nothing runs into it.
    let first = body.bytes().next()?;
    if !is_c(first) {
        return None;
    }
    crate::cll::camxes::starts_with_rafsi_string(&body[1..]).then(|| format!("{}a", first as char))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::constraints::morph_parse;
    use crate::cll::shapes::MorphClass;

    #[test]
    fn slinkui_named_form_fails() {
        assert!(
            failing_cv_prefix("slinku'i").is_some(),
            "slinku'i must fail C17"
        );
        let r = morph_parse("slinku'i");
        assert!(!r.classes.contains(&MorphClass::Fuhivla), "{r:?}");
        assert!(r
            .rejected_by
            .contains(&crate::cll::constraints::ConstraintId::C17));
    }

    #[test]
    fn stage4_spageti_ok() {
        assert_eq!(failing_cv_prefix("spageti"), None);
        let r = morph_parse("spageti");
        assert!(r.classes.contains(&MorphClass::Fuhivla), "{r:?}");
    }
}
