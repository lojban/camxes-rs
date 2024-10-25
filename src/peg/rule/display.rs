use std::fmt::{Display, Formatter};
use super::types::Rule;

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rule::Empty => write!(f, "()"),
            Rule::Any => write!(f, "."),
            Rule::Literal(text) => write!(
                f,
                "'{}'",
                text.replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\'', "\\'")
            ),
            Rule::NonTerminal(name) => write!(f, "{}", name),
            Rule::Range(start, end) => write!(f, "[{}-{}]", start, end),
            Rule::Class(symbols) => write!(
                f,
                "[{}]",
                symbols
                    .iter()
                    .map(|s| s
                        .replace('\\', "\\\\")
                        .replace('[', "\\[")
                        .replace(']', "\\]"))
                    .collect::<Vec<_>>()
                    .join("")
            ),
            Rule::Group(expr) => write!(f, "({})", expr),
            Rule::ZeroOrMore(expr) => write!(f, "{}*", expr),
            Rule::OneOrMore(expr) => write!(f, "{}+", expr),
            Rule::Optional(expr) => write!(f, "{}?", expr),
            Rule::And(expr) => write!(f, "&{}", expr),
            Rule::Not(expr) => write!(f, "!{}", expr),
            Rule::Choice(choices) => {
                let joined = choices
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" / ");
                write!(
                    f,
                    "{}",
                    if choices.len() > 1 {
                        format!("({})", joined)
                    } else {
                        joined
                    }
                )
            },
            Rule::Sequence(sequence) => {
                let joined = sequence
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(
                    f,
                    "{}",
                    if sequence.len() > 1 {
                        format!("({})", joined)
                    } else {
                        joined
                    }
                )
            },
        }
    }
}