//! camxes word-type mapping for the Earley engine (camxes-rs only; not part
//! of the vendored tergenzalvi code in `crate::cll`).
//!
//! The CLL pipeline says which classes a word *can* have (cmavo, gismu,
//! lujvo, fu'ivla, cmevla). This module picks the camxes word type for the
//! `morphology` API ([`word_class`]) and decides the rule-based
//! standard/obsolete flag ([`is_obsolete_form`]: a port of the camxes
//! syllable rules). Neither changes what the CLL pipeline accepts.

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

/// A lowercased word (pauses dropped, commas as syllable breaks) with the
/// camxes word-level predicates. Each returns the end of its match, PEG
/// style: ordered choice, no backtracking into a committed alternative.
struct Word {
    w: Vec<u8>,
}

type M = Option<usize>;

impl Word {
    fn new(text: &str) -> Self {
        // A comma is a syllable break (CLL §3.3), as in the camxes
        // morphology grammar: before a nucleus it separates syllables like
        // an apostrophe; before a consonant or an on-glide it is dropped.
        let raw: Vec<u8> = text
            .bytes()
            .filter(|&b| b != b'.')
            .map(|b| b.to_ascii_lowercase())
            .collect();
        let mut w = Vec::with_capacity(raw.len());
        for (i, &b) in raw.iter().enumerate() {
            if b != b',' {
                w.push(b);
                continue;
            }
            let next = raw.get(i + 1).copied().unwrap_or(0);
            let after = raw.get(i + 2).copied().unwrap_or(0);
            let glide = matches!(next, b'i' | b'u') && b"aeiouy".contains(&after);
            if b"aeiouy".contains(&next) && !glide {
                w.push(b'\'');
            }
        }
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
    fn obsolete_forms() {
        assert!(is_obsolete_form("kuarka", false));
        assert!(is_obsolete_form("cie'a'u", false));
        assert!(!is_obsolete_form("klama", false));
        assert!(is_obsolete_form("tiuanjous", true));
        assert!(!is_obsolete_form("djan", true));
    }
}
