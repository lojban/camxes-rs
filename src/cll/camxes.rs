//! camxes compatibility rules (local to camxes-rs, not part of
//! tergenzalvi).
//!
//! The CLL rules of this module tree say whether a word *can* be a cmavo,
//! gismu, lujvo, fu'ivla or cmevla. camxes, the standard morphology used by
//! the PEG engine, picks one class per word and accepts some forms the CLL
//! does not describe (experimental cmavo, apostrophe chains, ...; those
//! extensions are marked "camxes extension" in the code). This module holds
//! the rules that turn a CLL result into the camxes word class, and the
//! rules that recognize forms camxes rejects (obsolete forms).

use crate::cll::constraints::ConstrainedResult;
use crate::cll::lex::WordClass;
use crate::cll::phonology::is_c;
use crate::cll::shapes::MorphClass;

/// The camxes class of an accepted CLL result for the word `text`, or
/// `None` when the result is rejected or has no word class.
///
/// camxes tries cmevla, cmavo, gismu, fu'ivla, lujvo in that order, but its
/// fu'ivla rule excludes words that start with a rafsi string, so a word
/// that is both a CLL lujvo and a CLL fu'ivla is a fu'ivla exactly when it
/// does not start with one ([`starts_with_rafsi_string`]).
pub fn word_class(r: &ConstrainedResult, text: &str) -> Option<WordClass> {
    if !r.accepted {
        return None;
    }
    let has = |c: MorphClass| r.classes.contains(&c);
    if has(MorphClass::Cmevla) {
        Some(WordClass::Cmevla)
    } else if has(MorphClass::Cmavo) || has(MorphClass::CmavoCompound) {
        Some(WordClass::Cmavo)
    } else if has(MorphClass::Gismu) {
        Some(WordClass::Gismu)
    } else if has(MorphClass::Lujvo) && has(MorphClass::Fuhivla) {
        // camxes: a word starting with a rafsi string is a lujvo. Words
        // outside camxes morphology (obsolete forms, see
        // `is_obsolete_form`) fall back to the CLL lujvo rules (form-based
        // rafsi segmentation with valid hyphens).
        Some(
            if starts_with_rafsi_string(text)
                || (is_obsolete_form(text, false)
                    && crate::cll::constraints::lujvo_hyphen::check_lujvo(text).is_ok())
            {
                WordClass::Lujvo
            } else {
                WordClass::Fuhivla
            },
        )
    } else if has(MorphClass::Lujvo) {
        Some(WordClass::Lujvo)
    } else if has(MorphClass::Fuhivla) {
        Some(WordClass::Fuhivla)
    } else {
        None
    }
}

/// camxes `rafsi_string` at the start of the word: one or more rafsi that
/// end in a `y` hyphen, or in a final rafsi that ends the word. Words that
/// start like this are never fu'ivla in camxes.
pub fn starts_with_rafsi_string(text: &str) -> bool {
    Word::new(text).rafsi_string(0).is_some()
}

/// camxes extension (not CLL): a lujvo joined with `y` hyphens whose pieces
/// CLL cannot read as rafsi. camxes lets any brivla-like head end in a
/// `y` hyphen (`eiks-y-spe`, `abniien-y-bau`; a vowel-final head takes
/// `'y`: `baxmasi'-y-gu'e`) and lets the last piece be any brivla
/// (`cimn-y'-efku`, `batk-y-uidje`).
///
/// The word is cut after every `y` (and a following apostrophe). Each head
/// piece must contain a vowel and no `y`, and end in a consonant (before
/// `y`) or an apostrophe (`'y`); `last` decides whether the final piece is
/// a gismu, lujvo, fu'ivla or final rafsi.
pub fn is_y_hyphen_lujvo(text: &str, last: impl Fn(&str) -> bool) -> bool {
    let w: String = text
        .chars()
        .filter(|&c| c != '.')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if !w.bytes().all(|b| b.is_ascii_lowercase() || b == b'\'') {
        return false;
    }
    let bytes = w.as_bytes();
    let mut start = 0;
    let mut heads = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'y' {
            let head = &bytes[start..i];
            let ok = head.iter().any(|&b| crate::cll::phonology::is_v(b))
                && head.last().is_some_and(|&b| is_c(b) || b == b'\'')
                && !head.starts_with(b"'");
            if !ok {
                return false;
            }
            heads += 1;
            i += 1;
            if bytes.get(i) == Some(&b'\'') {
                i += 1;
            }
            start = i;
            continue;
        }
        i += 1;
    }
    heads > 0 && start < bytes.len() && last(&w[start..])
}

/// Short final rafsi shapes (CVV, CV'V, CCV with a permissible initial).
pub fn is_final_short_rafsi(piece: &str) -> bool {
    use crate::cll::phonology::{is_permissible_initial_pair, is_v};
    match piece.as_bytes() {
        [c, a, b] if is_c(*c) && is_v(*a) && is_v(*b) => {
            crate::cll::phonology::DIPHTHONG_OFF.contains(&&piece[1..])
        }
        [c, a, b'\'', b] => is_c(*c) && is_v(*a) && is_v(*b),
        [c1, c2, v] => is_c(*c1) && is_c(*c2) && is_v(*v) && is_permissible_initial_pair(*c1, *c2),
        _ => false,
    }
}

/// camxes extension (not CLL §4.3 "consonant cluster in the first five
/// letters"): camxes only asks that a brivla not fall apart into a cmavo
/// and another word. A fu'ivla-shaped word with a later cluster is kept when
/// its longest leading cmavo form (`fau'a` in `fau'arma`, `ra'oi` in
/// `ra'oijbo`) is followed by a consonant-initial rest that is not a word.
pub fn late_cluster_fuhivla(text: &str) -> bool {
    let w = Word::new(text);
    let letters = &w.w;
    if !letters.windows(2).any(|p| is_c(p[0]) && is_c(p[1])) {
        return false;
    }
    match cmavo_prefix_rest(&w) {
        Some(rest) => !is_word_sequence(&rest, 8),
        None => false,
    }
}

/// camxes `brivla_head = !cmavo ...`: a word that starts with a cmavo form
/// followed by a consonant-initial sequence of words (`mi` + `brainmu` in
/// `mibrainmu`) is not a brivla; camxes splits it.
pub fn cmavo_prefix_splits(text: &str) -> bool {
    let w = Word::new(text);
    // camxes `cmavo = !CVCy_lujvo ...`: `babylujvo` is a lujvo, not
    // `ba` + `by` + `lujvo`.
    let cvcy = matches!(w.w.as_slice(), [c1, v, c2, b'y', ..] if is_c(*c1) && crate::cll::phonology::is_v(*v) && is_c(*c2));
    !cvcy && cmavo_prefix_rest(&w).is_some_and(|rest| is_word_sequence(&rest, 8))
}

/// The whole text is a camxes `rafsi_string` (rafsi ending in a final
/// rafsi), e.g. the `xe'idaiseldi'e` tail of a y-hyphen lujvo.
pub fn is_rafsi_string(text: &str) -> bool {
    let w = Word::new(text);
    w.rafsi_string(0) == Some(w.w.len())
}

/// The rest of the word after its longest leading camxes cmavo form
/// `C?(N')*N`, when that rest starts with a consonant.
fn cmavo_prefix_rest(w: &Word) -> Option<String> {
    let mut p = w.zunsna(0).unwrap_or(0);
    loop {
        let e = w.nucleus(p)?;
        match w.h(e) {
            Some(after) => p = after,
            None => {
                p = e;
                break;
            }
        }
    }
    (p > 0 && p < w.w.len() && is_c(w.w[p]))
        .then(|| String::from_utf8_lossy(&w.w[p..]).into_owned())
}

/// camxes `lojban_word` after a cmavo: a word, or (recursively) a cmavo
/// followed by such a sequence (`lerenanmu` = `le` + `re` + `nanmu`).
/// Only camxes-valid (standard, see [`is_obsolete_form`]) words count, as in
/// camxes itself (`matriocka` is not split into `ma` + obsolete `triocka`).
fn is_word_sequence(text: &str, depth: usize) -> bool {
    let r = crate::cll::constraints::morph_parse(text);
    if r.accepted && !is_obsolete_form(text, r.classes.contains(&MorphClass::Cmevla)) {
        return true;
    }
    depth > 0
        && cmavo_prefix_rest(&Word::new(text))
            .is_some_and(|rest| is_word_sequence(&rest, depth - 1))
}

/// camxes reads a comma as a pause (a word break) unless a nucleus
/// follows it, in which case it is a syllable break like `'` (`da,ic` is one
/// name, `nu,IORK` is `nu` + `IORK`). Returns the byte offsets of the commas
/// in `text` that are pauses.
pub fn pause_commas(text: &str) -> Vec<usize> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    for (i, b) in lower.bytes().enumerate() {
        if b != b',' {
            continue;
        }
        let rest = Word {
            w: lower.as_bytes()[i + 1..].to_vec(),
        };
        if rest.nucleus(0).is_none() {
            out.push(i);
        }
    }
    out
}

/// True when `word` (one word of the given class) uses a form that CLL
/// Chapters 3–4 allow but camxes rejects (an obsolete form). The camxes
/// word-form rules are applied directly:
///
/// * a name must be a camxes `zifcme`: letters that read as nuclei,
///   on-glides, apostrophes and consonants, ending in a consonant (no vowel
///   hiatus such as `ou` in `guanJOUS`, no `n` before an affricate);
/// * any other word must split into camxes syllables (`any_syllable+`): each
///   consonant cluster is at most one coda consonant (or a syllabic
///   consonant) plus a valid onset, and the word starts with an onset. This rejects an on-glide right after a
///   consonant that cannot close the previous syllable (`kuarka`,
///   `fiesta`, `cia'o'e`, `gondii'i`), clusters that are not onsets
///   (`mastla`, `venzla`) and `n` before an affricate.
pub fn is_obsolete_form(word: &str, is_cmevla: bool) -> bool {
    let w = Word::new(word);
    if is_cmevla {
        !w.zifcme()
    } else {
        !w.syllables()
    }
}

/// A lowercased word (pauses dropped, commas read as apostrophes) with the
/// camxes word-level predicates. Each returns the end of its match, PEG
/// style: ordered choice, no backtracking into a committed alternative.
struct Word {
    w: Vec<u8>,
}

type M = Option<usize>;

impl Word {
    fn new(text: &str) -> Self {
        let w = text
            .bytes()
            .filter(|&b| b != b'.')
            .map(|b| match b.to_ascii_lowercase() {
                b',' => b'\'',
                b => b,
            })
            .collect();
        Word { w }
    }
    fn at(&self, i: usize) -> u8 {
        self.w.get(i).copied().unwrap_or(0)
    }
    fn lit(&self, i: usize, set: &[u8]) -> M {
        set.contains(&self.at(i)).then_some(i + 1)
    }
    fn pause(&self, i: usize) -> bool {
        i >= self.w.len()
    }
    fn affricate(&self, i: usize) -> M {
        match (self.at(i), self.at(i + 1)) {
            (b't', b'c' | b's') | (b'd', b'j' | b'z') => Some(i + 2),
            _ => None,
        }
    }
    fn zunsna(&self, i: usize) -> M {
        let c = self.at(i);
        if c == b'n' && self.affricate(i + 1).is_some() {
            return None;
        }
        (c != 0 && is_c(c)).then_some(i + 1)
    }
    fn syllabic(&self, i: usize) -> M {
        if self.at(i) == b'n' && self.affricate(i + 1).is_some() {
            return None;
        }
        self.lit(i, b"lmnr")
    }
    fn pa_zei_karsna(&self, i: usize) -> M {
        let e = self.lit(i, b"aeiou")?;
        self.nucleus(e).is_none().then_some(e)
    }
    fn re_zei_karsna(&self, i: usize) -> M {
        let e = match (self.at(i), self.at(i + 1)) {
            (b'a', b'u') if self.at(i + 2) != b'u' => i + 2,
            (b'a' | b'e' | b'o', b'i') if self.at(i + 2) != b'i' => i + 2,
            _ => return None,
        };
        self.nucleus(e).is_none().then_some(e)
    }
    fn y(&self, i: usize) -> M {
        let e = self.lit(i, b"y")?;
        let non_y_nucleus = self.at(e) != b'y' && self.nucleus(e).is_some();
        (!non_y_nucleus).then_some(e)
    }
    fn nucleus(&self, i: usize) -> M {
        self.pa_zei_karsna(i)
            .or_else(|| self.re_zei_karsna(i))
            .or_else(|| {
                let e = self.y(i)?;
                self.nucleus(e).is_none().then_some(e)
            })
    }
    fn h(&self, i: usize) -> M {
        let e = self.lit(i, b"'")?;
        self.nucleus(e).map(|_| e)
    }
    fn glaide(&self, i: usize) -> M {
        let e = self.lit(i, b"iu")?;
        self.nucleus(e).map(|_| e)
    }
    fn onset(&self, i: usize) -> M {
        let e = self
            .h(i)
            .or_else(|| self.glaide(i))
            .or_else(|| self.affricate(i))
            .unwrap_or_else(|| {
                let mut p = i;
                match self.at(p) {
                    b'c' | b's' if self.at(p + 1) != b'x' => p += 1,
                    b'j' | b'z' if !matches!(self.at(p + 1), b'n' | b'l' | b'r') => p += 1,
                    _ => {}
                }
                match self.at(p) {
                    b'p' | b'f' | b'b' | b'g' | b'v' | b'k' | b'x' | b'm' => p += 1,
                    b't' | b'd' if self.at(p + 1) != b'l' => p += 1,
                    b'n' if self.at(p + 1) != b'l'
                        && self.at(p + 1) != b'r'
                        && self.affricate(p + 1).is_none() =>
                    {
                        p += 1
                    }
                    _ => {}
                }
                if matches!(self.at(p), b'l' | b'r') {
                    p += 1;
                }
                p
            });
        (self.zunsna(e).is_none() && self.glaide(e).is_none()).then_some(e)
    }
    fn initial_pair(&self, i: usize) -> M {
        self.onset(i)?;
        let e = self.zunsna(self.zunsna(i)?)?;
        self.zunsna(e).is_none().then_some(e)
    }
    fn any_syllable(&self, i: usize) -> M {
        let a = self
            .onset(i)
            .and_then(|e| self.nucleus(e))
            .map(|e| self.coda(e).unwrap_or(e));
        a.or_else(|| self.consonantal_syllable(i))
    }
    fn consonantal_syllable(&self, i: usize) -> M {
        let e = self.zunsna(i)?;
        self.syllabic(e)?;
        self.coda(e)
    }
    fn coda(&self, i: usize) -> M {
        if self.any_syllable(i).is_none() {
            if let Some(e) = self.zunsna(i) {
                if self.any_syllable(e).is_some() {
                    return Some(e);
                }
            }
        }
        let mut p = i;
        if let Some(e) = self.syllabic(p) {
            p = e;
        }
        if let Some(e) = self.zunsna(p) {
            p = e;
        }
        self.pause(p).then_some(p)
    }
    fn slaka(&self, i: usize) -> M {
        let e = self.onset(i)?;
        if self.y(e).is_some() {
            return None;
        }
        let e = self.nucleus(e)?;
        Some(self.coda(e).unwrap_or(e))
    }
    fn stress(&self, i: usize) -> bool {
        let mut p = i;
        while let Some(e) = self.zunsna(p).or_else(|| self.glaide(p)) {
            p = e;
        }
        if let Some(e) = self.h(p) {
            p = e;
        }
        if let Some(e) = self.y(p) {
            p = e;
        }
        self.slaka(p).is_some_and(|e| self.pause(e))
    }
    fn final_syllable(&self, i: usize) -> M {
        let e = self.onset(i)?;
        if self.y(e).is_some() {
            return None;
        }
        let e = self.nucleus(e)?;
        self.pause(e).then_some(e)
    }
    fn cv(&self, i: usize) -> M {
        self.pa_zei_karsna(self.zunsna(i)?)
    }
    fn cvc(&self, i: usize) -> M {
        self.zunsna(self.cv(i)?)
    }
    fn ccv(&self, i: usize) -> M {
        self.pa_zei_karsna(self.initial_pair(i)?)
    }
    fn cvv(&self, i: usize) -> M {
        self.re_zei_karsna(self.zunsna(i)?)
    }
    fn cvc_ccv(&self, i: usize) -> M {
        self.cvc(i).or_else(|| self.ccv(i))
    }
    fn cvvr(&self, i: usize) -> M {
        let hv = || {
            let e = self.pa_zei_karsna(self.zunsna(i)?)?;
            if self.stress(e) {
                return None;
            }
            self.pa_zei_karsna(self.h(e)?)
        };
        let e = hv().or_else(|| self.cvv(i))?;
        let r = self.lit(e, b"r").filter(|&r| self.zunsna(r).is_some());
        let n = || self.lit(e, b"n").filter(|&n| self.at(n) == b'r');
        Some(r.or_else(n).unwrap_or(e))
    }
    fn cvc_ccv_cvv(&self, i: usize) -> M {
        self.cvc_ccv(i).or_else(|| self.cvvr(i))
    }
    fn long_rafsi(&self, i: usize) -> M {
        self.zunsna(self.cvc_ccv(i)?)
    }
    fn short_final_rafsi(&self, i: usize) -> M {
        self.final_syllable(i)?;
        let e = self
            .zunsna(i)
            .and_then(|e| self.re_zei_karsna(e))
            .or_else(|| self.ccv(i))?;
        self.pause(e).then_some(e)
    }
    fn gismu(&self, i: usize) -> M {
        let e = self.long_rafsi(i)?;
        if !self.stress(e) {
            return None;
        }
        self.final_syllable(e)?;
        let e = self.pa_zei_karsna(e)?;
        self.pause(e).then_some(e)
    }
    fn gismu_cvv_final_rafsi(&self, i: usize) -> M {
        self.gismu(i).or_else(|| {
            let e = self.cv(i)?;
            if !self.stress(e) {
                return None;
            }
            let e = self.h(e)?;
            self.final_syllable(e)?;
            let e = self.pa_zei_karsna(e)?;
            self.pause(e).then_some(e)
        })
    }
    fn long_or_cvc(&self, i: usize) -> M {
        self.long_rafsi(i).or_else(|| self.cvc(i))
    }
    fn y_rafsi(&self, i: usize) -> M {
        let e = self.long_or_cvc(i)?;
        if self.stress(e) {
            return None;
        }
        let e = self.y(e)?;
        Some(self.h(e).unwrap_or(e))
    }
    fn stressed_y_rafsi(&self, i: usize) -> M {
        let e = self.long_or_cvc(i)?;
        if !self.stress(e) {
            return None;
        }
        self.y(e)
    }
    fn hy_base(&self, i: usize) -> M {
        self.long_rafsi(i)
            .and_then(|e| self.pa_zei_karsna(e))
            .or_else(|| self.cvc_ccv_cvv(i))
    }
    fn hy_rafsi(&self, i: usize) -> M {
        let e = self.hy_base(i)?;
        if self.stress(e) {
            return None;
        }
        let e = self.y(self.h(e)?)?;
        Some(self.h(e).unwrap_or(e))
    }
    fn stressed_hy_rafsi(&self, i: usize) -> M {
        let e = self.hy_base(i)?;
        if !self.stress(e) {
            return None;
        }
        self.y(self.h(e)?)
    }
    fn y_less_rafsi(&self, i: usize) -> M {
        if self.y_rafsi(i).is_some()
            || self.stressed_y_rafsi(i).is_some()
            || self.hy_rafsi(i).is_some()
            || self.stressed_hy_rafsi(i).is_some()
        {
            return None;
        }
        let e = self.cvc_ccv_cvv(i)?;
        (!self.stress(e) && self.y(e).is_none() && self.h(e).is_none()).then_some(e)
    }
    /// camxes `zifcme` over the whole word.
    fn zifcme(&self) -> bool {
        if self.h(0).is_some() {
            return false;
        }
        let mut p = 0;
        loop {
            let next = self
                .nucleus(p)
                .or_else(|| self.glaide(p))
                .or_else(|| self.h(p))
                .or_else(|| self.zunsna(p).filter(|&e| !self.pause(e)));
            match next {
                Some(e) => p = e,
                None => break,
            }
        }
        self.zunsna(p).is_some_and(|e| self.pause(e))
    }
    /// The word starts with a camxes onset and splits into camxes syllables.
    fn syllables(&self) -> bool {
        if self.onset(0).is_none() {
            return false;
        }
        let mut p = 0;
        while !self.pause(p) {
            match self.any_syllable(p) {
                Some(e) if e > p => p = e,
                _ => return false,
            }
        }
        p > 0
    }
    fn rafsi_string(&self, i: usize) -> M {
        let mut p = i;
        while let Some(e) = self.y_less_rafsi(p) {
            p = e;
        }
        self.gismu_cvv_final_rafsi(p)
            .or_else(|| {
                let e = self.cvc_ccv_cvv(p)?;
                if !self.stress(e) || self.y(e).is_some() {
                    return None;
                }
                self.short_final_rafsi(e)
            })
            .or_else(|| self.y_rafsi(p))
            .or_else(|| self.stressed_y_rafsi(p))
            .or_else(|| {
                let q = self
                    .cvc_ccv_cvv(p)
                    .filter(|&e| self.stress(e) && self.y(e).is_none())
                    .unwrap_or(p);
                self.y(self.initial_pair(q)?)
            })
            .or_else(|| self.hy_rafsi(p))
            .or_else(|| self.stressed_hy_rafsi(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rafsi_string_starts() {
        for w in ["nilbra", "bakcmo", "zbasaigau", "eiksyspe"] {
            assert_eq!(starts_with_rafsi_string(w), w != "eiksyspe", "{w}");
        }
        for w in [
            "bicrbombu",
            "tirxpardu",
            "spageti",
            "kulnrfarsi",
            "klamrkalci",
        ] {
            assert!(!starts_with_rafsi_string(w), "{w}");
        }
    }

    #[test]
    fn y_hyphen_lujvo_pieces() {
        let last = |p: &str| is_final_short_rafsi(p) || p == "efku";
        assert!(is_y_hyphen_lujvo("eiksyspe", last));
        assert!(is_y_hyphen_lujvo("baxmasi'ygu'e", last));
        assert!(is_y_hyphen_lujvo("cimny'efku", last));
        assert!(!is_y_hyphen_lujvo("posytmo", last));
        assert!(!is_y_hyphen_lujvo("klama", last));
    }
}
