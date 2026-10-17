//! Letter classes — CLL §3.1–3.3, §4.1.
//!
//! - V = a e i o u  (NOT y) — §4.1.1
//! - C = bcdfgjklmnprstvxz (syllabic l/m/n/r still C) — §4.1.3
//! - Apostrophe `'` is a phoneme but neither C nor V for morphology — §3.3, §4.1
//! - y is restricted — §3.4, §4.2, §4.6, §4.8

/// CLL §4.1.1 — single Lojban vowel except y.
pub const VOWELS: &[u8] = b"aeiou";

/// CLL §4.1.3 — consonants excluding apostrophe.
pub const CONSONANTS: &[u8] = b"bcdfgjklmnprstvxz";

/// CLL §3.4 — y distribution is limited; still a letter.
pub const Y: u8 = b'y';

/// CLL §3.3 — apostrophe phoneme `[h]`.
pub const APOSTROPHE: u8 = b'\'';

/// CLL §3.3 — period marks pause (optional orthographic reminder).
pub const PERIOD: u8 = b'.';

/// CLL §3.3 — comma = syllable break without pause; non-contrastive for wordhood.
pub const COMMA: u8 = b',';

#[inline]
pub fn is_v(character: u8) -> bool {
    matches!(character, b'a' | b'e' | b'i' | b'o' | b'u')
}

#[inline]
pub fn is_c(character: u8) -> bool {
    CONSONANTS.contains(&character)
}

#[inline]
pub fn is_y(character: u8) -> bool {
    character == Y
}

/// Letters counted for “first five letters” of brivla — CLL §4.3:
/// y and apostrophe are **not** counted.
#[inline]
pub fn counts_for_brivla_window(character: u8) -> bool {
    is_c(character) || is_v(character)
}
