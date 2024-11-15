use super::constants::*;
use super::errors::GrammarError;
use super::types::Peg;
use crate::peg::parsing::ParseResult;
use crate::peg::rule::Rule;
use crate::peg::transformer::Transformer;
use std::collections::HashMap;
use std::sync::Arc;

type GrammarResult<T> = Result<T, GrammarError>;

impl Peg {
    pub fn new(start: &str, grammar: &str) -> GrammarResult<Self> {
        let parser = Self::bootstrap();
        match parser.parse(grammar) {
            ParseResult(_, _, Ok(tokens)) => Transformer {
                source: grammar.trim(),
            }
            .build(start, tokens)
            .map_err(GrammarError::from),
            ParseResult(_, _, Err(e)) => Err(GrammarError::from(e)),
        }
    }

    pub fn parse(&self, input: &str) -> ParseResult {
        Rule::NonTerminal(self.start.clone()).parse(self, input, 0, 0)
    }

    fn bootstrap() -> Self {
        let mut grammar_builder = RuleBuilder::new();
        define_operators(&mut grammar_builder);
        define_character_rules(&mut grammar_builder);
        define_nonterminal_rules(&mut grammar_builder);
        define_grammar_rules(&mut grammar_builder);

        Self {
            start: TEXT.to_string(),
            rules: Arc::new(grammar_builder.rules),
        }
    }
}

struct RuleBuilder {
    rules: HashMap<String, Rule>,
}

impl RuleBuilder {
    fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    fn add_rule(&mut self, name: &str, expr: Rule) -> Rule {
        self.rules.insert(name.to_string(), expr.clone());
        Rule::NonTerminal(name.to_string())
    }

    fn seq(&self, exprs: Vec<Rule>) -> Rule {
        Rule::Sequence(exprs)
    }

    fn choice(&self, exprs: Vec<Rule>) -> Rule {
        if exprs.len() == 1 {
            exprs[0].clone()
        } else {
            Rule::Choice(exprs)
        }
    }

    fn zero_or_more(&self, expr: Rule) -> Rule {
        Rule::ZeroOrMore(expr.boxed())
    }

    fn one_or_more(&self, expr: Rule) -> Rule {
        Rule::OneOrMore(expr.boxed())
    }

    fn optional(&self, expr: Rule) -> Rule {
        Rule::Optional(expr.boxed())
    }

    fn not(&self, expr: Rule) -> Rule {
        Rule::Not(expr.boxed())
    }

    fn build_quoted_literal(&self, quote: &str, spacing: &Rule) -> Rule {
        self.seq(vec![
            Rule::Literal(quote.to_string()),
            self.zero_or_more(self.seq(vec![
                self.not(Rule::Literal(quote.to_string())),
                Rule::NonTerminal(CHAR.to_string()),
            ])),
            Rule::Literal(quote.to_string()),
            spacing.clone(),
        ])
    }
}

fn define_operators(gb: &mut RuleBuilder) {
    let spacing = gb.add_rule(
        SPACING,
        gb.zero_or_more(gb.choice(vec![
            Rule::Literal(" ".to_string()),
            Rule::Literal("\n".to_string()),
            Rule::Literal("\r\n".to_string()),
        ])),
    );

    let operators = [
        (ARROW, "<-"),
        (SLASH, "/"),
        (AND, "&"),
        (NOT, "!"),
        (QUESTION, "?"),
        (STAR, "*"),
        (PLUS, "+"),
        (LPAR, "("),
        (RPAR, ")"),
        (DOT, "."),
    ];

    for (name, symbol) in operators {
        gb.add_rule(
            name,
            gb.seq(vec![Rule::Literal(symbol.to_string()), spacing.clone()]),
        );
    }

    gb.add_rule(EOF, gb.not(Rule::Any));
}

fn define_character_rules(gb: &mut RuleBuilder) {
    let char_rule = gb.choice(vec![
        gb.seq(vec![
            Rule::Literal("\\".to_string()),
            Rule::create_character_class(&["n", "r", "t", "'", "\"", "[", "]", "\\"]),
        ]),
        gb.seq(vec![
            Rule::Literal("\\".to_string()),
            gb.one_or_more(Rule::Range("0".to_string(), "9".to_string())),
            Rule::Literal(";".to_string()),
        ]),
        gb.seq(vec![gb.not(Rule::Literal("\\".to_string())), Rule::Any]),
    ]);
    gb.add_rule(CHAR, char_rule);

    let range = gb.add_rule(
        RANGE,
        gb.seq(vec![
            Rule::NonTerminal(CHAR.to_string()),
            Rule::Literal("-".to_string()),
            Rule::NonTerminal(CHAR.to_string()),
        ]),
    );

    gb.add_rule(
        CLASS_MEMBER,
        gb.choice(vec![range.clone(), Rule::NonTerminal(CHAR.to_string())]),
    );
}

fn define_nonterminal_rules(gb: &mut RuleBuilder) {
    let spacing = Rule::NonTerminal(SPACING.to_string());
    let identifier = define_identifier_rule(gb, &spacing);
    let literal = define_literal_rule(gb, &spacing);
    let class = define_class_rule(gb);

    let primary = gb.add_rule(
        PRIMARY,
        gb.choice(vec![
            gb.seq(vec![
                identifier.clone(),
                gb.not(Rule::NonTerminal(ARROW.to_string())),
            ]),
            gb.seq(vec![
                Rule::NonTerminal(LPAR.to_string()),
                Rule::NonTerminal(EXPR.to_string()),
                Rule::NonTerminal(RPAR.to_string()),
            ]),
            literal.clone(),
            class.clone(),
            Rule::NonTerminal(DOT.to_string()),
        ]),
    );

    let suffix = gb.add_rule(
        SUFFIX,
        gb.optional(gb.choice(vec![
            Rule::NonTerminal(QUESTION.to_string()),
            Rule::NonTerminal(STAR.to_string()),
            Rule::NonTerminal(PLUS.to_string()),
        ])),
    );

    let prefix = gb.add_rule(
        PREFIX,
        gb.optional(gb.choice(vec![
            Rule::NonTerminal(AND.to_string()),
            Rule::NonTerminal(NOT.to_string()),
        ])),
    );

    let sequence = gb.add_rule(
        SEQUENCE,
        gb.zero_or_more(gb.seq(vec![prefix.clone(), primary.clone(), suffix.clone()])),
    );

    gb.add_rule(
        EXPR,
        gb.seq(vec![
            sequence.clone(),
            gb.zero_or_more(gb.seq(vec![Rule::NonTerminal(SLASH.to_string()), sequence.clone()])),
        ]),
    );
}

fn define_grammar_rules(gb: &mut RuleBuilder) {
    let spacing = Rule::NonTerminal(SPACING.to_string());
    let definition = gb.add_rule(
        DEF,
        gb.seq(vec![
            Rule::NonTerminal(IDENT.to_string()),
            Rule::NonTerminal(ARROW.to_string()),
            Rule::NonTerminal(EXPR.to_string()),
        ]),
    );

    gb.add_rule(
        TEXT,
        gb.seq(vec![
            spacing.clone(),
            gb.zero_or_more(definition.clone()),
            Rule::NonTerminal(EOF.to_string()),
        ]),
    );
}

fn define_identifier_rule(gb: &mut RuleBuilder, spacing: &Rule) -> Rule {
    gb.add_rule(
        IDENT,
        gb.seq(vec![
            gb.choice(vec![
                Rule::Range("a".to_string(), "z".to_string()),
                Rule::Range("A".to_string(), "Z".to_string()),
                Rule::Literal("_".to_string()),
            ]),
            gb.zero_or_more(gb.choice(vec![
                Rule::Range("a".to_string(), "z".to_string()),
                Rule::Range("A".to_string(), "Z".to_string()),
                Rule::Literal("_".to_string()),
                Rule::Range("0".to_string(), "9".to_string()),
            ])),
            spacing.clone(),
        ]),
    )
}

fn define_literal_rule(gb: &mut RuleBuilder, spacing: &Rule) -> Rule {
    let single_quoted = gb.build_quoted_literal("'", spacing);
    let double_quoted = gb.build_quoted_literal("\"", spacing);

    gb.add_rule(LITERAL, gb.choice(vec![single_quoted, double_quoted]))
}

fn define_class_rule(gb: &mut RuleBuilder) -> Rule {
    let spacing = Rule::NonTerminal(SPACING.to_string());
    gb.add_rule(
        CLASS,
        gb.seq(vec![
            Rule::Literal("[".to_string()),
            gb.zero_or_more(gb.seq(vec![
                gb.not(Rule::Literal("]".to_string())),
                Rule::NonTerminal(CLASS_MEMBER.to_string()),
            ])),
            Rule::Literal("]".to_string()),
            spacing.clone(),
        ]),
    )
}
