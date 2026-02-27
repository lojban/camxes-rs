use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

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

/// Parse result: (cost, position, payload). Payload is Arc-wrapped so cloning is cheap (memo cache).
#[derive(Clone)]
pub struct ParseResult(pub u32, pub usize, pub Arc<Result<Vec<ParseNode>, ParseError>>);

impl std::fmt::Debug for ParseResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParseResult")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2)
            .finish()
    }
}

#[derive(Serialize)]
struct SerializableParseResult<'a> {
    cost: u32,
    position: usize,
    #[serde(flatten)]
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
            result: self.2.as_ref(),
        }
        .serialize(serializer)
    }
}


#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind")]
pub enum ErrorKind {
    UnexpectedEndOfInput,
    ExpressionDoesNotMatch,
    NotDidMatch(Vec<ParseNode>),
    NonTerminalDoesNotMatch,
    NonTerminalDoesNotExist(String),
}

/// Parse error with lazy line/column: only `position` is stored; use `line_column(input)` when needed.
#[derive(Clone, Debug, Serialize)]
pub struct ParseError {
    pub position: usize,
    /// Rule name or short description for error reporting (no full Rule clone).
    pub rule_name: String,
    pub error: ErrorKind,
    pub cause: Option<Box<ParseError>>,
}

/// Compute (1-based line, 1-based column) from input and byte position. O(position).
pub fn line_column(input: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for (i, c) in input.char_indices() {
        if i >= position {
            break;
        }
        if c == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

impl ParseError {
    /// Lazy line/column computation (call when formatting or reporting).
    pub fn line_column(&self, input: &str) -> (usize, usize) {
        line_column(input, self.position)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.cause {
            None => write!(
                f,
                "Encountered {} @ {} for '{}'",
                self.error, self.position, self.rule_name
            ),
            Some(inner) => write!(
                f,
                "Encountered {} @ {} for '{}'\n\tCaused by: {}",
                self.error, self.position, self.rule_name, inner
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
