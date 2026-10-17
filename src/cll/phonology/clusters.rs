//! Consonant pairs and triples — CLL §3.6–3.7.
//!
//! Medial pairs: allowed unless matching a prohibition (§3.6).
//! Initial pairs (non-names): exactly the 48 listed in §3.7.
//! Triples: first pair medial-OK, last pair initial-OK; forbid ndj/ndz/ntc/nts.
//! Fu'ivla-initial longer clusters: each adjacent pair must be initial-OK (§4.7.1).

/// CLL §3.7 — 48 permissible initial consonant pairs.
pub const INITIAL_PAIRS: &[&str] = &[
    "bl", "br", "cf", "ck", "cl", "cm", "cn", "cp", "cr", "ct", "dj", "dr", "dz", "fl", "fr", "gl",
    "gr", "jb", "jd", "jg", "jm", "jv", "kl", "kr", "ml", "mr", "pl", "pr", "sf", "sk", "sl", "sm",
    "sn", "sp", "sr", "st", "tc", "tr", "ts", "vl", "vr", "xl", "xr", "zb", "zd", "zg", "zm", "zv",
];

/// CLL §3.7 — forbidden triples.
pub const FORBIDDEN_TRIPLES: &[&str] = &["ndj", "ndz", "ntc", "nts"];

/// CLL §3.6 — specifically forbidden pairs (rule 4).
pub const FORBIDDEN_PAIRS: &[&str] = &["cx", "kx", "xc", "xk", "mz"];

const VOICED: &[u8] = b"bdgvjz";
const UNVOICED: &[u8] = b"ptkfcsx";
const SIBILANT_SET: &[u8] = b"cjsz";

fn is_voiced_obstruent(consonant: u8) -> bool {
    VOICED.contains(&consonant)
}
fn is_unvoiced_obstruent(consonant: u8) -> bool {
    UNVOICED.contains(&consonant)
}

/// CLL §3.6 medial pair predicate (by prohibition).
pub fn is_permissible_medial_pair(first: u8, second: u8) -> bool {
    if !super::letters::is_c(first) || !super::letters::is_c(second) {
        return false;
    }
    if first == second {
        return false; // no doubles
    }
    // Mixed voicing forbidden; l m n r are not in the voiced/unvoiced tables, so exempt.
    if (is_voiced_obstruent(first) && is_unvoiced_obstruent(second))
        || (is_unvoiced_obstruent(first) && is_voiced_obstruent(second))
    {
        return false;
    }
    if SIBILANT_SET.contains(&first) && SIBILANT_SET.contains(&second) {
        return false;
    }
    let pair = [first, second];
    if pair == *b"cx" || pair == *b"kx" || pair == *b"xc" || pair == *b"xk" || pair == *b"mz" {
        return false;
    }
    true
}

pub fn is_permissible_initial_pair(first: u8, second: u8) -> bool {
    let pair_bytes = [first, second];
    let Ok(pair) = std::str::from_utf8(&pair_bytes) else {
        return false;
    };
    INITIAL_PAIRS.contains(&pair)
}

/// CLL §3.7 medial triple: medial OK on first two, initial OK on last two; not forbidden.
pub fn is_permissible_triple(first: u8, second: u8, third: u8) -> bool {
    let triple_bytes = [first, second, third];
    let Ok(triple) = std::str::from_utf8(&triple_bytes) else {
        return false;
    };
    if FORBIDDEN_TRIPLES.contains(&triple) {
        return false;
    }
    is_permissible_medial_pair(first, second) && is_permissible_initial_pair(second, third)
}

/// CLL §4.7.1 — word-initial cluster of length ≥2: every adjacent pair is initial-OK.
pub fn is_permissible_initial_cluster(chars: &[u8]) -> bool {
    if chars.len() < 2 {
        return false;
    }
    chars
        .windows(2)
        .all(|pair| is_permissible_initial_pair(pair[0], pair[1]))
}

/// Enumerate all permissible medial pairs (finite; for CFG literal union).
pub fn all_medial_pairs() -> Vec<(u8, u8)> {
    let mut out = Vec::new();
    for &first in super::letters::CONSONANTS {
        for &second in super::letters::CONSONANTS {
            if is_permissible_medial_pair(first, second) {
                out.push((first, second));
            }
        }
    }
    out
}

/// Enumerate initial triples where each adjacent pair is initial-OK (§4.7.1).
pub fn all_initial_triples() -> Vec<(u8, u8, u8)> {
    let mut out = Vec::new();
    for &p1 in INITIAL_PAIRS {
        let b1 = p1.as_bytes();
        for &p2 in INITIAL_PAIRS {
            let b2 = p2.as_bytes();
            if b1[1] == b2[0] {
                out.push((b1[0], b1[1], b2[1]));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pairs_count_is_48() {
        assert_eq!(INITIAL_PAIRS.len(), 48);
    }

    #[test]
    fn medial_examples_from_cll() {
        assert!(is_permissible_medial_pair(b'f', b'l'));
        assert!(is_permissible_medial_pair(b'v', b'l'));
        assert!(is_permissible_medial_pair(b'l', b's'));
        assert!(is_permissible_medial_pair(b'l', b'z'));
        assert!(!is_permissible_medial_pair(b'b', b'f'));
        assert!(!is_permissible_medial_pair(b's', b'd'));
        assert!(!is_permissible_medial_pair(b't', b't'));
        assert!(!is_permissible_medial_pair(b'c', b'x'));
        assert!(!is_permissible_medial_pair(b'm', b'z'));
        assert!(!is_permissible_medial_pair(b'c', b's'));
        assert!(!is_permissible_medial_pair(b's', b'c')); // both sibilants — §3.6.3
    }

    #[test]
    fn forbidden_triples() {
        assert!(!is_permissible_triple(b'n', b'd', b'j'));
        assert!(!is_permissible_triple(b'n', b't', b's'));
    }

    #[test]
    fn fuhivla_initial_clusters() {
        assert!(is_permissible_initial_cluster(b"sp"));
        assert!(is_permissible_initial_cluster(b"spr")); // sp+pr
        assert!(!is_permissible_initial_cluster(b"kt"));
        assert!(!is_permissible_initial_cluster(b"trk")); // rk not initial
    }

    #[test]
    fn medial_pair_count_stable() {
        assert_eq!(all_medial_pairs().len(), 179);
    }
}
