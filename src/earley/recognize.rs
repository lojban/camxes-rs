//! Earley chart with lookahead symbols, plus preference-ordered derivation
//! extraction.

use super::{EarleyGrammar, NtKind, Sym};
use crate::peg::parsing::{ParseNode, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Item {
    prod: u32,
    dot: u32,
    origin: u32,
}

impl Item {
    #[inline]
    fn advance(self) -> Item {
        Item {
            dot: self.dot + 1,
            ..self
        }
    }
}

/// One Earley chart (main parse or a lookahead sub-recognition). Set `k`
/// holds position `base + k`, so a lookahead run near the end of the input
/// only allocates the sets it can reach.
struct Chart {
    base: usize,
    sets: Vec<Vec<Item>>,
    seen: Vec<FxHashSet<Item>>,
    waiters: Vec<FxHashMap<u32, Vec<Item>>>,
    predicted: Vec<FxHashSet<u32>>,
    eps: Vec<FxHashSet<u32>>,
}

impl Chart {
    fn new(base: usize, n: usize) -> Self {
        let len = n + 1 - base;
        Chart {
            base,
            sets: vec![Vec::new(); len],
            seen: vec![FxHashSet::default(); len],
            waiters: vec![FxHashMap::default(); len],
            predicted: vec![FxHashSet::default(); len],
            eps: vec![FxHashSet::default(); len],
        }
    }

    #[inline]
    fn add(&mut self, i: usize, item: Item) {
        let k = i - self.base;
        if self.seen[k].insert(item) {
            self.sets[k].push(item);
        }
    }
}

type Ends = Rc<Vec<u32>>;

pub(super) struct Parse<'g> {
    g: &'g EarleyGrammar,
    input: &'g [u8],
    /// Completed `(production, start)` → end positions (main chart only).
    done: FxHashMap<(u32, u32), Vec<u32>>,
    /// Lookahead decisions `(nonterminal, position)` → matches a prefix.
    pred_memo: FxHashMap<(u32, u32), bool>,
    pred_active: FxHashSet<(u32, u32)>,
    nt_pref: FxHashMap<(u32, u32), Ends>,
    seq_pref: FxHashMap<(u32, u32, u32), Ends>,
    pref_active: FxHashSet<(u32, u32)>,
    furthest: usize,
}

enum Mode {
    /// Record every completion (main parse).
    Record,
    /// Stop as soon as `nt` completes from the starting position.
    Prefix(u32),
}

impl<'g> Parse<'g> {
    pub(super) fn new(g: &'g EarleyGrammar, input: &'g [u8]) -> Self {
        Parse {
            g,
            input,
            done: FxHashMap::default(),
            pred_memo: FxHashMap::default(),
            pred_active: FxHashSet::default(),
            nt_pref: FxHashMap::default(),
            seq_pref: FxHashMap::default(),
            pref_active: FxHashSet::default(),
            furthest: 0,
        }
    }

    pub(super) fn furthest(&self) -> usize {
        self.furthest
    }

    /// Add the productions of `nt` that can start at `i` (FIRST/nullable filter).
    #[inline]
    fn predict(&self, chart: &mut Chart, nt: u32, i: usize) {
        let next = self.input.get(i).copied();
        for &p in &self.g.by_lhs[nt as usize] {
            let viable = self.g.prod_nullable[p as usize]
                || next.is_some_and(|c| self.g.prod_first[p as usize].contains(c));
            if viable {
                chart.add(
                    i,
                    Item {
                        prod: p,
                        dot: 0,
                        origin: i as u32,
                    },
                );
            }
        }
    }

    pub(super) fn run_main(&mut self) {
        let start = self.g.start;
        self.run(start, 0, Mode::Record);
    }

    /// Run Earley from `from` predicting `start`. In prefix mode returns true
    /// once `start` completes from `from`.
    fn run(&mut self, start: u32, from: usize, mode: Mode) -> bool {
        let n = self.input.len();
        let mut chart = Chart::new(from, n);
        chart.predicted[0].insert(start);
        self.predict(&mut chart, start, from);
        let record = matches!(mode, Mode::Record);
        let target = match mode {
            Mode::Prefix(nt) => Some(nt),
            Mode::Record => None,
        };
        for i in from..=n {
            let ki = i - from;
            if chart.sets[ki].is_empty() {
                continue;
            }
            if record {
                self.furthest = self.furthest.max(i);
            }
            let mut k = 0;
            while k < chart.sets[ki].len() {
                let item = chart.sets[ki][k];
                k += 1;
                let prod = &self.g.prods[item.prod as usize];
                if item.dot as usize == prod.rhs.len() {
                    let lhs = prod.lhs;
                    let origin = item.origin as usize;
                    if record {
                        let ends = self.done.entry((item.prod, item.origin)).or_default();
                        if !ends.contains(&(i as u32)) {
                            ends.push(i as u32);
                        }
                    }
                    if target == Some(lhs) && origin == from {
                        return true;
                    }
                    if origin == i {
                        chart.eps[ki].insert(lhs);
                    }
                    let ko = origin - from;
                    let waiting = chart.waiters[ko].get(&lhs).map_or(0, Vec::len);
                    for j in 0..waiting {
                        let w = chart.waiters[ko][&lhs][j];
                        chart.add(i, w.advance());
                    }
                    continue;
                }
                match prod.rhs[item.dot as usize] {
                    Sym::T(set) => {
                        if i < n && set.contains(self.input[i]) {
                            chart.add(i + 1, item.advance());
                        }
                    }
                    Sym::N(b) => {
                        chart.waiters[ki].entry(b).or_default().push(item);
                        if chart.predicted[ki].insert(b) {
                            self.predict(&mut chart, b, i);
                        }
                        if chart.eps[ki].contains(&b) {
                            chart.add(i, item.advance());
                        }
                    }
                    Sym::And(b) => {
                        if self.lookahead(b, i) {
                            chart.add(i, item.advance());
                        }
                    }
                    Sym::Not(b) => {
                        if !self.lookahead(b, i) {
                            chart.add(i, item.advance());
                        }
                    }
                }
            }
        }
        false
    }

    /// Does `nt` derive some prefix of `input[pos..]`?
    fn lookahead(&mut self, nt: u32, pos: usize) -> bool {
        // Cheap rejection: `nt` must consume input and cannot start here.
        if !self.g.nt_nullable[nt as usize]
            && self
                .input
                .get(pos)
                .is_none_or(|&c| !self.g.nt_first[nt as usize].contains(c))
        {
            return false;
        }
        let key = (nt, pos as u32);
        if let Some(&v) = self.pred_memo.get(&key) {
            return v;
        }
        if !self.pred_active.insert(key) {
            // Zero-width cycle through lookaheads: treat as no match.
            return false;
        }
        let v = self.run(nt, pos, Mode::Prefix(nt));
        self.pred_active.remove(&key);
        self.pred_memo.insert(key, v);
        v
    }

    pub(super) fn has_end(&self, nt: u32, start: usize, end: usize) -> bool {
        self.g.by_lhs[nt as usize].iter().any(|&p| {
            self.done
                .get(&(p, start as u32))
                .is_some_and(|ends| ends.contains(&(end as u32)))
        })
    }

    pub(super) fn preferred_end(&mut self, nt: u32, start: usize) -> Option<usize> {
        self.nt_ends(nt, start as u32).first().map(|&e| e as usize)
    }

    /// End positions of `nt` from `pos`, most PEG-preferred first.
    fn nt_ends(&mut self, nt: u32, pos: u32) -> Ends {
        if let Some(v) = self.nt_pref.get(&(nt, pos)) {
            return v.clone();
        }
        if !self.pref_active.insert((nt, pos)) {
            return Rc::new(Vec::new());
        }
        let mut out: Vec<u32> = Vec::new();
        for pi in 0..self.g.by_lhs[nt as usize].len() {
            let p = self.g.by_lhs[nt as usize][pi];
            if !self.done.contains_key(&(p, pos)) {
                continue;
            }
            let ends = self.seq_ends(p, 0, pos);
            for &e in ends.iter() {
                if !out.contains(&e) {
                    out.push(e);
                }
            }
        }
        self.pref_active.remove(&(nt, pos));
        let rc = Rc::new(out);
        self.nt_pref.insert((nt, pos), rc.clone());
        rc
    }

    /// Candidate ends of one symbol at `pos`, preferred first.
    fn sym_ends(&mut self, sym: Sym, pos: u32) -> Ends {
        match sym {
            Sym::T(set) => {
                let p = pos as usize;
                if p < self.input.len() && set.contains(self.input[p]) {
                    Rc::new(vec![pos + 1])
                } else {
                    Rc::new(Vec::new())
                }
            }
            Sym::N(b) => self.nt_ends(b, pos),
            Sym::And(b) => {
                if self.lookahead(b, pos as usize) {
                    Rc::new(vec![pos])
                } else {
                    Rc::new(Vec::new())
                }
            }
            Sym::Not(b) => {
                if self.lookahead(b, pos as usize) {
                    Rc::new(Vec::new())
                } else {
                    Rc::new(vec![pos])
                }
            }
        }
    }

    /// End positions of `prod.rhs[idx..]` from `pos`, preferred first.
    fn seq_ends(&mut self, prod: u32, idx: u32, pos: u32) -> Ends {
        let key = (prod, idx, pos);
        if let Some(v) = self.seq_pref.get(&key) {
            return v.clone();
        }
        let rhs_len = self.g.prods[prod as usize].rhs.len() as u32;
        let out = if idx == rhs_len {
            Rc::new(vec![pos])
        } else {
            let sym = self.g.prods[prod as usize].rhs[idx as usize];
            let first = self.sym_ends(sym, pos);
            let mut out: Vec<u32> = Vec::new();
            for &e in first.iter() {
                let rest = self.seq_ends(prod, idx + 1, e);
                for &r in rest.iter() {
                    if !out.contains(&r) {
                        out.push(r);
                    }
                }
            }
            Rc::new(out)
        };
        self.seq_pref.insert(key, out.clone());
        out
    }

    /// Most preferred derivation of `nt` over `[start, end)`.
    pub(super) fn derive(&mut self, nt: u32, start: usize, end: usize) -> Vec<ParseNode> {
        let (s, e) = (start as u32, end as u32);
        let mut children = Vec::new();
        for pi in 0..self.g.by_lhs[nt as usize].len() {
            let p = self.g.by_lhs[nt as usize][pi];
            if !self.done.get(&(p, s)).is_some_and(|v| v.contains(&e)) {
                continue;
            }
            if !self.seq_ends(p, 0, s).contains(&e) {
                continue;
            }
            self.split(p, s, e, &mut children);
            break;
        }
        match self.g.kinds[nt as usize] {
            NtKind::Named => vec![ParseNode::NonTerminal {
                name: self.g.names[nt as usize].clone(),
                span: Span(start, end),
                children,
            }],
            NtKind::Helper => children,
            NtKind::Literal => vec![ParseNode::Terminal {
                span: Span(start, end),
            }],
        }
    }

    fn split(&mut self, prod: u32, start: u32, end: u32, out: &mut Vec<ParseNode>) {
        let len = self.g.prods[prod as usize].rhs.len() as u32;
        let mut pos = start;
        for idx in 0..len {
            let sym = self.g.prods[prod as usize].rhs[idx as usize];
            let cands = self.sym_ends(sym, pos);
            let mut chosen = None;
            for &c in cands.iter() {
                if self.seq_ends(prod, idx + 1, c).contains(&end) {
                    chosen = Some(c);
                    break;
                }
            }
            let Some(c) = chosen else { return };
            match sym {
                Sym::T(_) => out.push(ParseNode::Terminal {
                    span: Span(pos as usize, c as usize),
                }),
                Sym::N(b) => out.extend(self.derive(b, pos as usize, c as usize)),
                Sym::And(_) | Sym::Not(_) => {}
            }
            pos = c;
        }
    }
}
