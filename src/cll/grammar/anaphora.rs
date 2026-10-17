//! Discourse annotations derived from the selected CLL tree, not grammar rules.
//! CLL §§7.5, 7.13, 8.5 and 17.9: letteral defaults, explicit definitions,
//! and cancellation. Multi-letter initials follow the name/tanru sequence.
use super::{earley::ParseNode, CmavoLexicon, Lexeme};

#[derive(Debug, PartialEq, Eq)]
pub struct LetteralReference {
    pub source_start: usize,
    pub source_end: usize,
    pub target: Option<(usize, usize, String)>,
    pub assigned: bool,
}

struct Entry<'a> {
    node: &'a ParseNode,
    ancestors: Vec<&'a ParseNode>,
    quoted: bool,
}

fn main_terminals<'a>(node: &'a ParseNode, out: &mut Vec<&'a ParseNode>) {
    for child in &node.children {
        if matches!(child.symbol.as_str(), "sumti" | "relative-clauses") {
            continue;
        }
        if child.terminal {
            out.push(child);
        } else {
            main_terminals(child, out);
        }
    }
}

fn entries<'a>(
    node: &'a ParseNode,
    ancestors: &mut Vec<&'a ParseNode>,
    quoted: bool,
    out: &mut Vec<Entry<'a>>,
) {
    let mut terminals = Vec::new();
    if node.symbol == "sumti-6" {
        main_terminals(node, &mut terminals);
    }
    let quoted = quoted
        || terminals
            .first()
            .is_some_and(|n| matches!(n.symbol.as_str(), "LU" | "LOhU" | "ZOI" | "ZO"));
    out.push(Entry {
        node,
        ancestors: ancestors.clone(),
        quoted,
    });
    ancestors.push(node);
    for child in &node.children {
        entries(child, ancestors, quoted, out);
    }
    ancestors.pop();
}

fn normalized(surface: &str) -> String {
    surface
        .to_lowercase()
        .chars()
        .filter(|c| !matches!(c, '.' | ',' | ' '))
        .collect()
}

fn letter(surface: &str) -> Option<char> {
    let s = normalized(surface);
    let chars: Vec<_> = s.chars().collect();
    match chars.as_slice() {
        [c, 'y'] if "bcdfgjklmnprstvxz".contains(*c) => Some(*c),
        [c, 'b', 'u'] if "aeiou".contains(*c) => Some(*c),
        _ => None,
    }
}

fn letter_key(node: &ParseNode, lexemes: &[Lexeme]) -> Option<String> {
    let mut terminals = Vec::new();
    main_terminals(node, &mut terminals);
    let letters: Vec<_> = terminals
        .iter()
        .filter(|n| !matches!(n.symbol.as_str(), "BOI" | "KU"))
        .collect();
    if letters.len() == 2 && letters[1].symbol == "BU" {
        let surface = normalized(&lexemes.get(letters[0].start)?.surface);
        if CmavoLexicon::global().selmaho(&surface) == Some("NU") {
            return Some(format!("NU:{surface}"));
        }
    }
    let mut key = String::new();
    let mut terminals = terminals.into_iter().peekable();
    while let Some(terminal) = terminals.next() {
        if terminals.peek().is_some_and(|next| next.symbol == "BU") {
            terminals.next();
            key.push(letter(&format!(
                "{}bu",
                normalized(&lexemes.get(terminal.start)?.surface)
            ))?);
            continue;
        }
        match terminal.symbol.as_str() {
            "BY" => key.push(letter(&lexemes.get(terminal.start)?.surface)?),
            "BOI" | "KU" => (),
            _ => return None,
        }
    }
    (!key.is_empty()).then_some(key)
}

fn sumti_key(node: &ParseNode, lexemes: &[Lexeme]) -> Option<String> {
    let mut terminals = Vec::new();
    main_terminals(node, &mut terminals);
    if terminals.len() == 1 && terminals[0].symbol == "KOhA" {
        let key = normalized(&lexemes.get(terminals[0].start)?.surface);
        if matches!(
            key.as_str(),
            "ko'a" | "ko'e" | "ko'i" | "ko'o" | "ko'u" | "fo'a" | "fo'e" | "fo'i" | "fo'o" | "fo'u"
        ) {
            return Some(key);
        }
    }
    letter_key(node, lexemes)
}

fn initials(node: &ParseNode, lexemes: &[Lexeme]) -> Option<String> {
    let mut terminals = Vec::new();
    main_terminals(node, &mut terminals);
    if !terminals
        .iter()
        .any(|n| matches!(n.symbol.as_str(), "LA" | "LE"))
    {
        return None;
    }
    // Ignore nested descriptions and abstraction bodies. Their predicates are
    // not tanru units of this description's selbri.
    fn heads(node: &ParseNode, lexemes: &[Lexeme], out: &mut String) {
        if matches!(node.symbol.as_str(), "sumti" | "relative-clauses") {
            return;
        }
        if node.symbol == "tanru-unit-2" {
            let mut ts = Vec::new();
            main_terminals(node, &mut ts);
            if ts.first().is_some_and(|n| n.symbol == "NU") {
                return;
            }
        }
        if node.terminal && matches!(node.symbol.as_str(), "BRIVLA" | "CMENE") {
            if let Some(c) = lexemes
                .get(node.start)
                .and_then(|l| normalized(&l.surface).chars().next())
            {
                out.push(c);
            }
        } else {
            for child in &node.children {
                heads(child, lexemes, out);
            }
        }
    }
    let mut key = String::new();
    for child in &node.children {
        heads(child, lexemes, &mut key);
    }
    (!key.is_empty()).then_some(key)
}

fn abstraction_head(node: &ParseNode, lexemes: &[Lexeme]) -> Option<String> {
    // Find this description's selbri without entering nested sumti or relatives.
    fn selbri(node: &ParseNode) -> Option<&ParseNode> {
        if matches!(node.symbol.as_str(), "sumti" | "relative-clauses") {
            return None;
        }
        if node.symbol == "selbri" {
            return Some(node);
        }
        node.children.iter().find_map(selbri)
    }
    let mut terminals = Vec::new();
    main_terminals(node, &mut terminals);
    if !terminals
        .iter()
        .any(|n| matches!(n.symbol.as_str(), "LA" | "LE"))
    {
        return None;
    }
    let head = node.children.iter().find_map(selbri)?;
    terminals.clear();
    main_terminals(head, &mut terminals);
    let first = terminals.first()?;
    (first.symbol == "NU").then(|| normalized(&lexemes[first.start].surface))
}

/// Lexeme boundaries after unquoted da'o, including non-formal word attachments.
pub fn reference_resets(tree: &ParseNode, lexemes: &[Lexeme]) -> Vec<usize> {
    let mut nodes = Vec::new();
    entries(tree, &mut Vec::new(), false, &mut nodes);
    let mut resets = Vec::new();
    for entry in nodes.iter().filter(|e| {
        e.node.terminal && (!e.quoted || matches!(e.node.symbol.as_str(), "LIhU" | "LEhU"))
    }) {
        if let Some(lexeme) = lexemes.get(entry.node.start) {
            if lexeme.terminal == "DAhO"
                || lexeme
                    .attached_indicators
                    .iter()
                    .any(|(_, terminal)| terminal == "DAhO")
            {
                resets.push(entry.node.end);
            }
        }
    }
    resets.sort_unstable();
    resets.dedup();
    resets
}

fn sumti_references(tree: &ParseNode, lexemes: &[Lexeme]) -> Vec<LetteralReference> {
    let mut nodes = Vec::new();
    entries(tree, &mut Vec::new(), false, &mut nodes);
    let sumti: Vec<_> = nodes
        .iter()
        .filter(|e| e.node.symbol == "sumti" && !e.quoted)
        .map(|e| e.node)
        .collect();
    let resets = reference_resets(tree, lexemes);
    let mut assignments = Vec::new();
    let mut definitions = Vec::new();
    for entry in nodes.iter().filter(|e| e.node.terminal && !e.quoted) {
        let Some(outer) = entry
            .ancestors
            .iter()
            .rev()
            .find(|n| n.symbol == "sumti")
            .copied()
        else {
            continue;
        };
        let surface = lexemes
            .get(entry.node.start)
            .map(|l| normalized(&l.surface))
            .unwrap_or_default();
        if entry.node.symbol == "GOI" && surface == "goi" {
            let inner = sumti
                .iter()
                .find(|n| n.start >= entry.node.end && n.end <= outer.end)
                .copied();
            if let Some(inner) = inner {
                if let Some(key) = sumti_key(inner, lexemes) {
                    assignments.push((key, inner.end, outer));
                    definitions.push(inner.start);
                } else if let Some(key) = sumti_key(outer, lexemes) {
                    assignments.push((key, inner.end, inner));
                    definitions.push(outer.start);
                }
            }
        } else if entry.node.symbol == "NOI" && surface == "voi" {
            // A described assignable pro-sumti supplies its own referent; don't replace it
            // with the heuristic antecedent for the same initials.
            if let Some(key) = sumti_key(outer, lexemes) {
                assignments.push((key, outer.end, outer));
                definitions.push(outer.start);
            }
        }
    }
    assignments.sort_by_key(|(_, at, _)| *at);
    let mut links = Vec::new();
    for own in &sumti {
        let Some(key) = sumti_key(own, lexemes) else {
            continue;
        };
        if definitions.contains(&own.start) {
            continue;
        }
        let reset = resets
            .iter()
            .copied()
            .filter(|at| *at <= own.start)
            .max()
            .unwrap_or(0);
        let assignment = assignments.iter().rev().find(|(assigned_key, at, _)| {
            assigned_key == &key && *at <= own.start && (reset == 0 || *at > reset)
        });
        let target = assignment.map(|(_, _, target)| *target).or_else(|| {
            sumti
                .iter()
                .copied()
                .filter(|n| {
                    n.end <= own.start
                        && n.start >= reset
                        && if let Some(marker) = key.strip_prefix("NU:") {
                            abstraction_head(n, lexemes).as_deref() == Some(marker)
                        } else {
                            initials(n, lexemes).is_some_and(|initials| initials.starts_with(&key))
                        }
                })
                .max_by_key(|n| (n.end, n.start))
        });
        links.push(LetteralReference {
            source_start: own.start,
            source_end: own.end,
            target: target.map(|n| (n.start, n.end, n.symbol.clone())),
            assigned: assignment.is_some(),
        });
    }
    links
}

/// Letteral references, including defining voi clauses and goi bindings.
pub fn letteral_references(tree: &ParseNode, lexemes: &[Lexeme]) -> Vec<LetteralReference> {
    sumti_references(tree, lexemes)
        .into_iter()
        .filter(|reference| lexemes[reference.source_start].terminal != "KOhA")
        .collect()
}

/// The ten ko'a/fo'a-series pro-sumti share the defining goi/voi mechanism
/// with letterals, but have no initial-letter fallback (CLL §§7.5, 8.5).
pub fn pro_sumti_references(tree: &ParseNode, lexemes: &[Lexeme]) -> Vec<LetteralReference> {
    sumti_references(tree, lexemes)
        .into_iter()
        .filter(|reference| lexemes[reference.source_start].terminal == "KOhA")
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProBridiReference {
    pub source_start: usize,
    pub source_end: usize,
    pub target: Option<(usize, usize, String)>,
}

fn broda_word(node: &ParseNode, lexemes: &[Lexeme]) -> Option<String> {
    let surface = normalized(&lexemes.get(node.start)?.surface);
    (node.terminal
        && node.symbol == "BRIVLA"
        && matches!(
            surface.as_str(),
            "broda" | "brode" | "brodi" | "brodo" | "brodu"
        ))
    .then_some(surface)
}

fn bare_broda<'a>(unit: &'a ParseNode, lexemes: &[Lexeme]) -> Option<(&'a ParseNode, String)> {
    let mut terminals = Vec::new();
    main_terminals(unit, &mut terminals);
    if terminals.len() != 1 {
        return None;
    }
    broda_word(terminals[0], lexemes).map(|key| (terminals[0], key))
}

/// CLL §7.5: cei binds the broda series to a bridi, including its arguments,
/// tense and negation. Links retain the bridi rather than substituting text.
/// A description/selbri fragment has no independent sentence, so its local
/// selbri is the available antecedent instead of the enclosing main bridi.
pub fn pro_bridi_references(tree: &ParseNode, lexemes: &[Lexeme]) -> Vec<ProBridiReference> {
    let mut nodes = Vec::new();
    entries(tree, &mut Vec::new(), false, &mut nodes);
    let resets = reference_resets(tree, lexemes);
    let mut assignments = Vec::new();
    let mut definitions = Vec::new();
    for entry in nodes
        .iter()
        .filter(|e| e.node.terminal && e.node.symbol == "CEI" && !e.quoted)
    {
        let Some(tanru) = entry
            .ancestors
            .iter()
            .rev()
            .find(|n| n.symbol == "tanru-unit")
            .copied()
        else {
            continue;
        };
        let units: Vec<_> = nodes
            .iter()
            .filter(|e| {
                e.node.symbol == "tanru-unit-1"
                    && e.ancestors
                        .iter()
                        .rev()
                        .find(|n| n.symbol == "tanru-unit")
                        .is_some_and(|parent| std::ptr::eq(*parent, tanru))
            })
            .map(|e| e.node)
            .collect();
        let before = units
            .iter()
            .copied()
            .filter(|n| n.end <= entry.node.start)
            .max_by_key(|n| n.end);
        let after = units
            .iter()
            .copied()
            .filter(|n| n.start >= entry.node.end)
            .min_by_key(|n| n.start);
        let (Some(before), Some(after)) = (before, after) else {
            continue;
        };
        // The named pro-bridi may occur on either side of cei. In a chain,
        // broda cei brode defines the right-hand name using the left bridi.
        let Some((definition, key)) =
            bare_broda(after, lexemes).or_else(|| bare_broda(before, lexemes))
        else {
            continue;
        };
        let context = entry
            .ancestors
            .iter()
            .rev()
            .find(|n| matches!(n.symbol.as_str(), "sentence" | "sumti-tail"));
        let target = if context.is_some_and(|n| n.symbol == "sentence") {
            context.copied()
        } else {
            entry
                .ancestors
                .iter()
                .rev()
                .find(|n| n.symbol == "selbri")
                .copied()
        }
        .unwrap_or(tanru);
        assignments.push((key, after.end, target));
        definitions.push(definition.start);
    }
    assignments.sort_by_key(|(_, at, _)| *at);
    nodes
        .iter()
        .filter(|e| !e.quoted && !definitions.contains(&e.node.start))
        .filter_map(|entry| {
            let key = broda_word(entry.node, lexemes)?;
            let reset = resets
                .iter()
                .copied()
                .filter(|at| *at <= entry.node.start)
                .max()
                .unwrap_or(0);
            let target = assignments
                .iter()
                .rev()
                .find(|(assigned, at, _)| {
                    assigned == &key && *at <= entry.node.start && (reset == 0 || *at > reset)
                })
                .map(|(_, _, target)| (target.start, target.end, target.symbol.clone()));
            Some(ProBridiReference {
                source_start: entry.node.start,
                source_end: entry.node.end,
                target,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sumti_targets(input: &str) -> Vec<Option<String>> {
        let (parsed, tree) = crate::cll::grammar::parse_as_with_tree(input, "text");
        assert!(parsed.accepted, "{input}: {:?}", parsed.gaps);
        pro_sumti_references(&tree.expect("selected derivation"), &parsed.lexemes)
            .into_iter()
            .map(|r| {
                r.target.map(|(start, end, _)| {
                    parsed.lexemes[start..end]
                        .iter()
                        .map(|l| l.surface.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
            })
            .collect()
    }

    #[test]
    fn voi_defines_all_ten_assignable_pro_sumti() {
        for key in [
            "ko'a", "ko'e", "ko'i", "ko'o", "ko'u", "fo'a", "fo'e", "fo'i", "fo'o", "fo'u",
        ] {
            let definition = format!("{key} voi prenu ku'o");
            assert_eq!(
                sumti_targets(&format!("{definition} cu klama .i {key} cu melbi")),
                vec![Some(definition)]
            );
        }
    }

    #[test]
    fn pro_sumti_definitions_rebind_and_cancel() {
        assert_eq!(
            sumti_targets(
                "ko'a voi prenu ku'o cu klama .i ko'a voi gerku ku'o cu cadzu .i ko'a cu melbi"
            ),
            vec![Some("ko'a voi gerku ku'o".into())]
        );
        assert_eq!(
            sumti_targets("ko'a voi prenu ku'o cu klama da'o .i ko'a cu melbi"),
            vec![None]
        );
        assert_eq!(
            sumti_targets("la .alis. goi ko'a cu klama .i ko'a cu melbi"),
            vec![Some("la alis goi ko'a".into())]
        );
        assert_eq!(
            sumti_targets("ko'a goi la .alis. cu klama .i ko'a cu melbi"),
            vec![Some("la alis".into())]
        );
    }

    #[test]
    fn incidental_and_veridical_clauses_do_not_define_pro_sumti() {
        assert_eq!(
            sumti_targets("ko'a poi prenu ku'o cu klama .i ko'a cu melbi"),
            vec![None, None]
        );
        assert_eq!(
            sumti_targets("ko'a noi prenu ku'o cu klama .i ko'a cu melbi"),
            vec![None, None]
        );
        assert_eq!(
            sumti_targets("mi cusku lu ko'a voi prenu ku'o cu klama li'u .i ko'a cu melbi"),
            vec![None]
        );
    }

    fn bridi_targets(input: &str) -> Vec<Option<String>> {
        let (parsed, tree) = crate::cll::grammar::parse_as_with_tree(input, "text");
        assert!(parsed.accepted, "{input}: {:?}", parsed.gaps);
        pro_bridi_references(&tree.expect("selected derivation"), &parsed.lexemes)
            .into_iter()
            .map(|r| {
                r.target.map(|(start, end, _)| {
                    parsed.lexemes[start..end]
                        .iter()
                        .map(|l| l.surface.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
            })
            .collect()
    }

    #[test]
    fn cei_binds_all_five_pro_bridi_to_the_whole_bridi() {
        for key in ["broda", "brode", "brodi", "brodo", "brodu"] {
            let antecedent = format!("mi klama cei {key} le zarci");
            assert_eq!(
                bridi_targets(&format!("{antecedent} .i do {key}")),
                vec![Some(antecedent)]
            );
        }
    }

    #[test]
    fn cei_chains_rebinds_and_supports_either_direction() {
        assert_eq!(
            bridi_targets("mi klama cei broda cei brode .i do broda .i do brode"),
            vec![Some("mi klama cei broda cei brode".into()); 2]
        );
        assert_eq!(
            bridi_targets("mi klama cei broda .i do cadzu cei broda .i mi broda"),
            vec![Some("do cadzu cei broda".into())]
        );
        assert_eq!(
            bridi_targets("mi broda cei klama .i do broda"),
            vec![Some("mi broda cei klama".into())]
        );
    }

    #[test]
    fn pro_bridi_preserve_tense_negation_and_argument_context() {
        assert_eq!(
            bridi_targets("mi pu na klama cei brode le zarci .i do brode"),
            vec![Some("mi pu na klama cei brode le zarci".into())]
        );
        assert_eq!(
            bridi_targets("mi klama le zarci .i go'i cei broda .i do broda"),
            vec![Some("go'i cei broda".into())]
        );
    }

    #[test]
    fn cei_in_a_description_uses_its_local_selbri() {
        assert_eq!(
            bridi_targets("le klama cei broda cu melbi .i le broda cu blanu"),
            vec![Some("klama cei broda".into())]
        );
        assert_eq!(
            bridi_targets(
                "ti slasi je mlatu bo cidja lante gacri cei broda .i le crino broda cu barda"
            ),
            vec![Some(
                "ti slasi je mlatu bo cidja lante gacri cei broda".into()
            )]
        );
    }

    #[test]
    fn unassigned_or_cancelled_pro_bridi_have_no_antecedent() {
        assert_eq!(bridi_targets("mi broda .i do brode"), vec![None, None]);
        assert_eq!(
            bridi_targets("mi klama cei broda da'o .i do broda"),
            vec![None]
        );
        assert_eq!(
            bridi_targets("mi klama cei broda .i da'o do broda"),
            vec![None]
        );
        assert_eq!(
            bridi_targets("mi klama cei broda da'o .i do cadzu cei broda .i mi broda"),
            vec![Some("do cadzu cei broda".into())]
        );
    }

    #[test]
    fn quoted_cei_and_broda_do_not_affect_outer_discourse() {
        assert_eq!(
            bridi_targets("mi klama cei broda .i mi cusku lu do cadzu cei broda li'u .i do broda"),
            vec![Some("mi klama cei broda".into())]
        );
        assert_eq!(bridi_targets("mi cusku zo broda"), vec![]);
    }
    fn targets(input: &str) -> Vec<Option<String>> {
        let (parsed, tree) = crate::cll::grammar::parse_as_with_tree(input, "text");
        assert!(parsed.accepted, "{input}: {:?}", parsed.gaps);
        let tree = tree.expect("selected derivation");
        letteral_references(&tree, &parsed.lexemes)
            .into_iter()
            .map(|r| {
                r.target.map(|(start, end, _)| {
                    parsed.lexemes[start..end]
                        .iter()
                        .map(|l| l.surface.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
            })
            .collect()
    }
    #[test]
    fn names_and_tanru_initials() {
        for (description, letteral) in [
            ("la .djan. .braun.", "dy by"),
            ("la gleki", "gy"),
            ("la gleki prenu", "gy py"),
            ("le gleki prenu", "gy py"),
        ] {
            let links = targets(&format!("{description} cu klama .i {letteral} cu melbi"));
            assert_eq!(links.len(), 1);
            assert!(
                links[0]
                    .as_ref()
                    .is_some_and(|t| normalized(t) == normalized(description)),
                "{links:?}"
            );
        }
    }
    #[test]
    fn latest_matching_complete_sumti() {
        assert_eq!(
            targets("la gleki cu klama .i la gerku cu cadzu .i gy cu melbi")
                .last()
                .unwrap()
                .as_deref(),
            Some("la gerku")
        );
    }
    #[test]
    fn nu_letterals_match_the_exact_abstraction_marker() {
        for marker in [
            "nu", "ka", "si'o", "du'u", "jei", "li'i", "mu'e", "ni", "pu'u", "su'u", "za'i", "zu'o",
        ] {
            let description = format!("lo {marker} mi klama kei ku");
            assert_eq!(
                targets(&format!("{description} cu melbi .i {marker} bu cu pluka")),
                vec![Some(description)]
            );
        }
        assert_eq!(
            targets("lo nu mi klama kei ku cu melbi .i lo ka do cadzu kei ku cu pluka .i nu bu cu barda"),
            vec![Some("lo nu mi klama kei ku".into())]
        );
        assert_eq!(
            targets("lo ka mi klama kei ku cu melbi .i nu bu cu pluka"),
            vec![None]
        );
    }

    #[test]
    fn nu_letterals_only_use_completed_descriptions_with_a_leading_nu() {
        assert_eq!(
            targets("lo nu mi klama kei ku cu melbi .i lo nu nu bu cu pluka kei ku cu barda .i nu bu cu cadzu"),
            vec![Some("lo nu mi klama kei ku".into()), Some("lo nu nu bu cu pluka kei ku".into())]
        );
        for description in [
            "lo nupre",
            "lo melbi nu mi klama kei ku",
            "lo se nu mi klama kei ku",
        ] {
            assert_eq!(
                targets(&format!("{description} cu melbi .i nu bu cu pluka")),
                vec![None]
            );
        }
    }

    #[test]
    fn nu_letteral_assignments_override_defaults_and_daho_cancels_them() {
        for assignment in [
            "la mlatu goi nu bu",
            "nu bu goi la mlatu",
            "nu bu voi mlatu ku'o",
        ] {
            let links = targets(&format!(
                "{assignment} cu melbi .i lo nu mi klama kei ku cu pluka .i nu bu cu barda"
            ));
            assert!(
                links
                    .last()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|t| t.contains("mlatu")),
                "{links:?}"
            );
            assert_eq!(
                targets(&format!("{assignment} cu melbi da'o .i lo nu mi klama kei ku cu pluka .i nu bu cu barda")).last().unwrap().as_deref(),
                Some("lo nu mi klama kei ku")
            );
        }
        assert_eq!(
            targets("lo nu mi klama kei ku cu melbi da'o .i nu bu cu pluka"),
            vec![None]
        );
    }
    #[test]
    fn explicit_goi_overrides_newer_initials_in_either_direction() {
        for assigned in ["la mlatu goi gy", "gy goi la mlatu"] {
            let links = targets(&format!(
                "{assigned} cu klama .i la gerku cu cadzu .i gy cu melbi"
            ));
            assert!(
                links
                    .last()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|t| t.contains("mlatu")),
                "{links:?}"
            );
        }
    }
    #[test]
    fn voi_definition_and_daho_cancellation() {
        assert!(
            targets("gy voi mlatu cu klama .i la gerku cu cadzu .i gy cu melbi")
                .last()
                .unwrap()
                .as_ref()
                .is_some_and(|t| t.contains("voi mlatu"))
        );
        assert_eq!(
            targets("la mlatu goi gy cu klama da'o .i gy cu melbi").last(),
            Some(&None)
        );
        assert_eq!(
            targets("la gleki cu klama da'o .i gy cu melbi").last(),
            Some(&None)
        );
        assert_eq!(
            targets("la mlatu goi gy cu klama da'o .i la gerku cu cadzu .i gy cu melbi")
                .last()
                .unwrap()
                .as_deref(),
            Some("la gerku")
        );
    }

    #[test]
    fn quotation_does_not_change_outer_assignments_or_defaults() {
        let links =
            targets("la mlatu goi gy cu cusku lu la gerku cu klama da'o li'u .i gy cu melbi");
        assert!(links
            .last()
            .unwrap()
            .as_ref()
            .is_some_and(|t| t.contains("mlatu")));
        assert_eq!(
            targets("mi cusku lu la gerku cu klama li'u .i gy cu melbi").last(),
            Some(&None)
        );
        assert_eq!(
            targets("la mlatu goi gy cu cusku lu mi klama li'u da'o .i gy cu melbi").last(),
            Some(&None)
        );
    }

    #[test]
    fn assignments_after_cancellation_and_rebinding() {
        assert!(targets("la mlatu da'o goi gy cu klama .i gy cu melbi")
            .last()
            .unwrap()
            .as_ref()
            .is_some_and(|t| t.contains("mlatu")));
        assert!(
            targets("la mlatu goi gy cu klama .i la prenu goi gy cu cadzu .i gy cu melbi")
                .last()
                .unwrap()
                .as_ref()
                .is_some_and(|t| t.contains("prenu"))
        );
        assert_eq!(
            targets("la mlatu goi gy da'o cu klama .i gy cu melbi").last(),
            Some(&None)
        );
    }

    #[test]
    fn vowel_and_single_letter_defaults_do_not_match_arbitrary_word_prefixes() {
        assert_eq!(
            targets("la .alis. cu klama .i .abu cu melbi")
                .last()
                .unwrap()
                .as_deref(),
            Some("la alis")
        );
        assert_eq!(
            targets("la gleki prenu cu klama .i gy cu melbi")
                .last()
                .unwrap()
                .as_deref(),
            Some("la gleki prenu")
        );
        assert_eq!(
            targets("la gleki cu klama .i gy ly cu melbi").last(),
            Some(&None)
        );
        assert_eq!(
            targets("lo nu la gerku cu klama kei cu melbi .i gy cu cadzu")
                .last()
                .unwrap()
                .as_deref(),
            Some("la gerku")
        );
    }
}
