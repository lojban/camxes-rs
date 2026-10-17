//! Constraint layer — CLL complement-style rules over shape-layer candidates.
//!
//! Each submodule cites CLL §§ and refuses to invent camxes/BPFK behavior where
//! the book is silent ([`ConstraintStatus::Gap`]).
//!
//! Public entry: [`constrain`] / [`morph_parse`] → [`ConstrainedResult`].

pub mod class_mutex;
pub mod cmevla_ban;
pub mod lujvo_hyphen;
pub mod options;
pub mod pauses;
pub mod rafsi_fuhivla;
pub mod slinkui;
pub mod stress;
pub mod tosmabru;

pub use options::MorphOptions;

use crate::cll::cfg::CandidateSpan;
use crate::cll::lex::WordClass;
use crate::cll::shapes::{recognize, MorphClass, ShapeResult};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConstraintId {
    C01,
    C02,
    C03,
    C04,
    C05,
    C06,
    C07,
    C08,
    C09,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
    C16,
    C17,
    C18,
    C19,
    C20,
    C21,
    C22,
    C23,
    C24,
}

impl ConstraintId {
    pub fn as_str(self) -> &'static str {
        match self {
            ConstraintId::C01 => "C01",
            ConstraintId::C02 => "C02",
            ConstraintId::C03 => "C03",
            ConstraintId::C04 => "C04",
            ConstraintId::C05 => "C05",
            ConstraintId::C06 => "C06",
            ConstraintId::C07 => "C07",
            ConstraintId::C08 => "C08",
            ConstraintId::C09 => "C09",
            ConstraintId::C10 => "C10",
            ConstraintId::C11 => "C11",
            ConstraintId::C12 => "C12",
            ConstraintId::C13 => "C13",
            ConstraintId::C14 => "C14",
            ConstraintId::C15 => "C15",
            ConstraintId::C16 => "C16",
            ConstraintId::C17 => "C17",
            ConstraintId::C18 => "C18",
            ConstraintId::C19 => "C19",
            ConstraintId::C20 => "C20",
            ConstraintId::C21 => "C21",
            ConstraintId::C22 => "C22",
            ConstraintId::C23 => "C23",
            ConstraintId::C24 => "C24",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintStatus {
    Stated,
    Gap,
    Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstraintMeta {
    pub id: &'static str,
    pub cll: &'static str,
    pub status: ConstraintStatus,
}

pub trait Constraint {
    fn meta(&self) -> ConstraintMeta;
    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reject {
    pub id: &'static str,
    pub cll: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstrainedResult {
    pub accepted: bool,
    pub classes: BTreeSet<MorphClass>,
    pub rejected_by: Vec<ConstraintId>,
    pub notes: Vec<String>,
    pub normalized: String,
}

fn push_id(constraint_ids: &mut Vec<ConstraintId>, id: ConstraintId) {
    if !constraint_ids.contains(&id) {
        constraint_ids.push(id);
    }
}

/// Apply CLL constraint filters with default options, including the §4.16 proposal.
pub fn constrain(original: &str, shape: &ShapeResult) -> ConstrainedResult {
    constrain_with(original, shape, MorphOptions::default())
}

/// Apply CLL constraint filters.
pub fn constrain_with(
    original: &str,
    shape: &ShapeResult,
    opts: MorphOptions,
) -> ConstrainedResult {
    let mut classes = shape.classes.clone();
    let mut rejected_by = Vec::new();
    let mut notes = Vec::new();
    let norm = shape.normalized.as_str();

    if !shape.accepted {
        // §4.16 experimental: shape CFG may reject proposal lujvo; recover when flagged.
        if opts.rafsi_fuhivla {
            if let Some(note) = rafsi_fuhivla::try_parse_experimental_lujvo(original) {
                let mut classes = BTreeSet::new();
                classes.insert(MorphClass::Lujvo);
                notes.push(note);
                push_id(&mut rejected_by, ConstraintId::C22);
                // C22 used as "experimental accepted" marker in notes; not a hard reject.
                rejected_by.retain(|id| *id != ConstraintId::C22);
                return ConstrainedResult {
                    accepted: true,
                    classes,
                    rejected_by,
                    notes,
                    normalized: shape.normalized.clone(),
                };
            }
        }
        return ConstrainedResult {
            accepted: false,
            classes,
            rejected_by,
            notes: vec!["shape layer rejected".into()],
            normalized: shape.normalized.clone(),
        };
    }

    let wordish: BTreeSet<MorphClass> = classes
        .iter()
        .copied()
        .filter(|c| c.to_word_class().is_some())
        .collect();
    if wordish.is_empty() {
        notes.push("fragment-only shape; constraints N/A".into());
        return ConstrainedResult {
            accepted: true,
            classes,
            rejected_by,
            notes,
            normalized: shape.normalized.clone(),
        };
    }

    // C19 (CLL §4.8 `la`/`lai`/`doi` ban in cmevla) is not applied: camxes
    // has no such ban (`lat`, `laus`). See `cmevla_ban` for the CLL rule.

    // C01/C02/C03/C05/C12/C16 class mutex
    let (classes2, rb2, mut n2) = class_mutex::filter_classes(norm, &classes);
    classes = classes2;
    for id in rb2 {
        push_id(&mut rejected_by, id);
    }
    notes.append(&mut n2);

    // C13/C14 lujvo hyphen / fall-apart
    if classes.contains(&MorphClass::Lujvo) {
        match lujvo_hyphen::check_lujvo(norm) {
            Ok(mut n) => notes.append(&mut n),
            Err((id, detail)) => {
                classes.remove(&MorphClass::Lujvo);
                push_id(&mut rejected_by, id);
                notes.push(detail);
            }
        }
    }

    // C15 tosmabru (§4.11.5 full algorithm)
    if classes.contains(&MorphClass::Lujvo) {
        match tosmabru::check_partial(norm) {
            tosmabru::TosmabruVerdict::Pass(note) => notes.push(note),
            tosmabru::TosmabruVerdict::Fail(detail) => {
                classes.remove(&MorphClass::Lujvo);
                push_id(&mut rejected_by, ConstraintId::C15);
                notes.push(detail);
            }
            tosmabru::TosmabruVerdict::Gap(note) => notes.push(note),
        }
    }

    // C17 slinku'i (§4.7.3 characterization) — Stage-4 concern; skip when
    // stage-3 `l`-hyphen already identified (C16) or Fuhivla already dropped.
    if classes.contains(&MorphClass::Fuhivla) {
        let (mut n17, reject_f) = slinkui::note_for(norm, &classes);
        notes.append(&mut n17);
        if reject_f {
            classes.remove(&MorphClass::Fuhivla);
            push_id(&mut rejected_by, ConstraintId::C17);
        }
    }

    // C18 Stage-4 extra tests — CLL: "not simple to characterize"; record Gap note
    // when form is fu'ivla and survived C16/C17. Positive shape + exclusions done.
    if classes.contains(&MorphClass::Fuhivla) {
        notes.push(
            "C18 GAP §4.7: Stage-4 'extra tests' not characterized in CLL; positive shape + C16/C17 applied"
                .into(),
        );
    }

    // §4.16 feature: cultural CCVVCV note
    if opts.rafsi_fuhivla {
        let body = norm.trim_matches('.');
        if rafsi_fuhivla::is_ccvvcv_fuhivla(body) {
            notes
                .push("C22 §4.16: CCVVCV cultural fu'ivla (rafsi-fu'ivla proposal enabled)".into());
        }
        if let Some(note) = rafsi_fuhivla::try_parse_experimental_lujvo(body) {
            if !classes.contains(&MorphClass::Lujvo) {
                classes.insert(MorphClass::Lujvo);
            }
            notes.push(note);
        }
    } else if rafsi_fuhivla::is_ccvvcv_fuhivla(norm.trim_matches('.')) {
        notes.push(
            "C22 §4.16: form matches CCVVCV cultural shape; rafsi-fu'ivla proposal OFF (settled morphology)"
                .into(),
        );
    }

    // Pauses
    match pauses::check_orthography(norm, &classes) {
        Ok(mut n) => notes.append(&mut n),
        Err((ids, detail)) => {
            for id in ids {
                push_id(&mut rejected_by, id);
            }
            notes.push(detail);
            classes.clear();
        }
    }

    // Stress
    match stress::check_original(original, norm, &classes) {
        stress::StressVerdict::Ok(mut n) => notes.append(&mut n),
        stress::StressVerdict::Skip(note) => notes.push(note),
        stress::StressVerdict::Fail(id, detail) => {
            push_id(&mut rejected_by, id);
            notes.push(detail);
            classes.clear();
        }
    }

    let still_word: BTreeSet<MorphClass> = classes
        .iter()
        .copied()
        .filter(|c| c.to_word_class().is_some())
        .collect();
    if !still_word.is_empty() {
        classes = still_word;
    }

    ConstrainedResult {
        accepted: !classes.is_empty(),
        classes,
        rejected_by,
        notes,
        normalized: shape.normalized.clone(),
    }
}

// Thread-local cache for settled-option morph_parse results.
// Continuous DP and stream segmentation re-query many overlapping prefixes;
// caching avoids rebuilding Earley charts. Cap prevents unbounded growth on
// infinite streams of novel tokens.
thread_local! {
    static MORPH_CACHE: std::cell::RefCell<std::collections::HashMap<String, ConstrainedResult>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

const MORPH_CACHE_CAP: usize = 8192;

/// Shape recognize + constrain using default options. Results are cached per thread.
pub fn morph_parse(input: &str) -> ConstrainedResult {
    if let Some(hit) = MORPH_CACHE.with(|cache| cache.borrow().get(input).cloned()) {
        return hit;
    }
    let result = constrain(input, &recognize(input));
    MORPH_CACHE.with(|cache| {
        let mut entries = cache.borrow_mut();
        if entries.len() >= MORPH_CACHE_CAP {
            entries.clear();
        }
        entries.insert(input.to_string(), result.clone());
    });
    result
}

/// Clear the thread-local morph_parse cache (tests / benches).
pub fn clear_morph_cache() {
    MORPH_CACHE.with(|c| c.borrow_mut().clear());
}

/// Shape recognize + constrain with options.
pub fn morph_parse_with(input: &str, opts: MorphOptions) -> ConstrainedResult {
    constrain_with(input, &recognize(input), opts)
}

/// Sketch-compatible per-span runner.
pub fn apply_all(cands: &[CandidateSpan]) -> Result<Vec<CandidateSpan>, Reject> {
    let mut out = Vec::new();
    for cand in cands {
        let mut classes = BTreeSet::new();
        match cand.proposed_class {
            WordClass::Cmavo => {
                classes.insert(MorphClass::Cmavo);
            }
            WordClass::Gismu => {
                classes.insert(MorphClass::Gismu);
            }
            WordClass::Lujvo => {
                classes.insert(MorphClass::Lujvo);
            }
            WordClass::Fuhivla => {
                classes.insert(MorphClass::Fuhivla);
            }
            WordClass::Cmevla => {
                classes.insert(MorphClass::Cmevla);
            }
        }
        let shape = ShapeResult {
            accepted: true,
            classes,
            normalized: cand.text.clone(),
        };
        let result = constrain(&cand.text, &shape);
        if result.accepted
            && result
                .classes
                .iter()
                .any(|class| class.to_word_class() == Some(cand.proposed_class))
        {
            out.push(cand.clone());
        } else if let Some(id) = result.rejected_by.first() {
            return Err(Reject {
                id: id.as_str(),
                cll: "constraints",
                detail: result.notes.join("; "),
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmevla_la_ban_not_applied() {
        // camxes extension: no `la` ban, so `laplas.` is a cmevla.
        assert!(cmevla_ban::banned_substring("laplas").is_some());
        let r = morph_parse("laplas.");
        assert!(r.classes.contains(&MorphClass::Cmevla), "{r:?}");
    }

    #[test]
    fn lujvo_fallapart_roireo() {
        let r = morph_parse("ro'ire'o");
        assert!(!r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn fuhivla_exclusions() {
        let r = morph_parse("xaceru");
        assert!(!r.classes.contains(&MorphClass::Fuhivla), "{r:?}");
        let r2 = morph_parse("kobra");
        assert!(!r2.classes.contains(&MorphClass::Fuhivla), "{r2:?}");
        assert!(r2.classes.contains(&MorphClass::Gismu), "{r2:?}");
    }

    #[test]
    fn gismu_preferred_over_fuhivla() {
        let r = morph_parse("creka");
        assert!(r.classes.contains(&MorphClass::Gismu));
        assert!(!r.classes.contains(&MorphClass::Fuhivla));
    }

    #[test]
    fn gerzda_tosmabru_joint_ok() {
        let r = morph_parse("gerzda");
        assert!(r.accepted);
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn rafsi_fuhivla_proposal_is_on_by_default() {
        let r = morph_parse("tci'ilykemcantutra");
        assert!(r.accepted, "proposal lujvo accepted by default: {r:?}");
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn settled_options_disable_rafsi_fuhivla_proposal() {
        let r = morph_parse_with("tci'ilykemcantutra", MorphOptions::settled());
        assert!(
            !r.accepted,
            "proposal lujvo rejected in settled mode: {r:?}"
        );
    }

    #[test]
    fn rafsi_fuhivla_feature_on() {
        let r = morph_parse_with(
            "tci'ilykemcantutra",
            MorphOptions::default().with_rafsi_fuhivla(true),
        );
        assert!(r.accepted, "{r:?}");
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }
}
