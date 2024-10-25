use crate::peg::parsing::ParseError;
use crate::peg::transformer::TransformError;

#[derive(Clone, Debug)]
pub enum GrammarError {
    Parse(ParseError),
    Transform(TransformError),
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
