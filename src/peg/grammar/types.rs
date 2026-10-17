use crate::peg::annotations::Annotation;
use crate::peg::parsing::ParseResult;
use crate::peg::rule::Rule;
use std::cell::{Cell, RefCell};
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
    /// Optional cap on the stack bytes one parse may use (see
    /// [`Peg::set_stack_budget`]).
    pub(crate) stack_budget: Option<usize>,
    /// Lowest stack address the current parse may reach (0 = not armed).
    pub(crate) stack_floor: Cell<usize>,
    pub(crate) recursion_exceeded: Cell<bool>,
}

/// Saved delimiter stack; `None` when the grammar has no annotations.
pub(crate) type DelimiterSnapshot = Option<Vec<Vec<u8>>>;

impl Peg {
    /// The cap set with [`Peg::set_stack_budget`], if any.
    pub fn stack_budget(&self) -> Option<usize> {
        self.stack_budget
    }

    /// Cap the stack bytes a single parse may use before it fails with
    /// [`ErrorKind::RecursionLimit`](crate::peg::parsing::ErrorKind::RecursionLimit).
    /// `None` (the default) allows the current thread's whole stack minus a
    /// safety margin, or [`FALLBACK_STACK_BUDGET`](crate::peg::stack::FALLBACK_STACK_BUDGET)
    /// where the stack size cannot be queried. A cap never lets a parse go
    /// past the thread's real stack.
    pub fn set_stack_budget(&mut self, bytes: Option<usize>) {
        self.stack_budget = bytes;
    }

    /// Set the stack guard for a parse starting here.
    pub(crate) fn arm_stack_guard(&self) {
        self.recursion_exceeded.set(false);
        self.stack_floor
            .set(crate::peg::stack::stack_floor(self.stack_budget));
    }

    /// True (and the parse is marked failed) when the stack is nearly used up.
    #[inline]
    pub(crate) fn out_of_stack(&self) -> bool {
        if self.recursion_exceeded.get() {
            return true;
        }
        if self.stack_floor.get() == 0 {
            // Rule::parse called directly, not through Peg::parse.
            self.arm_stack_guard();
        }
        if crate::peg::stack::stack_address() < self.stack_floor.get() {
            self.recursion_exceeded.set(true);
            true
        } else {
            false
        }
    }

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
