//! Earley chart with lookahead symbols, plus preference-ordered derivation
//! extraction.

use super::{EarleyGrammar, NtKind, Sym};
use crate::peg::annotations::delimiter_key;
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
/// Preference memos `(nt_pref, seq_pref)` of one extraction mode.
type PrefMemos = (
    FxHashMap<(u32, u32), Ends>,
    FxHashMap<(u32, u32, u32), Ends>,
);
/// Matches of one quote from a position: `(open end, middle end, close end)`,
/// preferred first.
type QuoteMatches = Rc<Vec<(u32, u32, u32)>>;

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
    /// Preference memos of the other mode (swapped in by `full_mode`).
    other_pref: PrefMemos,
    /// All end positions `(nonterminal, position)` → ascending ends, used by
    /// quotes (sub-recognitions also record into `done`).
    all_ends: FxHashMap<(u32, u32), Ends>,
    ends_active: FxHashSet<(u32, u32)>,
    quote_memo: FxHashMap<(u32, u32), QuoteMatches>,
    /// PEG mode: every nonterminal keeps only its most preferred end, so the
    /// extracted derivation is the one a PEG would build (no backtracking
    /// into a nonterminal that already matched).
    peg_mode: bool,
    furthest: usize,
    /// Lowest stack address the recursive helpers may reach (see
    /// [`crate::peg::stack`]); past it the parse is abandoned.
    stack_floor: usize,
    out_of_stack: bool,
}

enum Mode {
    /// Record every completion (main parse).
    Record,
    /// Stop as soon as `nt` completes from the starting position.
    Prefix(u32),
    /// Record every completion and collect all ends of `nt` from the start.
    Ends(u32),
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
            other_pref: Default::default(),
            all_ends: FxHashMap::default(),
            ends_active: FxHashSet::default(),
            quote_memo: FxHashMap::default(),
            peg_mode: false,
            furthest: 0,
            stack_floor: crate::peg::stack::stack_floor(None),
            out_of_stack: false,
        }
    }

    /// Switch derivation extraction to PEG mode (see `peg_mode`). Call
    /// before any extraction.
    pub(super) fn set_peg_mode(&mut self) {
        self.peg_mode = true;
    }

    /// Run `f` with PEG mode off (all ends of every nonterminal), keeping
    /// each mode's preference memos apart.
    fn full_mode<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        if !self.peg_mode {
            return f(self);
        }
        self.swap_pref();
        self.peg_mode = false;
        let r = f(self);
        self.peg_mode = true;
        self.swap_pref();
        r
    }

    fn swap_pref(&mut self) {
        std::mem::swap(&mut self.nt_pref, &mut self.other_pref.0);
        std::mem::swap(&mut self.seq_pref, &mut self.other_pref.1);
    }

    pub(super) fn furthest(&self) -> usize {
        self.furthest
    }

    /// True once the recursion got too deep for the thread's stack: the
    /// results are then incomplete and the parse must be reported failed.
    pub(super) fn out_of_stack(&self) -> bool {
        self.out_of_stack
    }

    #[inline]
    fn too_deep(&mut self) -> bool {
        if !self.out_of_stack && crate::peg::stack::stack_address() < self.stack_floor {
            self.out_of_stack = true;
        }
        self.out_of_stack
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
        self.run(start, 0, Mode::Record, &mut Vec::new());
    }

    /// Run Earley from `from` predicting `start`. In prefix mode returns true
    /// once `start` completes from `from`; in ends mode pushes every end of
    /// `start` from `from` onto `ends` (ascending).
    fn run(&mut self, start: u32, from: usize, mode: Mode, ends: &mut Vec<u32>) -> bool {
        let n = self.input.len();
        let mut chart = Chart::new(from, n);
        chart.predicted[0].insert(start);
        self.predict(&mut chart, start, from);
        let main = matches!(mode, Mode::Record);
        let record = !matches!(mode, Mode::Prefix(_));
        let (target, collect) = match mode {
            Mode::Prefix(nt) => (Some(nt), None),
            Mode::Ends(nt) => (None, Some(nt)),
            Mode::Record => (None, None),
        };
        for i in from..=n {
            let ki = i - from;
            if chart.sets[ki].is_empty() {
                continue;
            }
            if main {
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
                    if collect == Some(lhs) && origin == from && ends.last() != Some(&(i as u32)) {
                        ends.push(i as u32);
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
                    Sym::Quote(q) => {
                        let matches = self.quote_matches(q, i);
                        for &(_, _, end) in matches.iter() {
                            chart.add(end as usize, item.advance());
                        }
                    }
                }
            }
        }
        false
    }

    /// Every end of `nt` from `pos`, ascending (memoized sub-recognition).
    fn ends_of(&mut self, nt: u32, pos: usize) -> Ends {
        if !self.g.nt_nullable[nt as usize]
            && self
                .input
                .get(pos)
                .is_none_or(|&c| !self.g.nt_first[nt as usize].contains(c))
        {
            return Rc::new(Vec::new());
        }
        let key = (nt, pos as u32);
        if let Some(v) = self.all_ends.get(&key) {
            return v.clone();
        }
        if self.too_deep() {
            return Rc::new(Vec::new());
        }
        if !self.ends_active.insert(key) {
            return Rc::new(Vec::new());
        }
        let mut ends = Vec::new();
        self.run(nt, pos, Mode::Ends(nt), &mut ends);
        self.ends_active.remove(&key);
        let rc = Rc::new(ends);
        self.all_ends.insert(key, rc.clone());
        rc
    }

    /// Matches of quote `q` from `pos`: the opening delimiter, then the
    /// middle up to the **first** position where a closing delimiter with
    /// the same [`delimiter_key`] follows. Longer delimiters are preferred.
    fn quote_matches(&mut self, q: u32, pos: usize) -> QuoteMatches {
        let key = (q, pos as u32);
        if let Some(v) = self.quote_memo.get(&key) {
            return v.clone();
        }
        let quote = self.g.quotes[q as usize];
        let mut out = Vec::new();
        let opens = self.ends_of(quote.open, pos);
        for &open_end in opens.iter().rev() {
            let delimiter = delimiter_key(&self.input[pos..open_end as usize]);
            if delimiter.is_empty() {
                continue;
            }
            let middles = self.ends_of(quote.middle, open_end as usize);
            for &middle_end in middles.iter() {
                let closes = self.ends_of(quote.close, middle_end as usize);
                let before = out.len();
                for &close_end in closes.iter().rev() {
                    let word = &self.input[middle_end as usize..close_end as usize];
                    if close_end > middle_end && delimiter_key(word) == delimiter {
                        out.push((open_end, middle_end, close_end));
                    }
                }
                if out.len() > before {
                    break;
                }
            }
        }
        let rc = Rc::new(out);
        self.quote_memo.insert(key, rc.clone());
        rc
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
        if self.too_deep() {
            return false;
        }
        if !self.pred_active.insert(key) {
            // Zero-width cycle through lookaheads: treat as no match.
            return false;
        }
        let v = self.run(nt, pos, Mode::Prefix(nt), &mut Vec::new());
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
        if self.too_deep() {
            return Rc::new(Vec::new());
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
            if self.peg_mode && !out.is_empty() {
                break;
            }
        }
        if self.peg_mode {
            out.truncate(1);
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
            Sym::Quote(q) => {
                let mut ends: Vec<u32> = Vec::new();
                for &(_, _, end) in self.quote_matches(q, pos as usize).iter() {
                    if !ends.contains(&end) {
                        ends.push(end);
                    }
                }
                Rc::new(ends)
            }
        }
    }

    /// End positions of `prod.rhs[idx..]` from `pos`, preferred first.
    fn seq_ends(&mut self, prod: u32, idx: u32, pos: u32) -> Ends {
        let key = (prod, idx, pos);
        if let Some(v) = self.seq_pref.get(&key) {
            return v.clone();
        }
        if self.too_deep() {
            return Rc::new(Vec::new());
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
        if self.too_deep() {
            return children;
        }
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
                Sym::Quote(q) => {
                    let quote = self.g.quotes[q as usize];
                    let matches = self.quote_matches(q, pos as usize);
                    if let Some(&(open_end, middle_end, _)) =
                        matches.iter().find(|&&(_, _, end)| end == c)
                    {
                        let (open_end, middle_end) = (open_end as usize, middle_end as usize);
                        out.extend(self.derive(quote.open, pos as usize, open_end));
                        // The quoted words end where the matching delimiter
                        // starts, not at the middle's greedy PEG end.
                        out.extend(
                            self.full_mode(|p| p.derive(quote.middle, open_end, middle_end)),
                        );
                        out.extend(self.derive(quote.close, middle_end, c as usize));
                    }
                }
                Sym::And(_) | Sym::Not(_) => {}
            }
            pos = c;
        }
    }
}
