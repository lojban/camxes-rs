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
