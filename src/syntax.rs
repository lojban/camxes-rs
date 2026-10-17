//! Whole-text Lojban **syntax** check with either engine.
//!
//! Both engines run the same camxes grammar
//! ([`crate::grammars::LOJBAN_GRAMMAR`]); the text is accepted when the
//! start rule `text` covers all of it. Parse trees are returned but their
//! shape is an engine detail (the Earley tree is the derivation it picked).
//!
//! * `Engine::Peg` (feature `peg`): PEG semantics (ordered choice, greedy repetition,
//!   no backtracking into a committed choice).
//! * `Engine::Earley` (feature `earley`): first the PEG-preferred
//!   derivation; if that does not cover the text, any derivation of the
//!   grammar that does. This is a superset of the PEG result.
//!
//! `zoi`/`la'o` quotes: the grammar marks the delimiter rules with
//! ilmentufa's `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ` annotations, and both
//! engines enforce them: `zoi X. … .X` is accepted only when the closing word
//! equals the opening one (case, pauses and commas ignored, `h` ≡ `'`), and
//! the quote ends at the first such word, so the quoted text never contains
//! the delimiter as a separate word; `zoi gy. hello .dy glico` is rejected.
//! The PEG engine uses a delimiter stack, the Earley engine a quote symbol
//! (see `peg` and `earley`); both produce `zoi_open` /
//! `zoi_word` / `zoi_close` nodes.
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

use crate::grammars::LOJBAN_GRAMMAR;
use crate::morphology::Engine;
#[cfg(feature = "peg")]
use crate::peg::grammar::Peg;
use crate::peg::parsing::ParseNode;
use std::fmt;
#[cfg(feature = "earley")]
use std::sync::OnceLock;

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

/// The camxes syntax grammar compiled for the Earley engine.
#[cfg(feature = "earley")]
pub fn earley_grammar() -> &'static crate::earley::EarleyGrammar {
    static G: OnceLock<crate::earley::EarleyGrammar> = OnceLock::new();
    G.get_or_init(|| {
        crate::earley::EarleyGrammar::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1)
            .expect("embedded Lojban grammar compiles")
    })
}

/// Parse a whole Lojban text. Letters are lowercased first (capitals only
/// mark stress). Returns the parse tree on success.
pub fn parse(input: &str, engine: Engine) -> Result<Vec<ParseNode>, SyntaxError> {
    let text = input.to_ascii_lowercase();
    let n = text.len();
    let fail = |engine: &'static str, position: usize, message: String| SyntaxError {
        engine,
        position,
        message,
    };
    match engine {
        #[cfg(feature = "peg")]
        Engine::Peg => {
            let r = PEG.with(|p| p.parse(&text));
            match r.2.as_ref() {
                Ok(nodes) if r.1 == n => Ok(nodes.clone()),
                Ok(_) => Err(fail(
                    "peg",
                    r.1,
                    "text does not cover the whole input".into(),
                )),
                Err(e) => Err(fail("peg", e.position, e.to_string())),
            }
        }
        #[cfg(feature = "earley")]
        Engine::Earley => {
            let g = earley_grammar();
            let r = g.parse(&text);
            if let Ok(nodes) = r.2.as_ref() {
                if r.1 == n {
                    return Ok(nodes.clone());
                }
            }
            let c = g.parse_complete(&text);
            match c.2.as_ref() {
                Ok(nodes) => Ok(nodes.clone()),
                Err(e) => Err(fail("earley", r.1.max(e.position), e.to_string())),
            }
        }
    }
}

/// True when `engine` accepts the whole text.
pub fn accepts(input: &str, engine: Engine) -> bool {
    parse(input, engine).is_ok()
}
