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
    Group(Arc<Box<Rule>>),
    ZeroOrMore(Arc<Box<Rule>>),
    OneOrMore(Arc<Box<Rule>>),
    Optional(Arc<Box<Rule>>),
    And(Arc<Box<Rule>>),
    Not(Arc<Box<Rule>>),
    Choice(Vec<Rule>),
    Sequence(Vec<Rule>),
}

impl Rule {
    pub fn boxed(self) -> Arc<Box<Rule>> {
        Arc::new(Box::new(self))
    }

    pub fn create_character_class(chars: &[&str]) -> Rule {
        Rule::Class(chars.iter().map(|&c| c.to_string()).collect())
    }
}
