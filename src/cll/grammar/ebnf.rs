//! CLL Ch.21 EBNF → CFG expander.
//!
//! Operators (CLL Ch.21 notes):
//! - juxtaposition = concat
//! - `|` alternation (loosest)
//! - `&` and/or: `A & B` ⇒ `A | B | A B` (binds tighter than `|`)
//! - `...` repetition of construct to the left ⇒ one-or-more; `[X] ...` ⇒ `X*`
//! - `[]` optional
//! - `()` grouping
//! - `#` ⇒ `[free ...]` i.e. `free*`
//! - `/elidable/` ⇒ token-or-epsilon recognition superset with omission provenance.
//!   Chapter 21's YACC gap rules use parser error recovery, so this conversion
//!   alone does not encode the permitted tree relation.
//! - `/TERM#/` ⇒ `/TERM/` then `#` (free* stays outside the elision; CLL typesetting)

use super::cfg::{ElidableSite, NtId, Production, Symbol, TermId, TokenGrammar};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Debug, Clone)]
enum Expr {
    Term(String),
    Nt(String),
    /// `#` free-modifier slot
    Hash,
    Seq(Vec<Expr>),
    Alt(Vec<Expr>),
    /// A & B & …
    And(Vec<Expr>),
    Opt(Box<Expr>),
    /// one or more (from `...`)
    Plus(Box<Expr>),
    /// Elidable `/…/`: permissive optional recognition with omission metadata.
    Elidable(Box<Expr>),
}

#[derive(Debug)]
pub struct CompileStats {
    pub source_rules: usize,
    pub cfg_productions: usize,
    pub nonterminals: usize,
    pub terminals: usize,
    pub and_expansions: usize,
    pub hash_sites: usize,
    pub elidable_sites: usize,
    pub plus_sites: usize,
}

struct Compiler {
    nt_names: Vec<String>,
    nt_index: HashMap<String, NtId>,
    term_names: Vec<String>,
    term_index: HashMap<String, TermId>,
    productions: Vec<Production>,
    fresh: usize,
    and_expansions: usize,
    hash_sites: usize,
    elidable_sites: usize,
    plus_sites: usize,
    /// Shared free* NT once created
    free_star: Option<NtId>,
    elidables: HashMap<usize, ElidableSite>,
    source_rule: String,
}

impl Compiler {
    fn new() -> Self {
        Self {
            nt_names: Vec::new(),
            nt_index: HashMap::new(),
            term_names: Vec::new(),
            term_index: HashMap::new(),
            productions: Vec::new(),
            fresh: 0,
            and_expansions: 0,
            hash_sites: 0,
            elidable_sites: 0,
            plus_sites: 0,
            free_star: None,
            elidables: HashMap::new(),
            source_rule: String::new(),
        }
    }

    fn intern_nt(&mut self, name: &str) -> NtId {
        if let Some(&id) = self.nt_index.get(name) {
            return id;
        }
        let id = self.nt_names.len() as NtId;
        self.nt_names.push(name.to_string());
        self.nt_index.insert(name.to_string(), id);
        id
    }

    fn intern_term(&mut self, name: &str) -> TermId {
        if let Some(&id) = self.term_index.get(name) {
            return id;
        }
        let id = self.term_names.len() as TermId;
        self.term_names.push(name.to_string());
        self.term_index.insert(name.to_string(), id);
        id
    }

    fn fresh_nt(&mut self, hint: &str) -> NtId {
        let name = format!("@{hint}_{}", self.fresh);
        self.fresh += 1;
        self.intern_nt(&name)
    }

    fn add_prod(&mut self, lhs: NtId, rhs: Vec<Symbol>) {
        self.productions.push(Production { lhs, rhs });
    }

    fn ensure_free_star(&mut self) -> NtId {
        if let Some(id) = self.free_star {
            return id;
        }
        let free = self.intern_nt("free");
        let star = self.intern_nt("@free_star");
        // free* ::= ε | free free*
        self.add_prod(star, vec![]);
        self.add_prod(star, vec![Symbol::Nt(free), Symbol::Nt(star)]);
        self.free_star = Some(star);
        star
    }

    /// Compile expr to a sequence of symbols (possibly introducing NTs).
    fn compile_expr(&mut self, e: &Expr) -> Vec<Symbol> {
        match e {
            Expr::Term(t) => vec![Symbol::Term(self.intern_term(t))],
            Expr::Nt(n) => vec![Symbol::Nt(self.intern_nt(n))],
            Expr::Hash => {
                self.hash_sites += 1;
                vec![Symbol::Nt(self.ensure_free_star())]
            }
            Expr::Seq(xs) => {
                let mut out = Vec::new();
                for x in xs {
                    out.extend(self.compile_expr(x));
                }
                out
            }
            Expr::Alt(xs) => {
                let nt = self.fresh_nt("alt");
                for x in xs {
                    let rhs = self.compile_expr(x);
                    self.add_prod(nt, rhs);
                }
                vec![Symbol::Nt(nt)]
            }
            Expr::And(xs) => {
                // Small selections are cheaper to enumerate. Larger ones use
                // a linear-size CFG rather than 2^k - 1 alternatives.
                self.and_expansions += 1;
                let parts: Vec<Vec<Symbol>> = xs.iter().map(|x| self.compile_expr(x)).collect();
                if parts.len() <= 4 {
                    let nt = self.fresh_nt("and");
                    for mask in 1..(1u32 << parts.len()) {
                        let mut rhs = Vec::new();
                        for (i, part) in parts.iter().enumerate() {
                            if mask & (1 << i) != 0 {
                                rhs.extend(part.iter().copied());
                            }
                        }
                        self.add_prod(nt, rhs);
                    }
                    vec![Symbol::Nt(nt)]
                } else {
                    let mut optional_suffix = self.fresh_nt("and_optional");
                    self.add_prod(optional_suffix, vec![]);
                    self.add_prod(optional_suffix, parts.last().unwrap().clone());
                    let mut nonempty_suffix = self.fresh_nt("and_nonempty");
                    self.add_prod(nonempty_suffix, parts.last().unwrap().clone());
                    for part in parts[..parts.len() - 1].iter().rev() {
                        let optional = self.fresh_nt("and_optional");
                        self.add_prod(optional, vec![Symbol::Nt(optional_suffix)]);
                        let mut take = part.clone();
                        take.push(Symbol::Nt(optional_suffix));
                        self.add_prod(optional, take.clone());

                        let nonempty = self.fresh_nt("and_nonempty");
                        self.add_prod(nonempty, take);
                        self.add_prod(nonempty, vec![Symbol::Nt(nonempty_suffix)]);
                        optional_suffix = optional;
                        nonempty_suffix = nonempty;
                    }
                    vec![Symbol::Nt(nonempty_suffix)]
                }
            }
            Expr::Opt(inner) => {
                let nt = self.fresh_nt("opt");
                let rhs = self.compile_expr(inner);
                self.add_prod(nt, vec![]);
                self.add_prod(nt, rhs);
                vec![Symbol::Nt(nt)]
            }
            Expr::Plus(inner) => {
                self.plus_sites += 1;
                if let Expr::Opt(body) = inner.as_ref() {
                    // (ε | X)+ has the language X*. Avoid empty iterations
                    // caused solely by the optional wrapper.
                    let nt = self.fresh_nt("star");
                    let x = self.compile_expr(body);
                    self.add_prod(nt, vec![]);
                    let mut rec = vec![Symbol::Nt(nt)];
                    rec.extend(x);
                    self.add_prod(nt, rec);
                    vec![Symbol::Nt(nt)]
                } else {
                    let nt = self.fresh_nt("plus");
                    let x = self.compile_expr(inner);
                    self.add_prod(nt, x.clone());
                    let mut rec = vec![Symbol::Nt(nt)];
                    rec.extend(x);
                    self.add_prod(nt, rec);
                    vec![Symbol::Nt(nt)]
                }
            }
            Expr::Elidable(inner) => {
                self.elidable_sites += 1;
                let nt = self.fresh_nt("elidable");
                let rhs = self.compile_expr(inner);
                let terminal = rhs
                    .iter()
                    .find_map(|s| match s {
                        Symbol::Term(t) => Some(self.term_names[*t as usize].clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "elidable".into());
                self.elidables.insert(
                    self.productions.len(),
                    ElidableSite {
                        terminal,
                        source_rule: self.source_rule.clone(),
                    },
                );
                self.add_prod(nt, vec![]);
                self.add_prod(nt, rhs);
                vec![Symbol::Nt(nt)]
            }
        }
    }
}

/// Parse and compile CLL Ch.21 EBNF text into a [`TokenGrammar`].
pub fn compile_ebnf(src: &str, start: &str) -> Result<(TokenGrammar, CompileStats), String> {
    let mut rules: Vec<(String, Expr)> = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, expr)) = line.split_once('=') else {
            return Err(format!("line {}: expected name = expr", lineno + 1));
        };
        let name = name.trim().to_string();
        let expr = parse_expr(expr.trim()).map_err(|e| format!("line {}: {e}", lineno + 1))?;
        rules.push((name, expr));
    }

    let source_rules = rules.len();
    let mut compiler = Compiler::new();

    // Pre-intern all LHS names so mutual recursion works.
    for (name, _) in &rules {
        compiler.intern_nt(name);
    }
    let source_nts: HashSet<NtId> = rules
        .iter()
        .map(|(name, _)| compiler.nt_index[name])
        .collect();
    // Special NTs for informal terminals
    let any_word = compiler.intern_nt("any-word");
    let anything = compiler.intern_nt("anything");

    for (name, expr) in &rules {
        compiler.source_rule = name.clone();
        let lhs = compiler.intern_nt(name);
        // Top-level alt: if Alt, each branch is a production; else one prod.
        match expr {
            Expr::Alt(expressions) => {
                for expression in expressions {
                    let rhs = compiler.compile_expr(expression);
                    compiler.add_prod(lhs, rhs);
                }
            }
            other => {
                let rhs = compiler.compile_expr(other);
                compiler.add_prod(lhs, rhs);
            }
        }
    }

    // any-word ::= BRIVLA | CMENE | <every selma'o terminal>
    // anything ::= FOREIGN — the ZOI / LA'O body arrives from the pre-lex
    // quote step as one opaque lexeme (CLL 19.10); it is deliberately *not*
    // an any-word, so it can only appear between the quote delimiters.
    let brivla = compiler.intern_term("BRIVLA");
    let cmevla = compiler.intern_term("CMENE");
    compiler.add_prod(any_word, vec![Symbol::Term(brivla)]);
    compiler.add_prod(any_word, vec![Symbol::Term(cmevla)]);
    // Collect selma'o-like terms already interned + ensure common ones exist
    let term_names_snapshot: Vec<String> = compiler.term_names.clone();
    for term_name in &term_names_snapshot {
        if term_name == "BRIVLA" || term_name == "CMENE" {
            continue;
        }
        let tid = compiler.intern_term(term_name);
        compiler.add_prod(any_word, vec![Symbol::Term(tid)]);
    }
    let foreign = compiler.intern_term(super::lexeme::FOREIGN_TERMINAL);
    compiler.add_prod(anything, vec![Symbol::Term(foreign)]);

    // Ensure free exists (may be empty if # never used before free rule — but free is in source)
    let _ = compiler.intern_nt("free");
    let _ = compiler.ensure_free_star();

    let stats = CompileStats {
        source_rules,
        cfg_productions: compiler.productions.len(),
        nonterminals: compiler.nt_names.len(),
        terminals: compiler.term_names.len(),
        and_expansions: compiler.and_expansions,
        hash_sites: compiler.hash_sites,
        elidable_sites: compiler.elidable_sites,
        plus_sites: compiler.plus_sites,
    };

    let mut grammar = TokenGrammar::from_parts(
        compiler.nt_names,
        compiler.term_names,
        compiler.productions,
        start,
    );
    grammar.source_nts = source_nts;
    grammar.elidables = compiler.elidables;
    Ok((grammar, stats))
}

// --- EBNF expression parser ---

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok {
    Name(String), // lowercase NT or UPPER terminal
    Hash,         // #
    LBrack,
    RBrack,
    LParen,
    RParen,
    Pipe,
    Amp,
    Dots,             // ...
    Elidable(String), // raw inner of /.../
}

fn tokenize(input: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        if character.is_whitespace() {
            index += 1;
            continue;
        }
        match character {
            '#' => {
                out.push(Tok::Hash);
                index += 1;
            }
            '[' => {
                out.push(Tok::LBrack);
                index += 1;
            }
            ']' => {
                out.push(Tok::RBrack);
                index += 1;
            }
            '(' => {
                out.push(Tok::LParen);
                index += 1;
            }
            ')' => {
                out.push(Tok::RParen);
                index += 1;
            }
            '|' => {
                out.push(Tok::Pipe);
                index += 1;
            }
            '&' => {
                out.push(Tok::Amp);
                index += 1;
            }
            '/' => {
                // elidable: /.../  (inner may contain # and names)
                index += 1;
                let start = index;
                while index < chars.len() && chars[index] != '/' {
                    index += 1;
                }
                if index >= chars.len() {
                    return Err("unterminated elidable /.../".into());
                }
                let inner: String = chars[start..index].iter().collect();
                out.push(Tok::Elidable(inner));
                index += 1; // closing /
            }
            '.' if index + 2 < chars.len()
                && chars[index + 1] == '.'
                && chars[index + 2] == '.' =>
            {
                out.push(Tok::Dots);
                index += 3;
            }
            _ if character.is_ascii_alphanumeric() || character == '-' => {
                let start = index;
                index += 1;
                while index < chars.len()
                    && (chars[index].is_ascii_alphanumeric()
                        || chars[index] == '-'
                        || chars[index] == '\'')
                {
                    index += 1;
                }
                let name: String = chars[start..index].iter().collect();
                out.push(Tok::Name(name));
            }
            _ => return Err(format!("unexpected char {character:?} at {index}")),
        }
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    index: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.index)
    }
    fn bump(&mut self) -> Option<Tok> {
        if self.index < self.toks.len() {
            let token = self.toks[self.index].clone();
            self.index += 1;
            Some(token)
        } else {
            None
        }
    }
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_alt()
    }
    fn parse_alt(&mut self) -> Result<Expr, String> {
        let mut expressions = vec![self.parse_and()?];
        while matches!(self.peek(), Some(Tok::Pipe)) {
            self.bump();
            expressions.push(self.parse_and()?);
        }
        if expressions.len() == 1 {
            Ok(expressions.pop().unwrap())
        } else {
            Ok(Expr::Alt(expressions))
        }
    }
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut expressions = vec![self.parse_concat()?];
        while matches!(self.peek(), Some(Tok::Amp)) {
            self.bump();
            expressions.push(self.parse_concat()?);
        }
        if expressions.len() == 1 {
            Ok(expressions.pop().unwrap())
        } else {
            Ok(Expr::And(expressions))
        }
    }
    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut expressions = Vec::new();
        while self.starts_atom() {
            expressions.push(self.parse_postfix()?);
        }
        if expressions.is_empty() {
            return Err("expected expression".into());
        }
        if expressions.len() == 1 {
            Ok(expressions.pop().unwrap())
        } else {
            Ok(Expr::Seq(expressions))
        }
    }
    fn starts_atom(&self) -> bool {
        matches!(
            self.peek(),
            Some(Tok::Name(_) | Tok::Hash | Tok::LBrack | Tok::LParen | Tok::Elidable(_))
        )
    }
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expression = self.parse_atom()?;
        if matches!(self.peek(), Some(Tok::Dots)) {
            self.bump();
            expression = Expr::Plus(Box::new(expression));
        }
        Ok(expression)
    }
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Some(Tok::Name(name)) => {
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_uppercase())
                    .unwrap_or(false)
                {
                    Ok(Expr::Term(name))
                } else {
                    Ok(Expr::Nt(name))
                }
            }
            Some(Tok::Hash) => Ok(Expr::Hash),
            Some(Tok::LBrack) => {
                let inner = self.parse_expr()?;
                match self.bump() {
                    Some(Tok::RBrack) => Ok(Expr::Opt(Box::new(inner))),
                    _ => Err("expected ]".into()),
                }
            }
            Some(Tok::LParen) => {
                let inner = self.parse_expr()?;
                match self.bump() {
                    Some(Tok::RParen) => Ok(inner),
                    _ => Err("expected )".into()),
                }
            }
            Some(Tok::Elidable(inner)) => {
                // CLL writes `/BOI#/` meaning elidable BOI then `#` (= free*).
                // The `#` is typeset inside the slashes but is NOT part of what
                // may be omitted — otherwise `xy. xi ky.` (lerfu + free, no BOI)
                // and `li re xi pa` cannot attach free after an elided terminator.
                // Rewrite `/α#/` → `/α/` `#` (= Opt(α) · free*).
                let trimmed = inner.trim_end();
                if let Some(rest) = trimmed.strip_suffix('#') {
                    let rest = rest.trim_end();
                    if rest.is_empty() {
                        Ok(Expr::Hash)
                    } else {
                        let sub = parse_expr(rest)?;
                        Ok(Expr::Seq(vec![Expr::Elidable(Box::new(sub)), Expr::Hash]))
                    }
                } else {
                    let sub = parse_expr(&inner)?;
                    Ok(Expr::Elidable(Box::new(sub)))
                }
            }
            other => Err(format!("unexpected token {other:?}")),
        }
    }
}

fn parse_expr(input: &str) -> Result<Expr, String> {
    let toks = tokenize(input)?;
    let mut parser = Parser { toks, index: 0 };
    let expression = parser.parse_expr()?;
    if parser.index != parser.toks.len() {
        return Err(format!("trailing tokens starting at {:?}", parser.peek()));
    }
    Ok(expression)
}

/// Terminal names referenced by a compiled grammar (selma'o + BRIVLA/CMENE).
pub fn terminal_set(grammar: &TokenGrammar) -> BTreeSet<String> {
    grammar.term_names.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::grammar::cll21_grammar;
    use crate::cll::grammar::earley::recognizes;

    fn zero_width_analysis(grammar: &TokenGrammar) -> (Vec<NtId>, Vec<bool>) {
        let mut nullable = vec![false; grammar.n_nts()];
        loop {
            let mut changed = false;
            for p in &grammar.productions {
                if !nullable[p.lhs as usize]
                    && p.rhs
                        .iter()
                        .all(|s| matches!(s, Symbol::Nt(nt) if nullable[*nt as usize]))
                {
                    nullable[p.lhs as usize] = true;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let mut edges = vec![Vec::new(); grammar.n_nts()];
        for p in &grammar.productions {
            for (index, s) in p.rhs.iter().enumerate() {
                if let Symbol::Nt(nt) = s {
                    if p.rhs.iter().enumerate().all(|(i, other)| {
                        i == index || matches!(other, Symbol::Nt(n) if nullable[*n as usize])
                    }) {
                        edges[p.lhs as usize].push(*nt as usize);
                    }
                }
            }
        }
        fn visit(node: usize, edges: &[Vec<usize>], states: &mut [u8], cycles: &mut Vec<usize>) {
            states[node] = 1;
            for &next in &edges[node] {
                if states[next] == 1 {
                    cycles.push(next);
                } else if states[next] == 0 {
                    visit(next, edges, states, cycles);
                }
            }
            states[node] = 2;
        }
        let mut states = vec![0; grammar.n_nts()];
        let mut cycles = Vec::new();
        for node in 0..grammar.n_nts() {
            if states[node] == 0 {
                visit(node, &edges, &mut states, &mut cycles);
            }
        }
        (cycles.into_iter().map(|id| id as NtId).collect(), nullable)
    }

    #[test]
    fn cll_cfg_has_no_zero_width_recursion() {
        let grammar = cll21_grammar();
        let (cycles, nullable) = zero_width_analysis(grammar);
        assert!(cycles.is_empty());
        for (id, name) in grammar.nt_names.iter().enumerate() {
            if name.starts_with("@opt_") {
                assert!(
                    !grammar.productions.iter().any(|p| {
                        p.lhs as usize == id
                            && !p.rhs.is_empty()
                            && p.rhs
                                .iter()
                                .all(|s| matches!(s, Symbol::Nt(nt) if nullable[*nt as usize]))
                    }),
                    "redundant nullable optional helper {name}"
                );
            }
        }
        let (cyclic, _) = compile_ebnf("s = [s] | A\n", "s").unwrap();
        assert!(!zero_width_analysis(&cyclic).0.is_empty());
    }

    #[test]
    fn parse_simple_alt_and_optional() {
        let expression = parse_expr("A | [B] C").unwrap();
        assert!(matches!(expression, Expr::Alt(_)));
    }

    #[test]
    fn and_accepts_each_nonempty_ordered_selection() {
        let (grammar, st) = compile_ebnf("x = A & B\n", "x").unwrap();
        assert_eq!(st.and_expansions, 1);
        for words in [&["A"][..], &["B"][..], &["A", "B"][..]] {
            let input: Vec<_> = words
                .iter()
                .map(|word| grammar.term(word).unwrap())
                .collect();
            assert!(recognizes(&grammar, grammar.start, &input));
        }
        assert!(!recognizes(&grammar, grammar.start, &[]));
        assert!(!recognizes(
            &grammar,
            grammar.start,
            &[grammar.term("B").unwrap(), grammar.term("A").unwrap()]
        ));
    }

    #[test]
    fn and_keeps_nullable_operands_without_subset_expansion() {
        let (grammar, _) = compile_ebnf("x = [A] & B & [C] & D & [E]\n", "x").unwrap();
        for words in [&[][..], &["B"][..], &["A", "B", "C", "D", "E"][..]] {
            let input: Vec<_> = words
                .iter()
                .map(|word| grammar.term(word).unwrap())
                .collect();
            assert!(recognizes(&grammar, grammar.start, &input));
        }
    }

    #[test]
    fn factored_and_accepts_all_subsequences() {
        let (grammar, _) = compile_ebnf("x = A & B & C & D & E\n", "x").unwrap();
        for mask in 1..32 {
            let input: Vec<_> = (0..5)
                .filter(|i| mask & (1 << i) != 0)
                .map(|i| grammar.term(["A", "B", "C", "D", "E"][i]).unwrap())
                .collect();
            assert!(recognizes(&grammar, grammar.start, &input), "mask={mask}");
        }
        assert!(!recognizes(&grammar, grammar.start, &[]));
        assert!(!recognizes(
            &grammar,
            grammar.start,
            &[grammar.term("B").unwrap(), grammar.term("A").unwrap()]
        ));
    }

    #[test]
    fn repeated_optional_item_is_factored_as_star() {
        let (grammar, stats) = compile_ebnf("x = [A] ... B\n", "x").unwrap();
        assert_eq!(stats.plus_sites, 1);
        assert!(grammar
            .nt_names
            .iter()
            .any(|name| name.starts_with("@star_")));
        assert!(!grammar
            .nt_names
            .iter()
            .any(|name| name.starts_with("@opt_")));
        for words in [&["B"][..], &["A", "B"][..], &["A", "A", "B"][..]] {
            let input: Vec<_> = words
                .iter()
                .map(|word| grammar.term(word).unwrap())
                .collect();
            assert!(recognizes(&grammar, grammar.start, &input));
        }
        assert!(!recognizes(&grammar, grammar.start, &[]));
    }

    #[test]
    fn and_compilation_grows_linearly() {
        fn size(n: usize) -> usize {
            let operands: Vec<_> = (0..n).map(|i| format!("A{i}")).collect();
            let source = format!("x = {}\n", operands.join(" & "));
            compile_ebnf(&source, "x").unwrap().0.n_productions()
        }
        assert_eq!(size(24) - size(12), 12 * 5);
    }
}
