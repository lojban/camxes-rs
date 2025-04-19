use crate::peg::rule::Rule;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct Span(pub usize, pub usize);

#[derive(Clone, Debug)]
pub enum ParseNode {
    Terminal(Span),
    NonTerminal(String, Span, Vec<ParseNode>),
}

#[derive(Clone, Debug)]
pub struct ParseResult(pub u32, pub usize, pub Result<Vec<ParseNode>, ParseError>);

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnexpectedEndOfInput,
    ExpressionDoesNotMatch,
    NotDidMatch(Vec<ParseNode>),
    NonTerminalDoesNotMatch,
    NonTerminalDoesNotExist(String),
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub position: usize,
    pub expression: Rule,
    pub error: ErrorKind,
    pub cause: Option<Box<ParseError>>,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            None => write!(
                f,
                "Encountered {} @ {} for '{}'",
                self.error, self.position, self.expression
            ),
            Some(inner) => write!(
                f,
                "Encountered {} @ {} for '{}'\n\tCaused by: {}",
                self.error, self.position, self.expression, inner
            ),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ErrorKind::ExpressionDoesNotMatch => write!(f, "Expression does not match"),
            ErrorKind::NotDidMatch(nodes) => {
                write!(f, "Not predicate matched {} nodes", nodes.len())
            }
            ErrorKind::NonTerminalDoesNotMatch => write!(f, "Non-terminal does not match"),
            ErrorKind::NonTerminalDoesNotExist(name) => {
                write!(f, "Non-terminal rule '{}' does not exist", name)
            }
        }
    }
}
