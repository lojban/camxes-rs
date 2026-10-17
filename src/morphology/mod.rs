//! Engine-independent Lojban **morphology** API.
//!
//! Both engines answer the same question for a piece of Lojban text: how does
//! it split into words, and what is each word's class (cmavo, gismu, lujvo,
//! fu'ivla, cmevla)? Internal trees differ between engines and are not part of
//! this result.
//!
//! * [`Engine::Peg`] (default, always available) runs the camxes-style PEG
//!   morphology grammar [`crate::grammars::LOJBAN_MORPHOLOGY_GRAMMAR`].
//! * `Engine::Earley` (Cargo feature `earley`) runs the same grammar on the
//!   [`crate::earley`] engine and falls back to the CLL-grounded CFG/Earley
//!   morphology of [tergenzalvi](https://github.com/lagleki/tergenzalvi)
//!   (vendored as `camxes_rs::cll`). Besides everything the PEG grammar accepts
//!   it also accepts the older CLL dialect that the PEG rejects (for example
//!   obsolete fu'ivla such as `kuarka`). Each word is flagged
//!   [`WordDialect::Standard`] or [`WordDialect::Obsolete`] by rule.
//!
//! ```rust
//! use camxes_rs::morphology::{analyze, Engine, WordClass};
//!
//! let m = analyze("lenu klama", Engine::Peg).unwrap();
//! let shape: Vec<(&str, WordClass)> = m.words.iter().map(|w| (w.text.as_str(), w.class)).collect();
//! assert_eq!(shape, vec![("le", WordClass::Cmavo), ("nu", WordClass::Cmavo), ("klama", WordClass::Gismu)]);
//! ```

mod peg;

#[cfg(feature = "earley")]
pub use earley::camxes_morphology_grammar;

#[cfg(feature = "earley")]
mod earley;

use serde::Serialize;
use std::fmt;

/// Lojban word class (CLL §4.1 classes with brivla subtypes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WordClass {
    Cmavo,
    Gismu,
    Lujvo,
    Fuhivla,
    Cmevla,
}

impl WordClass {
    pub fn as_str(self) -> &'static str {
        match self {
            WordClass::Cmavo => "cmavo",
            WordClass::Gismu => "gismu",
            WordClass::Lujvo => "lujvo",
            WordClass::Fuhivla => "fu'ivla",
            WordClass::Cmevla => "cmevla",
        }
    }

    /// True for gismu, lujvo, and fu'ivla.
    pub fn is_brivla(self) -> bool {
        matches!(
            self,
            WordClass::Gismu | WordClass::Lujvo | WordClass::Fuhivla
        )
    }
}

impl fmt::Display for WordClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One word of a morphological analysis. Compound cmavo are always split into
/// their simple cmavo, so a cluster such as `lenu` yields two words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MorphWord {
    /// Surface text as written in the input (no surrounding pauses/spaces).
    pub text: String,
    /// Byte span `(start, end)` in the input.
    pub span: (usize, usize),
    pub class: WordClass,
    /// Chapter 21 selma'o of a cmavo when known (needs feature `earley`,
    /// which ships the cmavo lexicon). Always `None` for non-cmavo.
    pub selmaho: Option<String>,
    /// Which word-form rules this word satisfies; see [`WordDialect`].
    pub word_dialect: WordDialect,
}

impl MorphWord {
    /// True when the word's form is valid only under the obsolete (CLL)
    /// rules, i.e. the standard (camxes) word rules reject it.
    pub fn is_obsolete(&self) -> bool {
        self.word_dialect == WordDialect::Obsolete
    }

    /// Word type label in the style of the lensisku dictionary, e.g.
    /// `fu'ivla` or `obsolete fu'ivla`.
    pub fn type_label(&self) -> String {
        match self.word_dialect {
            WordDialect::Standard => self.class.as_str().to_string(),
            WordDialect::Obsolete => format!("obsolete {}", self.class),
        }
    }
}

/// Per-word dialect, decided by **rules** (no word lists):
///
/// * `Standard` — the word, on its own, is a word of the given class under
///   the standard camxes morphology rules
///   ([`crate::grammars::LOJBAN_MORPHOLOGY_GRAMMAR`]). Every word the PEG
///   engine produces is standard.
/// * `Obsolete` — the camxes rules reject the word (or give it another
///   class), but the CLL Chapter 3–4 rules of tergenzalvi accept it with this
///   class. Typical obsolete forms: on-glide diphthongs `iV`/`uV` inside
///   brivla and names (`kuarka`, `guanJOUS`), diphthong + apostrophe + vowel
///   (`gondii'i`), `CiV'V'V` cmavo (`cia'o'e`), and fu'ivla shapes that camxes
///   reads differently (`mastla`). Only produced by `Engine::Earley`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WordDialect {
    #[default]
    Standard,
    Obsolete,
}

/// Which morphology rules produced an analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dialect {
    /// camxes morphology grammar, PEG-preferred reading (both engines).
    Camxes,
    /// CLL (Chapters 3–4) morphology from tergenzalvi: the older dialect,
    /// e.g. obsolete fu'ivla such as `kuarka` (Earley engine only).
    Cll,
    /// camxes grammar rules, but a derivation the PEG cannot reach because it
    /// commits to an earlier alternative (Earley engine only, last resort).
    CamxesComplete,
}

/// Normalized morphology: the word sequence of the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Morphology {
    pub words: Vec<MorphWord>,
    pub dialect: Dialect,
}

impl Morphology {
    /// Engine-comparable key: `(text, class)` for each word.
    pub fn signature(&self) -> Vec<(String, WordClass)> {
        self.words
            .iter()
            .map(|w| (w.text.to_ascii_lowercase(), w.class))
            .collect()
    }

    /// True when any word is obsolete (see [`WordDialect`]).
    pub fn has_obsolete(&self) -> bool {
        self.words.iter().any(MorphWord::is_obsolete)
    }

    /// Compact rendering, e.g. `le/cmavo nu/cmavo klama/gismu`.
    pub fn render(&self) -> String {
        self.words
            .iter()
            .map(|w| format!("{}/{}", w.text, w.class))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Which morphology engine to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Engine {
    /// camxes-style PEG grammar (default).
    #[default]
    Peg,
    /// tergenzalvi CLL CFG/Earley morphology (feature `earley`).
    #[cfg(feature = "earley")]
    Earley,
}

impl Engine {
    pub fn name(self) -> &'static str {
        match self {
            Engine::Peg => "peg",
            #[cfg(feature = "earley")]
            Engine::Earley => "earley",
        }
    }

    /// All engines compiled into this build.
    pub fn available() -> &'static [Engine] {
        #[cfg(feature = "earley")]
        {
            &[Engine::Peg, Engine::Earley]
        }
        #[cfg(not(feature = "earley"))]
        {
            &[Engine::Peg]
        }
    }
}

impl std::str::FromStr for Engine {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "peg" | "camxes" => Ok(Engine::Peg),
            #[cfg(feature = "earley")]
            "earley" | "bnf" | "cfg" | "tergenzalvi" => Ok(Engine::Earley),
            other => Err(format!(
                "unknown or disabled morphology engine `{other}` (available: {})",
                Engine::available()
                    .iter()
                    .map(|e| e.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}

/// Why an input has no morphological analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MorphologyError {
    pub engine: &'static str,
    /// Byte offset where analysis stopped (best effort).
    pub position: usize,
    pub message: String,
}

impl fmt::Display for MorphologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} morphology rejected input at byte {}: {}",
            self.engine, self.position, self.message
        )
    }
}

impl std::error::Error for MorphologyError {}

/// Split `input` into words and classify them with the chosen engine.
pub fn analyze(input: &str, engine: Engine) -> Result<Morphology, MorphologyError> {
    let mut m = match engine {
        Engine::Peg => peg::analyze(input)?,
        #[cfg(feature = "earley")]
        Engine::Earley => earley::analyze(input)?,
    };
    annotate_selmaho(&mut m);
    Ok(m)
}

#[cfg(feature = "earley")]
fn annotate_selmaho(m: &mut Morphology) {
    let lexicon = crate::cll::grammar::CmavoLexicon::global();
    for w in &mut m.words {
        if w.class == WordClass::Cmavo {
            w.selmaho = lexicon.selmaho(&w.text).map(str::to_string);
        }
    }
}

#[cfg(not(feature = "earley"))]
fn annotate_selmaho(_m: &mut Morphology) {}

/// Split a compound-cmavo surface into simple cmavo spans (relative byte
/// offsets). A new simple cmavo starts at every consonant; `y`, vowels,
/// apostrophes and commas continue the current one (CLL §4.2: cmavo are
/// `(C)V`, `(C)VV`, `(C)V'V`, `Cy`, ...).
pub(crate) fn split_cmavo_cluster(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        let c = b.to_ascii_lowercase();
        let consonant =
            c.is_ascii_alphabetic() && !matches!(c, b'a' | b'e' | b'i' | b'o' | b'u' | b'y');
        if consonant && i > start {
            out.push((start, i));
            start = i;
        }
    }
    if start < bytes.len() {
        out.push((start, bytes.len()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(input: &str, engine: Engine) -> Vec<(String, WordClass)> {
        analyze(input, engine)
            .unwrap_or_else(|e| panic!("{input}: {e}"))
            .signature()
    }

    fn s(words: &[(&str, WordClass)]) -> Vec<(String, WordClass)> {
        words.iter().map(|(t, c)| (t.to_string(), *c)).collect()
    }

    #[test]
    fn split_cluster() {
        assert_eq!(split_cmavo_cluster("lenu"), vec![(0, 2), (2, 4)]);
        assert_eq!(split_cmavo_cluster("abu"), vec![(0, 1), (1, 3)]);
        assert_eq!(split_cmavo_cluster("a'o"), vec![(0, 3)]);
        assert_eq!(split_cmavo_cluster("bybu"), vec![(0, 2), (2, 4)]);
    }

    #[test]
    fn peg_basic_classes() {
        use WordClass::*;
        assert_eq!(sig("klama", Engine::Peg), s(&[("klama", Gismu)]));
        assert_eq!(sig("klamydja", Engine::Peg), s(&[("klamydja", Lujvo)]));
        assert_eq!(sig(".djan.", Engine::Peg), s(&[("djan", Cmevla)]));
        assert_eq!(
            sig("a'o di'a kanro", Engine::Peg),
            s(&[("a'o", Cmavo), ("di'a", Cmavo), ("kanro", Gismu)])
        );
    }

    #[test]
    fn peg_rejects_partial() {
        assert!(analyze("klamq", Engine::Peg).is_err());
        assert!(analyze("", Engine::Peg).is_err());
    }

    #[test]
    fn engine_from_str() {
        assert_eq!("peg".parse::<Engine>().unwrap(), Engine::Peg);
        assert!("nope".parse::<Engine>().is_err());
    }

    #[cfg(feature = "earley")]
    #[test]
    fn earley_matches_peg_on_common_words() {
        for input in [
            "klama",
            "klamydja",
            ".djan.",
            "a'o di'a kanro",
            "lenu klama",
            "abu zei sance",
            "coi",
            "zoskaigau",
        ] {
            assert_eq!(
                sig(input, Engine::Earley),
                sig(input, Engine::Peg),
                "{input}"
            );
        }
    }

    #[cfg(feature = "earley")]
    #[test]
    fn earley_accepts_cll_dialect() {
        // `matriocka` is accepted by both dialects; the CLL path alone must
        // still classify it.
        let m = earley::analyze_cll("matriocka").unwrap();
        assert_eq!(m.signature(), s(&[("matriocka", WordClass::Fuhivla)]));
        assert_eq!(m.dialect, Dialect::Cll);
    }

    #[cfg(feature = "earley")]
    #[test]
    fn earley_cmavo_selmaho() {
        let m = analyze("lenu", Engine::Earley).unwrap();
        let sm: Vec<_> = m.words.iter().map(|w| w.selmaho.as_deref()).collect();
        assert_eq!(sm, vec![Some("LE"), Some("NU")]);
    }
}
