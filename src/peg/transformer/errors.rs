use std::fmt::{Display, Formatter};

/// Error types for the transformer module
#[derive(Clone, Debug)]
pub enum TransformError {
    /// Error when CST contains multiple root nodes
    CstShouldOnlyHaveOneRoot(String),
    /// Error when CST doesn't start with a grammar node
    CstShouldStartWithGrammar(String),
    /// Error when encountering an unexpected parse_node
    UnExpectedToken(String),
    /// Error when a non-terminal reference is ambiguous
    AmbiguousNonTerminal(String),
    /// Error when identifier is empty
    EmptyIdentifier,
    /// Error when parse_node count doesn't match expected
    WrongNumberOfTokens(String),
}

impl Display for TransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::CstShouldOnlyHaveOneRoot(msg) => 
                write!(f, "CST should have exactly one root: {}", msg),
            TransformError::CstShouldStartWithGrammar(msg) => 
                write!(f, "CST should start with grammar: {}", msg),
            TransformError::UnExpectedToken(token) => 
                write!(f, "Unexpected token: {}", token),
            TransformError::AmbiguousNonTerminal(msg) => 
                write!(f, "Ambiguous non-terminal reference: {}", msg),
            TransformError::EmptyIdentifier => 
                write!(f, "Empty identifier"),
            TransformError::WrongNumberOfTokens(msg) => 
                write!(f, "Wrong number of tokens: {}", msg),
        }
    }
}