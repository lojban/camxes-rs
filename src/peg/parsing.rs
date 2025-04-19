use crate::peg::rule::Rule;
use serde::Serialize; // Import Serialize
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize)] // Add Serialize
pub struct Span(pub usize, pub usize);

#[derive(Clone, Debug, Serialize)] // Add Serialize
#[serde(tag = "type")] // Use tagged enum representation for clarity in JSON
pub enum ParseNode {
    Terminal { span: Span },
    NonTerminal {
        name: String,
        span: Span,
        children: Vec<ParseNode>,
    },
}

// Custom Debug implementation for ParseResult to avoid deriving Debug on Result<...>
#[derive(Clone)]
pub struct ParseResult(pub u32, pub usize, pub Result<Vec<ParseNode>, ParseError>);

impl std::fmt::Debug for ParseResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParseResult")
         .field(&self.0) // cost
         .field(&self.1) // position
         .field(&self.2) // result (Vec<ParseNode> or ParseError)
         .finish()
    }
}


// Make ParseResult serializable
#[derive(Serialize)] // Add Serialize
struct SerializableParseResult<'a> {
    cost: u32,
    position: usize,
    #[serde(flatten)] // Flatten Result into the main structure
    result: &'a Result<Vec<ParseNode>, ParseError>,
}

impl Serialize for ParseResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableParseResult {
            cost: self.0,
            position: self.1,
            result: &self.2,
        }
        .serialize(serializer)
    }
}


#[derive(Clone, Debug, Serialize)] // Add Serialize
#[serde(tag = "kind")] // Use tagged enum representation
pub enum ErrorKind {
    UnexpectedEndOfInput,
    ExpressionDoesNotMatch,
    NotDidMatch(Vec<ParseNode>),
    NonTerminalDoesNotMatch,
    NonTerminalDoesNotExist(String),
}

#[derive(Clone, Debug, Serialize)] // Add Serialize
pub struct ParseError {
    pub position: usize,
    #[serde(skip)] // Skip serializing the full Rule enum for now to avoid complexity
    pub expression: Rule, // Consider serializing Rule name or simplified representation if needed
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
