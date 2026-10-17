//! Elidable recovery during Earley chart construction, without enumerating
//! trees. The production path resolves each boundary and retains one
//! predecessor per item and discards resolved alternative links.
//!
//! Costs are omission counts at successive token boundaries, compared
//! lexicographically. Fewer early closures wins: material stays in the open
//! construction until an explicit terminator or the complete syntax requires
//! closure. Positive production costs break ties and eliminate vacuous nullable
//! cycles. There is no prefix commitment: a preferred *incomplete* derivation
//! cannot suppress a valid complete parse.

use super::{ChartTrace, Item, ParseNode};
use crate::cll::grammar::cfg::{NtId, Symbol, TokenGrammar};
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::rc::Rc;

type OmissionProfile = Vec<(usize, usize)>;

fn add_omissions(a: &[(usize, usize)], b: &[(usize, usize)]) -> OmissionProfile {
    let mut merged = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        match (a.get(i), b.get(j)) {
            (Some(&(position, count)), Some(&(other, amount))) if position == other => {
                merged.push((position, count + amount));
                i += 1;
                j += 1;
            }
            (Some(&value), Some(&other)) if value.0 < other.0 => {
                merged.push(value);
                i += 1;
            }
            (Some(&value), None) => {
                merged.push(value);
                i += 1;
            }
            (_, Some(&value)) => {
                merged.push(value);
                j += 1;
            }
            _ => unreachable!(),
        }
    }
    merged
}

/// Sparse nonnegative vector; a missing boundary has count zero. The final
/// component counts production nodes, making gratuitous epsilon loops costly.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Cost {
    omissions: Rc<[(usize, usize)]>,
    nodes: usize,
}

impl Ord for Cost {
    fn cmp(&self, other: &Self) -> Ordering {
        if Rc::ptr_eq(&self.omissions, &other.omissions) {
            return self.nodes.cmp(&other.nodes);
        }
        let mut a = self.omissions.iter();
        let mut b = other.omissions.iter();
        loop {
            match (a.next(), b.next()) {
                (None, None) => return self.nodes.cmp(&other.nodes),
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some((i, x)), Some((j, y))) => {
                    // At the earlier occupied boundary the other count is zero.
                    let order = j.cmp(i).then(x.cmp(y));
                    if order != Ordering::Equal {
                        return order;
                    }
                }
            }
        }
    }
}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Cost {
    fn plus(&self, other: &Self) -> Self {
        let omissions = if self.omissions.is_empty() {
            Rc::clone(&other.omissions)
        } else if other.omissions.is_empty() {
            Rc::clone(&self.omissions)
        } else {
            add_omissions(&self.omissions, &other.omissions).into()
        };
        Self {
            omissions,
            nodes: self.nodes + other.nodes,
        }
    }
}

#[derive(Clone, Copy)]
enum Family {
    Seed,
    /// Empty production for any compiled `/terminator/` site. This is a
    /// conditional recovery, not an unconditional CFG epsilon in the selected
    /// derivation.
    Recover,
    Scan(usize),
    Complete(usize, usize),
}

impl Family {
    /// The one source of recovery cost for every elidable site. Scan and
    /// completion families inherit the costs of their children.
    fn initial_profile(self, boundary: usize) -> Option<OmissionProfile> {
        match self {
            Self::Seed => Some(Vec::new()),
            Self::Recover => Some(vec![(boundary, 1)]),
            Self::Scan(_) | Self::Complete(_, _) => None,
        }
    }
}

struct Edge {
    parent: usize,
    family: Family,
    pending: usize,
}

/// Production tree path: retain one cost and predecessor per Earley item.
/// All alternative links for one boundary are solved together and then
/// discarded. Earlier boundaries are immutable by the time Earley advances.
pub(super) struct CompactBuilder {
    items: Vec<(usize, Item)>,
    ids: HashMap<(usize, Item), usize>,
    costs: Vec<Option<Cost>>,
    profiles: HashSet<Rc<[(usize, usize)]>>,
    chosen: Vec<Option<Family>>,
    pending: Vec<Vec<Edge>>,
    completed_here: HashMap<NtId, Vec<(Item, usize)>>,
    #[cfg(test)]
    pending_links: usize,
    #[cfg(test)]
    peak_pending_links: usize,
}

impl CompactBuilder {
    pub(super) fn new(input_len: usize) -> Self {
        Self {
            items: Vec::new(),
            ids: HashMap::new(),
            costs: Vec::new(),
            profiles: HashSet::new(),
            chosen: Vec::new(),
            pending: (0..=input_len).map(|_| Vec::new()).collect(),
            completed_here: HashMap::new(),
            #[cfg(test)]
            pending_links: 0,
            #[cfg(test)]
            peak_pending_links: 0,
        }
    }

    fn add(&mut self, end: usize, parent: usize, family: Family) {
        self.pending[end].push(Edge {
            parent,
            family,
            pending: 0,
        });
        #[cfg(test)]
        {
            self.pending_links += 1;
            self.peak_pending_links = self.peak_pending_links.max(self.pending_links);
        }
    }

    fn candidate(&self, edge: &Edge) -> Option<Cost> {
        let omissions = edge.family.initial_profile(self.items[edge.parent].0);
        match edge.family {
            Family::Seed | Family::Recover => Some(Cost {
                omissions: omissions.unwrap().into(),
                nodes: 1,
            }),
            // A scan adds no recovery or production node. Share the complete
            // profile instead of cloning its potentially long sparse vector.
            Family::Scan(prefix) => self.costs[prefix].clone(),
            Family::Complete(prefix, child) => Some(
                self.costs[prefix]
                    .as_ref()?
                    .plus(self.costs[child].as_ref()?),
            ),
        }
    }

    fn intern_profile(&mut self, profile: Rc<[(usize, usize)]>) -> Rc<[(usize, usize)]> {
        if let Some(existing) = self.profiles.get(profile.as_ref()) {
            return Rc::clone(existing);
        }
        self.profiles.insert(Rc::clone(&profile));
        profile
    }

    fn tree(&self, grammar: &TokenGrammar, id: usize) -> ParseNode {
        let (end, item) = self.items[id];
        let prod = &grammar.productions[item.prod];
        let mut children = Vec::new();
        let mut prefix = id;
        loop {
            match self.chosen[prefix].expect("productive prefix") {
                Family::Seed | Family::Recover => break,
                Family::Scan(prev) => {
                    let (position, scanned) = self.items[prefix];
                    let Symbol::Term(term) = grammar.productions[scanned.prod].rhs[scanned.dot - 1]
                    else {
                        unreachable!()
                    };
                    children.push(ParseNode {
                        symbol: grammar.term_names[term as usize].clone(),
                        start: position - 1,
                        end: position,
                        terminal: true,
                        production: None,
                        children: Vec::new(),
                    });
                    prefix = prev;
                }
                Family::Complete(prev, child) => {
                    children.push(self.tree(grammar, child));
                    prefix = prev;
                }
            }
        }
        children.reverse();
        ParseNode {
            symbol: grammar.nt_names[prod.lhs as usize].clone(),
            start: item.origin,
            end,
            terminal: false,
            production: Some(item.prod as u32),
            children,
        }
    }

    pub(super) fn finish(
        self,
        grammar: &TokenGrammar,
        start: NtId,
        input_len: usize,
    ) -> Option<ParseNode> {
        let root = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, (end, item))| {
                *end == input_len
                    && item.origin == 0
                    && grammar.productions[item.prod].lhs == start
                    && item.dot == grammar.productions[item.prod].rhs.len()
            })
            .filter_map(|(id, _)| self.costs[id].as_ref().map(|cost| (cost, id)))
            .min()
            .map(|(_, id)| id)?;
        Some(self.tree(grammar, root))
    }
}

impl ChartTrace for CompactBuilder {
    fn insert(&mut self, end: usize, item: Item, recovery: bool) {
        debug_assert!(!recovery || item.dot == 0 && item.origin == end);
        let id = self.items.len();
        assert!(self.ids.insert((end, item), id).is_none());
        self.items.push((end, item));
        self.costs.push(None);
        self.chosen.push(None);
        if item.dot == 0 {
            self.add(
                end,
                id,
                if recovery {
                    Family::Recover
                } else {
                    Family::Seed
                },
            );
        }
    }

    fn scan(&mut self, end: usize, prefix: Item) {
        let parent = Item {
            dot: prefix.dot + 1,
            ..prefix
        };
        if let Some(&parent_id) = self.ids.get(&(end, parent)) {
            let prefix_id = self.ids[&(end - 1, prefix)];
            self.add(end, parent_id, Family::Scan(prefix_id));
        }
    }

    fn complete(&mut self, end: usize, prefix: Item, child: Item) {
        let parent = Item {
            dot: prefix.dot + 1,
            ..prefix
        };
        if let Some(&parent_id) = self.ids.get(&(end, parent)) {
            let prefix_id = self.ids[&(child.origin, prefix)];
            let child_id = self.ids[&(end, child)];
            self.add(end, parent_id, Family::Complete(prefix_id, child_id));
        }
    }

    fn item_count(&self) -> usize {
        self.items.len()
    }

    fn begin_boundary(&mut self) {
        self.completed_here.clear();
    }

    fn completed_here(&self, end: usize, nt: NtId, waiter: Item) -> Vec<Item> {
        let waiter_id = self.ids[&(end, waiter)];
        self.completed_here
            .get(&nt)
            .map(|children| {
                children
                    .iter()
                    .filter(|(_, prior_count)| waiter_id >= *prior_count)
                    .map(|(item, _)| *item)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn record_completed(&mut self, end: usize, item: Item, lhs: NtId, prior_count: usize) {
        if item.origin == end {
            self.completed_here
                .entry(lhs)
                .or_default()
                .push((item, prior_count));
        }
    }

    fn finish_boundary(&mut self, end: usize) {
        let mut edges = std::mem::take(&mut self.pending[end]);
        #[cfg(test)]
        {
            self.pending_links -= edges.len();
        }
        let mut uses: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut heap = BinaryHeap::new();
        for (edge_id, edge) in edges.iter_mut().enumerate() {
            let mut child = |id: usize| {
                if self.items[id].0 == end {
                    edge.pending += 1;
                    uses.entry(id).or_default().push(edge_id);
                }
            };
            match edge.family {
                Family::Seed | Family::Recover => {}
                Family::Scan(a) => child(a),
                Family::Complete(a, b) => {
                    child(a);
                    child(b);
                }
            }
            if edge.pending == 0 {
                if let Some(cost) = self.candidate(edge) {
                    heap.push(Reverse((cost, edge.parent, edge_id)));
                }
            }
        }
        while let Some(Reverse((mut cost, id, edge_id))) = heap.pop() {
            if self.costs[id].is_some() {
                continue;
            }
            cost.omissions = self.intern_profile(cost.omissions);
            self.costs[id] = Some(cost);
            self.chosen[id] = Some(edges[edge_id].family);
            if let Some(dependents) = uses.get(&id) {
                for &dependent in dependents {
                    let edge = &mut edges[dependent];
                    edge.pending -= 1;
                    if edge.pending == 0 {
                        if let Some(cost) = self.candidate(edge) {
                            heap.push(Reverse((cost, edge.parent, dependent)));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cll::grammar::{cll21_grammar, compile_ebnf, earley::derive_one, parse_as};

    /// Manual state probe. Run with
    /// `cargo test --features grammar --lib compact_link_probe -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn compact_link_probe() {
        for depth in [1, 2, 4, 8, 16] {
            let input = format!(
                "mi djica le nu {}mi klama le zarci",
                "do djica le nu ".repeat(depth - 1)
            );
            let parsed = parse_as(&input, "text");
            assert!(parsed.accepted);
            let grammar = cll21_grammar();
            let mut compact = CompactBuilder::new(parsed.term_ids.len());
            super::super::build_chart_with_trace(
                grammar,
                &[grammar.start],
                &parsed.term_ids,
                Some(&mut compact),
            );
            let unique: HashMap<_, _> = compact
                .costs
                .iter()
                .flatten()
                .map(|cost| (Rc::as_ptr(&cost.omissions), cost.omissions.len()))
                .collect();
            let unique_profile_entries: usize = unique.values().sum();
            let distinct_profiles: HashSet<_> = compact
                .costs
                .iter()
                .flatten()
                .map(|cost| cost.omissions.clone())
                .collect();
            let distinct_profile_entries: usize = distinct_profiles.iter().map(|p| p.len()).sum();
            eprintln!(
                "depth={depth} tokens={} items={} peak_pending_links={} logical_profile_entries={} unique_profile_entries={} distinct_profile_entries={} distinct_profiles={}",
                parsed.term_ids.len(),
                compact.items.len(),
                compact.peak_pending_links,
                compact.costs.iter().flatten().map(|cost| cost.omissions.len()).sum::<usize>(),
                unique_profile_entries,
                distinct_profile_entries,
                distinct_profiles.len()
            );
            assert_eq!(compact.pending_links, 0);
            assert!(compact.pending.iter().all(Vec::is_empty));
        }
    }

    #[test]
    fn later_closure_wins_even_if_it_requires_more_omissions() {
        for rules in ["A /E/ B | A B /E/ /F/ /G/", "A B /E/ /F/ /G/ | A /E/ B"] {
            let (g, _) = compile_ebnf(&format!("s = {rules}\n"), "s").unwrap();
            let t = derive_one(&g, g.start, &[g.term("A").unwrap(), g.term("B").unwrap()]).unwrap();
            let omissions = t.elisions(&g);
            assert_eq!(omissions.len(), 3);
            assert!(omissions.iter().all(|e| e.position == 2));
        }
    }

    #[test]
    fn continuation_falls_back_to_earlier_closure_when_it_cannot_complete() {
        let (g, _) = compile_ebnf("s = A /E/ B | A B /F/ C | A B C /G/\n", "s").unwrap();
        for (words, expected_terminator, expected_position) in
            [(&["A", "B"][..], "E", 1), (&["A", "B", "C"][..], "G", 3)]
        {
            let tokens: Vec<_> = words.iter().map(|word| g.term(word).unwrap()).collect();
            let tree = derive_one(&g, g.start, &tokens).unwrap();
            let omissions = tree.elisions(&g);
            assert_eq!(omissions.len(), 1);
            assert_eq!(omissions[0].terminal, expected_terminator);
            assert_eq!(omissions[0].position, expected_position);
        }
    }

    #[test]
    fn every_chapter_21_terminator_uses_the_same_recovery_rule() {
        let mut terminals: Vec<_> = cll21_grammar()
            .elidables
            .values()
            .map(|site| site.terminal.as_str())
            .collect();
        terminals.sort_unstable();
        terminals.dedup();
        assert_eq!(terminals.len(), 21);

        for terminal in terminals {
            // Both complete readings exist. The recovery after B is selected.
            let (g, _) =
                compile_ebnf(&format!("s = A /{terminal}/ B | A B /{terminal}/\n"), "s").unwrap();
            let input = [g.term("A").unwrap(), g.term("B").unwrap()];
            let tree = derive_one(&g, g.start, &input).unwrap();
            assert_eq!(tree.elisions(&g)[0].position, 2, "{terminal}");

            // The later closure cannot complete here, so recovery at A wins.
            let (fallback, _) =
                compile_ebnf(&format!("s = A /{terminal}/ B | A B /{terminal}/ C\n"), "s").unwrap();
            let input = [fallback.term("A").unwrap(), fallback.term("B").unwrap()];
            let tree = derive_one(&fallback, fallback.start, &input).unwrap();
            let elisions = tree.elisions(&fallback);
            assert_eq!(elisions.len(), 1, "{terminal}");
            assert_eq!(elisions[0].terminal, terminal);
            assert_eq!(elisions[0].position, 1, "{terminal}");
        }
    }

    #[test]
    fn nullable_cycles_have_finite_preferred_trees() {
        let (g, _) = compile_ebnf("s = [s] | A /E/\n", "s").unwrap();
        for tokens in [vec![], vec![g.term("A").unwrap()]] {
            let t = derive_one(&g, g.start, &tokens).unwrap();
            if !tokens.is_empty() {
                assert_eq!(t.elisions(&g)[0].position, tokens.len());
            }
            fn nodes(t: &ParseNode) -> usize {
                1 + t.children.iter().map(nodes).sum::<usize>()
            }
            assert!(nodes(&t) < 12);
        }
    }

    #[test]
    fn compact_ambiguity_does_not_enumerate_catalan_many_trees() {
        let (g, _) = compile_ebnf("s = s s | A\n", "s").unwrap();
        let tokens = vec![g.term("A").unwrap(); 24];
        let mut compact = CompactBuilder::new(tokens.len());
        super::super::build_chart_with_trace(&g, &[g.start], &tokens, Some(&mut compact));
        assert!(compact.items.len() < 10_000);
        assert!(compact.peak_pending_links < 20_000);
        assert_eq!(compact.pending_links, 0);
        assert!(compact.pending.iter().all(Vec::is_empty));
        let t = compact.finish(&g, g.start, tokens.len()).unwrap();
        assert_eq!((t.start, t.end), (0, 24));
    }

    #[test]
    fn sparse_cost_order_and_addition_agree_with_dense_vectors() {
        let dense: Vec<[usize; 3]> = (0..27).map(|n| [n / 9, (n / 3) % 3, n % 3]).collect();
        fn sparse(v: &[usize; 3]) -> Cost {
            Cost {
                omissions: v
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|(_, n)| *n != 0)
                    .collect::<Vec<_>>()
                    .into(),
                nodes: 1,
            }
        }
        for a in &dense {
            for b in &dense {
                assert_eq!(sparse(a).cmp(&sparse(b)), a.cmp(b));
                let mut sum = sparse(&[a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
                sum.nodes = 2;
                assert_eq!(sparse(a).plus(&sparse(b)), sum);
            }
        }
    }
}
