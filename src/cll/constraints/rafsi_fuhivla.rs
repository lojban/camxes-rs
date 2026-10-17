//! CLL §4.16 rafsi-fu'ivla proposal (experimental; feature-gated).
//!
//! When [`super::options::MorphOptions::rafsi_fuhivla`] is on:
//! - Type-4 fu'ivla of form CCVVCV may drop the final vowel to form a stem
//!   rafsi (treated like a 4-letter rafsi, so a following y-hyphen is used).
//! - Example: `tci'ile` → stem `tci'il` → `tci'ilykemcantutra`.

use crate::cll::phonology::{is_c, is_v};
use crate::cll::tables::{RafsiLexicon, RafsiShape};

/// CCVVCV cultural form (§4.16): CC + (VV|V'V) + C + V.
pub fn is_ccvvcv_fuhivla(body: &str) -> bool {
    let bytes = body.as_bytes();
    // CC + V'V + C + V
    if bytes.len() == 7
        && is_c(bytes[0])
        && is_c(bytes[1])
        && is_v(bytes[2])
        && bytes[3] == b'\''
        && is_v(bytes[4])
        && is_c(bytes[5])
        && is_v(bytes[6])
    {
        return true;
    }
    // CC + diphthong + C + V (len 6): ai/ei/oi/au
    if bytes.len() == 6
        && is_c(bytes[0])
        && is_c(bytes[1])
        && is_v(bytes[2])
        && is_v(bytes[3])
        && is_c(bytes[4])
        && is_v(bytes[5])
    {
        let diphthong = [bytes[2], bytes[3]];
        return matches!(&diphthong[..], b"ai" | b"ei" | b"oi" | b"au");
    }
    false
}

/// Stem rafsi = CCVVCV minus final vowel (§4.16).
pub fn cultural_stem(body: &str) -> Option<String> {
    if !is_ccvvcv_fuhivla(body) {
        return None;
    }
    let bytes = body.as_bytes();
    Some(
        std::str::from_utf8(&bytes[..bytes.len() - 1])
            .ok()?
            .to_string(),
    )
}

/// Try to accept an experimental §4.16 lujvo: cultural-stem + y + lexicon tail.
pub fn try_parse_experimental_lujvo(body: &str) -> Option<String> {
    let body = body.trim_matches('.');
    // Find first 'y' that could be the long-rafsi hyphen after the stem.
    let y_at = body.find('y')?;
    if y_at < 5 {
        return None;
    }
    let stem = &body[..y_at];
    let rest = &body[y_at + 1..];
    if rest.is_empty() {
        return None;
    }
    // Stem must be CCVVCV-minus-V (i.e. adding a vowel yields CCVVCV).
    // Try restoring a/e/i/o/u.
    let mut ok_stem = false;
    for &v in b"aeiou" {
        let mut full = stem.to_string();
        full.push(v as char);
        if is_ccvvcv_fuhivla(&full) {
            ok_stem = true;
            break;
        }
    }
    if !ok_stem {
        return None;
    }
    let lex = RafsiLexicon::global();
    let segs = lex.segment_lujvo(rest);
    if segs.is_empty() {
        // Rest might itself be a single final gismu / short chain without being
        // in the segmenter if it's one piece — try whole rest as gismu.
        if lex.is_gismu(rest) || !lex.segment_lujvo(rest).is_empty() {
            return Some(format!(
                "C22 §4.16: experimental rafsi-fu'ivla lujvo (`{stem}`-y+…)"
            ));
        }
        // Try: remainder as concatenation starting with known rafsi — already did.
        return None;
    }
    // Prefer segmentations that don't begin mid-word oddly; any non-empty is OK.
    let _ = RafsiShape::Long4;
    Some(format!(
        "C22 §4.16: experimental rafsi-fu'ivla lujvo (`{stem}`-y + lexicon tail); {} seg(s)",
        segs.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tciile_shape() {
        assert!(is_ccvvcv_fuhivla("tci'ile"));
        assert_eq!(cultural_stem("tci'ile").as_deref(), Some("tci'il"));
    }

    #[test]
    fn experimental_lujvo_example() {
        let n = try_parse_experimental_lujvo("tci'ilykemcantutra");
        assert!(n.is_some(), "{n:?}");
    }
}
