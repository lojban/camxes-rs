//! Break-up of a brivla candidate into a leading cmavo and following words
//! — CLL §4.6 and §4.7.
//!
//! CLL §4.7 (fu'ivla rule 3): a fu'ivla "must not be gismu or lujvo, or any
//! combination of cmavo, gismu, and lujvo". CLL §4.6: hyphens keep a lujvo
//! from falling apart "into a CVV-form cmavo and a gismu" or "a cmavo, and
//! the remainder of the word as a valid lujvo". Both say: a word that reads
//! as a cmavo followed by cmavo / gismu / lujvo is not a brivla.
//!
//! The leading cmavo is the longest prefix of the form `C? N ('N)*`, where a
//! nucleus `N` is a vowel or an off-glide diphthong (§4.2 cmavo forms,
//! including the experimental three-or-more-vowel forms). `y` is not taken:
//! a `Cy` cmavo must be followed by a pause unless another `Cy` follows
//! (§4.9 rule 6). The rest must start with a consonant (a vowel-initial word
//! needs a pause before it, §4.9 rule 3).

use crate::cll::phonology::{is_c, is_v};
use crate::cll::shapes::MorphClass;

const OFF_GLIDE: [&[u8]; 4] = [b"ai", b"ei", b"oi", b"au"];

/// The rest of `text` after its longest leading cmavo form, when that rest
/// starts with a consonant.
fn cmavo_prefix_rest(text: &str) -> Option<&str> {
    let w = text.as_bytes();
    let mut p = 0;
    if w.first().copied().is_some_and(is_c) {
        p = 1;
    }
    loop {
        if !w.get(p).copied().is_some_and(is_v) {
            return None;
        }
        p += if OFF_GLIDE.iter().any(|d| w[p..].starts_with(d)) {
            2
        } else {
            1
        };
        if w.get(p) == Some(&b'\'') && w.get(p + 1).copied().is_some_and(is_v) {
            p += 1;
        } else {
            break;
        }
    }
    (p > 0 && w.get(p).copied().is_some_and(is_c)).then(|| &text[p..])
}

/// `text` is a run of cmavo, gismu and lujvo with no pause needed between
/// them (`renanmu` = `re` + `nanmu`).
fn is_cmavo_gismu_lujvo_run(text: &str, depth: usize) -> bool {
    let r = crate::cll::constraints::morph_parse(text);
    if r.accepted
        && r.classes.iter().any(|c| {
            matches!(
                c,
                MorphClass::Cmavo
                    | MorphClass::CmavoCompound
                    | MorphClass::Gismu
                    | MorphClass::Lujvo
            )
        })
    {
        return true;
    }
    depth > 0
        && cmavo_prefix_rest(text).is_some_and(|rest| is_cmavo_gismu_lujvo_run(rest, depth - 1))
}

/// Whether `text` breaks up into a leading cmavo and a following run of
/// cmavo / gismu / lujvo (`lerenanmu` = `le` + `re` + `nanmu`), so that it
/// is not a brivla (CLL §4.6, §4.7).
pub fn cmavo_prefix_splits(text: &str) -> bool {
    let body = text.trim_matches('.');
    if !body.is_ascii() {
        return false;
    }
    cmavo_prefix_rest(body).is_some_and(|rest| {
        // A rest starting with a consonant cluster is the tosmabru (C15,
        // §4.6) or slinku'i (C17, §4.7) case, which those filters decide.
        !rest.as_bytes().get(1).copied().is_some_and(is_c) && is_cmavo_gismu_lujvo_run(rest, 8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_up_into_cmavo_and_gismu() {
        assert!(cmavo_prefix_splits("lerenanmu"));
        assert!(!cmavo_prefix_splits("spageti"));
        assert!(!cmavo_prefix_splits("babylujvo"));
        // `ba` + `blujvo`: left to the tosmabru test (C15).
        assert!(!cmavo_prefix_splits("bablujvo"));
        // A fu'ivla after a cmavo is not one of the forms CLL lists.
        assert!(!cmavo_prefix_splits("matriocka"));
    }
}
