use std::collections::HashMap;
use std::sync::Arc;
use crate::peg::rule::Rule;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct PEG {
    pub rules: Arc<HashMap<String, Rule>>,
    pub start: String,
}

impl Display for PEG {
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
