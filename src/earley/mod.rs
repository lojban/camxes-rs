//! Earley engine for camxes-rs grammars — an alternative to the PEG engine.
//!
//! [`EarleyGrammar::new`] accepts exactly the grammar text that
//! [`Peg::new`](crate::peg::grammar::Peg::new) accepts and compiles it to a
//! context-free grammar extended with zero-width lookahead predicates:
//!
//! * `a / b` becomes an (unordered) alternation; `X*`, `X+`, `X?` become
//!   right-recursive helper nonterminals; literals, classes, ranges and `.`
//!   become byte terminals.
//! * `&X` / `!X` become lookahead symbols. They are decided by a separate
//!   Earley recognition of `X` from the current position ("does `X` derive
//!   some prefix of the remaining input?"), memoized per `(X, position)`.
//!
//! Recognition is a standard Earley chart with ε-completion handling (the
//! Aycock–Horspool refinement done dynamically). From the chart, a single
//! derivation is selected with PEG-style preferences: earlier alternatives
//! first, then longer repetitions. For grammars written as PEGs this usually
//! reproduces the PEG's own derivation, while the chart still knows every
//! other derivation (see [`EarleyGrammar::parse_complete`]).
//!
//! The output uses the same [`ParseResult`] / [`ParseNode`] types as the PEG
//! engine. Nodes are emitted for named rules and for terminals; helper
//! nonterminals introduced by the compiler are flattened away.

mod compile;
mod recognize;

pub use compile::CompileError;

use crate::peg::parsing::{ErrorKind, ParseError, ParseNode, ParseResult};
use std::sync::Arc;

/// 256-bit byte set used for terminals.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ByteSet([u64; 4]);

impl ByteSet {
    pub(crate) const EMPTY: ByteSet = ByteSet([0; 4]);
    pub(crate) const ALL: ByteSet = ByteSet([u64::MAX; 4]);

    pub(crate) fn union(&mut self, other: &ByteSet) -> bool {
        let mut changed = false;
        for (a, b) in self.0.iter_mut().zip(other.0.iter()) {
            let next = *a | *b;
            changed |= next != *a;
            *a = next;
        }
        changed
    }

    pub(crate) fn insert(&mut self, b: u8) {
        self.0[(b >> 6) as usize] |= 1u64 << (b & 63);
    }

    #[inline]
    pub(crate) fn contains(&self, b: u8) -> bool {
        self.0[(b >> 6) as usize] & (1u64 << (b & 63)) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Sym {
    /// One input byte from the set.
    T(ByteSet),
    /// Nonterminal.
    N(u32),
    /// Positive lookahead `&X` (zero width).
    And(u32),
    /// Negative lookahead `!X` (zero width).
    Not(u32),
}

/// How a nonterminal shows up in the output tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NtKind {
    /// A rule named in the grammar text: emitted as `ParseNode::NonTerminal`.
    Named,
    /// Compiler helper: children are spliced into the parent.
    Helper,
    /// Multi-byte literal or class member: emitted as one `ParseNode::Terminal`.
    Literal,
}

#[derive(Clone, Debug)]
pub(crate) struct Prod {
    pub(crate) lhs: u32,
    pub(crate) rhs: Vec<Sym>,
}

/// A camxes-rs grammar compiled for the Earley engine. Unlike
/// [`Peg`](crate::peg::grammar::Peg) it holds no per-parse state, so it is
/// `Send + Sync` and can be shared between threads.
#[derive(Clone, Debug)]
pub struct EarleyGrammar {
    pub(crate) start: u32,
    pub(crate) names: Vec<String>,
    pub(crate) kinds: Vec<NtKind>,
    pub(crate) prods: Vec<Prod>,
    pub(crate) by_lhs: Vec<Vec<u32>>,
    /// Bytes that can start a derivation of each nonterminal.
    pub(crate) nt_first: Vec<ByteSet>,
    /// Nonterminal may match without consuming input (lookaheads count as
    /// zero-width, so this is conservative).
    pub(crate) nt_nullable: Vec<bool>,
    /// Same two facts per production, used to filter predictions.
    pub(crate) prod_first: Vec<ByteSet>,
    pub(crate) prod_nullable: Vec<bool>,
}

impl EarleyGrammar {
    /// Compile `grammar` (camxes-rs PEG syntax) with start rule `start`.
    pub fn new(start: &str, grammar: &str) -> Result<Self, CompileError> {
        compile::compile(start, grammar)
    }

    /// Number of nonterminals (named rules plus helpers).
    pub fn nonterminal_count(&self) -> usize {
        self.names.len()
    }

    /// Number of CFG productions after compilation.
    pub fn production_count(&self) -> usize {
        self.prods.len()
    }

    /// Parse like [`Peg::parse`](crate::peg::grammar::Peg::parse): the start
    /// rule is matched against a prefix of `input`, preferring the
    /// PEG-preferred derivation. `ParseResult.1` is the end of that match.
    pub fn parse(&self, input: &str) -> ParseResult {
        let mut chart = recognize::Parse::new(self, input.as_bytes());
        chart.run_main();
        match chart.preferred_end(self.start, 0) {
            Some(end) => {
                let nodes = chart.derive(self.start, 0, end);
                ParseResult(1, end, Arc::new(Ok(nodes)))
            }
            None => self.failure(chart.furthest()),
        }
    }

    /// Parse requiring the start rule to span the whole `input`. Returns the
    /// most PEG-preferred derivation among those covering all input, so it
    /// also succeeds where a PEG would have committed to a shorter match.
    pub fn parse_complete(&self, input: &str) -> ParseResult {
        let mut chart = recognize::Parse::new(self, input.as_bytes());
        chart.run_main();
        let n = input.len();
        if chart.has_end(self.start, 0, n) {
            let nodes = chart.derive(self.start, 0, n);
            ParseResult(1, n, Arc::new(Ok(nodes)))
        } else {
            self.failure(chart.furthest())
        }
    }

    /// Like [`parse`](Self::parse) but returns `(preferred, complete)`: the
    /// PEG-preferred match end (if any) and whether some derivation spans all
    /// of `input`, computed from a single chart.
    pub fn analyze_ends(&self, input: &str) -> (Option<usize>, bool) {
        let mut chart = recognize::Parse::new(self, input.as_bytes());
        chart.run_main();
        (
            chart.preferred_end(self.start, 0),
            chart.has_end(self.start, 0, input.len()),
        )
    }

    /// Serialize [`parse`](Self::parse) output as pretty JSON (same shape as
    /// [`Peg::parse_to_json`](crate::peg::grammar::Peg::parse_to_json)).
    pub fn parse_to_json(&self, input: &str) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.parse(input))
    }

    fn failure(&self, position: usize) -> ParseResult {
        ParseResult(
            1,
            0,
            Arc::new(Err(ParseError {
                position,
                rule_name: self.names[self.start as usize].clone(),
                error: ErrorKind::NonTerminalDoesNotMatch,
                cause: None,
            })),
        )
    }
}

/// Keep `ParseNode` in scope for docs.
#[allow(dead_code)]
type _Node = ParseNode;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peg::grammar::Peg;

    fn shape(nodes: &[ParseNode]) -> String {
        nodes
            .iter()
            .map(|n| match n {
                ParseNode::Terminal { span } => format!("{}-{}", span.0, span.1),
                ParseNode::NonTerminal { name, span, children } => {
                    format!("{name}[{}-{}]({})", span.0, span.1, shape(children))
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn same_as_peg(start: &str, grammar: &str, inputs: &[&str]) {
        let peg = Peg::new(start, grammar).unwrap();
        let earley = EarleyGrammar::new(start, grammar).unwrap();
        for input in inputs {
            let p = peg.parse(input);
            let e = earley.parse(input);
            match (p.2.as_ref(), e.2.as_ref()) {
                (Ok(pn), Ok(en)) => {
                    assert_eq!(p.1, e.1, "end for {input:?}");
                    assert_eq!(shape(pn), shape(en), "tree for {input:?}");
                }
                (Err(_), Err(_)) => {}
                (a, b) => panic!("{input:?}: peg {a:?} vs earley {b:?}"),
            }
        }
    }

    #[test]
    fn calculator_matches_peg() {
        let g = r#"
        calc <- additive !.
        additive <- multiplicative (('+' / '-') multiplicative)*
        multiplicative <- primary (('*' / '/') primary)*
        primary <- number / '(' additive ')'
        number <- [0-9]+
        "#;
        same_as_peg("calc", g, &["1", "1+2*3", "(1+2)*3", "12*(3-4)/5", "1+", "x"]);
    }

    #[test]
    fn lookahead_and_literals() {
        let g = r#"
        start <- word+
        word <- &[a-z] 'hello' ' '? / !'hello' [a-z]+ ' '?
        "#;
        same_as_peg("start", g, &["hello world ", "hello", "helloworld", "abc hello", ""]);
    }

    #[test]
    fn complete_parse_beyond_peg_commitment() {
        // PEG commits to the first alternative `'a'` and leaves "ab" unmatched.
        let g = "s <- ('a' / 'ab') 'c'?";
        let earley = EarleyGrammar::new("s", g).unwrap();
        assert_eq!(earley.parse("abc").1, 1);
        let full = earley.parse_complete("abc");
        assert!(full.2.is_ok());
        assert_eq!(full.1, 3);
    }

    #[test]
    fn missing_rule_is_a_compile_error() {
        assert!(EarleyGrammar::new("s", "s <- 'a' undefined / 'b'").is_err());
    }

    #[test]
    fn grammar_is_send_sync() {
        fn check<T: Send + Sync>() {}
        check::<EarleyGrammar>();
    }
}
