use super::errors::TransformError;
use crate::peg::grammar::PEG;
use crate::peg::grammar::{
    AND, ARROW, CHAR, CLASS, CLASS_MEMBER, DEF, DOT, EOF, EXPR, IDENT, LITERAL, LPAR, NOT, PLUS,
    PREFIX, PRIMARY, QUESTION, RANGE, RPAR, SEQUENCE, SLASH, SPACING, STAR, SUFFIX, TEXT,
};
use crate::peg::parsing::{ParseNode, Span};
use crate::peg::rule::Rule;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

type Result<T> = std::result::Result<T, TransformError>;

pub struct Transformer<'a> {
    pub source: &'a str,
}

impl Transformer<'_> {
    pub fn build(&self, start_rule: &str, cst: Vec<ParseNode>) -> Result<PEG> {
        match &cst[..] {
            [ParseNode::NonTerminal(name, _, tokens)] if name == TEXT => Ok(PEG {
                rules: self.build_grammar_rules(tokens)?,
                start: start_rule.to_string(),
            }),
            [ParseNode::NonTerminal(n, _, _)] => Err(TransformError::CstShouldStartWithGrammar(
                format!("Found '{n}' instead!"),
            )),
            _ => Err(TransformError::CstShouldOnlyHaveOneRoot(
                "Invalid root structure".into(),
            )),
        }
    }

    fn build_grammar_rules(&self, tokens: &[ParseNode]) -> Result<Arc<HashMap<String, Rule>>> {
        let (rules, refs) = tokens
            .iter()
            .skip(1)
            .take_while(|t| !matches!(t, ParseNode::NonTerminal(n, _, _) if n == EOF))
            .try_fold(
                (HashMap::new(), HashSet::new()),
                |(mut rules, mut refs), parse_node| {
                    let (name, expr, new_refs) = self.process_rule(parse_node)?;
                    rules.insert(name, expr);
                    refs.extend(new_refs);
                    Ok((rules, refs))
                },
            )?;

        let defined: HashSet<_> = rules.keys().cloned().collect();
        let undefined: Vec<_> = refs.difference(&defined).cloned().collect();
        if !undefined.is_empty() {
            return Err(TransformError::AmbiguousNonTerminal(format!(
                "Missing rules: [{}]!",
                undefined.join(", ")
            )));
        }
        Ok(Arc::new(rules))
    }

    fn process_rule(&self, parse_node: &ParseNode) -> Result<(String, Rule, HashSet<String>)> {
        let [id, arrow, expr] = Self::get_tokens(DEF, parse_node)?.as_slice() else {
            return Err(TransformError::WrongNumberOfTokens(
                "Definition needs 3 tokens".into(),
            ));
        };
        let name = self.extract_identifier(id)?;
        if !Self::is_token(ARROW, arrow) {
            return Err(TransformError::UnExpectedToken(ARROW.into()));
        }
        let (expr, refs) = self.convert_rule(expr)?;
        Ok((name, expr, refs))
    }

    fn extract_identifier(&self, parse_node: &ParseNode) -> Result<String> {
        let result = Self::get_tokens(IDENT, parse_node)?
            .iter()
            .take_while(|t| !Self::is_token(SPACING, t))
            .filter_map(|t| match t {
                ParseNode::Terminal(Span(s, e)) => Some(self.source[*s..*e].to_string()),
                _ => None,
            })
            .collect::<String>();
        if result.is_empty() {
            Err(TransformError::EmptyIdentifier)
        } else {
            Ok(result)
        }
    }

    fn convert_rule(&self, parse_node: &ParseNode) -> Result<(Rule, HashSet<String>)> {
        let chunks: Vec<_> = Self::get_tokens(EXPR, parse_node)?.chunks(2).collect();
        let (exprs, refs): (Vec<_>, HashSet<_>) =
            chunks
                .iter()
                .try_fold((vec![], HashSet::new()), |(mut exprs, mut refs), chunk| {
                    let (expr, new_refs) = self.convert_sequence(&chunk[0])?;
                    exprs.push(expr);
                    refs.extend(new_refs);
                    if chunk.get(1).map_or(false, |t| !Self::is_token(SLASH, t)) {
                        return Err(TransformError::UnExpectedToken(SLASH.into()));
                    }
                    Ok((exprs, refs))
                })?;
        Ok(match exprs.len() {
            0 => (Rule::Empty, refs),
            1 => (exprs[0].clone(), refs),
            _ => (Rule::Choice(exprs), refs),
        })
    }

    fn convert_sequence(&self, parse_node: &ParseNode) -> Result<(Rule, HashSet<String>)> {
        let (exprs, refs): (Vec<_>, HashSet<_>) = Self::get_tokens(SEQUENCE, parse_node)?
            .chunks(3)
            .take_while(|c| c.len() == 3)
            .try_fold((vec![], HashSet::new()), |(mut exprs, mut refs), chunk| {
                let (mut expr, new_refs) = self.convert_primary(&chunk[1])?;
                expr = self.apply_suffix(&chunk[2], expr)?;
                expr = self.apply_prefix(&chunk[0], expr)?;
                exprs.push(expr);
                refs.extend(new_refs);
                Ok((exprs, refs))
            })?;
        Ok(match exprs.len() {
            0 => (Rule::Empty, refs),
            1 => (exprs[0].clone(), refs),
            _ => (Rule::Sequence(exprs), refs),
        })
    }

    fn apply_prefix(&self, parse_node: &ParseNode, expr: Rule) -> Result<Rule> {
        Ok(match Self::get_tokens(PREFIX, parse_node)?.first() {
            Some(t) if Self::is_token(AND, t) => Rule::And(expr.boxed()),
            Some(t) if Self::is_token(NOT, t) => Rule::Not(expr.boxed()),
            Some(_) => {
                return Err(TransformError::UnExpectedToken(format!(
                    "Expected {AND}/{NOT}"
                )))
            }
            None => expr,
        })
    }

    fn apply_suffix(&self, parse_node: &ParseNode, expr: Rule) -> Result<Rule> {
        Ok(match Self::get_tokens(SUFFIX, parse_node)?.first() {
            Some(t) if Self::is_token(QUESTION, t) => Rule::Optional(expr.boxed()),
            Some(t) if Self::is_token(STAR, t) => Rule::ZeroOrMore(expr.boxed()),
            Some(t) if Self::is_token(PLUS, t) => Rule::OneOrMore(expr.boxed()),
            Some(_) => return Err(TransformError::UnExpectedToken("Invalid suffix".into())),
            None => expr,
        })
    }

    fn convert_primary(&self, parse_node: &ParseNode) -> Result<(Rule, HashSet<String>)> {
        let tokens = Self::get_tokens(PRIMARY, parse_node)?;
        match tokens.as_slice() {
            [open, expr, close] if Self::is_token(LPAR, open) && Self::is_token(RPAR, close) => {
                self.convert_rule(expr)
            }
            [t] => match Self::get_name(t)? {
                IDENT => {
                    let id = self.extract_identifier(t)?;
                    Ok((Rule::NonTerminal(id.clone()), HashSet::from([id])))
                }
                LITERAL => Ok((self.convert_literal(t)?, HashSet::new())),
                CLASS => Ok((self.convert_class(t)?, HashSet::new())),
                DOT => Ok((Rule::Any, HashSet::new())),
                _ => Err(TransformError::UnExpectedToken("Invalid primary".into())),
            },
            _ => Err(TransformError::WrongNumberOfTokens(
                "Primary needs 1 or 3 tokens".into(),
            )),
        }
    }

    fn unescape_char(&self, parse_node: &ParseNode) -> Result<String> {
        match parse_node {
            ParseNode::NonTerminal(name, Span(s, e), _) if name == CHAR => Ok(self.source[*s..*e]
                .replace("\\'", "'")
                .replace("\\\"", "\"")
                .replace("\\[", "[")
                .replace("\\]", "]")
                .replace("\\\\", "\\")),
            _ => Err(TransformError::UnExpectedToken(
                "Invalid char parse_node".into(),
            )),
        }
    }

    fn convert_literal(&self, parse_node: &ParseNode) -> Result<Rule> {
        let tokens = Self::get_tokens(LITERAL, parse_node)?;
        let content =
            tokens[1..tokens.len() - 2]
                .iter()
                .try_fold(String::new(), |mut acc, t| {
                    acc.push_str(&self.unescape_char(t)?);
                    Ok(acc)
                })?;
        Ok(Rule::Literal(content))
    }

    fn convert_class(&self, parse_node: &ParseNode) -> Result<Rule> {
        let tokens = Self::get_tokens(CLASS, parse_node)?;
        let (parts, symbols) = tokens[1..tokens.len() - 2].iter().try_fold(
            (vec![], HashSet::new()),
            |(mut parts, mut symbols), member| {
                for t in Self::get_tokens(CLASS_MEMBER, member)? {
                    if Self::is_token(CHAR, t) {
                        symbols.insert(self.unescape_char(t)?);
                    } else {
                        parts.push(self.convert_range(t)?);
                    }
                }
                Ok((parts, symbols))
            },
        )?;
        let mut all_parts = parts;
        if !symbols.is_empty() {
            all_parts.push(Rule::Class(symbols));
        }
        Ok(match all_parts.len() {
            0 => Rule::Empty,
            1 => all_parts.remove(0),
            _ => Rule::Choice(all_parts),
        })
    }

    fn convert_range(&self, parse_node: &ParseNode) -> Result<Rule> {
        let tokens = Self::get_tokens(RANGE, parse_node)?;
        if tokens.len() != 3 {
            return Err(TransformError::WrongNumberOfTokens(
                "Range needs 3 tokens".into(),
            ));
        }
        Ok(Rule::Range(
            self.unescape_char(&tokens[0])?,
            self.unescape_char(&tokens[2])?,
        ))
    }

    fn get_tokens<'a>(name: &str, parse_node: &'a ParseNode) -> Result<&'a Vec<ParseNode>> {
        match parse_node {
            ParseNode::NonTerminal(n, _, tokens) if n == name => Ok(tokens),
            ParseNode::NonTerminal(n, _, _) => Err(TransformError::UnExpectedToken(format!(
                "Expected {name}, got {n}"
            ))),
            _ => Err(TransformError::UnExpectedToken(format!("Expected {name}"))),
        }
    }

    fn is_token(name: &str, parse_node: &ParseNode) -> bool {
        matches!(parse_node, ParseNode::NonTerminal(n, _, _) if n == name)
    }

    fn get_name(parse_node: &ParseNode) -> Result<&str> {
        match parse_node {
            ParseNode::NonTerminal(name, _, _) => Ok(name),
            _ => Err(TransformError::UnExpectedToken(
                "Expected non-terminal".into(),
            )),
        }
    }
}
