use super::types::Rule;
use crate::peg::grammar::Peg;
use crate::peg::parsing::{ErrorKind, ParseError, ParseNode, ParseResult, Span};
use log::{debug, log_enabled, Level};
use std::sync::Arc;

impl Rule {
    pub fn parse(&self, peg: &Peg, input: &str, position: usize, depth: usize) -> ParseResult {
        match self {
            Rule::Empty => ParseResult(1, position, Arc::new(Ok(vec![]))),

            Rule::Any => {
                if position < input.len() {
                    ParseResult(
                        1,
                        position + 1,
                        Arc::new(Ok(vec![ParseNode::Terminal {
                            span: Span(position, position + 1),
                        }])),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Arc::new(Err(ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::UnexpectedEndOfInput,
                            cause: None,
                        })),
                    )
                }
            }

            Rule::Literal(pattern) => {
                // Fast path: single-byte literal
                let matched = if pattern.len() == 1 {
                    position < input.len() && input.as_bytes()[position] == pattern.as_bytes()[0]
                } else {
                    input[position..].starts_with(pattern)
                };
                if matched {
                    let len = pattern.len();
                    ParseResult(
                        1,
                        position + len,
                        Arc::new(Ok(vec![ParseNode::Terminal {
                            span: Span(position, position + len),
                        }])),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Arc::new(Err(ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        })),
                    )
                }
            }

            Rule::NonTerminal(name) => {
                let key = (name.clone(), position);

                if let Some(cached_result) = peg.memo.borrow().get(&key) {
                    if log_enabled!(Level::Debug) {
                        debug!(
                            "{}cache hit {name} @ {position} -> {}",
                            "│".repeat(depth),
                            cached_result.1
                        );
                    }
                    return cached_result.clone();
                }

                if log_enabled!(Level::Debug) {
                    debug!("{}parsing {name} @ {position}", "│".repeat(depth));
                }

                let rule = match peg.rules.get(name) {
                    Some(r) => r,
                    None => {
                        let err = ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::NonTerminalDoesNotExist(name.clone()),
                            cause: None,
                        };
                        let res = ParseResult(1, position, Arc::new(Err(err)));
                        peg.memo.borrow_mut().insert(key, res.clone());
                        return res;
                    }
                };

                let result = match rule.parse(peg, input, position, depth + 1) {
                    ParseResult(cost, new_pos, ref payload) => match payload.as_ref() {
                        Ok(matches) => ParseResult(
                            cost,
                            new_pos,
                            Arc::new(Ok(vec![ParseNode::NonTerminal {
                                name: name.clone(),
                                span: Span(position, new_pos),
                                children: matches.clone(),
                            }])),
                        ),
                        Err(inner) => ParseResult(
                            cost,
                            position,
                            Arc::new(Err(ParseError {
                                position,
                                rule_name: self.error_description(),
                                error: ErrorKind::NonTerminalDoesNotMatch,
                                cause: Some(Box::new(inner.clone())),
                            })),
                        ),
                    },
                };

                if log_enabled!(Level::Debug) {
                    debug!(
                        "{}└{} {} @ {} -> {}",
                        "│".repeat(depth),
                        if result.2.is_ok() { "ok" } else { "err" },
                        name,
                        position,
                        result.1
                    );
                }

                peg.memo.borrow_mut().insert(key, result.clone());
                result
            }

            Rule::Choice(choices) => {
                for choice in choices {
                    let res = choice.parse(peg, input, position, depth);
                    if res.2.is_ok() {
                        return res;
                    }
                }
                ParseResult(
                    1,
                    position,
                    Arc::new(Err(ParseError {
                        position,
                        rule_name: self.error_description(),
                        error: ErrorKind::ExpressionDoesNotMatch,
                        cause: None,
                    })),
                )
            }

            Rule::Sequence(sequence) => {
                let mut captures = Vec::with_capacity(sequence.len());
                let mut pos = position;

                for expr in sequence {
                    let res = expr.parse(peg, input, pos, depth);
                    match res.2.as_ref() {
                        Ok(m) => {
                            pos = res.1;
                            captures.extend(m.iter().cloned());
                        }
                        Err(e) => {
                            return ParseResult(1, position, Arc::new(Err(e.clone())));
                        }
                    }
                }
                ParseResult(1, pos, Arc::new(Ok(captures)))
            }

            Rule::ZeroOrMore(expr) => {
                let mut captures = Vec::with_capacity(8);
                let mut pos = position;

                loop {
                    let res = expr.parse(peg, input, pos, depth);
                    match res.2.as_ref() {
                        Ok(m) => {
                            pos = res.1;
                            captures.extend(m.iter().cloned());
                        }
                        Err(_) => break,
                    }
                }
                ParseResult(1, pos, Arc::new(Ok(captures)))
            }

            Rule::OneOrMore(expr) => Rule::Sequence(vec![
                Rule::Group(expr.clone()),
                Rule::ZeroOrMore(expr.clone()),
            ])
            .parse(peg, input, position, depth),

            Rule::Optional(expr) => {
                let res = expr.parse(peg, input, position, depth);
                if res.2.is_ok() {
                    ParseResult(1, res.1, res.2)
                } else {
                    ParseResult(1, position, Arc::new(Ok(vec![])))
                }
            }

            Rule::And(expr) => {
                let res = expr.parse(peg, input, position, depth);
                if res.2.is_ok() {
                    ParseResult(1, position, Arc::new(Ok(vec![])))
                } else {
                    let e = res.2.as_ref().clone().err().unwrap();
                    ParseResult(1, position, Arc::new(Err(e)))
                }
            }

            Rule::Not(expr) => {
                let res = expr.parse(peg, input, position, depth);
                match res.2.as_ref() {
                    Ok(m) => ParseResult(
                        1,
                        position,
                        Arc::new(Err(ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::NotDidMatch(m.clone()),
                            cause: None,
                        })),
                    ),
                    Err(_) => ParseResult(1, position, Arc::new(Ok(vec![]))),
                }
            }

            Rule::Group(expr) => expr.parse(peg, input, position, depth),

            Rule::Range(start, end) => {
                if position < input.len() {
                    let c = input[position..].chars().next().unwrap();
                    if start.chars().next().unwrap() <= c && c <= end.chars().next().unwrap() {
                        ParseResult(
                            1,
                            position + 1,
                            Arc::new(Ok(vec![ParseNode::Terminal {
                                span: Span(position, position + 1),
                            }])),
                        )
                    } else {
                        ParseResult(
                            1,
                            position,
                            Arc::new(Err(ParseError {
                                position,
                                rule_name: self.error_description(),
                                error: ErrorKind::ExpressionDoesNotMatch,
                                cause: None,
                            })),
                        )
                    }
                } else {
                    ParseResult(
                        1,
                        position,
                        Arc::new(Err(ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        })),
                    )
                }
            }

            Rule::Class(symbols) => {
                // Longest match first: sort by length descending
                let mut syms: Vec<&String> = symbols.iter().collect();
                syms.sort_by(|a, b| b.len().cmp(&a.len()));
                let matched = syms
                    .into_iter()
                    .find(|s| input[position..].starts_with(s.as_str()));
                if let Some(symbol) = matched {
                    let len = symbol.len();
                    ParseResult(
                        1,
                        position + len,
                        Arc::new(Ok(vec![ParseNode::Terminal {
                            span: Span(position, position + len),
                        }])),
                    )
                } else {
                    ParseResult(
                        1,
                        position,
                        Arc::new(Err(ParseError {
                            position,
                            rule_name: self.error_description(),
                            error: ErrorKind::ExpressionDoesNotMatch,
                            cause: None,
                        })),
                    )
                }
            }
        }
    }
}
