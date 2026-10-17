//! Word-class mutual exclusion — CLL §4.1–4.3, §4.7.

use super::{Constraint, ConstraintId, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::{is_c, is_v, is_y};
use crate::cll::shapes::{has_cc_in_first_five_ignoring_y_apostrophe, MorphClass};
use std::collections::BTreeSet;

pub struct ClassMutex;

impl Constraint for ClassMutex {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C01,C16",
            cll: "§4.1; §4.3; §4.7.3",
            status: ConstraintStatus::Stated,
        }
    }
    fn check(&self, _cand: &CandidateSpan) -> Result<(), Reject> {
        Ok(())
    }
}

pub fn filter_classes(
    norm: &str,
    classes: &BTreeSet<MorphClass>,
) -> (BTreeSet<MorphClass>, Vec<ConstraintId>, Vec<String>) {
    let mut out = classes.clone();
    let mut rejected = Vec::new();
    let mut notes = Vec::new();
    let body = norm.trim_matches('.').to_string();

    let ends_c = body
        .bytes()
        .rev()
        .find(|&c| c != b',' && c != b'.')
        .map(is_c)
        .unwrap_or(false);
    let has_cc = has_cc_in_first_five_ignoring_y_apostrophe(&body);
    let gismu_shape = is_gismu_shape(&body);
    let has_y = body.bytes().any(is_y);

    if ends_c {
        let before = out.len();
        out.retain(|c| {
            matches!(
                c,
                MorphClass::Cmevla
                    | MorphClass::MedialPair
                    | MorphClass::InitialPair
                    | MorphClass::ConsonantTriple
                    | MorphClass::Diphthong
            )
        });
        if out.len() < before {
            rejected.push(ConstraintId::C03);
            notes.push("C03 §4.8: consonant-final → cmevla only".into());
        }
    } else if out.remove(&MorphClass::Cmevla) {
        rejected.push(ConstraintId::C03);
        notes.push("C03 §4.3: vowel-final → not cmevla".into());
    }

    if !has_cc {
        let mut stripped = false;
        // camxes extension: see `crate::cll::camxes::late_cluster_fuhivla`.
        let keep_fuhivla =
            out.contains(&MorphClass::Fuhivla) && crate::cll::camxes::late_cluster_fuhivla(norm);
        for brivla in [MorphClass::Gismu, MorphClass::Lujvo, MorphClass::Fuhivla] {
            if keep_fuhivla && brivla == MorphClass::Fuhivla {
                continue;
            }
            if out.remove(&brivla) {
                stripped = true;
            }
        }
        if stripped {
            rejected.push(ConstraintId::C02);
            notes.push("C02 §4.3: no CC in first five → not brivla".into());
        }
    } else {
        // camxes extension: see `crate::cll::camxes::cmavo_prefix_splits`.
        if [MorphClass::Gismu, MorphClass::Lujvo, MorphClass::Fuhivla]
            .iter()
            .any(|c| out.contains(c))
            && crate::cll::camxes::cmavo_prefix_splits(norm)
        {
            for brivla in [MorphClass::Gismu, MorphClass::Lujvo, MorphClass::Fuhivla] {
                out.remove(&brivla);
            }
            rejected.push(ConstraintId::C02);
            notes.push("camxes: starts with a cmavo followed by words → not brivla".into());
        }
        let mut stripped = false;
        if out.remove(&MorphClass::Cmavo) {
            stripped = true;
        }
        if out.remove(&MorphClass::CmavoCompound) {
            stripped = true;
        }
        if stripped {
            rejected.push(ConstraintId::C05);
            notes.push("C05 §4.2: CC present → not cmavo".into());
        }
    }

    if gismu_shape && out.contains(&MorphClass::Gismu) {
        if out.remove(&MorphClass::Fuhivla) {
            rejected.push(ConstraintId::C12);
            notes.push("C12 §4.4: gismu shape preferred over fu'ivla".into());
        }
        if out.remove(&MorphClass::Lujvo) {
            rejected.push(ConstraintId::C12);
            notes.push("C12 §4.4: gismu shape preferred over lujvo".into());
        }
    }

    // C16: fu'ivla must not be gismu / lujvo / cmavo-combination (§4.7.3).
    // Without rafsi lexicon (C23), "is a lujvo" is not decidable from shape alone —
    // stage-3 fu'ivla (…r/n/l…) overgenerate as Lujvo in the CFG. Only apply the
    // decidable exclusions here; lujvo-vs-fu'ivla uniqueness is partial.
    if out.contains(&MorphClass::Fuhivla) {
        let mut drop_f = false;
        if out.contains(&MorphClass::Gismu) {
            drop_f = true;
            notes.push("C16 §4.7.3: fu'ivla excluded — also gismu".into());
        }
        if out.contains(&MorphClass::Cmavo) || out.contains(&MorphClass::CmavoCompound) {
            drop_f = true;
            notes.push("C16 §4.7.3: fu'ivla excluded — cmavo / cmavo-combination".into());
        }
        // Stage-3 marker: 4-letter + l + body — `l` is not a lujvo hyphen (§4.6).
        if out.contains(&MorphClass::Lujvo) && has_stage3_l_hyphen(&body) {
            out.remove(&MorphClass::Lujvo);
            rejected.push(ConstraintId::C01);
            notes.push(
                "C01/C16: `l`-hyphen is stage-3 fu'ivla only (§4.6/§4.7); drop lujvo reading"
                    .into(),
            );
        } else if out.contains(&MorphClass::Lujvo) {
            notes.push(
                "C16 partial: Lujvo∩Fuhivla overgeneration kept — lexicon segmentation may still leave Lujvo∩Fuhivla ambiguous without unique decomposition"
                    .into(),
            );
        }
        if drop_f {
            out.remove(&MorphClass::Fuhivla);
            rejected.push(ConstraintId::C16);
        }
    }

    if has_y && out.remove(&MorphClass::Fuhivla) {
        rejected.push(ConstraintId::C16);
        notes.push("C16/§4.7.4: fu'ivla may not contain y".into());
    }

    // If a future or experimental form derives both starts, use its complete
    // simple-cmavo reading. The compound CFG separately requires a pause
    // before each following V-initial piece, so au is already unambiguous.
    if out.contains(&MorphClass::Cmavo) && out.remove(&MorphClass::CmavoCompound) {
        notes.push(
            "C01 §4.2: simple cmavo shape takes precedence over compound decomposition".into(),
        );
    }

    let word: Vec<_> = out
        .iter()
        .copied()
        .filter(|c| c.to_word_class().is_some())
        .collect();
    if word.len() > 1 {
        notes.push(format!(
            "C01 partial: multiple classes remain {:?} (stream-break GAP §4.1)",
            word
        ));
    }

    (out, rejected, notes)
}

fn is_gismu_shape(body: &str) -> bool {
    let letters: Vec<u8> = body.bytes().filter(|&c| c != b',' && c != b'.').collect();
    if letters.len() != 5 || letters.contains(&b'\'') || letters.iter().any(|&c| is_y(c)) {
        return false;
    }
    if !is_c(letters[0]) || !is_v(letters[4]) {
        return false;
    }
    if is_c(letters[1]) && is_v(letters[2]) && is_c(letters[3]) {
        return crate::cll::phonology::is_permissible_initial_pair(letters[0], letters[1]);
    }
    if is_v(letters[1]) && is_c(letters[2]) && is_c(letters[3]) {
        return crate::cll::phonology::is_permissible_medial_pair(letters[2], letters[3]);
    }
    false
}

/// §4.7 stage-3: 4-letter rafsi + `l` hyphen + body (`l` not allowed in lujvo §4.6).
fn has_stage3_l_hyphen(body: &str) -> bool {
    let letters: Vec<u8> = body.bytes().filter(|&c| c != b',' && c != b'.').collect();
    // Look for … C V C C l C … or … C C V C l C …
    for index in 0..letters.len() {
        if letters[index] == b'l'
            && index >= 4
            && index + 1 < letters.len()
            && is_c(letters[index + 1])
        {
            let pre = &letters[index - 4..index];
            let cvcc = is_c(pre[0]) && is_v(pre[1]) && is_c(pre[2]) && is_c(pre[3]);
            let ccvc = is_c(pre[0]) && is_c(pre[1]) && is_v(pre[2]) && is_c(pre[3]);
            if cvcc || ccvc {
                return true;
            }
        }
    }
    false
}
