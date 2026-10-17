//! Token-level CFG for CLL Ch.21 (feature `grammar`).
//!
//! Distinct from [`crate::cll::cfg`]: morphology Earley consumes UTF-8 bytes;
//! this CFG consumes selma'o / BRIVLA / CMENE terminal ids.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::OnceLock;

pub type NtId = u16;
pub type TermId = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Nt(NtId),
    Term(TermId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Production {
    pub lhs: NtId,
    pub rhs: Vec<Symbol>,
}

/// A CLL slash-delimited terminator site. Retained after compilation so tree
/// selection distinguishes implicit closures from ordinary optional syntax.
#[derive(Debug, Clone)]
pub struct ElidableSite {
    pub terminal: String,
    pub source_rule: String,
}

#[derive(Debug, Clone)]
pub struct TokenGrammar {
    pub nt_names: Vec<String>,
    pub term_names: Vec<String>,
    pub productions: Vec<Production>,
    pub by_lhs: Vec<Vec<usize>>,
    pub start: NtId,
    /// Nonterminals corresponding to source grammar rules. Generated EBNF
    /// helpers are excluded when comparing syntax-tree structure.
    pub source_nts: HashSet<NtId>,
    pub(crate) prediction: OnceLock<super::earley::PredictionFirst>,
    pub elidables: HashMap<usize, ElidableSite>,
    pub term_by_name: HashMap<String, TermId>,
    pub nt_by_name: HashMap<String, NtId>,
}

impl TokenGrammar {
    pub fn from_parts(
        nt_names: Vec<String>,
        term_names: Vec<String>,
        productions: Vec<Production>,
        start_name: &str,
    ) -> Self {
        let nonterminal_count = nt_names.len();
        let mut by_lhs = vec![Vec::new(); nonterminal_count];
        for (index, production) in productions.iter().enumerate() {
            by_lhs[production.lhs as usize].push(index);
        }
        let nt_by_name: HashMap<String, NtId> = nt_names
            .iter()
            .enumerate()
            .map(|(index, nonterminal_count)| (nonterminal_count.clone(), index as NtId))
            .collect();
        let term_by_name: HashMap<String, TermId> = term_names
            .iter()
            .enumerate()
            .map(|(index, nonterminal_count)| (nonterminal_count.clone(), index as TermId))
            .collect();
        let start = *nt_by_name
            .get(start_name)
            .unwrap_or_else(|| panic!("missing start NT {start_name}"));
        Self {
            nt_names,
            term_names,
            productions,
            by_lhs,
            start,
            source_nts: (0..nonterminal_count).map(|index| index as NtId).collect(),
            prediction: OnceLock::new(),
            elidables: HashMap::new(),
            term_by_name,
            nt_by_name,
        }
    }

    pub fn nt(&self, name: &str) -> Option<NtId> {
        self.nt_by_name.get(name).copied()
    }

    pub fn term(&self, name: &str) -> Option<TermId> {
        self.term_by_name.get(name).copied()
    }

    pub fn n_productions(&self) -> usize {
        self.productions.len()
    }

    pub fn n_nts(&self) -> usize {
        self.nt_names.len()
    }

    pub fn n_terms(&self) -> usize {
        self.term_names.len()
    }
}

impl fmt::Display for TokenGrammar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "TokenGrammar ({} nts, {} terms, {} prods, start={}):",
            self.nt_names.len(),
            self.term_names.len(),
            self.productions.len(),
            self.nt_names[self.start as usize]
        )?;
        for production in &self.productions {
            write!(formatter, "  {} ->", self.nt_names[production.lhs as usize])?;
            if production.rhs.is_empty() {
                write!(formatter, " ε")?;
            }
            for symbol in &production.rhs {
                match symbol {
                    Symbol::Nt(nonterminal_count) => {
                        write!(formatter, " {}", self.nt_names[*nonterminal_count as usize])?
                    }
                    Symbol::Term(t) => write!(formatter, " {}", self.term_names[*t as usize])?,
                }
            }
            writeln!(formatter)?;
        }
        Ok(())
    }
}
