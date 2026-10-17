//! Whole-text Lojban **syntax** check with either engine.
//!
//! Both engines run the same camxes grammar
//! ([`crate::grammars::LOJBAN_GRAMMAR`]); the text is accepted when the
//! start rule `text` covers all of it. Parse trees are returned but their
//! shape is an engine detail (the Earley tree is the derivation it picked).
//!
//! * [`Engine::Peg`]: PEG semantics (ordered choice, greedy repetition,
//!   no backtracking into a committed choice).
//! * `Engine::Earley` (feature `earley`): first the PEG-preferred
//!   derivation; if that does not cover the text, any derivation of the
//!   grammar that does. This is a superset of the PEG result.
//!
//! Known limitation: the grammar's `zoi`/`la'o` quotes need delimiter
//! back-references (`#: PUSH`/`#: POP-EQ` in ilmentufa) that neither engine
//! implements. The PEG engine's greedy repetition then swallows the closing
//! delimiter and rejects `zoi gy. text .gy`; the Earley engine accepts it but
//! does not check that both delimiters are the same word.
//!
//! ```rust
//! use camxes_rs::morphology::Engine;
//! use camxes_rs::syntax;
//!
//! assert!(syntax::accepts("lo gerku cu batci lo mlatu", Engine::Peg));
//! assert!(!syntax::accepts("lo cu klama", Engine::Peg));
//! ```

use crate::grammars::LOJBAN_GRAMMAR;
use crate::morphology::Engine;
use crate::peg::grammar::Peg;
use crate::peg::parsing::ParseNode;
use std::fmt;
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

thread_local! {
    // `Peg` memoizes in a `RefCell`, so keep one parser per thread.
    static PEG: Peg = Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).expect("embedded Lojban grammar compiles");
}

/// The camxes syntax grammar compiled for the Earley engine.
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
