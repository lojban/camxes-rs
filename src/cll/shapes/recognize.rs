//! Shape-layer recognition — normalize orthography, run Earley CFG, classify.
//!
//! This is the public morphology **shape** API (overgenerator). Constraint-layer
//! uniqueness (tosmabru, slinku'i, la-ban, …) is intentionally out of scope.

use crate::cll::cfg::earley::{recognize_starts, recognizes};
use crate::cll::cfg::grammar::{morph_grammar, shape_nt_id, ShapeNt};
use crate::cll::phonology::{
    classify_diphthong, is_c, is_permissible_initial_cluster, is_permissible_initial_pair,
    is_permissible_medial_pair, is_permissible_triple, is_v, is_y,
};
use std::collections::BTreeSet;

/// Morphological shape classes returned by the overgenerator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MorphClass {
    Cmavo,
    CmavoCompound,
    Gismu,
    Lujvo,
    Fuhivla,
    Cmevla,
    /// §3.6 medial pair fragment
    MedialPair,
    /// §3.7 initial pair fragment
    InitialPair,
    /// §3.7 triple fragment
    ConsonantTriple,
    /// §3.4 diphthong fragment
    Diphthong,
}

impl MorphClass {
    pub fn to_word_class(self) -> Option<crate::cll::lex::WordClass> {
        use crate::cll::lex::WordClass::*;
        Some(match self {
            MorphClass::Cmavo | MorphClass::CmavoCompound => Cmavo,
            MorphClass::Gismu => Gismu,
            MorphClass::Lujvo => Lujvo,
            MorphClass::Fuhivla => Fuhivla,
            MorphClass::Cmevla => Cmevla,
            _ => return None,
        })
    }
}

/// Result of shape-layer recognition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeResult {
    /// True if any shape / fragment class matched.
    pub accepted: bool,
    /// All matching shape classes (overgeneration OK).
    pub classes: BTreeSet<MorphClass>,
    /// Normalized byte string actually fed to the CFG.
    pub normalized: String,
}

/// Normalize CLL orthography for shape recognition:
/// - lowercase (stress capitals → plain letters)
/// - keep commas as syllable-break marks (§3.3; block illegal vowel adjacency)
/// - keep letters, apostrophe, period (pause)
/// - drop spaces (multiword phrases are not isolated-word shape targets)
pub fn normalize_orthography(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            'A'..='Z' => out.push(ch.to_ascii_lowercase()),
            'a'..='z' | '\'' | '.' | ',' => out.push(ch),
            _ => {}
        }
    }
    out
}

/// CLL §3.6–3.7 cluster check used as a shape-layer filter (C20).
/// `y`, `'`, `.`, and `,` break clusters.
pub fn phonotactics_ok(word: &[u8]) -> bool {
    let mut byte_index = 0;
    while byte_index < word.len() {
        let byte = word[byte_index];
        if byte == b'.' || byte == b',' || byte == b'\'' || is_y(byte) || !is_c(byte) {
            byte_index += 1;
            continue;
        }
        let cluster_start = byte_index;
        while byte_index < word.len() && is_c(word[byte_index]) {
            byte_index += 1;
        }
        if !cluster_ok(&word[cluster_start..byte_index]) {
            return false;
        }
    }
    true
}

fn cluster_ok(cluster: &[u8]) -> bool {
    if cluster.len() <= 1 {
        return true;
    }
    // Pairwise medial permissibility (§3.6 / §4.8).
    // Named forbidden triples (ndj/ndz/ntc/nts) are enforced for *fragment*
    // fixtures via `is_permissible_triple`; cmevla examples like `nederlants.`
    // contain `nts` as adjacent medial-OK pairs (§4.8 pairwise rule).
    // Stricter §3.7 triple-initial / forbidden-triple filters → constraint layer.
    cluster
        .windows(2)
        .all(|w| is_permissible_medial_pair(w[0], w[1]))
}

fn fuhivla_initial_ok(word: &[u8]) -> bool {
    let word_without_pauses = trim_pauses(word);
    let mut cluster_end = 0;
    while cluster_end < word_without_pauses.len() && is_c(word_without_pauses[cluster_end]) {
        cluster_end += 1;
    }
    if cluster_end < 2 {
        return true;
    }
    is_permissible_initial_cluster(&word_without_pauses[..cluster_end])
}

fn trim_pauses(word: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = word.len();
    while start < end && (word[start] == b'.' || word[start] == b',') {
        start += 1;
    }
    while end > start && (word[end - 1] == b'.' || word[end - 1] == b',') {
        end -= 1;
    }
    &word[start..end]
}

fn classify_via_cfg(norm: &[u8]) -> BTreeSet<MorphClass> {
    let grammar = morph_grammar();
    let starts = [
        ShapeNt::CmavoSimple,
        ShapeNt::CmavoCompound,
        ShapeNt::Gismu,
        ShapeNt::Lujvo,
        ShapeNt::Fuhivla,
        ShapeNt::Cmevla,
    ];
    let ids: Vec<_> = starts
        .iter()
        .map(|start| shape_nt_id(grammar, *start))
        .collect();
    let matched = recognize_starts(grammar, &ids, norm);
    let mut classes = BTreeSet::new();
    for (nt, id) in starts.iter().zip(ids.iter()) {
        if matched.contains(id) {
            classes.insert(match nt {
                ShapeNt::CmavoSimple => MorphClass::Cmavo,
                ShapeNt::CmavoCompound => MorphClass::CmavoCompound,
                ShapeNt::Gismu => MorphClass::Gismu,
                ShapeNt::Lujvo => MorphClass::Lujvo,
                ShapeNt::Fuhivla => MorphClass::Fuhivla,
                ShapeNt::Cmevla => MorphClass::Cmevla,
                _ => continue,
            });
        }
    }
    if classes.contains(&MorphClass::Fuhivla) && !fuhivla_initial_ok(norm) {
        classes.remove(&MorphClass::Fuhivla);
    }
    classes
}

fn classify_fragment(norm: &[u8]) -> BTreeSet<MorphClass> {
    let mut classes = BTreeSet::new();
    if norm.len() == 2 {
        let first_byte = norm[0];
        let second_byte = norm[1];
        if is_permissible_medial_pair(first_byte, second_byte) {
            classes.insert(MorphClass::MedialPair);
        }
        if is_permissible_initial_pair(first_byte, second_byte) {
            classes.insert(MorphClass::InitialPair);
        }
        if let Ok(s) = std::str::from_utf8(norm) {
            if classify_diphthong(s).is_some() {
                classes.insert(MorphClass::Diphthong);
            }
        }
    } else if norm.len() == 3 {
        if is_permissible_triple(norm[0], norm[1], norm[2]) {
            classes.insert(MorphClass::ConsonantTriple);
        }
        let grammar = morph_grammar();
        if let Some(nonterminal_id) = grammar.nt("ConsonantTriple") {
            if recognizes(grammar, nonterminal_id, norm) {
                classes.insert(MorphClass::ConsonantTriple);
            }
        }
    }
    classes
}

fn is_pure_consonant_fragment(norm: &[u8]) -> bool {
    norm.len() <= 3 && !norm.is_empty() && norm.iter().all(|&c| is_c(c))
}

/// Recognize an orthographic string at the **shape** layer.
pub fn recognize(input: &str) -> ShapeResult {
    let normalized = normalize_orthography(input);
    if normalized.is_empty() {
        return ShapeResult {
            accepted: false,
            classes: BTreeSet::new(),
            normalized,
        };
    }
    let primary = recognize_normalized(normalized.clone());
    if primary.accepted || !normalized.contains(',') {
        return primary;
    }
    // Commas are orthographic syllable marks (§3.3). If the marked form is not
    // a CFG shape (e.g. `DI,ky,jvo`), retry with commas stripped — but only when
    // stripping does not create a bare illegal vowel pair (V,V hiatus cases like
    // `kore,a` must keep the comma).
    let stripped: String = normalized.chars().filter(|&c| c != ',').collect();
    if stripped == normalized {
        return primary;
    }
    if bare_illegal_vowel_adjacency(&stripped) {
        return primary;
    }
    let mut alt = recognize_normalized(stripped);
    // Preserve original normalized (with commas) for display.
    alt.normalized = normalized;
    if alt.accepted {
        alt
    } else {
        primary
    }
}

fn bare_illegal_vowel_adjacency(word: &str) -> bool {
    let bytes = word.as_bytes();
    let mut byte_index = 0;
    while byte_index + 1 < bytes.len() {
        if is_v(bytes[byte_index]) && is_v(bytes[byte_index + 1]) {
            let pair = [bytes[byte_index], bytes[byte_index + 1]];
            if !matches!(&pair[..], b"ai" | b"ei" | b"oi" | b"au") {
                return true;
            }
        }
        byte_index += 1;
    }
    false
}

fn recognize_normalized(normalized: String) -> ShapeResult {
    // A written period at either edge belongs to the stream boundary, not to
    // the word's CFG derivation. Keep it in `normalized` for pause constraints.
    // Internal periods remain in the CFG input for compound cmavo.
    if normalized.starts_with("..") || normalized.ends_with("..") {
        return ShapeResult {
            accepted: false,
            classes: BTreeSet::new(),
            normalized,
        };
    }
    let body = normalized.trim_matches('.');
    let bytes = body.as_bytes();

    // Pure consonant fragments (pairs/triples fixtures) — table predicates only.
    if is_pure_consonant_fragment(bytes) {
        let classes = classify_fragment(bytes);
        return ShapeResult {
            accepted: !classes.is_empty(),
            classes,
            normalized,
        };
    }

    // Diphthong-only fragments (ai, iy, …)
    if bytes.len() == 2 {
        if let Ok(s) = std::str::from_utf8(bytes) {
            if classify_diphthong(s).is_some() {
                let mut classes = BTreeSet::new();
                classes.insert(MorphClass::Diphthong);
                // Also allow cmavo shape for off/on glides
                if phonotactics_ok(bytes) {
                    classes.extend(classify_via_cfg(bytes));
                }
                return ShapeResult {
                    accepted: true,
                    classes,
                    normalized,
                };
            }
        }
    }

    if !phonotactics_ok(bytes) {
        return ShapeResult {
            accepted: false,
            classes: BTreeSet::new(),
            normalized,
        };
    }

    let classes = classify_via_cfg(bytes);
    ShapeResult {
        accepted: !classes.is_empty(),
        classes,
        normalized,
    }
}

pub fn accept(input: &str) -> bool {
    recognize(input).accepted
}

pub fn classify(input: &str) -> BTreeSet<MorphClass> {
    recognize(input).classes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmavo_and_gismu_smoke() {
        assert!(accept(".a"));
        assert!(accept("ba"));
        assert!(accept("creka"));
        assert!(classify("creka").contains(&MorphClass::Gismu));
        assert!(classify("ba").contains(&MorphClass::Cmavo));
    }

    #[test]
    fn boundary_pause_does_not_change_au_word_shape() {
        let plain = recognize("au");
        let dotted = recognize(".au");
        assert_eq!(plain.classes, dotted.classes);
        assert!(plain.classes.contains(&MorphClass::Cmavo));
        assert!(!plain.classes.contains(&MorphClass::CmavoCompound));
        assert_eq!(dotted.normalized, ".au");
    }

    #[test]
    fn phonotaxis_fragments() {
        assert!(classify("fl").contains(&MorphClass::MedialPair));
        assert!(classify("bl").contains(&MorphClass::InitialPair));
        assert!(!accept("bf"));
        assert!(!accept("mz"));
        assert!(!accept("ndj"));
        assert!(!accept("sc"));
    }

    #[test]
    fn bad_lujvo_clusters_rejected() {
        assert!(!accept("patta'a"));
        assert!(!accept("mudsiclu"));
        assert!(!accept("ro'irre'o"));
        assert!(!accept("djeimz."));
    }

    #[test]
    fn fuhivla_initial() {
        assert!(accept("spraile"));
        assert!(!accept("ktraile"));
        assert!(!accept("trkaile"));
        assert!(accept("tricrxaceru"));
        assert!(accept("bongnanba"));
    }

    #[test]
    fn normalize_case_and_comma_fallback() {
        let r = recognize("DJOsefin.");
        assert_eq!(r.normalized, "djosefin.");
        let r2 = recognize("kat,r,in.");
        assert!(r2.accepted, "{r2:?}");
        assert!(
            r2.normalized == "kat,r,in." || r2.normalized == "katrin.",
            "normalized={}",
            r2.normalized
        );
        let r3 = recognize("DI,ky,jvo");
        assert!(r3.accepted, "{r3:?}");
        assert!(r3.classes.contains(&MorphClass::Lujvo));
        let r4 = recognize("bangrkore,a");
        assert!(r4.accepted, "{r4:?}");
    }
}
