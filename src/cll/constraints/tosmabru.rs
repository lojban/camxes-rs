//! tosmabru test — CLL §4.11.5 (full algorithm as stated) + §4.13 joint illustration.
//!
//! CLL text implemented (no camxes extras):
//! > Test all forms with one or more initial CVC-form rafsi — with the pattern
//! > "CVC … CVC + X" — for "tosmabru failure". In order to fail, X must either
//! > be a CVCCV long rafsi that happens to have a permissible initial pair as
//! > the consonant cluster, or is something which has caused a y-hyphen to be
//! > inserted between the preceding CVC and X in step 4.b.
//! >
//! > a. Examine all the C/C consonant pairs up to the first y-hyphen, or up to
//! >    the end of the word in case there are no y-hyphens. These are "joints".
//! > b. If all of those joints are permissible initials, then the trial word
//! >    will break up into a cmavo and a shorter brivla, so we need to add a
//! >    "y"-hyphen at the first joint. If not, the word will not break up.
//! > c. Install a y-hyphen at the first such joint.
//!
//! Recognition reading: a candidate lujvo **fails** C15 when the algorithm
//! would require a y-hyphen at the first joint and that hyphen is absent.

use super::{Constraint, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::{is_c, is_permissible_initial_pair, is_v};
use crate::cll::tables::{classify_rafsi_shape, LujvoSegmentation, RafsiShape};

pub struct Tosmabru;

impl Constraint for Tosmabru {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C15",
            cll: "§4.11.5",
            status: ConstraintStatus::Stated,
        }
    }

    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject> {
        match check_partial(&cand.text) {
            TosmabruVerdict::Fail(detail) => Err(Reject {
                id: "C15",
                cll: "§4.11.5",
                detail,
            }),
            _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum TosmabruVerdict {
    Pass(String),
    Fail(String),
    /// Not applicable / residual (should not reject).
    Gap(String),
}

pub fn check_partial(norm: &str) -> TosmabruVerdict {
    let body = norm.trim_matches('.');
    let segs = super::lujvo_segmentations(body);

    // Prefer lexicon segmentations that already satisfy step-4 hyphens except
    // possibly the tosmabru y (step 5). We evaluate each segmentation.
    if !segs.is_empty() {
        let mut any_pass = false;
        let mut fail_notes = Vec::new();
        let mut na_notes = Vec::new();
        for seg in &segs {
            match evaluate_segmentation(seg) {
                TosmabruVerdict::Pass(n) => {
                    any_pass = true;
                    na_notes.push(n);
                }
                TosmabruVerdict::Fail(n) => fail_notes.push(n),
                TosmabruVerdict::Gap(n) => na_notes.push(n),
            }
        }
        if any_pass {
            return TosmabruVerdict::Pass(
                na_notes
                    .into_iter()
                    .find(|n| n.starts_with("C15"))
                    .unwrap_or_else(|| {
                        "C15 §4.11.5: at least one segmentation passes tosmabru".into()
                    }),
            );
        }
        if let Some(f) = fail_notes.into_iter().next() {
            return TosmabruVerdict::Fail(f);
        }
        return TosmabruVerdict::Gap(
            na_notes
                .into_iter()
                .next()
                .unwrap_or_else(|| "C15: no applicable tosmabru pattern".into()),
        );
    }

    // Structural fallback (no lexicon rafsi parse): letter-pattern CVC+CVCCV
    // as named by the algorithm. Used for the etymon `tosmabru` itself
    // (CLL names the test after this pattern; `tos` is not a modern rafsi).
    if let Some(v) = structural_cvc_cvccv(body) {
        return v;
    }

    TosmabruVerdict::Gap("C15: no CVC…CVC+X pattern / no lexicon segmentation".into())
}

fn evaluate_segmentation(seg: &LujvoSegmentation) -> TosmabruVerdict {
    let n = seg.pieces.len();
    if n < 2 {
        return TosmabruVerdict::Gap("C15: <2 rafsi".into());
    }

    // Leading run of CVC rafsi (one or more), then X = remainder.
    let mut k = 0usize;
    while k < n && seg.pieces[k].shape == RafsiShape::Cvc {
        k += 1;
    }
    if k == 0 {
        // §4.13 gerzda joint illustration: CV fall-off when leftover CC not initial.
        return joint_fallapart_note(seg);
    }
    if k == n {
        return TosmabruVerdict::Gap("C15: all-CVC (no X)".into());
    }

    let x_pieces = &seg.pieces[k..];
    let last_cvc = &seg.pieces[k - 1];

    let x_is_cvccv_initial =
        x_pieces.len() == 1 && is_cvccv_with_initial_cluster(&x_pieces[0].form);
    let y_before_x = last_cvc.hyphen_after == Some('y');
    // Step 4.b y between last CVC and X: hyphen_after y on last CVC, and the
    // medial between last CVC and X would have been illegal without it.
    let y_from_4b = y_before_x && x_forced_y_by_medial(last_cvc, &x_pieces[0]);

    if !(x_is_cvccv_initial || y_from_4b || (y_before_x && x_is_cvccv_initial)) {
        // X does not meet "in order to fail" precondition.
        // Still record §4.13-style joint note when useful.
        if k == 1 && x_pieces.len() == 1 {
            return joint_fallapart_note(seg);
        }
        return TosmabruVerdict::Gap(
            "C15 §4.11.5: X not CVCCV-with-initial-cluster and no 4.b y — tosmabru N/A".into(),
        );
    }

    // Collect joints up to (but not past) the first y-hyphen.
    let mut joints: Vec<(u8, u8, usize)> = Vec::new(); // (a,b, piece_index of left)
    for i in 0..k {
        let left = &seg.pieces[i];
        if left.hyphen_after == Some('y') {
            // First y stops examination; joint at this boundary is where y sits.
            break;
        }
        let right_form = if i + 1 < k {
            seg.pieces[i + 1].form.as_str()
        } else {
            x_pieces[0].form.as_str()
        };
        let Some(a) = last_consonant(&left.form) else {
            continue;
        };
        let Some(b) = first_consonant(right_form) else {
            continue;
        };
        joints.push((a, b, i));
        if left.hyphen_after.is_some() {
            // r/n hyphen is not y; continue? CLL says "up to the first y-hyphen".
            // Non-y hyphens don't stop the scan.
        }
    }

    if joints.is_empty() {
        // y already at the first CVC boundary (vacuous joints before y).
        if seg.pieces[0].hyphen_after == Some('y') {
            return TosmabruVerdict::Pass("C15 §4.11.5: tosmabru y already at first joint".into());
        }
        return TosmabruVerdict::Gap("C15: no joints collected".into());
    }

    let all_initial = joints
        .iter()
        .all(|&(a, b, _)| is_permissible_initial_pair(a, b));

    if !all_initial {
        let bad = joints
            .iter()
            .find(|&&(a, b, _)| !is_permissible_initial_pair(a, b))
            .map(|&(a, b, _)| format!("{}{}", a as char, b as char))
            .unwrap_or_default();
        return TosmabruVerdict::Pass(format!(
            "C15 §4.11.5: joint `{bad}` not initial-OK — will not fall apart; no tosmabru y"
        ));
    }

    // All joints initial-OK → need y at first joint.
    let first_idx = joints[0].2;
    if seg.pieces[first_idx].hyphen_after == Some('y') {
        return TosmabruVerdict::Pass(
            "C15 §4.11.5: all joints initial-OK; tosmabru y present at first joint".into(),
        );
    }
    let (a, b, _) = joints[0];
    TosmabruVerdict::Fail(format!(
        "C15 §4.11.5 tosmabru failure: all joints initial-OK (first `{}{}`); need y-hyphen at first joint",
        a as char, b as char
    ))
}

fn joint_fallapart_note(seg: &LujvoSegmentation) -> TosmabruVerdict {
    // CLL §4.13 gerzda: "ge cannot fall off the front, because the following
    // word would begin with rz, which is not a permissible initial consonant pair".
    let first = &seg.pieces[0];
    if first.form.len() >= 2 && seg.pieces.len() >= 2 {
        let rest_start = {
            // After peeling CV from a CVC/CVV-ish front, leftover begins with
            // the final C of the first rafsi + start of next.
            let first_rafsi_bytes = first.form.as_bytes();
            if first_rafsi_bytes.len() >= 3
                && is_c(first_rafsi_bytes[0])
                && is_v(first_rafsi_bytes[1])
                && is_c(first_rafsi_bytes[2])
            {
                let next_c = first_consonant(&seg.pieces[1].form);
                next_c.map(|next_consonant| (first_rafsi_bytes[2], next_consonant))
            } else {
                None
            }
        };
        if let Some((a, b)) = rest_start {
            if !is_permissible_initial_pair(a, b) {
                return TosmabruVerdict::Pass(format!(
                    "C15 §4.13 joint: front CV cannot fall off — `{a}{b}` not initial-OK",
                    a = a as char,
                    b = b as char
                ));
            }
        }
    }
    TosmabruVerdict::Gap("C15: non-CVC-prefix form; tosmabru N/A".into())
}

fn x_forced_y_by_medial(
    left: &crate::cll::tables::RafsiPiece,
    right: &crate::cll::tables::RafsiPiece,
) -> bool {
    let Some(a) = last_consonant(&left.form) else {
        return false;
    };
    let Some(b) = first_consonant(&right.form) else {
        return false;
    };
    !crate::cll::phonology::is_permissible_medial_pair(a, b)
}

fn is_cvccv_with_initial_cluster(form: &str) -> bool {
    let letters: Vec<u8> = form.bytes().filter(|&c| c != b'\'').collect();
    // CVCCV: C V C C V with medial CC a permissible initial.
    if letters.len() == 5
        && is_c(letters[0])
        && is_v(letters[1])
        && is_c(letters[2])
        && is_c(letters[3])
        && is_v(letters[4])
    {
        return is_permissible_initial_pair(letters[2], letters[3]);
    }
    false
}

/// Letter-pattern reading for CVC+CVCCV when lexicon lacks the CVC rafsi
/// (e.g. etymon `tosmabru`).
fn structural_cvc_cvccv(body: &str) -> Option<TosmabruVerdict> {
    let letters: Vec<u8> = body
        .bytes()
        .filter(|&c| c != b'.' && c != b',' && c != b'\'')
        .collect();
    // Try splits: 3+5, 3+3+5, 3+3+3+5, …
    if letters.len() < 8 {
        return None;
    }
    // Single CVC + CVCCV (len 8)
    if letters.len() == 8 {
        let cvc = &letters[..3];
        let rest = &letters[3..];
        if is_c(cvc[0]) && is_v(cvc[1]) && is_c(cvc[2]) && is_cvccv_bytes(rest) {
            let joint_ok = is_permissible_initial_pair(cvc[2], rest[0]);
            let medial_ok = is_permissible_initial_pair(rest[2], rest[3]);
            if medial_ok && joint_ok {
                return Some(TosmabruVerdict::Fail(format!(
                    "C15 §4.11.5 structural tosmabru failure on `{body}`: CVC+CVCCV joints/medial all initial-OK (source=cll_derived)"
                )));
            }
            if medial_ok && !joint_ok {
                return Some(TosmabruVerdict::Pass(format!(
                    "C15 structural: joint not initial-OK on `{body}`"
                )));
            }
        }
    }
    // Also try with y present: CVC y CVCCV
    if let Some(yi) = body.find('y') {
        if yi == 3 && body.len() == 9 {
            // tosymabru-style
            return Some(TosmabruVerdict::Pass(
                "C15 structural: y present after leading CVC (tosmabru repaired)".into(),
            ));
        }
    }
    let _ = classify_rafsi_shape; // keep import used in tests path
    None
}

fn is_cvccv_bytes(word_bytes: &[u8]) -> bool {
    word_bytes.len() == 5
        && is_c(word_bytes[0])
        && is_v(word_bytes[1])
        && is_c(word_bytes[2])
        && is_c(word_bytes[3])
        && is_v(word_bytes[4])
}

fn last_consonant(form: &str) -> Option<u8> {
    form.bytes().rev().find(|&c| is_c(c))
}

fn first_consonant(form: &str) -> Option<u8> {
    form.bytes().find(|&c| is_c(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::constraints::morph_parse;
    use crate::cll::shapes::MorphClass;

    #[test]
    fn gerzda_passes_joint() {
        let v = check_partial("gerzda");
        match v {
            TosmabruVerdict::Pass(n) => assert!(n.contains("C15"), "{n}"),
            other => panic!("{other:?}"),
        }
        let r = morph_parse("gerzda");
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn tosmabru_etymon_fails_structural() {
        // CLL names the test after this pattern; derived from §4.11.5 algorithm.
        let v = check_partial("tosmabru");
        assert!(matches!(v, TosmabruVerdict::Fail(_)), "{v:?}");
    }

    #[test]
    fn bablujvo_fails_tosmabru() {
        // bab (CVC) + lujvo (CVCCV, jv initial-OK), joint bl initial-OK.
        let v = check_partial("bablujvo");
        assert!(matches!(v, TosmabruVerdict::Fail(_)), "{v:?}");
        let r = morph_parse("bablujvo");
        assert!(!r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn babylujvo_passes() {
        let v = check_partial("babylujvo");
        assert!(matches!(v, TosmabruVerdict::Pass(_)), "{v:?}");
        let r = morph_parse("babylujvo");
        assert!(r.classes.contains(&MorphClass::Lujvo), "{r:?}");
    }

    #[test]
    fn sacfanva_not_tosmabru_fail() {
        // fanva medial `nv` is not initial-OK → X does not qualify.
        let v = check_partial("sacfanva");
        assert!(
            !matches!(v, TosmabruVerdict::Fail(_)),
            "should not tosmabru-fail: {v:?}"
        );
    }
}
