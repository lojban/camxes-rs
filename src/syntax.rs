//! Whole-text Lojban **syntax** check with either engine.
//!
//! * `Engine::Peg` (feature `peg`) runs the camxes grammar
//!   (`grammars::LOJBAN_GRAMMAR`) with PEG semantics (ordered
//!   choice, greedy repetition, no backtracking into a committed choice);
//!   the text is accepted when the start rule `text` covers all of it.
//! * `Engine::Earley` (feature `earley`) runs tergenzalvi's CLL pipeline
//!   (vendored as `camxes_rs::cll`): CLL morphology and stream segmentation,
//!   selma'o from the cmavo lexicon (`src/cll/data/cmavo_selmaho.tsv`),
//!   Chapter 19 SI/SA/SU erasure, then an Earley parse of the CLL Chapter 21
//!   EBNF (`src/cll/data/cll21.ebnf`) with elidable terminators
//!   (`camxes_rs::cll::grammar::parse_text`).
//!
//! The two engines use different grammars, so their parse trees differ and
//! their acceptance can differ on edge cases. The Earley tree has one node
//! per Chapter 21 rule and one node per selma'o / `BRIVLA` / `CMENE` /
//! `FOREIGN` terminal, with byte spans into the input.
//!
//! `zoi`/`la'o` quotes need matching delimiters in both engines: `zoi X. …
//! .X` is accepted only when the closing word equals the opening one (case,
//! pauses and commas ignored, `h` ≡ `'`), and the quote ends at the first
//! such word; `zoi gy. hello .dy glico` is rejected. The PEG grammar marks
//! the delimiter rules with ilmentufa's `#: PUSH` / `#: PEEK-DIFF` /
//! `#: POP-EQ` annotations (`zoi_open` / `zoi_word` / `zoi_close` nodes);
//! the CLL pipeline cuts the quote out before morphology (one `FOREIGN`
//! terminal for the quoted text).
//!
//! ```rust
//! use camxes_rs::morphology::Engine;
//! use camxes_rs::syntax;
//!
//! assert!(syntax::accepts("lo gerku cu batci lo mlatu", Engine::default()));
//! assert!(!syntax::accepts("lo cu klama", Engine::default()));
//!
//! for &engine in Engine::available() {
//!     assert!(syntax::accepts("zoi gy. hello world .gy glico", engine));
//!     assert!(syntax::accepts("mi klama la'o dy. New York .dy", engine));
//!     assert!(!syntax::accepts("zoi gy. hello .dy glico", engine));
//! }
//! ```

#[cfg(feature = "peg")]
use crate::grammars::LOJBAN_GRAMMAR;
use crate::morphology::Engine;
#[cfg(feature = "peg")]
use crate::peg::grammar::Peg;
use crate::peg::parsing::ParseNode;
use std::fmt;

/// Why a text was rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    pub engine: &'static str,
    /// Byte offset where the parse stopped (end of the longest match).
    pub position: usize,
    pub message: String,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} syntax rejected input at byte {}: {}",
            self.engine, self.position, self.message
        )
    }
}

impl std::error::Error for SyntaxError {}

#[cfg(feature = "peg")]
thread_local! {
    // `Peg` memoizes in a `RefCell`, so keep one parser per thread.
    static PEG: Peg = Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).expect("embedded Lojban grammar compiles");
}

/// Parse a whole Lojban text. Returns the parse tree on success. The PEG
/// engine lowercases the text first (capitals only mark stress); the CLL
/// pipeline reads capitals as stress marks.
pub fn parse(input: &str, engine: Engine) -> Result<Vec<ParseNode>, SyntaxError> {
    match engine {
        #[cfg(feature = "peg")]
        Engine::Peg => peg_parse(input),
        #[cfg(feature = "earley")]
        Engine::Earley => cll_parse(input),
    }
}

#[cfg(feature = "peg")]
fn peg_parse(input: &str) -> Result<Vec<ParseNode>, SyntaxError> {
    let text = input.to_ascii_lowercase();
    let fail = |position: usize, message: String| SyntaxError {
        engine: "peg",
        position,
        message,
    };
    let r = PEG.with(|p| p.parse(&text));
    match r.2.as_ref() {
        Ok(nodes) if r.1 == text.len() => Ok(nodes.clone()),
        Ok(_) => Err(fail(r.1, "text does not cover the whole input".into())),
        Err(e) => Err(fail(e.position, e.to_string())),
    }
}

/// Earley engine: tergenzalvi's CLL Chapter 21 pipeline.
#[cfg(feature = "earley")]
fn cll_parse(input: &str) -> Result<Vec<ParseNode>, SyntaxError> {
    use crate::cll::grammar::{cll21_grammar, parse_text};
    let r = parse_text(input);
    let spans = lexeme_spans(input, &r.lexemes);
    // The CLL orthography normalizer silently drops characters that are not
    // Lojban letters (`valsi---` would read as `valsi`); outside `zoi` /
    // `la'o` bodies such characters are an error, as in the PEG engine.
    let foreign: Vec<(usize, usize)> = r
        .lexemes
        .iter()
        .zip(&spans)
        .filter(|(l, _)| l.terminal == crate::cll::grammar::FOREIGN_TERMINAL)
        .map(|(_, s)| *s)
        .collect();
    if let Some((pos, ch)) = input.char_indices().find(|&(i, c)| {
        !(c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || matches!(c, '\'' | ',' | '.'))
            && !foreign.iter().any(|&(s, e)| s <= i && i < e)
    }) {
        return Err(SyntaxError {
            engine: "earley",
            position: pos,
            message: format!("`{ch}` is not Lojban orthography"),
        });
    }
    // The CLL lexer drops words it cannot analyse (recording a gap) and
    // parses the rest; a whole-text check must reject such texts.
    if !r.accepted || !r.gaps.is_empty() {
        let bad = r
            .lexemes
            .iter()
            .position(|l| l.terminal == "GAP" || l.terminal == "UNKNOWN_CMAVO");
        let gap_word = r
            .gaps
            .first()
            .and_then(|g| g.split('"').nth(1))
            .filter(|w| !w.is_empty())
            .and_then(|w| {
                input
                    .to_ascii_lowercase()
                    .find(w.to_ascii_lowercase().as_str())
            });
        let position = gap_word.or(bad.map(|i| spans[i].0)).unwrap_or(0);
        let message = if r.gaps.is_empty() {
            "no CLL Chapter 21 derivation of `text`".to_string()
        } else {
            r.gaps.join("; ")
        };
        return Err(SyntaxError {
            engine: "earley",
            position,
            message,
        });
    }
    let tree = r.tree().ok_or_else(|| SyntaxError {
        engine: "earley",
        position: 0,
        message: "no CLL Chapter 21 derivation of `text`".into(),
    })?;
    let structure = tree.structure(cll21_grammar());
    Ok(structure
        .roots
        .iter()
        .map(|n| convert(n, &spans, input.len()))
        .collect())
}

/// Byte span of each lexeme in `input`, found left to right (lexemes are
/// lowercased surface words; erased words and pauses are skipped).
#[cfg(feature = "earley")]
fn lexeme_spans(input: &str, lexemes: &[crate::cll::grammar::Lexeme]) -> Vec<(usize, usize)> {
    let lower = input.to_ascii_lowercase();
    let mut cursor = 0;
    lexemes
        .iter()
        .map(|l| {
            let needle = l.surface.to_ascii_lowercase();
            match lower
                .get(cursor..)
                .and_then(|rest| rest.find(needle.as_str()))
            {
                Some(rel) if !needle.is_empty() => {
                    let start = cursor + rel;
                    cursor = start + needle.len();
                    (start, cursor)
                }
                _ => (cursor, cursor),
            }
        })
        .collect()
}

#[cfg(feature = "earley")]
fn convert(
    node: &crate::cll::grammar::earley::StructuralNode,
    spans: &[(usize, usize)],
    len: usize,
) -> ParseNode {
    use crate::peg::parsing::Span;
    let start = spans.get(node.start).map_or(len, |s| s.0);
    let end = if node.end > node.start {
        spans.get(node.end - 1).map_or(start, |s| s.1)
    } else {
        start
    };
    let children = if node.terminal {
        vec![ParseNode::Terminal {
            span: Span(start, end),
        }]
    } else {
        node.children
            .iter()
            .map(|c| convert(c, spans, len))
            .collect()
    };
    ParseNode::NonTerminal {
        name: node.symbol.clone(),
        span: Span(start, end),
        children,
    }
}

/// True when `engine` accepts the whole text.
pub fn accepts(input: &str, engine: Engine) -> bool {
    parse(input, engine).is_ok()
}
