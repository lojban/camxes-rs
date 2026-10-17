//! Public parse pipeline: morph segment → lexeme → Ch.19 DERIVED SI/SA/SU
//! erasure → Earley over Ch.21 CFG.

use super::cfg::{NtId, TermId, TokenGrammar};
use super::earley::{recognize_starts, recognize_starts_with_tree, ParseNode};
use super::ebnf::{compile_ebnf, CompileStats};
use super::lexeme::{lexeme_stream, Lexeme, LexemeResult};
use std::sync::OnceLock;

const CLL21_EBNF: &str = include_str!("../data/cll21.ebnf");

static GRAMMAR: OnceLock<(TokenGrammar, CompileStats)> = OnceLock::new();

fn grammar_cell() -> &'static (TokenGrammar, CompileStats) {
    GRAMMAR.get_or_init(|| {
        compile_ebnf(CLL21_EBNF, "text").unwrap_or_else(|e| panic!("CLL Ch.21 EBNF compile: {e}"))
    })
}

/// Compiled CLL Ch.21 grammar (cached).
pub fn cll21_grammar() -> &'static TokenGrammar {
    &grammar_cell().0
}

pub fn cll21_stats() -> &'static CompileStats {
    &grammar_cell().1
}

#[derive(Debug, Clone)]
pub struct GrammarResult {
    pub accepted: bool,
    pub start: String,
    pub lexemes: Vec<Lexeme>,
    pub term_ids: Vec<TermId>,
    pub gaps: Vec<String>,
    /// Morph / lexeme / erasure notes (includes `DERIVED Ch.19 SI|SA|SU: …`
    /// when metalinguistic erasure ran).
    pub notes: Vec<String>,
    /// Other start symbols that also matched (sentence, …) when probed.
    pub also_matched: Vec<String>,
}

impl GrammarResult {
    /// Select a complete derivation using continuation-before-elision scope.
    /// Kept lazy so recognition-only callers do not allocate tree-selection state.
    pub fn tree(&self) -> Option<super::earley::ParseNode> {
        if !self.accepted {
            return None;
        }
        let grammar = cll21_grammar();
        super::earley::derive_one(grammar, grammar.nt(&self.start)?, &self.term_ids)
    }
}

/// Full pipeline from orthographic text, starting at Ch.21 `text`.
pub fn parse_text(input: &str) -> GrammarResult {
    parse_as(input, "text")
}

/// Parse starting at an arbitrary Ch.21 nonterminal (e.g. `sentence`, `sumti`).
pub fn parse_as(input: &str, start_name: &str) -> GrammarResult {
    parse_as_internal(input, start_name, None).0
}

/// Parse and select a tree using one Earley chart for both requested and
/// diagnostic start symbols.
pub fn parse_as_with_tree(input: &str, start_name: &str) -> (GrammarResult, Option<ParseNode>) {
    parse_as_internal(input, start_name, Some(usize::MAX))
}

/// The web view omits very large trees; avoid allocating their tree-selection state.
#[cfg(test)]
pub(crate) fn parse_as_with_tree_up_to(
    input: &str,
    start_name: &str,
    max_lexemes: usize,
) -> (GrammarResult, Option<ParseNode>) {
    parse_as_internal(input, start_name, Some(max_lexemes))
}

fn parse_as_internal(
    input: &str,
    start_name: &str,
    tree_lexeme_limit: Option<usize>,
) -> (GrammarResult, Option<ParseNode>) {
    let grammar = cll21_grammar();
    let lexed = lexeme_stream(input);
    let mut notes = lexed.morph_notes.clone();
    let mut gaps = lexed.gaps.clone();

    let (term_ids, map_notes) = map_lexemes_to_terms(grammar, &lexed.lexemes);
    notes.extend(map_notes);

    let start = match grammar.nt(start_name) {
        Some(id) => id,
        None => {
            gaps.push(format!("unknown start NT {start_name}"));
            return (
                GrammarResult {
                    accepted: false,
                    start: start_name.into(),
                    lexemes: lexed.lexemes,
                    term_ids,
                    gaps,
                    notes,
                    also_matched: vec![],
                },
                None,
            );
        }
    };

    // Refuse if any GAP/UNKNOWN terminals slipped through
    let has_bad = lexed
        .lexemes
        .iter()
        .any(|l| l.terminal == "GAP" || l.terminal == "UNKNOWN_CMAVO");
    if has_bad {
        gaps.push("syntax skipped: lexeme layer emitted GAP/UNKNOWN_CMAVO".into());
        return (
            GrammarResult {
                accepted: false,
                start: start_name.into(),
                lexemes: lexed.lexemes,
                term_ids,
                gaps,
                notes,
                also_matched: vec![],
            },
            None,
        );
    }

    // Recognition and diagnostic starts share one Earley chart.
    let probe = ["text", "sentence", "sumti", "selbri", "fragment"];
    let mut ids: Vec<NtId> = vec![start];
    let mut probes: Vec<(&str, NtId)> = Vec::new();
    for candidate_start_name in probe {
        if let Some(id) = grammar.nt(candidate_start_name) {
            if !ids.contains(&id) {
                ids.push(id);
            }
            probes.push((candidate_start_name, id));
        }
    }
    let (matched, tree) = if tree_lexeme_limit.is_some_and(|limit| lexed.lexemes.len() <= limit) {
        recognize_starts_with_tree(grammar, &ids, start, &term_ids)
    } else {
        (recognize_starts(grammar, &ids, &term_ids), None)
    };
    let accepted = matched.contains(&start);
    let also_matched: Vec<String> = probes
        .iter()
        .filter(|(_, id)| matched.contains(id))
        .map(|(candidate_start_name, _)| (*candidate_start_name).to_string())
        .filter(|candidate_start_name| candidate_start_name != start_name)
        .collect();

    (
        GrammarResult {
            accepted,
            start: start_name.into(),
            lexemes: lexed.lexemes,
            term_ids,
            gaps,
            notes,
            also_matched,
        },
        tree,
    )
}

fn map_lexemes_to_terms(grammar: &TokenGrammar, lexemes: &[Lexeme]) -> (Vec<TermId>, Vec<String>) {
    let mut ids = Vec::with_capacity(lexemes.len());
    let mut notes = Vec::new();
    for lex in lexemes {
        match grammar.term(&lex.terminal) {
            Some(id) => ids.push(id),
            None => {
                notes.push(format!(
                    "terminal {} not in Ch.21 term set — inserting as unmatched",
                    lex.terminal
                ));
                // Use a sentinel: if term missing, invent won't work mid-parse;
                // push an impossible id by allocating? Better skip and fail.
                // Allocate into a side channel — for recognition, use max TermId
                // that won't match any production.
                ids.push(TermId::MAX);
            }
        }
    }
    (ids, notes)
}

/// Expose lexeme layer alone.
pub fn lexeme(input: &str) -> LexemeResult {
    lexeme_stream(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_chart_preserves_selected_trees_and_diagnostics() {
        for (input, start) in [
            ("mi viska le gerku", "text"),
            ("le broda joi le brodi", "text"),
            ("mi djica le nu do klama le zarci", "text"),
            ("le xamgu be do noi barda cu zdani", "text"),
            ("li su'i pa re ci", "sumti"),
            ("mi viska le", "text"),
        ] {
            let ordinary = parse_as(input, start);
            let (shared, tree) = parse_as_with_tree(input, start);
            assert_eq!(shared.accepted, ordinary.accepted, "{input}");
            assert_eq!(shared.also_matched, ordinary.also_matched, "{input}");
            assert_eq!(tree, ordinary.tree(), "{input}");
        }
        let (result, tree) = parse_as_with_tree_up_to("mi viska le gerku", "text", 1);
        assert!(result.accepted);
        assert!(tree.is_none());
    }

    #[test]
    fn grammar_loads_full_ebnf() {
        let st = cll21_stats();
        assert_eq!(st.source_rules, 88, "{st:?}");
        assert!(
            st.cfg_productions > 200,
            "expected expanded CFG; got {} prods",
            st.cfg_productions
        );
        assert!(st.hash_sites > 0);
        assert!(st.elidable_sites > 0);
        assert!(st.and_expansions > 0);
        let grammar = cll21_grammar();
        assert_eq!(grammar.source_nts.len(), st.source_rules);
        assert!(grammar.nt("text").is_some());
        assert!(grammar.nt("sentence").is_some());
        assert!(grammar.term("BRIVLA").is_some());
        assert!(grammar.term("CMENE").is_some());
        assert!(grammar.term("KOhA").is_some());
        assert!(grammar.term("LE").is_some());
        assert!(grammar.term("CU").is_some());
        eprintln!(
            "CLL21 CFG: rules={} prods={} nts={} terms={} &#{} //{} &{} ...{}",
            st.source_rules,
            st.cfg_productions,
            st.nonterminals,
            st.terminals,
            st.hash_sites,
            st.elidable_sites,
            st.and_expansions,
            st.plus_sites
        );
    }

    #[test]
    fn smoke_mi_viska_le_gerku() {
        let r = parse_text("mi viska le gerku");
        assert!(
            r.accepted,
            "expected accept; lexemes={:?} gaps={:?} notes={:?}",
            r.lexemes, r.gaps, r.notes
        );
        let terms: Vec<_> = r.lexemes.iter().map(|l| l.terminal.as_str()).collect();
        assert_eq!(terms, ["KOhA", "BRIVLA", "LE", "BRIVLA"]);
    }

    #[test]
    fn smoke_with_cu() {
        let r = parse_text("mi cu viska le gerku");
        assert!(r.accepted, "{r:?}");
    }

    #[test]
    fn sumti_and_selbri_fragments() {
        let s = parse_as("le gerku", "sumti");
        assert!(s.accepted, "sumti: {s:?}");
        let selbri_result = parse_as("viska", "selbri");
        assert!(selbri_result.accepted, "selbri: {selbri_result:?}");
        let sent = parse_as("mi viska", "sentence");
        assert!(sent.accepted, "sentence: {sent:?}");
    }

    #[test]
    fn reject_scrambled() {
        let r = parse_text("gerku mi le");
        // Morphology may still lex; syntax should fail as text
        assert!(
            !r.accepted,
            "scrambled stream should not be text; got {r:?}"
        );
    }

    #[test]
    fn lexeme_roundtrip_cmavo() {
        let t = crate::cll::grammar::lexeme::terminals_only("mi le cu");
        assert_eq!(t, vec!["KOhA", "LE", "CU"]);
        // Mid-stream indicators stripped before Earley (DERIVED Ch.21 `word` rule).
        let stripped = crate::cll::grammar::lexeme::terminals_only("mi le cu ui nai");
        assert_eq!(stripped, vec!["KOhA", "LE", "CU"]);
        // Leading indicators kept for `text` prefix.
        let lead = crate::cll::grammar::lexeme::terminals_only("ui nai mi");
        assert_eq!(lead, vec!["UI", "NAI", "KOhA"]);
        let attached = parse_text("mi au");
        assert!(attached.accepted);
        assert_eq!(attached.lexemes.len(), 1);
        assert_eq!(attached.lexemes[0].surface, "mi");
        assert_eq!(
            attached.lexemes[0].attached_indicators,
            vec![("au".into(), "UI".into())]
        );
    }
}
