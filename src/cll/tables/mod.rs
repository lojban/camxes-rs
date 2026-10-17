//! Rafsi / gismu lexicon for constraints C13 / C23.
//!
//! ## Provenance
//! CLL Chapter 4 **does not print** a full gismu↔rafsi assignment table (DocBook
//! `04.xml` / korpora2 `cll/4.md` contain worked examples only). CLL §4.11
//! explicitly presupposes “a complete list of gismu and their assigned rafsi.”
//!
//! This crate embeds the **LLG baselined** lists dated **1994-09-26**
//! (`lojban.org/publications/wordlists/{gismu,rafsi.txt}`), contemporaneous with
//! CLL — **not** modern jbovlaste experimental rafsi. Long (4-letter / 5-letter)
//! rafsi are derived for every baselined gismu per CLL §4.6.
//!
//! See the repository README and `rafsi_provenance.json`.

mod rafsi_data;

use rafsi_data::{
    FINAL5_RAFSI_COUNT, GISMU_COUNT, LONG4_RAFSI_COUNT, RAFSI_ENTRIES, RAFSI_ENTRY_COUNT,
    SHORT_RAFSI_COUNT,
};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Kind of rafsi form (CLL §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RafsiKind {
    Short,
    Long4,
    Final5,
}

impl RafsiKind {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "short" => Some(Self::Short),
            "long4" => Some(Self::Long4),
            "final5" => Some(Self::Final5),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RafsiEntry {
    pub rafsi: &'static str,
    pub source: &'static str,
    pub kind: RafsiKind,
}

/// CV shape family of a rafsi spelling (for hyphen rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RafsiShape {
    Cvc,
    Ccv,
    Cvv,
    Long4,
    Final5,
}

pub fn classify_rafsi_shape(rafsi: &str) -> Option<RafsiShape> {
    let rafsi_bytes = rafsi.as_bytes();
    match rafsi_bytes {
        [c1, v, c2] if is_c(*c1) && is_v(*v) && is_c(*c2) => Some(RafsiShape::Cvc),
        [c1, c2, v] if is_c(*c1) && is_c(*c2) && is_v(*v) => Some(RafsiShape::Ccv),
        [c1, v1, v2] if is_c(*c1) && is_v(*v1) && is_v(*v2) => Some(RafsiShape::Cvv),
        [c1, v1, b'\'', v2] if is_c(*c1) && is_v(*v1) && is_v(*v2) => Some(RafsiShape::Cvv),
        [c1, v, c2, c3] if is_c(*c1) && is_v(*v) && is_c(*c2) && is_c(*c3) => {
            Some(RafsiShape::Long4)
        }
        [c1, c2, v, c3] if is_c(*c1) && is_c(*c2) && is_v(*v) && is_c(*c3) => {
            Some(RafsiShape::Long4)
        }
        [c1, v1, c2, c3, v2] if is_c(*c1) && is_v(*v1) && is_c(*c2) && is_c(*c3) && is_v(*v2) => {
            Some(RafsiShape::Final5)
        }
        [c1, c2, v1, c3, v2] if is_c(*c1) && is_c(*c2) && is_v(*v1) && is_c(*c3) && is_v(*v2) => {
            Some(RafsiShape::Final5)
        }
        _ => None,
    }
}

fn is_c(b: u8) -> bool {
    crate::cll::phonology::is_c(b)
}
fn is_v(b: u8) -> bool {
    crate::cll::phonology::is_v(b)
}

/// One rafsi piece in a segmentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RafsiPiece {
    pub form: String,
    pub shape: RafsiShape,
    pub source: String,
    pub hyphen_after: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LujvoSegmentation {
    pub pieces: Vec<RafsiPiece>,
}

/// Lookup tables over the embedded LLG/CLL-era lexicon.
#[derive(Debug, Default)]
pub struct RafsiLexicon {
    by_rafsi: HashMap<&'static str, Vec<RafsiEntry>>,
    spellings: HashSet<&'static str>,
    gismu: HashSet<&'static str>,
}

impl RafsiLexicon {
    pub fn load_embedded() -> Self {
        let mut by_rafsi: HashMap<&'static str, Vec<RafsiEntry>> = HashMap::new();
        let mut spellings = HashSet::new();
        let mut gismu = HashSet::new();
        for &(rafsi, source, kind_s) in RAFSI_ENTRIES {
            let kind = RafsiKind::parse(kind_s).expect("kind");
            if kind == RafsiKind::Final5 {
                gismu.insert(rafsi);
            }
            spellings.insert(rafsi);
            by_rafsi.entry(rafsi).or_default().push(RafsiEntry {
                rafsi,
                source,
                kind,
            });
        }
        Self {
            by_rafsi,
            spellings,
            gismu,
        }
    }

    pub fn global() -> &'static RafsiLexicon {
        static LEX: OnceLock<RafsiLexicon> = OnceLock::new();
        LEX.get_or_init(RafsiLexicon::load_embedded)
    }

    pub fn entry_count(&self) -> usize {
        RAFSI_ENTRY_COUNT
    }
    pub fn gismu_count(&self) -> usize {
        GISMU_COUNT
    }
    pub fn short_count(&self) -> usize {
        SHORT_RAFSI_COUNT
    }
    pub fn long4_count(&self) -> usize {
        LONG4_RAFSI_COUNT
    }
    pub fn final5_count(&self) -> usize {
        FINAL5_RAFSI_COUNT
    }

    pub fn is_rafsi(&self, s: &str) -> bool {
        self.spellings.contains(s)
    }

    pub fn is_gismu(&self, s: &str) -> bool {
        self.gismu.contains(s)
    }

    pub fn entries_for(&self, rafsi: &str) -> &[RafsiEntry] {
        self.by_rafsi
            .get(rafsi)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Lexicon-valid segmentations of a lujvo body (hyphens as present in the string).
    pub fn segment_lujvo(&self, body: &str) -> Vec<LujvoSegmentation> {
        let body = body.as_bytes();
        let mut out = Vec::new();
        let mut path = Vec::new();
        self.dfs_segment(body, 0, &mut path, &mut out);
        out
    }

    fn dfs_segment(
        &self,
        body: &[u8],
        i: usize,
        path: &mut Vec<RafsiPiece>,
        out: &mut Vec<LujvoSegmentation>,
    ) {
        if i == body.len() {
            if path.is_empty() {
                return;
            }
            let last = path.last().unwrap();
            if matches!(last.shape, RafsiShape::Cvc | RafsiShape::Long4) {
                return;
            }
            if last.hyphen_after.is_some() {
                return;
            }
            out.push(LujvoSegmentation {
                pieces: path.clone(),
            });
            return;
        }

        for len in 3..=5 {
            if i + len > body.len() {
                break;
            }
            let Ok(cand) = std::str::from_utf8(&body[i..i + len]) else {
                continue;
            };
            if !self.is_rafsi(cand) {
                continue;
            }
            let Some(shape) = classify_rafsi_shape(cand) else {
                continue;
            };
            let at_end = i + len == body.len();
            if at_end && matches!(shape, RafsiShape::Cvc | RafsiShape::Long4) {
                continue;
            }
            let source = self
                .entries_for(cand)
                .first()
                .map(|e| e.source.to_string())
                .unwrap_or_default();

            if at_end {
                path.push(RafsiPiece {
                    form: cand.to_string(),
                    shape,
                    source,
                    hyphen_after: None,
                });
                self.dfs_segment(body, i + len, path, out);
                path.pop();
                continue;
            }

            let rest_i = i + len;
            let hyph = match body.get(rest_i).copied() {
                Some(b'y') => Some('y'),
                Some(b'r') => Some('r'),
                Some(b'n') => Some('n'),
                _ => None,
            };
            if let Some(h) = hyph {
                path.push(RafsiPiece {
                    form: cand.to_string(),
                    shape,
                    source: source.clone(),
                    hyphen_after: Some(h),
                });
                self.dfs_segment(body, rest_i + 1, path, out);
                path.pop();
            }
            path.push(RafsiPiece {
                form: cand.to_string(),
                shape,
                source,
                hyphen_after: None,
            });
            self.dfs_segment(body, rest_i, path, out);
            path.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_counts_sane() {
        let lex = RafsiLexicon::global();
        assert!(lex.gismu_count() >= 1300);
        assert!(lex.short_count() >= 1400);
        assert_eq!(
            lex.entry_count(),
            lex.short_count() + lex.long4_count() + lex.final5_count()
        );
        assert!(lex.is_rafsi("ger"));
        assert!(lex.is_rafsi("ge'u"));
        assert!(lex.is_rafsi("zda"));
        assert!(lex.is_rafsi("soi"));
        assert!(lex.is_rafsi("sai"));
        assert!(lex.is_gismu("gerku"));
    }

    #[test]
    fn segment_soirsai() {
        let lex = RafsiLexicon::global();
        let segs = lex.segment_lujvo("soirsai");
        assert!(
            segs.iter().any(|s| {
                s.pieces.len() == 2
                    && s.pieces[0].form == "soi"
                    && s.pieces[0].hyphen_after == Some('r')
                    && s.pieces[1].form == "sai"
            }),
            "segs={segs:?}"
        );
    }

    #[test]
    fn segment_roinreo() {
        let lex = RafsiLexicon::global();
        let segs = lex.segment_lujvo("ro'inre'o");
        assert!(
            segs.iter().any(|s| {
                s.pieces.len() == 2
                    && s.pieces[0].form == "ro'i"
                    && s.pieces[0].hyphen_after == Some('n')
                    && s.pieces[1].form == "re'o"
            }),
            "segs={segs:?}"
        );
    }

    #[test]
    fn segment_patytaa() {
        let lex = RafsiLexicon::global();
        let segs = lex.segment_lujvo("patyta'a");
        assert!(
            segs.iter().any(|s| {
                s.pieces.len() == 2
                    && s.pieces[0].form == "pat"
                    && s.pieces[0].hyphen_after == Some('y')
                    && s.pieces[1].form == "ta'a"
            }),
            "segs={segs:?}"
        );
    }
}
