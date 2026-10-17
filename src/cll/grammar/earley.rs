//! Token-stream Earley recognizer for [`super::cfg::TokenGrammar`].
//! Recognition uses the permissive CFG. Selected-tree extraction also applies
//! the complete-parse elidable-recovery policy to marked packed-chart edges.
//!
//! Uses the Aycock–Horspool nullable refinement: when the next symbol is a
//! nullable nonterminal, the dot advances in the same set. Without that,
//! completing `B → ε` once fails to advance waiters that appear later in the
//! same set (common with `/elidable/` → optional and `[…]` tails), which
//! falsely rejected CLL examples that omit KEI/KU/KUhO/etc.

use super::cfg::{NtId, Symbol, TermId, TokenGrammar};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Item {
    prod: usize,
    dot: usize,
    origin: usize,
}

/// Precompute nullable nonterminals (empty derivation).
fn nullable_nts(grammar: &TokenGrammar) -> Vec<bool> {
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
                Symbol::Term(_) => false,
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

/// FIRST sets for every production suffix. An item is useful at a boundary
/// only if its remaining symbols can start with the next token or derive
/// epsilon. The latter is essential for omitted terminators and same-chart
/// nullable completion.
#[derive(Debug, Clone)]
pub(crate) struct PredictionFirst {
    nullable_nts: Vec<bool>,
    terminal_bits: Vec<Vec<Vec<u64>>>,
    nullable: Vec<Vec<bool>>,
}

impl PredictionFirst {
    fn new(grammar: &TokenGrammar) -> Self {
        let nullable_nts = nullable_nts(grammar);
        let words = grammar.term_names.len().div_ceil(64);
        let mut first_nts = vec![vec![0u64; words]; grammar.nt_names.len()];

        let rhs_first = |rhs: &[Symbol], first_nts: &[Vec<u64>]| {
            let mut bits = vec![0u64; words];
            let mut nullable = true;
            for symbol in rhs {
                match symbol {
                    Symbol::Term(term) => {
                        let index = *term as usize;
                        bits[index / 64] |= 1u64 << (index % 64);
                        nullable = false;
                        break;
                    }
                    Symbol::Nt(nt) => {
                        for (word, contribution) in bits.iter_mut().zip(&first_nts[*nt as usize]) {
                            *word |= contribution;
                        }
                        if !nullable_nts[*nt as usize] {
                            nullable = false;
                            break;
                        }
                    }
                }
            }
            (bits, nullable)
        };

        loop {
            let mut changed = false;
            for production in &grammar.productions {
                let (bits, _) = rhs_first(&production.rhs, &first_nts);
                for (current, contribution) in
                    first_nts[production.lhs as usize].iter_mut().zip(bits)
                {
                    let merged = *current | contribution;
                    changed |= merged != *current;
                    *current = merged;
                }
            }
            if !changed {
                break;
            }
        }

        let mut terminal_bits = Vec::with_capacity(grammar.productions.len());
        let mut nullable = Vec::with_capacity(grammar.productions.len());
        for production in &grammar.productions {
            let (bits, empty): (Vec<_>, Vec<_>) = (0..=production.rhs.len())
                .map(|dot| rhs_first(&production.rhs[dot..], &first_nts))
                .unzip();
            terminal_bits.push(bits);
            nullable.push(empty);
        }
        Self {
            nullable_nts,
            terminal_bits,
            nullable,
        }
    }

    fn allows(&self, production: usize, dot: usize, next: Option<TermId>) -> bool {
        if self.nullable[production][dot] {
            return true;
        }
        let Some(term) = next else {
            return false;
        };
        let index = term as usize;
        self.terminal_bits[production][dot]
            .get(index / 64)
            .is_some_and(|word| word & (1u64 << (index % 64)) != 0)
    }
}

/// Recognize whether `start` derives `input` exactly.
pub fn recognizes(grammar: &TokenGrammar, start: NtId, input: &[TermId]) -> bool {
    recognize_starts(grammar, &[start], input).contains(&start)
}

/// Which of the candidate start symbols derive `input` exactly.
pub fn recognize_starts(
    grammar: &TokenGrammar,
    starts: &[NtId],
    input: &[TermId],
) -> BTreeSet<NtId> {
    let input_len = input.len();
    if grammar.productions.is_empty()
        || starts.is_empty()
        || starts.iter().any(|&s| s as usize >= grammar.by_lhs.len())
    {
        return BTreeSet::new();
    }

    let chart = build_chart(grammar, starts, input);
    matched_starts(grammar, starts, input_len, &chart)
}

fn matched_starts(
    grammar: &TokenGrammar,
    starts: &[NtId],
    input_len: usize,
    chart: &[Vec<Item>],
) -> BTreeSet<NtId> {
    let mut ok = BTreeSet::new();
    for item in &chart[input_len] {
        let prod = &grammar.productions[item.prod];
        if item.origin == 0 && item.dot == prod.rhs.len() && starts.contains(&prod.lhs) {
            ok.insert(prod.lhs);
        }
    }
    ok
}

/// Return diagnostic matches and the selected tree from one Earley chart.
pub(crate) fn recognize_starts_with_tree(
    grammar: &TokenGrammar,
    starts: &[NtId],
    selected: NtId,
    input: &[TermId],
) -> (BTreeSet<NtId>, Option<ParseNode>) {
    if grammar.productions.is_empty()
        || starts.is_empty()
        || starts.iter().any(|&s| s as usize >= grammar.by_lhs.len())
        || !starts.contains(&selected)
    {
        return (BTreeSet::new(), None);
    }
    let mut trace = elision::CompactBuilder::new(input.len());
    let chart = build_chart_with_trace(grammar, starts, input, Some(&mut trace));
    let matched = matched_starts(grammar, starts, input.len(), &chart);
    let tree = if matched.contains(&selected) {
        trace.finish(grammar, selected, input.len())
    } else {
        None
    };
    (matched, tree)
}

/// Build the Earley chart used by both recognition and one-tree extraction.
fn build_chart(grammar: &TokenGrammar, starts: &[NtId], input: &[TermId]) -> Vec<Vec<Item>> {
    build_chart_with_trace(grammar, starts, input, None::<&mut elision::CompactBuilder>)
}

trait ChartTrace {
    fn insert(&mut self, end: usize, item: Item, recovery: bool);
    fn scan(&mut self, end: usize, prefix: Item);
    fn complete(&mut self, end: usize, prefix: Item, child: Item);
    fn item_count(&self) -> usize;
    fn begin_boundary(&mut self);
    fn completed_here(&self, end: usize, nt: NtId, waiter: Item) -> Vec<Item>;
    fn record_completed(&mut self, end: usize, item: Item, lhs: NtId, prior_count: usize);
    fn finish_boundary(&mut self, end: usize);
}

fn build_chart_with_trace<T: ChartTrace>(
    grammar: &TokenGrammar,
    starts: &[NtId],
    input: &[TermId],
    mut trace: Option<&mut T>,
) -> Vec<Vec<Item>> {
    let input_len = input.len();
    let first = grammar
        .prediction
        .get_or_init(|| PredictionFirst::new(grammar));
    let nullable = &first.nullable_nts;
    let mut chart: Vec<Vec<Item>> = vec![Vec::new(); input_len + 1];
    let mut membership: Vec<HashSet<Item>> = vec![HashSet::new(); input_len + 1];
    let mut predicted: Vec<HashSet<NtId>> = vec![HashSet::new(); input_len + 1];
    // Completion consults only items awaiting the completed nonterminal.
    // Sparse maps avoid allocating one list per grammar nonterminal at every
    // input boundary.
    let mut waiters: Vec<HashMap<NtId, Vec<Item>>> = vec![HashMap::new(); input_len + 1];

    let enqueue = |chart: &mut Vec<Vec<Item>>,
                   membership: &mut Vec<HashSet<Item>>,
                   waiters: &mut Vec<HashMap<NtId, Vec<Item>>>,
                   trace: &mut Option<&mut T>,
                   chart_index: usize,
                   item: Item| {
        if !first.allows(item.prod, item.dot, input.get(chart_index).copied()) {
            return;
        }
        if membership[chart_index].insert(item) {
            if let Some(trace) = trace.as_deref_mut() {
                trace.insert(
                    chart_index,
                    item,
                    grammar.elidables.contains_key(&item.prod),
                );
            }
            if let Some(Symbol::Nt(nt)) = grammar.productions[item.prod].rhs.get(item.dot) {
                waiters[chart_index].entry(*nt).or_default().push(item);
            }
            chart[chart_index].push(item);
        }
    };

    for &start_nt in starts {
        for &production_index in &grammar.by_lhs[start_nt as usize] {
            enqueue(
                &mut chart,
                &mut membership,
                &mut waiters,
                &mut trace,
                0,
                Item {
                    prod: production_index,
                    dot: 0,
                    origin: 0,
                },
            );
        }
    }

    for chart_index in 0..=input_len {
        if let Some(trace) = trace.as_deref_mut() {
            trace.begin_boundary();
        }
        let mut item_index = 0;
        while item_index < chart[chart_index].len() {
            let item = chart[chart_index][item_index];
            item_index += 1;
            let prod = &grammar.productions[item.prod];
            if item.dot < prod.rhs.len() {
                match prod.rhs[item.dot] {
                    Symbol::Term(t) => {
                        if chart_index < input_len && input[chart_index] == t {
                            enqueue(
                                &mut chart,
                                &mut membership,
                                &mut waiters,
                                &mut trace,
                                chart_index + 1,
                                Item {
                                    prod: item.prod,
                                    dot: item.dot + 1,
                                    origin: item.origin,
                                },
                            );
                            if let Some(trace) = trace.as_deref_mut() {
                                trace.scan(chart_index + 1, item);
                            }
                        }
                    }
                    Symbol::Nt(nonterminal) => {
                        // Each nonterminal's productions need prediction only
                        // once at a boundary, however many items await it.
                        if predicted[chart_index].insert(nonterminal) {
                            for &production_index in &grammar.by_lhs[nonterminal as usize] {
                                enqueue(
                                    &mut chart,
                                    &mut membership,
                                    &mut waiters,
                                    &mut trace,
                                    chart_index,
                                    Item {
                                        prod: production_index,
                                        dot: 0,
                                        origin: chart_index,
                                    },
                                );
                            }
                        }
                        // Aycock–Horspool: nullable B also advances the waiter now
                        if nullable[nonterminal as usize] {
                            enqueue(
                                &mut chart,
                                &mut membership,
                                &mut waiters,
                                &mut trace,
                                chart_index,
                                Item {
                                    prod: item.prod,
                                    dot: item.dot + 1,
                                    origin: item.origin,
                                },
                            );
                        }
                        if let Some(trace) = trace.as_deref_mut() {
                            for child in trace.completed_here(chart_index, nonterminal, item) {
                                trace.complete(chart_index, item, child);
                            }
                        }
                    }
                }
            } else {
                let lhs = prod.lhs;
                let origin_index = item.origin;
                let prior_trace_items = trace.as_ref().map(|trace| trace.item_count());
                // Capture the current length: same-set nullable completions
                // may enqueue more waiters, which advance through the
                // nullable shortcut when they are processed.
                let waiting_count = waiters[origin_index].get(&lhs).map_or(0, Vec::len);
                for waiting_index in 0..waiting_count {
                    let waiting_item = waiters[origin_index][&lhs][waiting_index];
                    enqueue(
                        &mut chart,
                        &mut membership,
                        &mut waiters,
                        &mut trace,
                        chart_index,
                        Item {
                            prod: waiting_item.prod,
                            dot: waiting_item.dot + 1,
                            origin: waiting_item.origin,
                        },
                    );
                    if let Some(trace) = trace.as_deref_mut() {
                        trace.complete(chart_index, waiting_item, item);
                    }
                }
                if let (Some(trace), Some(prior_count)) = (trace.as_deref_mut(), prior_trace_items)
                {
                    trace.record_completed(chart_index, item, lhs, prior_count);
                }
            }
        }
        if let Some(trace) = trace.as_deref_mut() {
            trace.finish_boundary(chart_index);
        }
    }

    chart
}

/// One concrete Chapter 21 derivation. Offsets are lexeme indices, not bytes.
/// Selected by elision scope, with deterministic ties for other ambiguities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseNode {
    pub symbol: String,
    pub start: usize,
    pub end: usize,
    pub terminal: bool,
    /// Index of the CFG production selected for this nonterminal. Terminals
    /// have no production. Helper productions are retained in the tree.
    pub production: Option<u32>,
    pub children: Vec<ParseNode>,
}

/// An implicit terminator in the selected derivation. Positions and owner
/// spans are lexeme indices; several nested constructions can close at one gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elision {
    pub terminal: String,
    pub position: usize,
    pub source_rule: String,
    pub owner_start: usize,
    pub owner_end: usize,
}

/// A syntax tree with compiler-generated EBNF helpers removed. Source-rule
/// nodes and written terminals retain their hierarchy and token spans.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralTree {
    pub roots: Vec<StructuralNode>,
    pub elisions: Vec<StructuralElision>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralNode {
    pub symbol: String,
    pub start: usize,
    pub end: usize,
    pub terminal: bool,
    pub children: Vec<StructuralNode>,
}

/// `owner_path` indexes `roots`, then successive `children`. It distinguishes
/// nested owners even when source rule and token span happen to be identical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralElision {
    pub terminal: String,
    pub position: usize,
    pub source_rule: String,
    pub owner_path: Vec<usize>,
}

impl ParseNode {
    /// Project a raw CFG derivation to the source-rule structure used for
    /// attachment tests. All explicit terminal leaves are retained; omitted
    /// terminators are recorded with their owning source-rule path.
    pub fn structure(&self, grammar: &TokenGrammar) -> StructuralTree {
        fn append(
            raw: &ParseNode,
            grammar: &TokenGrammar,
            siblings: &mut Vec<StructuralNode>,
            path: &mut Vec<usize>,
            ancestors: &mut Vec<(String, Vec<usize>)>,
            elisions: &mut Vec<StructuralElision>,
        ) {
            if let Some(site) = raw
                .production
                .and_then(|prod| grammar.elidables.get(&(prod as usize)))
            {
                // A tree from this grammar always has the owner; a tree
                // paired with some other grammar just records no elision.
                if let Some(owner_path) = ancestors
                    .iter()
                    .rev()
                    .find(|(name, _)| name == &site.source_rule)
                    .map(|(_, path)| path.clone())
                {
                    elisions.push(StructuralElision {
                        terminal: site.terminal.clone(),
                        position: raw.start,
                        source_rule: site.source_rule.clone(),
                        owner_path,
                    });
                }
            }

            let source_rule = !raw.terminal
                && raw.production.is_some_and(|prod| {
                    grammar
                        .productions
                        .get(prod as usize)
                        .is_some_and(|p| grammar.source_nts.contains(&p.lhs))
                });
            if raw.terminal || source_rule {
                path.push(siblings.len());
                if source_rule {
                    ancestors.push((raw.symbol.clone(), path.clone()));
                }
                let mut children = Vec::new();
                for child in &raw.children {
                    append(child, grammar, &mut children, path, ancestors, elisions);
                }
                if source_rule {
                    ancestors.pop();
                }
                siblings.push(StructuralNode {
                    symbol: raw.symbol.clone(),
                    start: raw.start,
                    end: raw.end,
                    terminal: raw.terminal,
                    children,
                });
                path.pop();
            } else {
                for child in &raw.children {
                    append(child, grammar, siblings, path, ancestors, elisions);
                }
            }
        }

        let mut structure = StructuralTree {
            roots: Vec::new(),
            elisions: Vec::new(),
        };
        append(
            self,
            grammar,
            &mut structure.roots,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut structure.elisions,
        );
        structure
    }

    /// Recover elisions in yield order, preserving the owning source-rule span.
    pub fn elisions(&self, grammar: &TokenGrammar) -> Vec<Elision> {
        fn visit<'a>(
            node: &'a ParseNode,
            grammar: &TokenGrammar,
            ancestors: &mut Vec<&'a ParseNode>,
            out: &mut Vec<Elision>,
        ) {
            if let Some(site) = node
                .production
                .and_then(|p| grammar.elidables.get(&(p as usize)))
            {
                let owner = ancestors
                    .iter()
                    .rev()
                    .find(|n| n.symbol == site.source_rule)
                    .copied()
                    .unwrap_or(node);
                out.push(Elision {
                    terminal: site.terminal.clone(),
                    position: node.start,
                    source_rule: site.source_rule.clone(),
                    owner_start: owner.start,
                    owner_end: owner.end,
                });
            }
            ancestors.push(node);
            for child in &node.children {
                visit(child, grammar, ancestors, out);
            }
            ancestors.pop();
        }
        let mut out = Vec::new();
        visit(self, grammar, &mut Vec::new(), &mut out);
        out
    }
}

// Kept in a child module so recognition and forest extraction share the exact
// same chart/item implementation, including nullable waiter handling.
#[path = "elision.rs"]
mod elision;

/// Extract the preferred complete derivation. At the first token boundary where
/// readings differ, continue an open construction instead of silently closing
/// it. Only complete parses compete; this intentionally permits `le nanmu joi
/// le ninmu` without importing YACC's prefix-commitment restriction.
pub fn derive_one(grammar: &TokenGrammar, start: NtId, input: &[TermId]) -> Option<ParseNode> {
    if start as usize >= grammar.by_lhs.len() {
        return None;
    }
    let mut trace = elision::CompactBuilder::new(input.len());
    build_chart_with_trace(grammar, &[start], input, Some(&mut trace));
    trace.finish(grammar, start, input.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::grammar::cfg::{Production, Symbol, TokenGrammar};
    use crate::cll::grammar::{cll21_grammar, compile_ebnf, parse_as};

    #[test]
    #[ignore]
    fn shared_chart_size_probe() {
        let grammar = cll21_grammar();
        let starts: Vec<_> = ["text", "sentence", "sumti", "selbri", "fragment"]
            .into_iter()
            .filter_map(|name| grammar.nt(name))
            .collect();
        for (label, input, root) in [
            ("basic", "mi viska le gerku".to_string(), "text"),
            (
                "abstraction",
                "mi djica le nu do klama le zarci".to_string(),
                "text",
            ),
            (
                "nested-8",
                format!(
                    "mi djica le nu {}mi klama le zarci",
                    "do djica le nu ".repeat(7)
                ),
                "text",
            ),
            (
                "nested-16",
                format!(
                    "mi djica le nu {}mi klama le zarci",
                    "do djica le nu ".repeat(15)
                ),
                "text",
            ),
            ("sumti-mex", "li su'i pa re ci".to_string(), "sumti"),
        ] {
            let parsed = parse_as(&input, root);
            assert!(parsed.accepted);
            let single = build_chart(grammar, &[grammar.nt(root).unwrap()], &parsed.term_ids);
            let combined = build_chart(grammar, &starts, &parsed.term_ids);
            eprintln!(
                "{label} tokens={} single_items={} combined_items={}",
                parsed.term_ids.len(),
                single.iter().map(Vec::len).sum::<usize>(),
                combined.iter().map(Vec::len).sum::<usize>()
            );
        }
    }

    #[test]
    fn toy_token_earley() {
        // S -> A S | A ; A is term 0
        let g = TokenGrammar::from_parts(
            vec!["S".into()],
            vec!["A".into()],
            vec![
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Term(0), Symbol::Nt(0)],
                },
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Term(0)],
                },
            ],
            "S",
        )
        .unwrap();
        assert!(recognizes(&g, 0, &[0]));
        assert!(recognizes(&g, 0, &[0, 0, 0]));
        assert!(!recognizes(&g, 0, &[]));
    }

    #[test]
    fn derives_a_real_tree_with_terminal_spans() {
        let grammar = TokenGrammar::from_parts(
            vec!["S".into(), "N".into()],
            vec!["A".into()],
            vec![
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Nt(1), Symbol::Term(0)],
                },
                Production {
                    lhs: 1,
                    rhs: vec![],
                },
            ],
            "S",
        )
        .unwrap();
        let tree = derive_one(&grammar, 0, &[0]).expect("accepted derivation");
        assert_eq!(tree.symbol, "S");
        assert_eq!(tree.production, Some(0));
        assert_eq!((tree.start, tree.end), (0, 1));
        assert_eq!(tree.children[0].symbol, "N");
        assert_eq!(tree.children[0].production, Some(1));
        assert_eq!((tree.children[0].start, tree.children[0].end), (0, 0));
        assert_eq!(tree.children[1].symbol, "A");
        assert!(tree.children[1].terminal);
        assert_eq!(tree.children[1].production, None);
    }

    #[test]
    fn nullable_tail_aycock() {
        // S -> A N ; N -> ε | B ; A,B terms
        // Input [A] must accept via N → ε even if another A N completion
        // path also predicts N earlier in the same set.
        let g = TokenGrammar::from_parts(
            vec!["S".into(), "N".into()],
            vec!["A".into(), "B".into()],
            vec![
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Term(0), Symbol::Nt(1)],
                },
                Production {
                    lhs: 1,
                    rhs: vec![],
                },
                Production {
                    lhs: 1,
                    rhs: vec![Symbol::Term(1)],
                },
            ],
            "S",
        )
        .unwrap();
        assert!(recognizes(&g, 0, &[0]), "A with nullable N");
        assert!(recognizes(&g, 0, &[0, 1]), "A B");
        assert!(!recognizes(&g, 0, &[1]));
    }

    #[test]
    fn first_filter_keeps_nullable_prefix_and_left_recursion() {
        let nullable_prefix = TokenGrammar::from_parts(
            vec!["S".into(), "N".into()],
            vec!["A".into(), "B".into()],
            vec![
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Nt(1), Symbol::Term(1)],
                },
                Production {
                    lhs: 1,
                    rhs: vec![],
                },
                Production {
                    lhs: 1,
                    rhs: vec![Symbol::Term(0)],
                },
            ],
            "S",
        )
        .unwrap();
        assert!(recognizes(&nullable_prefix, 0, &[1]));
        assert!(recognizes(&nullable_prefix, 0, &[0, 1]));
        assert!(derive_one(&nullable_prefix, 0, &[1]).is_some());

        let left_recursive = TokenGrammar::from_parts(
            vec!["S".into()],
            vec!["A".into(), "B".into()],
            vec![
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Nt(0), Symbol::Term(0)],
                },
                Production {
                    lhs: 0,
                    rhs: vec![Symbol::Term(1)],
                },
            ],
            "S",
        )
        .unwrap();
        assert!(recognizes(&left_recursive, 0, &[1, 0, 0]));
        assert!(derive_one(&left_recursive, 0, &[1, 0, 0]).is_some());
    }

    #[test]
    fn structural_projection_ignores_optional_helper_shape() {
        let (optional, _) = compile_ebnf("s = [A] B\n", "s").unwrap();
        let (alternatives, _) = compile_ebnf("s = A B | B\n", "s").unwrap();
        for words in [&["A", "B"][..], &["B"][..]] {
            let input_for = |grammar: &TokenGrammar| {
                words
                    .iter()
                    .map(|word| grammar.term(word).unwrap())
                    .collect::<Vec<_>>()
            };
            let first = derive_one(&optional, optional.start, &input_for(&optional)).unwrap();
            let second =
                derive_one(&alternatives, alternatives.start, &input_for(&alternatives)).unwrap();
            assert_eq!(first.structure(&optional), second.structure(&alternatives));
        }
    }
}
