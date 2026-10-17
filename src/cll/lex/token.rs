//! Token definitions — CLL §4.1 three word classes + brivla subtypes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordClass {
    /// §4.2
    Cmavo,
    /// §4.4
    Gismu,
    /// §4.5
    Lujvo,
    /// §4.7
    Fuhivla,
    /// §4.8 (cmevla / cmevla)
    Cmevla,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Explicit pause — §3.3 / §4.9
    Pause,
    Word {
        class: WordClass,
        text: String,
    },
    /// Reserved / experimental forms under feature flag — §4.2
    ExperimentalCmavo(String),
}

impl Token {
    /// Map toward Ch. 21 terminal families (documentation helper).
    pub fn ch21_terminal_family(&self) -> &'static str {
        match self {
            Token::Pause => "# / pause (not a selma'o)",
            Token::Word {
                class: WordClass::Cmevla,
                ..
            } => "CMENE",
            Token::Word {
                class: WordClass::Gismu | WordClass::Lujvo | WordClass::Fuhivla,
                ..
            } => "BRIVLA",
            Token::Word {
                class: WordClass::Cmavo,
                ..
            } => "selma'o (post-morph classification)",
            Token::ExperimentalCmavo(_) => "experimental cmavo",
        }
    }
}
