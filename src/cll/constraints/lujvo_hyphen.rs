//! Lujvo hyphen legality — CLL §4.6, §4.11 (C13) + fall-apart (C14).
//!
//! C23 lexicon (embedded LLG/CLL-era rafsi list) is used to validate rafsi
//! segmentations and decide when `y` / `r` / `n` hyphens are required.

use super::{Constraint, ConstraintId, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::{is_c, is_permissible_medial_pair};
use crate::cll::shapes::{has_cc_in_first_five_ignoring_y_apostrophe, recognize, MorphClass};
use crate::cll::tables::{LujvoSegmentation, RafsiLexicon, RafsiShape};

pub struct LujvoHyphen;

impl Constraint for LujvoHyphen {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C13,C14,C23",
            cll: "§4.6; §4.11",
            status: ConstraintStatus::Stated,
        }
    }

    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject> {
        if cand.proposed_class != crate::cll::lex::WordClass::Lujvo {
            return Ok(());
        }
        match check_lujvo(&cand.text) {
            Ok(_) => Ok(()),
            Err((id, detail)) => Err(Reject {
                id: id.as_str(),
                cll: "§4.6",
                detail,
            }),
        }
    }
}

fn last_consonant(form: &str) -> Option<u8> {
    form.bytes().rev().find(|&b| is_c(b))
}

fn first_consonant(form: &str) -> Option<u8> {
    form.bytes().find(|&b| is_c(b))
}

/// Form-aware hyphen requirement (CLL §4.6).
pub fn required_hyphen_forms(
    prev_shape: RafsiShape,
    prev_form: &str,
    next_shape: RafsiShape,
    next_form: &str,
    prev_index: usize,
    total_parts: usize,
) -> Option<char> {
    match prev_shape {
        RafsiShape::Long4 => Some('y'),
        RafsiShape::Cvc => {
            let a = last_consonant(prev_form)?;
            let b = first_consonant(next_form)?;
            // CLL §4.11 step 4.b only — illegal medial → y.
            // Tosimabru y (step 5) is enforced by C15; see hyphen_ok.
            if !is_permissible_medial_pair(a, b) {
                return Some('y');
            }
            None
        }
        RafsiShape::Cvv => {
            // CLL §4.6 only constrains a CVV rafsi in the *initial* position:
            // always hyphenated in a 3+ part lujvo, and in a two-part lujvo
            // when the remainder would otherwise lack a consonant cluster
            // (CVV-CVV, CVV-CVC/CV, CVV-CCVCV). A medial CVV (zer-xai-gau)
            // needs no hyphen: the initial rafsi already supplies the cluster.
            if prev_index != 0 {
                return None;
            }
            let initial_force = total_parts > 2;
            let pair_need = match next_shape {
                RafsiShape::Cvv | RafsiShape::Final5 | RafsiShape::Long4 | RafsiShape::Cvc => true,
                RafsiShape::Ccv => false,
            };
            if !(initial_force || pair_need) {
                return None;
            }
            // Exception: CVV+CCV even as a 2-part form needs no hyphen (cluster).
            // For 3+ parts with initial CVV+CCV, CLL still requires hyphen after
            // initial CVV ("must always be followed by a hyphen").
            if next_shape == RafsiShape::Ccv && !initial_force {
                return None;
            }
            if next_form.as_bytes().first() == Some(&b'r') {
                Some('n')
            } else {
                Some('r')
            }
        }
        RafsiShape::Ccv | RafsiShape::Final5 => None,
    }
}

fn hyphen_ok(seg: &LujvoSegmentation) -> Result<(), String> {
    let piece_count = seg.pieces.len();
    if piece_count < 2 {
        return Err("lujvo needs ≥2 rafsi".into());
    }
    for piece_index in 0..piece_count - 1 {
        let previous_piece = &seg.pieces[piece_index];
        let next_piece = &seg.pieces[piece_index + 1];
        let required_hyphen = required_hyphen_forms(
            previous_piece.shape,
            &previous_piece.form,
            next_piece.shape,
            &next_piece.form,
            piece_index,
            piece_count,
        );
        match (required_hyphen, previous_piece.hyphen_after) {
            (None, None) => {}
            (Some(h), Some(g)) if h == g => {}
            (Some(h), None) => {
                return Err(format!(
                    "C13 §4.6: missing `{h}`-hyphen after rafsi `{}` before `{}`",
                    previous_piece.form, next_piece.form
                ));
            }
            (None, Some('y')) if previous_piece.shape != RafsiShape::Cvc => {
                // §4.6: `y` only ever separates consonants (after a CVC rafsi,
                // or the tosmabru `y` of §4.11.5, also after CVC); it never
                // follows a vowel-final rafsi (mra-y-bau, pe'o-y-bau).
                return Err(format!(
                    "C13 §4.6: illegal `y`-hyphen after vowel-final rafsi `{}` before `{}`",
                    previous_piece.form, next_piece.form
                ));
            }
            (None, Some('y')) => {
                // Possibly required by §4.11.5 tosmabru (C15). Tentatively allow;
                // C15 rejects if the y is not justified, and rejects if missing when needed.
            }
            (None, Some(g)) => {
                return Err(format!(
                    "C13 §4.6/§4.11: illegal `{g}`-hyphen after rafsi `{}` before `{}` (not required)",
                    previous_piece.form, next_piece.form
                ));
            }
            (Some(h), Some(g)) => {
                return Err(format!(
                    "C13 §4.6: wrong hyphen after `{}` before `{}`: need `{h}`, found `{g}`",
                    previous_piece.form, next_piece.form
                ));
            }
        }
    }
    if seg.pieces[piece_count - 1].hyphen_after.is_some() {
        return Err("trailing hyphen after final rafsi".into());
    }
    Ok(())
}

pub fn check_lujvo(norm: &str) -> Result<Vec<String>, (ConstraintId, String)> {
    let mut notes = Vec::new();
    let body = norm.trim_matches('.');
    let shape = recognize(body);
    let has_cc = has_cc_in_first_five_ignoring_y_apostrophe(body);

    // C14: fall-apart into cmavo when no brivla CC (§4.6 ro'ire'o).
    if !has_cc
        && (shape.classes.contains(&MorphClass::Cmavo)
            || shape.classes.contains(&MorphClass::CmavoCompound))
    {
        return Err((
            ConstraintId::C14,
            "C14 §4.6: form falls apart as cmavo / cmavo-compound (no brivla CC)".into(),
        ));
    }

    // §4.6: the `y` hyphen only ever stands between consonants (after a
    // CVC or 4-letter rafsi); a `y` right after a vowel is never a hyphen.
    // Checked on the shape so it also holds when the lexicon has no parse.
    let lower = body.to_ascii_lowercase();
    let lb = lower.as_bytes();
    if let Some(i) = (1..lb.len()).find(|&i| lb[i] == b'y' && !is_c(lb[i - 1])) {
        return Err((
            ConstraintId::C13,
            format!("C13 §4.6: `y` after a vowel at byte {i} is not a lujvo hyphen"),
        ));
    }

    let lex = RafsiLexicon::global();
    let segs = super::lujvo_segmentations(body);
    if segs.is_empty() && !super::current_options().camxes_dialect {
        // Strict CLL dialect: a lujvo is made of rafsi (§4.5), and CLL's
        // rafsi are the ones in its list.
        return Err((
            ConstraintId::C13,
            "C13 §4.5 (strict CLL dialect): no segmentation into CLL-era rafsi".into(),
        ));
    }
    if segs.is_empty() {
        notes.push(format!(
            "C23: no rafsi segmentation found in CLL-era lexicon ({} entries); C13 necessity not fully decidable for this form",
            lex.entry_count()
        ));
        return Ok(notes);
    }

    let mut ok: Vec<&LujvoSegmentation> = Vec::new();
    let mut errs = Vec::new();
    for seg in &segs {
        match hyphen_ok(seg) {
            Ok(()) => ok.push(seg),
            Err(e) => errs.push(e),
        }
    }

    if ok.is_empty() {
        let detail = errs
            .into_iter()
            .next()
            .unwrap_or_else(|| "C13: no hyphen-valid rafsi segmentation".into());
        return Err((ConstraintId::C13, detail));
    }

    let shown = ok[0]
        .pieces
        .iter()
        .map(|p| match p.hyphen_after {
            Some(h) => format!("{}-{h}", p.form),
            None => p.form.clone(),
        })
        .collect::<Vec<_>>()
        .join("+");
    notes.push(format!(
        "C13/C23 §4.6/§4.11: hyphen-valid rafsi segmentation ({shown}); {} lexicon-valid segmentation(s)",
        ok.len()
    ));
    Ok(notes)
}

/// Whether a candidate string has at least one hyphen-legal rafsi parse.
pub fn has_valid_rafsi_parse(norm: &str) -> bool {
    check_lujvo(norm).is_ok_and(|notes| notes.iter().any(|n| n.starts_with("C13/C23")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::constraints::morph_parse;

    #[test]
    fn camxes_dialect_recognizes_rafsi_shapes_outside_the_list() {
        use crate::cll::constraints::{with_options, MorphOptions};
        use crate::cll::tables::RafsiLexicon;
        // `gam` has a CVC rafsi shape but is not in the CLL-era list.
        assert!(!RafsiLexicon::global().is_rafsi("gam"));
        assert!(with_options(MorphOptions::camxes(), || {
            has_valid_rafsi_parse("gamkemdau")
        }));
        // Strict CLL dialect (default, settled): not a lujvo.
        for opts in [MorphOptions::default(), MorphOptions::settled()] {
            assert!(with_options(opts, || check_lujvo("gamkemdau").is_err()));
        }
        // Still subject to the hyphen rules: an n-hyphen only before `r`.
        assert!(with_options(MorphOptions::camxes(), || {
            check_lujvo("zeingamyba'o").is_err()
        }));
    }

    #[test]
    fn soirsai_r_hyphen_ok() {
        // CLL §4.5: soi + r + sai
        let r = check_lujvo("soirsai");
        assert!(r.is_ok(), "{r:?}");
        assert!(has_valid_rafsi_parse("soirsai"));
    }

    #[test]
    fn soisai_missing_r_rejected() {
        // CVV+CVV without hyphen — falls apart / missing r.
        let r = check_lujvo("soisai");
        assert!(r.is_err(), "{r:?}");
    }

    #[test]
    fn roinreo_n_before_r() {
        // CLL §4.6: ro'i + n + re'o (n before r)
        let r = check_lujvo("ro'inre'o");
        assert!(r.is_ok(), "{r:?}");
        let notes = r.unwrap();
        assert!(notes.iter().any(|n| n.contains("C13/C23")), "{notes:?}");
    }

    #[test]
    fn roirreo_double_r_not_valid_seg() {
        let r = check_lujvo("ro'irre'o");
        assert!(r.is_err() || !has_valid_rafsi_parse("ro'irre'o"), "{r:?}");
    }

    #[test]
    fn patytaa_y_after_cvc() {
        // CLL §4.6: pat + y + ta'a (tt illegal)
        let r = check_lujvo("patyta'a");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn pattaa_illegal_tt() {
        let r = morph_parse("patta'a");
        assert!(
            !r.accepted || !r.classes.contains(&MorphClass::Lujvo),
            "{r:?}"
        );
    }

    #[test]
    fn geurzdani_r_hyphen() {
        // CLL §4.13: ge'u + r + zdani
        let r = check_lujvo("ge'urzdani");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn geuzdani_missing_r() {
        let r = check_lujvo("ge'uzdani");
        assert!(r.is_err(), "{r:?}");
        if let Err((id, _)) = r {
            assert_eq!(id, ConstraintId::C13);
        }
    }

    #[test]
    fn geuzda_cvv_ccv_no_hyphen() {
        // CLL §4.13: ge'u + zda — no hyphen
        let r = check_lujvo("ge'uzda");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn gerkyzda_long4_y() {
        // CLL §4.13: gerk + y + zda
        let r = check_lujvo("gerkyzda");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn mudysiclu_y() {
        // CLL §4.6: mud + y + siclu
        let r = check_lujvo("mudysiclu");
        assert!(r.is_ok(), "{r:?}");
    }

    #[test]
    fn roireo_c14() {
        let r = check_lujvo("ro'ire'o");
        assert!(matches!(r, Err((ConstraintId::C14, _))), "{r:?}");
    }
}
