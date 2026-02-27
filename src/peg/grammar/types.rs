use crate::peg::parsing::ParseResult;
use crate::peg::rule::Rule;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Memo key: (rule_name, position). Using rustc_hash for faster hashing in the hot path.
pub type MemoMap = rustc_hash::FxHashMap<(String, usize), ParseResult>;

#[derive(Clone, Debug)]
pub struct Peg {
    pub rules: Arc<HashMap<String, Rule>>,
    pub start: String,
    pub memo: RefCell<MemoMap>,
}

impl Display for Peg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut rules: Vec<_> = self
            .rules
            .iter()
            .map(|(name, expr)| format!("\t{} <- {}", name, expr))
            .collect();
        rules.sort();
        write!(f, "PEG ({}) {{\n{}\n}}", self.start, rules.join("\n"))
    }
}
