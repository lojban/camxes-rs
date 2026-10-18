//! camxes-dialect lujvo with brivla/fu'ivla-stem rafsi
//! ([`super::MorphOptions::camxes_dialect`] only).
//!
//! CLL §4.5 knows only the short rafsi (CVC, CCV, CVV/CV'V), the 4-letter
//! form + `y` and the final 5-letter form. camxes (and current usage, cf. the
//! §4.16 proposal) also builds lujvo from the stem of any brivla:
//!
//! * a stem ending in a consonant, then `y` (camxes `fuhivla_head onset y`):
//!   `afxan`-y-`ru'u`, `kexmair`-y-`bau`, `bem`-y-`mantodi`;
//! * a stem ending in a vowel, then `'y` (camxes `fuhivla_head h y`):
//!   `baxmasi`-'y-`gu'e`, `olju`-'y-`taifu'i`;
//! * a `y` hyphen may be followed by an apostrophe, after which the next
//!   piece may begin with a vowel (camxes `y h?`): `ped`-y'-`utka`;
//! * the last piece may be any brivla-shaped tail (camxes `lujvo_core` ends
//!   in `fuhivla`): `bem`-y-`mantodi`, `dut`-y-`ku'arka`.
//!
//! Each piece must still be well formed: §3.7 consonant clusters
//! (permissible initial cluster at the start of a piece; inside, every
//! adjacent pair a permissible medial pair and none of the forbidden
//! triples `ndj`, `ndz`, `ntc`, `nts` — the pairwise test the fu'ivla
//! shapes use), vowels grouped as in §3.4–3.5 (diphthongs, or
//! a vowel followed by an on-glide), apostrophes only between vowels. A
//! vowel-initial piece needs a preceding pause or apostrophe (§4.9), or an
//! on-glide (`batky`-`uidje`). Standard short rafsi between the stems keep
//! the §4.6 `r`/`n` hyphen rule (`n` only before `r`). The analysis is used
//! only for words that no other CLL rule accepts and that contain `y`.

use crate::cll::phonology::{
    classify_diphthong, is_c, is_permissible_initial_cluster, is_permissible_initial_pair,
    is_permissible_medial_pair, is_v, FORBIDDEN_TRIPLES,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ctx {
    /// Word start (a pause precedes).
    Start,
    /// After a `y` hyphen.
    AfterY,
    /// After `y'` / `'y'`: an apostrophe precedes.
    AfterApostrophe,
    /// After a short rafsi without `y`.
    AfterRafsi,
}

/// Try to read `body` as a camxes-dialect lujvo; returns the pieces joined
/// with `+` when it is one and at least one piece is a non-CLL stem rafsi
/// or brivla tail.
pub fn try_parse(body: &str) -> Option<String> {
    let clean: String = body
        .trim_matches('.')
        .chars()
        .filter(|&c| c != ',')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    let b = clean.as_bytes();
    if !b.contains(&b'y')
        || b.iter()
            .any(|&c| !(is_letter(c) || c == b'y' || c == b'\''))
    {
        return None;
    }
    let mut pieces = Vec::new();
    if dfs(b, 0, Ctx::Start, false, &mut pieces) {
        Some(pieces.join("+"))
    } else {
        None
    }
}

fn is_letter(c: u8) -> bool {
    is_c(c) || is_v(c)
}

fn dfs(b: &[u8], pos: usize, ctx: Ctx, nonstd: bool, pieces: &mut Vec<String>) -> bool {
    let rest = &b[pos..];
    // Final piece.
    if !rest.contains(&b'y') {
        if let Some(final_std) = final_piece(rest, ctx) {
            if nonstd || !final_std {
                pieces.push(String::from_utf8_lossy(rest).into_owned());
                return true;
            }
        }
        return false;
    }
    // Short standard rafsi (no `y`), optionally with an r/n hyphen.
    for len in [3usize, 4] {
        if rest.len() <= len {
            continue;
        }
        let r = &rest[..len];
        if !short_rafsi(r) || !(ctx != Ctx::AfterRafsi || is_c(r[0])) {
            continue;
        }
        let cvv = r.len() == 4 || (is_v(r[1]) && is_v(r[2]));
        let mut ends = vec![len];
        if cvv && rest.len() > len + 1 {
            let (h, next) = (rest[len], rest[len + 1]);
            if (h == b'r' && is_c(next) && next != b'r') || (h == b'n' && next == b'r') {
                ends.push(len + 1);
            }
        }
        for end in ends {
            if !is_c(rest[end]) || rest[end] == b'y' {
                continue;
            }
            // Without a hyphen the two rafsi meet at a consonant pair.
            if end == len && is_c(r[len - 1]) && !is_permissible_medial_pair(r[len - 1], rest[end])
            {
                continue;
            }
            pieces.push(String::from_utf8_lossy(&rest[..end]).into_owned());
            if dfs(b, pos + end, Ctx::AfterRafsi, nonstd, pieces) {
                return true;
            }
            pieces.pop();
        }
    }
    // Stem rafsi: stem + `y` or stem + `'y`, then an optional apostrophe.
    let Some(k) = rest.iter().position(|&c| c == b'y') else {
        return false;
    };
    let stem = &rest[..k];
    if stem.is_empty() {
        return false;
    }
    let (core, apostrophe) = match stem.last() {
        Some(b'\'') => (&stem[..stem.len() - 1], true),
        _ => (stem, false),
    };
    let ok = if apostrophe {
        core.last().is_some_and(|&c| is_v(c)) && stem_ok(core, ctx) && hy_stem(core)
    } else {
        core.last().is_some_and(|&c| is_c(c)) && core.len() >= 3 && stem_ok(core, ctx)
    };
    if !ok {
        return false;
    }
    let std = !apostrophe && standard_y_rafsi(core);
    let mut end = k + 1;
    let mut next = Ctx::AfterY;
    if rest.get(end) == Some(&b'\'') {
        end += 1;
        next = Ctx::AfterApostrophe;
    }
    if end >= rest.len() {
        return false;
    }
    pieces.push(String::from_utf8_lossy(&rest[..end]).into_owned());
    if dfs(
        b,
        pos + end,
        next,
        nonstd || !std || next == Ctx::AfterApostrophe,
        pieces,
    ) {
        return true;
    }
    pieces.pop();
    false
}

/// CVC, CCV, CVV (ai/ei/oi/au) or CV'V.
fn short_rafsi(r: &[u8]) -> bool {
    match r {
        [c1, v, c2] if is_c(*c1) && is_v(*v) && is_c(*c2) => true,
        [c1, c2, v] if is_permissible_initial_pair(*c1, *c2) && is_v(*v) => true,
        [c, v1, v2]
            if is_c(*c)
                && matches!(
                    [*v1, *v2],
                    [b'a', b'i'] | [b'e', b'i'] | [b'o', b'i'] | [b'a', b'u']
                ) =>
        {
            true
        }
        [c, v1, b'\'', v2] if is_c(*c) && is_v(*v1) && is_v(*v2) => true,
        _ => false,
    }
}

/// CLL y-hyphen forms: CVC + y, CVCC + y, CCVC + y.
fn standard_y_rafsi(s: &[u8]) -> bool {
    match s {
        [c1, v, c2] => is_c(*c1) && is_v(*v) && is_c(*c2),
        [c1, x, y, c3] => {
            (is_c(*c1) && is_v(*x) && is_permissible_medial_pair(*y, *c3))
                || (is_permissible_initial_pair(*c1, *x) && is_v(*y) && is_c(*c3))
        }
        _ => false,
    }
}

/// A vowel-final stem before `'y`: two or more syllables, or a CCV / CVV
/// rafsi shape (camxes `hy_rafsi`); never a CV cmavo.
fn hy_stem(s: &[u8]) -> bool {
    syllables(s) >= 2 || (s.len() == 3 && short_rafsi(s))
}

/// The final piece; `Some(true)` for a CLL final rafsi or gismu shape,
/// `Some(false)` for any other brivla-shaped tail.
fn final_piece(f: &[u8], ctx: Ctx) -> Option<bool> {
    if f.is_empty() || !is_v(*f.last()?) || !stem_ok(f, ctx) {
        return None;
    }
    let off_glide = |a: u8, b: u8| {
        matches!(
            [a, b],
            [b'a', b'i'] | [b'e', b'i'] | [b'o', b'i'] | [b'a', b'u']
        )
    };
    let std = match f {
        [c, x, v] => {
            (is_c(*c) && off_glide(*x, *v)) || (is_permissible_initial_pair(*c, *x) && is_v(*v))
        }
        [c, v1, b'\'', v2] => is_c(*c) && is_v(*v1) && is_v(*v2),
        [c1, x, y, c3, v2] => {
            ((is_c(*c1) && is_v(*x) && is_permissible_medial_pair(*y, *c3))
                || (is_permissible_initial_pair(*c1, *x) && is_v(*y) && is_c(*c3)))
                && is_v(*v2)
        }
        _ => false,
    };
    if std {
        return Some(true);
    }
    (syllables(f) >= 2).then_some(false)
}

/// Vowel groups (syllable nuclei) in `s`, counting a vowel + on-glide
/// diphthong as two.
fn syllables(s: &[u8]) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < s.len() {
        if is_v(s[i]) {
            let start = i;
            while i < s.len() && is_v(s[i]) {
                i += 1;
            }
            n += group_count(&s[start..i]).unwrap_or(1);
        } else {
            i += 1;
        }
    }
    n
}

/// Split a vowel run into syllables (§3.4–3.5): diphthongs from the left,
/// a single vowel only at the end or before an on-glide (`i`/`u` + vowel).
fn group_count(run: &[u8]) -> Option<usize> {
    let mut i = 0;
    let mut n = 0;
    while i < run.len() {
        if i + 1 < run.len()
            && classify_diphthong(std::str::from_utf8(&run[i..i + 2]).ok()?)
                .is_some_and(|d| !matches!(d, crate::cll::phonology::DiphthongClass::NameOnly))
        {
            i += 2;
        } else if i + 1 == run.len() || (matches!(run[i + 1], b'i' | b'u') && i + 2 < run.len()) {
            i += 1;
        } else {
            return None;
        }
        n += 1;
    }
    Some(n)
}

/// A piece without `y`: §3.7 clusters, §3.4–3.5 vowel groups, apostrophes
/// between vowels, and a vowel start only where §4.9 allows it.
fn stem_ok(s: &[u8], ctx: Ctx) -> bool {
    if s.is_empty() || !s.iter().any(|&c| is_v(c)) {
        return false;
    }
    for (i, &c) in s.iter().enumerate() {
        if c == b'\'' && !(i > 0 && i + 1 < s.len() && is_v(s[i - 1]) && is_v(s[i + 1])) {
            return false;
        }
    }
    let lead = s.iter().take_while(|&&c| is_c(c)).count();
    if lead == 0 {
        let glide = s.len() > 1 && matches!(s[0], b'i' | b'u') && is_v(s[1]);
        let allowed = match ctx {
            Ctx::Start | Ctx::AfterApostrophe => true,
            Ctx::AfterY => glide,
            Ctx::AfterRafsi => false,
        };
        if !allowed {
            return false;
        }
    } else if lead >= 2 && !is_permissible_initial_cluster(&s[..lead]) {
        return false;
    }
    // Clusters after the first vowel; vowel runs.
    let mut i = lead;
    while i < s.len() {
        if is_c(s[i]) {
            let start = i;
            while i < s.len() && is_c(s[i]) {
                i += 1;
            }
            let cl = &s[start..i];
            if !cl
                .windows(2)
                .all(|w| is_permissible_medial_pair(w[0], w[1]))
                || cl
                    .windows(3)
                    .any(|w| FORBIDDEN_TRIPLES.contains(&std::str::from_utf8(w).unwrap_or("")))
            {
                return false;
            }
        } else if is_v(s[i]) {
            let start = i;
            while i < s.len() && is_v(s[i]) {
                i += 1;
            }
            if group_count(&s[start..i]).is_none() {
                return false;
            }
        } else {
            i += 1;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stem_rafsi_lujvo() {
        for w in [
            "afxanyru'u",
            "bemymantodi",
            "pedy'utka",
            "baxmasi'ygu'e",
            "kexmairybau",
            "olju'ytaifu'i",
            "batkyuidje",
            "velski'y'esku",
            "prejumydu'a'y'utka",
        ] {
            assert!(try_parse(w).is_some(), "{w}");
        }
    }

    #[test]
    fn not_stem_rafsi_lujvo() {
        // CLL lujvo (no non-CLL piece), cmavo + 'y, vowel after a plain y,
        // a forbidden cluster, n-hyphen before a non-r consonant.
        for w in [
            "klamydja",
            "si'ygu'e",
            "bemyantodi",
            "bemzmyru'u",
            "zainkemgau",
        ] {
            assert!(try_parse(w).is_none(), "{w}");
        }
    }
}
