//! Minimal in-crate Earley recognizer for pure CFGs.
//!
//! No lookahead predicates, no PEG `&`/`!`. Ambiguous grammars are fine;
//! this module only answers recognition (and which start symbols succeed).
//!
//! Uses the Aycock–Horspool nullable refinement (same as `grammar::earley`):
//! when the next symbol is a nullable nonterminal, the dot advances in the
//! same set. Needed because the morph CFG has real ε-productions (`f_onset`,
//! `c_onset`, `c_link`); classic Earley can miss waiters predicted after an
//! ε-completion in the same set.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

/// Nonterminal id (index into [`Grammar::nt_names`]).
pub type NtId = u16;

/// Terminal is a single UTF-8 byte (Lojban orthography is ASCII here).
pub type Term = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Nt(NtId),
    T(Term),
}

#[derive(Debug, Clone)]
pub struct Production {
    pub lhs: NtId,
    pub rhs: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Grammar {
    pub nt_names: Vec<&'static str>,
    pub productions: Vec<Production>,
    /// Productions grouped by lhs for predictor.
    pub by_lhs: Vec<Vec<usize>>,
}

impl Grammar {
    pub fn new(nt_names: Vec<&'static str>, productions: Vec<Production>) -> Self {
        let nonterminal_count = nt_names.len();
        let mut by_lhs = vec![Vec::new(); nonterminal_count];
        for (production_index, production) in productions.iter().enumerate() {
            by_lhs[production.lhs as usize].push(production_index);
        }
        Self {
            nt_names,
            productions,
            by_lhs,
        }
    }

    pub fn nt(&self, name: &str) -> Option<NtId> {
        self.nt_names
            .iter()
            .position(|n| *n == name)
            .map(|i| i as NtId)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Item {
    prod: u16,
    dot: u16,
    origin: u16,
}

/// One selected derivation of a morphology shape, including literal letters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MorphParseNode {
    pub symbol: String,
    pub start: usize,
    pub end: usize,
    pub production: Option<usize>,
    pub children: Vec<MorphParseNode>,
    pub terminal: bool,
}

/// Precompute nullable nonterminals (empty derivation).
fn nullable_nts(grammar: &Grammar) -> Vec<bool> {
    let nonterminal_count = grammar.nt_names.len();
    let mut nullable = vec![false; nonterminal_count];
    let mut changed = true;
    while changed {
        changed = false;
        for production in &grammar.productions {
            if nullable[production.lhs as usize] {
                continue;
            }
            let all_null = production.rhs.iter().all(|symbol| match symbol {
                Symbol::T(_) => false,
                Symbol::Nt(nonterminal) => nullable[*nonterminal as usize],
            });
            if all_null {
                nullable[production.lhs as usize] = true;
                changed = true;
            }
        }
    }
    nullable
}

/// Recognize whether `start` derives `input` exactly.
pub fn recognizes(grammar: &Grammar, start: NtId, input: &[u8]) -> bool {
    recognize_starts(grammar, &[start], input).contains(&start)
}

/// Which of the candidate start symbols derive `input` exactly.
pub fn recognize_starts(grammar: &Grammar, starts: &[NtId], input: &[u8]) -> BTreeSet<NtId> {
    completed_start_productions(grammar, starts, input)
        .into_iter()
        .map(|index| grammar.productions[index].lhs)
        .collect()
}

/// All root productions that can derive the complete input.
pub fn completed_start_productions(
    grammar: &Grammar,
    starts: &[NtId],
    input: &[u8],
) -> BTreeSet<usize> {
    if grammar.productions.is_empty() || starts.is_empty() {
        return BTreeSet::new();
    }

    let chart = build_chart(grammar, starts, input);
    let mut matching_productions = BTreeSet::new();
    for item in &chart[input.len()] {
        let prod = &grammar.productions[item.prod as usize];
        if item.origin == 0 && (item.dot as usize) == prod.rhs.len() && starts.contains(&prod.lhs) {
            matching_productions.insert(item.prod as usize);
        }
    }
    matching_productions
}

fn build_chart(grammar: &Grammar, starts: &[NtId], input: &[u8]) -> Vec<Vec<Item>> {
    let input_len = input.len();
    let nullable = nullable_nts(grammar);

    // chart[k] = items with dot after consuming k terminals
    let mut chart: Vec<Vec<Item>> = vec![Vec::new(); input_len + 1];
    let mut membership: Vec<HashSet<Item>> = vec![HashSet::new(); input_len + 1];

    let enqueue = |chart: &mut Vec<Vec<Item>>,
                   membership: &mut Vec<HashSet<Item>>,
                   chart_index: usize,
                   item: Item| {
        if membership[chart_index].insert(item) {
            chart[chart_index].push(item);
        }
    };

    // Seed: predict each start symbol from a synthetic S' → start ·
    for &start_nonterminal in starts {
        for &production_index in &grammar.by_lhs[start_nonterminal as usize] {
            enqueue(
                &mut chart,
                &mut membership,
                0,
                Item {
                    prod: production_index as u16,
                    dot: 0,
                    origin: 0,
                },
            );
        }
    }

    for chart_index in 0..=input_len {
        let mut item_index = 0;
        while item_index < chart[chart_index].len() {
            let item = chart[chart_index][item_index];
            item_index += 1;
            let prod = &grammar.productions[item.prod as usize];
            if (item.dot as usize) < prod.rhs.len() {
                match prod.rhs[item.dot as usize] {
                    Symbol::T(t) => {
                        // Scanner
                        if chart_index < input_len && input[chart_index] == t {
                            enqueue(
                                &mut chart,
                                &mut membership,
                                chart_index + 1,
                                Item {
                                    prod: item.prod,
                                    dot: item.dot + 1,
                                    origin: item.origin,
                                },
                            );
                        }
                    }
                    Symbol::Nt(nonterminal) => {
                        // Predictor
                        for &production_index in &grammar.by_lhs[nonterminal as usize] {
                            enqueue(
                                &mut chart,
                                &mut membership,
                                chart_index,
                                Item {
                                    prod: production_index as u16,
                                    dot: 0,
                                    origin: chart_index as u16,
                                },
                            );
                        }
                        // Aycock–Horspool: nullable B also advances the waiter now
                        if nullable[nonterminal as usize] {
                            enqueue(
                                &mut chart,
                                &mut membership,
                                chart_index,
                                Item {
                                    prod: item.prod,
                                    dot: item.dot + 1,
                                    origin: item.origin,
                                },
                            );
                        }
                    }
                }
            } else {
                // Completer: advance items waiting for prod.lhs at origin
                let lhs = prod.lhs;
                let origin_index = item.origin as usize;
                // Snapshot length — completer may grow chart[k]
                let waiting: Vec<Item> = chart[origin_index]
                    .iter()
                    .copied()
                    .filter(|waiting_item| {
                        let waiting_production = &grammar.productions[waiting_item.prod as usize];
                        (waiting_item.dot as usize) < waiting_production.rhs.len()
                            && matches!(waiting_production.rhs[waiting_item.dot as usize], Symbol::Nt(nt) if nt == lhs)
                    })
                    .collect();
                for waiting_item in waiting {
                    enqueue(
                        &mut chart,
                        &mut membership,
                        chart_index,
                        Item {
                            prod: waiting_item.prod,
                            dot: waiting_item.dot + 1,
                            origin: waiting_item.origin,
                        },
                    );
                }
            }
        }
    }

    chart
}

/// Extract one complete shape derivation for each accepted start symbol.
/// Ambiguous CFG paths are tried in grammar order; cyclic nullable paths are skipped.
pub fn derive_one(grammar: &Grammar, starts: &[NtId], input: &[u8]) -> Vec<(NtId, MorphParseNode)> {
    if input.len() > u16::MAX as usize || grammar.productions.is_empty() || starts.is_empty() {
        return Vec::new();
    }
    let chart = build_chart(grammar, starts, input);
    let mut completed = HashSet::new();
    let mut spans = HashSet::new();
    for (end, items) in chart.iter().enumerate() {
        for item in items {
            let production = &grammar.productions[item.prod as usize];
            if item.dot as usize == production.rhs.len() {
                let start = item.origin as usize;
                completed.insert((production.lhs, start, end, item.prod as usize));
                spans.insert((production.lhs, start, end));
            }
        }
    }
    let mut extractor = MorphExtractor {
        grammar,
        input,
        completed,
        spans,
        memo: HashMap::new(),
        active: HashSet::new(),
        steps: 0,
    };
    starts
        .iter()
        .filter_map(|&start| {
            extractor
                .derive_node(start, 0, input.len())
                .map(|node| (start, node))
        })
        .collect()
}

struct MorphExtractor<'a> {
    grammar: &'a Grammar,
    input: &'a [u8],
    completed: HashSet<(NtId, usize, usize, usize)>,
    spans: HashSet<(NtId, usize, usize)>,
    memo: HashMap<(NtId, usize, usize), MorphParseNode>,
    active: HashSet<(NtId, usize, usize)>,
    steps: usize,
}

impl MorphExtractor<'_> {
    fn derive_node(&mut self, nt: NtId, start: usize, end: usize) -> Option<MorphParseNode> {
        let key = (nt, start, end);
        if let Some(node) = self.memo.get(&key) {
            return Some(node.clone());
        }
        if self.steps >= 100_000 || !self.spans.contains(&key) || !self.active.insert(key) {
            return None;
        }
        self.steps += 1;
        let mut result = None;
        for offset in 0..self.grammar.by_lhs[nt as usize].len() {
            let production_index = self.grammar.by_lhs[nt as usize][offset];
            if !self.completed.contains(&(nt, start, end, production_index)) {
                continue;
            }
            let rhs = self.grammar.productions[production_index].rhs.clone();
            if let Some(children) = self.derive_rhs(&rhs, 0, start, end) {
                result = Some(MorphParseNode {
                    symbol: self.grammar.nt_names[nt as usize].to_owned(),
                    start,
                    end,
                    production: Some(production_index),
                    children,
                    terminal: false,
                });
                break;
            }
        }
        self.active.remove(&key);
        if let Some(node) = &result {
            self.memo.insert(key, node.clone());
        }
        result
    }

    fn derive_rhs(
        &mut self,
        rhs: &[Symbol],
        part: usize,
        cursor: usize,
        end: usize,
    ) -> Option<Vec<MorphParseNode>> {
        if self.steps >= 100_000 {
            return None;
        }
        if part == rhs.len() {
            return (cursor == end).then(Vec::new);
        }
        match rhs[part] {
            Symbol::T(letter) if cursor < end && self.input[cursor] == letter => {
                let mut rest = self.derive_rhs(rhs, part + 1, cursor + 1, end)?;
                rest.insert(
                    0,
                    MorphParseNode {
                        symbol: (letter as char).to_string(),
                        start: cursor,
                        end: cursor + 1,
                        production: None,
                        children: Vec::new(),
                        terminal: true,
                    },
                );
                Some(rest)
            }
            Symbol::T(_) => None,
            Symbol::Nt(nt) => {
                for split in cursor..=end {
                    if !self.spans.contains(&(nt, cursor, split)) {
                        continue;
                    }
                    let Some(child) = self.derive_node(nt, cursor, split) else {
                        continue;
                    };
                    if let Some(mut rest) = self.derive_rhs(rhs, part + 1, split, end) {
                        rest.insert(0, child);
                        return Some(rest);
                    }
                }
                None
            }
        }
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Grammar ({} nts, {} prods):",
            self.nt_names.len(),
            self.productions.len()
        )?;
        for p in &self.productions {
            write!(f, "  {} ->", self.nt_names[p.lhs as usize])?;
            if p.rhs.is_empty() {
                write!(f, " ε")?;
            }
            for s in &p.rhs {
                match s {
                    Symbol::Nt(n) => write!(f, " {}", self.nt_names[*n as usize])?,
                    Symbol::T(t) => write!(f, " '{}'", *t as char)?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy() -> (Grammar, NtId) {
        // S → "a" S | "a"
        let nt_names = vec!["S"];
        let s = 0u16;
        let productions = vec![
            Production {
                lhs: s,
                rhs: vec![Symbol::T(b'a'), Symbol::Nt(s)],
            },
            Production {
                lhs: s,
                rhs: vec![Symbol::T(b'a')],
            },
        ];
        (Grammar::new(nt_names, productions), s)
    }

    #[test]
    fn earley_toy_as() {
        let (g, s) = toy();
        assert!(recognizes(&g, s, b"a"));
        assert!(recognizes(&g, s, b"aaa"));
        assert!(!recognizes(&g, s, b""));
        assert!(!recognizes(&g, s, b"ab"));
        assert_eq!(completed_start_productions(&g, &[s], b"a"), [1].into());
        assert_eq!(completed_start_productions(&g, &[s], b"aaa"), [0].into());
        let derived = derive_one(&g, &[s], b"aaa");
        assert_eq!(derived[0].1.symbol, "S");
        assert_eq!(derived[0].1.children[0].symbol, "a");
        assert_eq!(
            derived[0].1.children[1].end - derived[0].1.children[1].start,
            2
        );
    }

    #[test]
    fn nullable_tail_aycock() {
        // S -> A N ; N -> ε | B ; A,B terms (bytes 'a','b')
        let nt_names = vec!["S", "N"];
        let productions = vec![
            Production {
                lhs: 0,
                rhs: vec![Symbol::T(b'a'), Symbol::Nt(1)],
            },
            Production {
                lhs: 1,
                rhs: vec![],
            },
            Production {
                lhs: 1,
                rhs: vec![Symbol::T(b'b')],
            },
        ];
        let g = Grammar::new(nt_names, productions);
        assert!(recognizes(&g, 0, b"a"), "A with nullable N");
        assert!(recognizes(&g, 0, b"ab"), "A B");
        assert!(!recognizes(&g, 0, b"b"));
    }
}
