use super::types::Rule;
use crate::peg::annotations::{delimiter_key, Annotation};
use crate::peg::grammar::Peg;
use crate::peg::parsing::{ErrorKind, ParseError, ParseNode, ParseResult, Span};
use log::{debug, log_enabled, Level};
use std::sync::Arc;

// `parse` only dispatches; each kind of rule is a separate non-inlined
// function so the recursive path keeps small stack frames (one frame holding
// every arm's locals made deep inputs overflow the stack, especially in
// debug builds). Positions always stay on character boundaries: every rule
// consumes whole characters (`.` and ranges) or whole literal strings.
impl Rule {
    pub fn parse(&self, peg: &Peg, input: &str, position: usize, depth: usize) -> ParseResult {
        match self {
            Rule::Empty => Self::matched(position, vec![]),
            Rule::Any => self.parse_any(input, position),
            Rule::Literal(pattern) => self.parse_literal(pattern, input, position),
            Rule::NonTerminal(name) => self.parse_nonterminal(name, peg, input, position, depth),
            Rule::Choice(choices) => self.parse_choice(choices, peg, input, position, depth),
            Rule::Sequence(sequence) => Self::parse_sequence(sequence, peg, input, position, depth),
            Rule::ZeroOrMore(expr) => Self::parse_star(expr, peg, input, position, depth),
            Rule::OneOrMore(expr) => Self::parse_plus(expr, peg, input, position, depth),
            Rule::Optional(expr) => Self::parse_optional(expr, peg, input, position, depth),
            Rule::And(expr) => Self::parse_and(expr, peg, input, position, depth),
            Rule::Not(expr) => self.parse_not(expr, peg, input, position, depth),
            Rule::Group(expr) => expr.parse(peg, input, position, depth),
            Rule::Range(start, end) => self.parse_range(start, end, input, position),
            Rule::Class(symbols) => self.parse_class(symbols, input, position),
        }
    }

    #[inline]
    fn matched(end: usize, nodes: Vec<ParseNode>) -> ParseResult {
        ParseResult(1, end, Arc::new(Ok(nodes)))
    }

    #[inline]
    fn terminal(position: usize, end: usize) -> ParseResult {
        Self::matched(
            end,
            vec![ParseNode::Terminal {
                span: Span(position, end),
            }],
        )
    }

    #[cold]
    #[inline(never)]
    fn fail(&self, position: usize, error: ErrorKind) -> ParseResult {
        ParseResult(
            1,
            position,
            Arc::new(Err(ParseError {
                position,
                rule_name: self.error_description(),
                error,
                cause: None,
            })),
        )
    }

    #[inline(never)]
    fn parse_any(&self, input: &str, position: usize) -> ParseResult {
        match input.get(position..).and_then(|rest| rest.chars().next()) {
            Some(c) => Self::terminal(position, position + c.len_utf8()),
            None => self.fail(position, ErrorKind::UnexpectedEndOfInput),
        }
    }

    #[inline(never)]
    fn parse_literal(&self, pattern: &str, input: &str, position: usize) -> ParseResult {
        let rest = input.as_bytes().get(position..).unwrap_or_default();
        let matched = if pattern.len() == 1 {
            // Fast path: single-byte literal
            rest.first() == pattern.as_bytes().first()
        } else {
            rest.starts_with(pattern.as_bytes())
        };
        if matched {
            Self::terminal(position, position + pattern.len())
        } else {
            self.fail(position, ErrorKind::ExpressionDoesNotMatch)
        }
    }

    #[inline(never)]
    fn parse_nonterminal(
        &self,
        name: &str,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let annotation = if peg.annotations.is_empty() {
            None
        } else {
            peg.annotations.get(name).copied()
        };
        if let Some(annotation) = annotation {
            return self.parse_annotated(peg, name, annotation, input, position, depth);
        }
        if peg.out_of_stack() {
            return Self::recursion_error(name, position);
        }
        let key = (name.to_string(), position);

        if let Some(cached_result) = peg.memo.borrow().get(&key) {
            if log_enabled!(Level::Debug) {
                Self::log_cache_hit(name, position, depth, cached_result.1);
            }
            return cached_result.clone();
        }

        if log_enabled!(Level::Debug) {
            debug!("{}parsing {name} @ {position}", "│".repeat(depth));
        }

        let result = match peg.rules.get(name) {
            Some(rule) => Self::wrap_nonterminal(
                self,
                name,
                position,
                rule.parse(peg, input, position, depth + 1),
            ),
            None => self.fail(
                position,
                ErrorKind::NonTerminalDoesNotExist(name.to_string()),
            ),
        };

        if log_enabled!(Level::Debug) {
            Self::log_result(name, position, depth, &result);
        }

        peg.memo.borrow_mut().insert(key, result.clone());
        result
    }

    /// A nonterminal's node around its body's result (or the error chain).
    fn wrap_nonterminal(&self, name: &str, position: usize, body: ParseResult) -> ParseResult {
        let ParseResult(cost, new_pos, payload) = body;
        match payload.as_ref() {
            Ok(matches) => ParseResult(
                cost,
                new_pos,
                Arc::new(Ok(vec![ParseNode::NonTerminal {
                    name: name.to_string(),
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
        }
    }

    #[cold]
    #[inline(never)]
    fn log_cache_hit(name: &str, position: usize, depth: usize, end: usize) {
        debug!(
            "{}cache hit {name} @ {position} -> {end}",
            "│".repeat(depth)
        );
    }

    #[cold]
    #[inline(never)]
    fn log_result(name: &str, position: usize, depth: usize, result: &ParseResult) {
        debug!(
            "{}└{} {} @ {} -> {}",
            "│".repeat(depth),
            if result.2.is_ok() { "ok" } else { "err" },
            name,
            position,
            result.1
        );
    }

    #[inline(never)]
    fn parse_choice(
        &self,
        choices: &[Rule],
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        for choice in choices {
            let res = choice.parse(peg, input, position, depth);
            if res.2.is_ok() {
                return res;
            }
        }
        self.fail(position, ErrorKind::ExpressionDoesNotMatch)
    }

    #[inline(never)]
    fn parse_sequence(
        sequence: &[Rule],
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let mut captures = Vec::with_capacity(sequence.len());
        let mut pos = position;
        let saved = peg.save_delimiters();

        for expr in sequence {
            let res = expr.parse(peg, input, pos, depth);
            match res.2.as_ref() {
                Ok(m) => {
                    pos = res.1;
                    captures.extend(m.iter().cloned());
                }
                Err(e) => {
                    peg.restore_delimiters(saved);
                    return ParseResult(1, position, Arc::new(Err(e.clone())));
                }
            }
        }
        Self::matched(pos, captures)
    }

    /// `expr*` from `position`, appending to `captures`; returns the end.
    fn star_from(
        expr: &Rule,
        peg: &Peg,
        input: &str,
        mut pos: usize,
        depth: usize,
        captures: &mut Vec<ParseNode>,
    ) -> usize {
        loop {
            let res = expr.parse(peg, input, pos, depth);
            match res.2.as_ref() {
                // A match that consumed nothing would repeat forever: stop before it.
                Ok(m) if res.1 != pos => {
                    pos = res.1;
                    captures.extend(m.iter().cloned());
                }
                Ok(_) | Err(_) => return pos,
            }
        }
    }

    #[inline(never)]
    fn parse_star(
        expr: &Rule,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let mut captures = Vec::with_capacity(8);
        let end = Self::star_from(expr, peg, input, position, depth, &mut captures);
        Self::matched(end, captures)
    }

    #[inline(never)]
    fn parse_plus(
        expr: &Rule,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let saved = peg.save_delimiters();
        let first = expr.parse(peg, input, position, depth);
        match first.2.as_ref() {
            Ok(m) => {
                let mut captures = m.clone();
                let end = if first.1 == position {
                    position
                } else {
                    Self::star_from(expr, peg, input, first.1, depth, &mut captures)
                };
                Self::matched(end, captures)
            }
            Err(e) => {
                peg.restore_delimiters(saved);
                ParseResult(1, position, Arc::new(Err(e.clone())))
            }
        }
    }

    #[inline(never)]
    fn parse_optional(
        expr: &Rule,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let res = expr.parse(peg, input, position, depth);
        if res.2.is_ok() {
            ParseResult(1, res.1, res.2)
        } else {
            Self::matched(position, vec![])
        }
    }

    #[inline(never)]
    fn parse_and(
        expr: &Rule,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let saved = peg.save_delimiters();
        let res = expr.parse(peg, input, position, depth);
        peg.restore_delimiters(saved);
        match res.2.as_ref() {
            Ok(_) => Self::matched(position, vec![]),
            Err(e) => ParseResult(1, position, Arc::new(Err(e.clone()))),
        }
    }

    #[inline(never)]
    fn parse_not(
        &self,
        expr: &Rule,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let saved = peg.save_delimiters();
        let res = expr.parse(peg, input, position, depth);
        peg.restore_delimiters(saved);
        match res.2.as_ref() {
            Ok(m) => self.fail(position, ErrorKind::NotDidMatch(m.clone())),
            Err(_) => Self::matched(position, vec![]),
        }
    }

    #[inline(never)]
    fn parse_range(&self, start: &str, end: &str, input: &str, position: usize) -> ParseResult {
        let c = input.get(position..).and_then(|rest| rest.chars().next());
        match (c, start.chars().next(), end.chars().next()) {
            (Some(c), Some(lo), Some(hi)) if lo <= c && c <= hi => {
                Self::terminal(position, position + c.len_utf8())
            }
            _ => self.fail(position, ErrorKind::ExpressionDoesNotMatch),
        }
    }

    #[inline(never)]
    fn parse_class(
        &self,
        symbols: &std::collections::HashSet<String>,
        input: &str,
        position: usize,
    ) -> ParseResult {
        // Longest match first.
        let rest = input.as_bytes().get(position..).unwrap_or_default();
        let matched = symbols
            .iter()
            .filter(|s| !s.is_empty() && rest.starts_with(s.as_bytes()))
            .map(String::len)
            .max();
        match matched {
            Some(len) => Self::terminal(position, position + len),
            None => self.fail(position, ErrorKind::ExpressionDoesNotMatch),
        }
    }

    /// A `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ` rule: parse the rule
    /// normally, then apply the delimiter-stack check to the matched text.
    /// PUSH results are memoized (the push is replayed on a cache hit); the
    /// other two depend on the stack and are not.
    fn parse_annotated(
        &self,
        peg: &Peg,
        name: &str,
        annotation: Annotation,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        if peg.out_of_stack() {
            return Self::recursion_error(name, position);
        }
        let result = if annotation == Annotation::Push {
            let memo_key = (name.to_string(), position);
            let cached = peg.memo.borrow().get(&memo_key).cloned();
            match cached {
                Some(cached) => cached,
                None => {
                    let fresh = Rule::NonTerminal(name.to_string())
                        .parse_unannotated(peg, input, position, depth);
                    peg.memo.borrow_mut().insert(memo_key, fresh.clone());
                    fresh
                }
            }
        } else {
            Rule::NonTerminal(name.to_string()).parse_unannotated(peg, input, position, depth)
        };
        if result.2.is_err() {
            return result;
        }
        let word = delimiter_key(input.as_bytes().get(position..result.1).unwrap_or_default());
        let mut stack = peg.delimiters.borrow_mut();
        let accepted = match annotation {
            Annotation::Push => {
                stack.push(word);
                true
            }
            Annotation::PeekDiff => stack.last() != Some(&word),
            Annotation::PopEq => {
                let equal = stack.last() == Some(&word);
                if equal {
                    stack.pop();
                }
                equal
            }
        };
        if accepted {
            result
        } else {
            ParseResult(
                1,
                position,
                Arc::new(Err(ParseError {
                    position,
                    rule_name: self.error_description(),
                    error: ErrorKind::NonTerminalDoesNotMatch,
                    cause: None,
                })),
            )
        }
    }

    /// The stack budget ran out (the whole parse is then reported as
    /// [`ErrorKind::RecursionLimit`] by [`Peg::parse`]).
    fn recursion_error(name: &str, position: usize) -> ParseResult {
        ParseResult(
            1,
            position,
            Arc::new(Err(ParseError {
                position,
                rule_name: name.to_string(),
                error: ErrorKind::RecursionLimit,
                cause: None,
            })),
        )
    }

    /// Parse a nonterminal's body without memoization or annotation checks.
    fn parse_unannotated(
        &self,
        peg: &Peg,
        input: &str,
        position: usize,
        depth: usize,
    ) -> ParseResult {
        let Rule::NonTerminal(name) = self else {
            return self.parse(peg, input, position, depth);
        };
        let Some(rule) = peg.rules.get(name) else {
            return ParseResult(
                1,
                position,
                Arc::new(Err(ParseError {
                    position,
                    rule_name: self.error_description(),
                    error: ErrorKind::NonTerminalDoesNotExist(name.clone()),
                    cause: None,
                })),
            );
        };
        match rule.parse(peg, input, position, depth + 1) {
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
        }
    }
}
