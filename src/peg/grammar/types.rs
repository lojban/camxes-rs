use crate::peg::annotations::Annotation;
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
    /// `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ` rules (see
    /// [`crate::peg::annotations`]).
    pub(crate) annotations: Arc<rustc_hash::FxHashMap<String, Annotation>>,
    /// Delimiter stack (normalized words) for annotated rules.
    pub(crate) delimiters: RefCell<Vec<Vec<u8>>>,
}

/// Saved delimiter stack; `None` when the grammar has no annotations.
pub(crate) type DelimiterSnapshot = Option<Vec<Vec<u8>>>;

impl Peg {
    /// Save the delimiter stack before something that may fail. Free for
    /// grammars without annotations; cheap (no allocation) while the stack
    /// is empty.
    #[inline]
    pub(crate) fn save_delimiters(&self) -> DelimiterSnapshot {
        if self.annotations.is_empty() {
            None
        } else {
            Some(self.delimiters.borrow().clone())
        }
    }

    #[inline]
    pub(crate) fn restore_delimiters(&self, saved: DelimiterSnapshot) {
        if let Some(saved) = saved {
            *self.delimiters.borrow_mut() = saved;
        }
    }
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
