//! Diphthongs and vowel pairs — CLL §3.4–3.5, §4.1–4.2.
//!
//! Off-glides (freely used in most word types): ai ei oi au — §3.4.
//! On-glides (stand-alone / names / borrowings; VV-form cmavo):
//!   ia ie ii io iu ua ue ui uo uu — §3.4; §4.2 “ten additional diphthongs”
//!   (printed table may be missing in some text dumps — recover from §3.4).
//! Name-only: iy uy — §3.4, §4.8.
//! Vowel pairs: any V'V as two syllables — §3.5.

pub const DIPHTHONG_OFF: &[&str] = &["ai", "ei", "oi", "au"];

/// The ten on-glide diphthongs of §3.4 (VV cmavo / names / borrowings).
pub const DIPHTHONG_ON: &[&str] = &["ia", "ie", "ii", "io", "iu", "ua", "ue", "ui", "uo", "uu"];

pub const DIPHTHONG_NAME_ONLY: &[&str] = &["iy", "uy"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiphthongClass {
    /// §3.4 first four — free in most word types.
    OffGlide,
    /// §3.4 next ten — cmavo VV / names / borrowings.
    OnGlide,
    /// §3.4 last two — cmevla only.
    NameOnly,
}

pub fn classify_diphthong(s: &str) -> Option<DiphthongClass> {
    if DIPHTHONG_OFF.contains(&s) {
        Some(DiphthongClass::OffGlide)
    } else if DIPHTHONG_ON.contains(&s) {
        Some(DiphthongClass::OnGlide)
    } else if DIPHTHONG_NAME_ONLY.contains(&s) {
        Some(DiphthongClass::NameOnly)
    } else {
        None
    }
}

/// §4.1.2 VV as used in morphology conventions: off-glide diphthong OR V'V pair.
/// On-glides are additional for VV-*cmavo* (§4.2), not the §4.1.2 VV abbreviation
/// used when describing brivla/rafsi shapes.
pub fn is_morphology_vv_core(s: &str) -> bool {
    if DIPHTHONG_OFF.contains(&s) {
        return true;
    }
    // V'V
    let bytes = s.as_bytes();
    bytes.len() == 3
        && super::letters::is_v(bytes[0])
        && bytes[1] == b'\''
        && super::letters::is_v(bytes[2])
}
