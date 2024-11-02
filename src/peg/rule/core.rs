use super::types::Rule;
use crate::peg::grammar::PEG;
use crate::peg::parsing::{ErrorKind, ParseError, ParseNode, ParseResult, Span};
use log::debug;

impl Rule {
    pub fn parse(&self, peg: &PEG, input: &str, position: usize, depth: usize) -> ParseResult {
        match self {
            Rule::Empty => ParseResult(1, position, Ok(vec![])),

            Rule::Any => {
                if position < input.len() {
                    ParseResult(
                        1,
                        position + 1,
                        Ok(vec![ParseNode::Terminal(Span(position, position + 1))]),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Err(ParseError {
                            position,
                            expression: self.clone(),
                            error: ErrorKind::UnexpectedEndOfInput,
                            cause: None,
                        }),
                    )
                }
            }

            Rule::Literal(pattern) => {
                if input[position..].starts_with(pattern) {
                    ParseResult(
                        1,
                        position + pattern.len(),
                        Ok(vec![ParseNode::Terminal(Span(
                            position,
                            position + pattern.len(),
                        ))]),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Err(ParseError {
                            position,
                            expression: self.clone(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        }),
                    )
                }
            }

            Rule::NonTerminal(name) => {
                debug!("{}parsing {name} @ {position}", "│".repeat(depth));
                let output =
                    match peg
                        .rules
                        .get(name)
                        .unwrap()
                        .parse(peg, input, position, depth + 1)
                    {
                        ParseResult(cost, new_pos, Ok(matches)) => ParseResult(
                            cost,
                            new_pos,
                            Ok(vec![ParseNode::NonTerminal(
                                name.clone(),
                                Span(position, new_pos),
                                matches,
                            )]),
                        ),
                        ParseResult(cost, _, Err(inner)) => ParseResult(
                            cost,
                            position,
                            Err(ParseError {
                                position,
                                expression: self.clone(),
                                error: ErrorKind::NonTerminalDoesNotMatch,
                                cause: Some(Box::from(inner)),
                            }),
                        ),
                    };
                debug!(
                    "{}└{} {} @ {} -> {}",
                    "│".repeat(depth),
                    if output.2.is_ok() { "ok" } else { "err" },
                    name,
                    position,
                    output.1
                );
                output
            }

            Rule::Choice(choices) => {
                for choice in choices {
                    if let ParseResult(cost, new_pos, Ok(matches)) =
                        choice.parse(peg, input, position, depth)
                    {
                        return ParseResult(cost, new_pos, Ok(matches));
                    }
                }
                ParseResult(
                    1,
                    position,
                    Err(ParseError {
                        position,
                        expression: self.clone(),
                        error: ErrorKind::ExpressionDoesNotMatch,
                        cause: None,
                    }),
                )
            }

            Rule::Sequence(sequence) => {
                let mut captures = vec![];
                let mut pos = position;

                for expr in sequence {
                    match expr.parse(peg, input, pos, depth) {
                        ParseResult(_, p, Ok(mut m)) => {
                            pos = p;
                            captures.append(&mut m);
                        }
                        ParseResult(_, _, Err(e)) => return ParseResult(1, position, Err(e)),
                    }
                }
                ParseResult(1, pos, Ok(captures))
            }

            Rule::ZeroOrMore(expr) => {
                let mut captures = vec![];
                let mut pos = position;

                while let ParseResult(_, p, Ok(mut m)) = expr.parse(peg, input, pos, depth) {
                    pos = p;
                    captures.append(&mut m);
                }
                ParseResult(1, pos, Ok(captures))
            }

            Rule::OneOrMore(expr) => Rule::Sequence(vec![
                Rule::Group(expr.clone()),
                Rule::ZeroOrMore(expr.clone()),
            ])
            .parse(peg, input, position, depth),

            Rule::Optional(expr) => match expr.parse(peg, input, position, depth) {
                ParseResult(_, pos, Ok(matches)) => ParseResult(1, pos, Ok(matches)),
                ParseResult(_, _, Err(_)) => ParseResult(1, position, Ok(vec![])),
            },

            Rule::And(expr) => match expr.parse(peg, input, position, depth) {
                ParseResult(_, _, Ok(_)) => ParseResult(1, position, Ok(vec![])),
                ParseResult(_, _, Err(e)) => ParseResult(1, position, Err(e)),
            },

            Rule::Not(expr) => match expr.parse(peg, input, position, depth) {
                ParseResult(_, _, Ok(m)) => ParseResult(
                    1,
                    position,
                    Err(ParseError {
                        position,
                        expression: self.clone(),
                        error: ErrorKind::NotDidMatch(m),
                        cause: None,
                    }),
                ),
                ParseResult(_, _, Err(_)) => ParseResult(1, position, Ok(vec![])),
            },

            Rule::Group(expr) => expr.parse(peg, input, position, depth),

            Rule::Range(start, end) => {
                if position < input.len() {
                    let c = input[position..].chars().next().unwrap();
                    if start.chars().next().unwrap() <= c && c <= end.chars().next().unwrap() {
                        ParseResult(
                            1,
                            position + 1,
                            Ok(vec![ParseNode::Terminal(Span(position, position + 1))]),
                        )
                    } else {
                        ParseResult(
                            1,
                            position,
                            Err(ParseError {
                                position,
                                expression: self.clone(),
                                error: ErrorKind::ExpressionDoesNotMatch,
                                cause: None,
                            }),
                        )
                    }
                } else {
                    ParseResult(
                        1,
                        position,
                        Err(ParseError {
                            position,
                            expression: self.clone(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        }),
                    )
                }
            }

            Rule::Class(symbols) => {
                if let Some(symbol) = symbols
                    .iter()
                    .find(|s| input[position..].starts_with(s.as_str()))
                {
                    ParseResult(
                        1,
                        position + symbol.len(),
                        Ok(vec![ParseNode::Terminal(Span(
                            position,
                            position + symbol.len(),
                        ))]),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Err(ParseError {
                            position,
                            expression: self.clone(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        }),
                    )
                }
            }
        }
    }
}
