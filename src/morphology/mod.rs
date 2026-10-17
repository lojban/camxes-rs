//! Engine-independent Lojban **morphology** API.
//!
//! Both engines answer the same question for a piece of Lojban text: how does
//! it split into words, and what is each word's class (cmavo, gismu, lujvo,
//! fu'ivla, cmevla)? Internal trees differ between engines and are not part of
//! this result.
//!
//! * `Engine::Peg` (Cargo feature `peg`, on by default) runs the
//!   camxes-style PEG morphology grammar
//!   `grammars::LOJBAN_MORPHOLOGY_GRAMMAR`.
//! * `Engine::Earley` (Cargo feature `earley`) runs the CLL Chapter 3–4
//!   morphology of [tergenzalvi](https://github.com/lagleki/tergenzalvi)
//!   (vendored as `camxes_rs::cll`): CFG/Earley word shapes, constraint
//!   filters and stream segmentation; a mapping layer (`morphology::camxes`,
//!   private) chooses the camxes word type. Besides the
//!   words the PEG grammar accepts it also accepts the older CLL dialect
//!   that the PEG rejects (for example obsolete fu'ivla such as `kuarka`).
//!   Each word is flagged [`WordDialect::Standard`] or
//!   [`WordDialect::Obsolete`] by rule.
//!
//! ```rust
//! use camxes_rs::morphology::{analyze, Engine, WordClass};
//!
//! // `Engine::default()` is `Engine::Peg` when the `peg` feature is on.
//! let m = analyze("lenu klama", Engine::default()).unwrap();
//! let shape: Vec<(&str, WordClass)> = m.words.iter().map(|w| (w.text.as_str(), w.class)).collect();
//! assert_eq!(shape, vec![("le", WordClass::Cmavo), ("nu", WordClass::Cmavo), ("klama", WordClass::Gismu)]);
//! ```

#[cfg(feature = "peg")]
mod peg;
#[cfg(feature = "peg")]
mod tree;

#[cfg(feature = "earley")]
mod camxes;
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
///   (`grammars::LOJBAN_MORPHOLOGY_GRAMMAR`). Every word the PEG
///   engine produces is standard.
/// * `Obsolete` — the camxes rules reject the word, but the CLL Chapter 3–4
///   rules of tergenzalvi accept it with this class. Decided by the camxes
///   word-form rules (`is_obsolete_form` in `src/morphology/camxes.rs`): a word
///   other than a name must split into camxes syllables (no on-glide right
///   after a consonant that cannot close a syllable, as in `kuarka`,
///   `fiesta`, `cia'o'e`, `gondii'i`; no consonant cluster that is not a
///   coda plus onset, as in `mastla`; no `n` before an affricate); a name
///   must be a camxes name form (no vowel hiatus such as `ou` in
///   `guanJOUS`). Only produced by `Engine::Earley`.
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
    /// camxes morphology grammar (PEG engine).
    Camxes,
    /// CLL (Chapters 3–4) morphology from tergenzalvi with the camxes
    /// compatibility rules (Earley engine).
    Cll,
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
    /// camxes-style PEG grammar (feature `peg`; the default engine).
    #[cfg(feature = "peg")]
    #[default]
    Peg,
    /// tergenzalvi CLL CFG/Earley morphology and Chapter 21 grammar (feature
    /// `earley`; the default engine when `peg` is disabled).
    #[cfg(feature = "earley")]
    #[cfg_attr(not(feature = "peg"), default)]
    Earley,
}

impl Engine {
    pub fn name(self) -> &'static str {
        match self {
            #[cfg(feature = "peg")]
            Engine::Peg => "peg",
            #[cfg(feature = "earley")]
            Engine::Earley => "earley",
        }
    }

    /// All engines compiled into this build.
    pub fn available() -> &'static [Engine] {
        #[cfg(all(feature = "peg", feature = "earley"))]
        {
            &[Engine::Peg, Engine::Earley]
        }
        #[cfg(all(feature = "peg", not(feature = "earley")))]
        {
            &[Engine::Peg]
        }
        #[cfg(all(not(feature = "peg"), feature = "earley"))]
        {
            &[Engine::Earley]
        }
    }
}

impl std::str::FromStr for Engine {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            #[cfg(feature = "peg")]
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

/// Options (dialect switches) of the CLL pipeline; see
/// [`earley_options`] and [`analyze_with_options`].
#[cfg(feature = "earley")]
pub use crate::cll::constraints::MorphOptions;

/// The options `Engine::Earley` uses: [`MorphOptions::camxes()`], i.e. CLL
/// Chapters 3–4 and the §4.16 rafsi fu'ivla proposal, with the single
/// camxes dialect switch on ([`MorphOptions::camxes_dialect`]: lujvo from
/// rafsi outside the CLL-era list, and `la`/`lai`/`doi` inside names). Use
/// [`analyze_with_options`] with [`MorphOptions::default()`] (or
/// [`MorphOptions::settled()`], which also drops §4.16) for strict CLL.
#[cfg(feature = "earley")]
pub fn earley_options() -> MorphOptions {
    MorphOptions::camxes()
}

/// Split `input` into words and classify them with the chosen engine.
/// `Engine::Earley` runs the CLL pipeline with `earley_options()` (feature `earley`).
pub fn analyze(input: &str, engine: Engine) -> Result<Morphology, MorphologyError> {
    match engine {
        #[cfg(feature = "peg")]
        Engine::Peg => {
            let mut m = peg::analyze(input)?;
            annotate_selmaho(&mut m);
            Ok(m)
        }
        #[cfg(feature = "earley")]
        Engine::Earley => analyze_with_options(input, earley_options()),
    }
}

/// `Engine::Earley` analysis with explicit CLL pipeline options (dialect).
/// The options decide which words the CLL pipeline accepts; each accepted
/// word's [`WordDialect`] is still decided by the camxes rules alone, so a
/// word accepted only through the camxes dialect (such as `volapuk`) is `Standard` when camxes accepts it.
#[cfg(feature = "earley")]
pub fn analyze_with_options(
    input: &str,
    options: MorphOptions,
) -> Result<Morphology, MorphologyError> {
    let mut m = crate::cll::constraints::with_options(options, || earley::analyze(input))?;
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

    #[cfg(feature = "peg")]
    fn sig(input: &str, engine: Engine) -> Vec<(String, WordClass)> {
        analyze(input, engine)
            .unwrap_or_else(|e| panic!("{input}: {e}"))
            .signature()
    }

    fn s(words: &[(&str, WordClass)]) -> Vec<(String, WordClass)> {
        words.iter().map(|(t, c)| (t.to_string(), *c)).collect()
    }

    #[cfg(feature = "earley")]
    #[test]
    fn earley_dialect_options() {
        use WordClass::*;
        // camxes dialect (Engine::Earley): `la` inside a name …
        let m = analyze("volapuk", Engine::Earley).unwrap();
        assert_eq!(m.signature(), s(&[("volapuk", Cmevla)]));
        assert!(!m.has_obsolete());
        // … and a lujvo from rafsi outside the CLL-era list (`gam`).
        let m = analyze("gamkemdau", Engine::Earley).unwrap();
        assert_eq!(m.signature(), s(&[("gamkemdau", Lujvo)]));
        // Strict CLL (one switch turns off both): §4.8 bans `la` in names,
        // and `gam` is not a CLL rafsi.
        let strict = analyze_with_options("volapuk", MorphOptions::default());
        assert!(strict.map_or(true, |m| m.signature() != s(&[("volapuk", Cmevla)])));
        let strict = analyze_with_options("gamkemdau", MorphOptions::default());
        assert!(strict.map_or(true, |m| m.signature() != s(&[("gamkemdau", Lujvo)])));
        // Consonant-only names with permissible pairs, in every dialect.
        for opts in [
            MorphOptions::settled(),
            MorphOptions::default(),
            MorphOptions::camxes(),
        ] {
            let m = analyze_with_options("la .c. cu klama", opts).unwrap();
            assert_eq!(m.words[1].class, Cmevla, "{opts:?}");
        }
    }

    #[test]
    fn split_cluster() {
        assert_eq!(split_cmavo_cluster("lenu"), vec![(0, 2), (2, 4)]);
        assert_eq!(split_cmavo_cluster("abu"), vec![(0, 1), (1, 3)]);
        assert_eq!(split_cmavo_cluster("a'o"), vec![(0, 3)]);
        assert_eq!(split_cmavo_cluster("bybu"), vec![(0, 2), (2, 4)]);
    }

    #[cfg(feature = "peg")]
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

    #[cfg(feature = "peg")]
    #[test]
    fn peg_rejects_partial() {
        assert!(analyze("klamq", Engine::Peg).is_err());
        assert!(analyze("", Engine::Peg).is_err());
    }

    #[cfg(feature = "peg")]
    #[test]
    fn engine_from_str() {
        assert_eq!("peg".parse::<Engine>().unwrap(), Engine::Peg);
        assert!("nope".parse::<Engine>().is_err());
    }

    #[cfg(all(feature = "peg", feature = "earley"))]
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
        let m = analyze("matriocka", Engine::Earley).unwrap();
        assert_eq!(m.signature(), s(&[("matriocka", WordClass::Fuhivla)]));
        assert_eq!(m.dialect, Dialect::Cll);
        let m = analyze("kuarka", Engine::Earley).unwrap();
        assert_eq!(m.signature(), s(&[("kuarka", WordClass::Fuhivla)]));
        assert!(m.words[0].is_obsolete());
    }

    #[cfg(feature = "earley")]
    #[test]
    fn earley_cmavo_selmaho() {
        let m = analyze("lenu", Engine::Earley).unwrap();
        let sm: Vec<_> = m.words.iter().map(|w| w.selmaho.as_deref()).collect();
        assert_eq!(sm, vec![Some("LE"), Some("NU")]);
    }
}
