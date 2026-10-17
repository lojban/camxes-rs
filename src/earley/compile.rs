//! camxes-rs PEG text → CFG with lookahead symbols.

use super::{ByteSet, EarleyGrammar, NtKind, Prod, Quote, Sym};
use crate::peg::annotations::{strip_annotations, Annotation};
use crate::peg::grammar::Peg;
use crate::peg::rule::Rule;
use std::collections::HashMap;
use std::fmt;

/// Grammar text could not be compiled.
#[derive(Debug, Clone)]
pub struct CompileError(pub String);

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "earley grammar compile error: {}", self.0)
    }
}

impl std::error::Error for CompileError {}

struct Builder {
    names: Vec<String>,
    kinds: Vec<NtKind>,
    prods: Vec<Prod>,
    named: HashMap<String, u32>,
}

impl Builder {
    fn named(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.named.get(name) {
            return id;
        }
        let id = self.new_nt(name.to_string(), NtKind::Named);
        self.named.insert(name.to_string(), id);
        id
    }

    fn new_nt(&mut self, name: String, kind: NtKind) -> u32 {
        let id = self.names.len() as u32;
        self.names.push(name);
        self.kinds.push(kind);
        id
    }

    fn helper(&mut self, kind: NtKind) -> u32 {
        let n = self.names.len();
        self.new_nt(format!("#{n}"), kind)
    }

    fn prod(&mut self, lhs: u32, rhs: Vec<Sym>) {
        self.prods.push(Prod { lhs, rhs });
    }

    fn bytes_nt(&mut self, alternatives: Vec<Vec<u8>>, kind: NtKind) -> u32 {
        let h = self.helper(kind);
        for alt in alternatives {
            let rhs = alt
                .into_iter()
                .map(|b| {
                    let mut s = ByteSet::EMPTY;
                    s.insert(b);
                    Sym::T(s)
                })
                .collect();
            self.prod(h, rhs);
        }
        h
    }

    /// Right-hand-side symbols for a rule used inside a sequence.
    fn seq_syms(&mut self, rule: &Rule, out: &mut Vec<Sym>) {
        match rule {
            Rule::Sequence(items) => {
                for r in items {
                    self.seq_syms(r, out);
                }
            }
            Rule::Group(inner) => self.seq_syms(inner, out),
            Rule::Empty => {}
            other => out.push(self.sym(other)),
        }
    }

    fn nt_of(&mut self, rule: &Rule) -> u32 {
        match self.sym(rule) {
            Sym::N(id) => id,
            s => {
                let h = self.helper(NtKind::Helper);
                self.prod(h, vec![s]);
                h
            }
        }
    }

    fn sym(&mut self, rule: &Rule) -> Sym {
        match rule {
            Rule::NonTerminal(name) => Sym::N(self.named(name)),
            Rule::Any => Sym::T(ByteSet::ALL),
            Rule::Literal(s) => {
                let b = s.as_bytes();
                if b.len() == 1 {
                    let mut set = ByteSet::EMPTY;
                    set.insert(b[0]);
                    Sym::T(set)
                } else {
                    Sym::N(self.bytes_nt(vec![b.to_vec()], NtKind::Literal))
                }
            }
            Rule::Range(a, b) => {
                let (a, b) = (a.as_bytes(), b.as_bytes());
                let mut set = ByteSet::EMPTY;
                if let (Some(&lo), Some(&hi)) = (a.first(), b.first()) {
                    for x in lo..=hi {
                        set.insert(x);
                    }
                }
                Sym::T(set)
            }
            Rule::Class(members) => {
                if members.iter().all(|m| m.len() == 1) {
                    let mut set = ByteSet::EMPTY;
                    for m in members {
                        set.insert(m.as_bytes()[0]);
                    }
                    Sym::T(set)
                } else {
                    // PEG tries longer members first.
                    let mut alts: Vec<Vec<u8>> =
                        members.iter().map(|m| m.as_bytes().to_vec()).collect();
                    alts.sort_by(|x, y| y.len().cmp(&x.len()).then(x.cmp(y)));
                    Sym::N(self.bytes_nt(alts, NtKind::Literal))
                }
            }
            Rule::Group(inner) => self.sym(inner),
            Rule::Empty => {
                let h = self.helper(NtKind::Helper);
                self.prod(h, vec![]);
                Sym::N(h)
            }
            Rule::Sequence(_) => {
                let h = self.helper(NtKind::Helper);
                let mut rhs = Vec::new();
                self.seq_syms(rule, &mut rhs);
                self.prod(h, rhs);
                Sym::N(h)
            }
            Rule::Choice(alts) => {
                let h = self.helper(NtKind::Helper);
                for alt in alts {
                    let mut rhs = Vec::new();
                    self.seq_syms(alt, &mut rhs);
                    self.prod(h, rhs);
                }
                Sym::N(h)
            }
            Rule::ZeroOrMore(inner) => Sym::N(self.star(inner)),
            Rule::OneOrMore(inner) => {
                let x = self.sym(inner);
                let star = self.star_of(x);
                let h = self.helper(NtKind::Helper);
                self.prod(h, vec![x, Sym::N(star)]);
                Sym::N(h)
            }
            Rule::Optional(inner) => {
                let x = self.sym(inner);
                let h = self.helper(NtKind::Helper);
                self.prod(h, vec![x]);
                self.prod(h, vec![]);
                Sym::N(h)
            }
            Rule::And(inner) => Sym::And(self.nt_of(inner)),
            Rule::Not(inner) => Sym::Not(self.nt_of(inner)),
        }
    }

    fn star(&mut self, inner: &Rule) -> u32 {
        let x = self.sym(inner);
        self.star_of(x)
    }

    /// `H -> x H | ε` (greedy alternative first).
    fn star_of(&mut self, x: Sym) -> u32 {
        let h = self.helper(NtKind::Helper);
        self.prod(h, vec![x, Sym::N(h)]);
        self.prod(h, vec![]);
        h
    }
}

pub(super) fn compile(start: &str, grammar: &str) -> Result<EarleyGrammar, CompileError> {
    // Reuse the PEG front end (bootstrap grammar + transformer) so both
    // engines accept identical grammar text.
    let (plain, annotations) = strip_annotations(grammar);
    let peg = Peg::new(start, &plain).map_err(|e| CompileError(e.to_string()))?;
    let mut b = Builder {
        names: Vec::new(),
        kinds: Vec::new(),
        prods: Vec::new(),
        named: HashMap::new(),
    };
    let start_id = b.named(start);
    let mut rule_names: Vec<&String> = peg.rules.keys().collect();
    rule_names.sort();
    for name in rule_names {
        let lhs = b.named(name);
        let body = &peg.rules[name];
        match body {
            Rule::Choice(alts) => {
                for alt in alts {
                    let mut rhs = Vec::new();
                    b.seq_syms(alt, &mut rhs);
                    b.prod(lhs, rhs);
                }
            }
            other => {
                let mut rhs = Vec::new();
                b.seq_syms(other, &mut rhs);
                b.prod(lhs, rhs);
            }
        }
    }
    let quotes = compile_quotes(&mut b, &annotations)?;
    let mut by_lhs = vec![Vec::new(); b.names.len()];
    for (i, p) in b.prods.iter().enumerate() {
        by_lhs[p.lhs as usize].push(i as u32);
    }
    let (nt_first, nt_nullable, prod_first, prod_nullable) =
        first_sets(b.names.len(), &b.prods, &quotes);
    Ok(EarleyGrammar {
        start: start_id,
        names: b.names,
        kinds: b.kinds,
        prods: b.prods,
        by_lhs,
        nt_first,
        nt_nullable,
        prod_first,
        prod_nullable,
        quotes,
    })
}

/// Replace each `PUSH-rule … POP-EQ-rule` stretch of a production by a
/// [`Sym::Quote`]. `PEEK-DIFF` needs no symbol of its own: a quote ends at
/// the first closing delimiter equal to the opening one, so no quoted word
/// can equal it.
fn compile_quotes(
    b: &mut Builder,
    annotations: &[(String, Annotation)],
) -> Result<Vec<Quote>, CompileError> {
    let rule_id = |b: &Builder, name: &str| {
        b.named
            .get(name)
            .copied()
            .ok_or_else(|| CompileError(format!("annotated rule `{name}` is not defined")))
    };
    let mut opens = Vec::new();
    let mut closes = Vec::new();
    for (name, annotation) in annotations {
        match annotation {
            Annotation::Push => opens.push(rule_id(b, name)?),
            Annotation::PopEq => closes.push(rule_id(b, name)?),
            Annotation::PeekDiff => {
                rule_id(b, name)?;
            }
        }
    }
    let mut quotes = Vec::new();
    let mut used_opens = Vec::new();
    for pi in 0..b.prods.len() {
        let mut idx = 0;
        while idx < b.prods[pi].rhs.len() {
            let Sym::N(open) = b.prods[pi].rhs[idx] else {
                idx += 1;
                continue;
            };
            if !opens.contains(&open) {
                idx += 1;
                continue;
            }
            let close_at = b.prods[pi].rhs[idx + 1..]
                .iter()
                .position(|sym| matches!(sym, Sym::N(c) if closes.contains(c)))
                .map(|offset| idx + 1 + offset)
                .ok_or_else(|| {
                    CompileError(format!(
                        "`#: PUSH` rule `{}` is not followed by a `#: POP-EQ` rule",
                        b.names[open as usize]
                    ))
                })?;
            let Sym::N(close) = b.prods[pi].rhs[close_at] else {
                unreachable!("position matched a nonterminal")
            };
            let between: Vec<Sym> = b.prods[pi].rhs[idx + 1..close_at].to_vec();
            let middle = b.helper(NtKind::Helper);
            b.prod(middle, between);
            let q = quotes.len() as u32;
            quotes.push(Quote {
                open,
                middle,
                close,
            });
            used_opens.push(open);
            b.prods[pi].rhs.splice(idx..=close_at, [Sym::Quote(q)]);
            idx += 1;
        }
    }
    if let Some(&open) = opens.iter().find(|o| !used_opens.contains(o)) {
        return Err(CompileError(format!(
            "`#: PUSH` rule `{}` is never followed by a `#: POP-EQ` rule",
            b.names[open as usize]
        )));
    }
    Ok(quotes)
}

/// FIRST byte sets and (conservative) nullability by fixpoint iteration.
fn first_sets(
    nt_count: usize,
    prods: &[Prod],
    quotes: &[Quote],
) -> (Vec<ByteSet>, Vec<bool>, Vec<ByteSet>, Vec<bool>) {
    let mut nt_first = vec![ByteSet::EMPTY; nt_count];
    let mut nt_nullable = vec![false; nt_count];
    let mut prod_first = vec![ByteSet::EMPTY; prods.len()];
    let mut prod_nullable = vec![false; prods.len()];
    let mut changed = true;
    while changed {
        changed = false;
        for (pi, p) in prods.iter().enumerate() {
            let mut first = ByteSet::EMPTY;
            let mut nullable = true;
            for sym in &p.rhs {
                match *sym {
                    Sym::T(set) => {
                        first.union(&set);
                        nullable = false;
                    }
                    Sym::N(n) => {
                        first.union(&nt_first[n as usize]);
                        nullable = nt_nullable[n as usize];
                    }
                    Sym::Quote(q) => {
                        // A quote always consumes input (empty matches are
                        // discarded by the recognizer).
                        first.union(&nt_first[quotes[q as usize].open as usize]);
                        nullable = false;
                    }
                    Sym::And(_) | Sym::Not(_) => {}
                }
                if !nullable {
                    break;
                }
            }
            changed |= prod_first[pi].union(&first);
            changed |= nt_first[p.lhs as usize].union(&first);
            if nullable && !prod_nullable[pi] {
                prod_nullable[pi] = true;
                changed = true;
            }
            if nullable && !nt_nullable[p.lhs as usize] {
                nt_nullable[p.lhs as usize] = true;
                changed = true;
            }
        }
    }
    (nt_first, nt_nullable, prod_first, prod_nullable)
}
