use crate::peg::parsing::ParseError;
use crate::peg::transformer::TransformError;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum GrammarError {
    Parse(ParseError),
    Transform(TransformError),
}

impl Display for GrammarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GrammarError::Parse(err) => write!(f, "Parse error: {}", err),
            GrammarError::Transform(err) => write!(f, "Transform error: {}", err),
        }
    }
}

impl From<ParseError> for GrammarError {
    fn from(error: ParseError) -> Self {
        GrammarError::Parse(error)
    }
}

impl From<TransformError> for GrammarError {
    fn from(error: TransformError) -> Self {
        GrammarError::Transform(error)
    }
}
