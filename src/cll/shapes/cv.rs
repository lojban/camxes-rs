//! Abstract CV patterns used throughout CLL §4.

use crate::cll::phonology::{counts_for_brivla_window, is_c, is_v};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CvPattern {
    /// §4.2 V-form cmavo
    V,
    /// §4.2 CV-form cmavo
    Cv,
    /// §4.2 VV-form (diphthong or V'V; on-glides per §4.2)
    Vv,
    /// §4.2 CVV-form
    Cvv,
    /// §4.2 Cy letteral
    Cy,
    /// §4.4 / §4.6 CVC
    Cvc,
    /// §4.6 CCV (requires InitialCC)
    Ccv,
    /// §4.4 CVC/CV gismu
    CvcCv,
    /// §4.4 CCVCV gismu
    CcvCv,
    /// §4.6 4-letter rafsi
    CvcC,
    CcvC,
}

/// Brivla window check — CLL §4.3.2:
/// consonant pair must appear among the first five letters, ignoring y and '.
pub fn has_cc_in_first_five_ignoring_y_apostrophe(word: &str) -> bool {
    let mut window = Vec::new();
    for byte in word.bytes() {
        if counts_for_brivla_window(byte) {
            window.push(byte);
            if window.len() == 5 {
                break;
            }
        }
    }
    window.windows(2).any(|w| is_c(w[0]) && is_c(w[1]))
}

/// Final letter is a vowel (brivla) vs consonant (cmevla) — ignoring trailing pause.
pub fn ends_in_vowel(word: &str) -> bool {
    word.bytes()
        .rev()
        .find(|&c| c != b'.')
        .map(is_v)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brivla_window_examples() {
        assert!(has_cc_in_first_five_ignoring_y_apostrophe("bisycla")); // sc via s,c ignoring y
        assert!(has_cc_in_first_five_ignoring_y_apostrophe("ro'inre'o")); // nr
        assert!(!has_cc_in_first_five_ignoring_y_apostrophe("da'amei"));
    }
}
