use std::collections::HashSet;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Rule {
    Empty,
    Any,
    Literal(String),
    NonTerminal(String),
    Range(String, String),
    Class(HashSet<String>),
    Group(Arc<Rule>),
    ZeroOrMore(Arc<Rule>),
    OneOrMore(Arc<Rule>),
    Optional(Arc<Rule>),
    And(Arc<Rule>),
    Not(Arc<Rule>),
    Choice(Vec<Rule>),
    Sequence(Vec<Rule>),
}

impl Rule {
    pub fn boxed(self) -> Arc<Rule> {
        Arc::new(self)
    }

    pub fn create_character_class(chars: &[&str]) -> Rule {
        Rule::Class(chars.iter().map(|&c| c.to_string()).collect())
    }
}
