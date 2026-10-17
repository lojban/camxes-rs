//! ilmentufa-style rule annotations (`rule <- body #: PUSH`).
//!
//! camxes grammars mark the `zoi`/`la'o` delimiter rules with `#: PUSH`
//! (opening delimiter), `#: PEEK-DIFF` (quoted word, must differ from the
//! delimiter) and `#: POP-EQ` (closing delimiter, must equal it). Both
//! engines honor them, comparing words by [`delimiter_key`]:
//!
//! * PEG: a delimiter stack. A PUSH rule pushes the key of its match, a
//!   PEEK-DIFF rule fails when its match equals the top, a POP-EQ rule fails
//!   unless its match equals the top and then pops it. The stack is restored
//!   when a sequence fails and after predicates, and PEEK-DIFF / POP-EQ rules
//!   are not memoized (their result depends on the stack).
//! * Earley: each PUSH … POP-EQ stretch of a rule becomes one quote symbol
//!   that ends at the first closing word equal to the opening one.
//!
//! A PUSH rule must be closed by a POP-EQ rule in the same rule body.

/// Normalized delimiter word: ASCII lower case, pauses (`.`), commas, `?`,
/// `!` and whitespace dropped, `h` folded into `'` (camxes spells the
/// apostrophe either way).
pub(crate) fn delimiter_key(text: &[u8]) -> Vec<u8> {
    text.iter()
        .filter(|b| !matches!(b, b'.' | b',' | b'?' | b'!') && !b.is_ascii_whitespace())
        .map(|b| match b.to_ascii_lowercase() {
            b'h' => b'\'',
            other => other,
        })
        .collect()
}

/// Back-reference annotation on a grammar rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Annotation {
    Push,
    PeekDiff,
    PopEq,
}

impl Annotation {
    fn from_keyword(word: &str) -> Option<Self> {
        match word {
            "PUSH" => Some(Annotation::Push),
            "PEEK-DIFF" => Some(Annotation::PeekDiff),
            "POP-EQ" => Some(Annotation::PopEq),
            _ => None,
        }
    }
}

/// Remove trailing `#: KEYWORD` annotations from rule lines. Returns the
/// grammar text without them and the `(rule name, annotation)` pairs.
/// Lines whose `#:` suffix is not a known keyword are left untouched.
pub(crate) fn strip_annotations(grammar: &str) -> (String, Vec<(String, Annotation)>) {
    let mut text = String::with_capacity(grammar.len());
    let mut found = Vec::new();
    for line in grammar.split_inclusive('\n') {
        let (body, newline) = match line.strip_suffix('\n') {
            Some(body) => (body, "\n"),
            None => (line, ""),
        };
        let annotation = body.rfind("#:").and_then(|at| {
            let annotation = Annotation::from_keyword(body[at + 2..].trim())?;
            let rule = body[..at].split("<-").next()?.trim();
            (!rule.is_empty() && body[..at].contains("<-")).then_some((at, rule, annotation))
        });
        match annotation {
            Some((at, rule, annotation)) => {
                found.push((rule.to_string(), annotation));
                text.push_str(body[..at].trim_end());
            }
            None => text.push_str(body),
        }
        text.push_str(newline);
    }
    (text, found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiter_key_normalizes_like_camxes() {
        assert_eq!(delimiter_key(b"gy"), delimiter_key(b".GY. "));
        assert_eq!(delimiter_key(b"xu'u"), delimiter_key(b"xuhu"));
        assert_eq!(delimiter_key(b"dja,n"), delimiter_key(b"djan"));
        assert_ne!(delimiter_key(b"gy"), delimiter_key(b"dy"));
    }

    #[test]
    fn strips_known_annotations_only() {
        let (text, found) =
            strip_annotations("a <- b #: PUSH\nb <- 'x'\nc <- d #: POP-EQ\nd <- '#:' #: OTHER\n");
        assert_eq!(text, "a <- b\nb <- 'x'\nc <- d\nd <- '#:' #: OTHER\n");
        assert_eq!(
            found,
            vec![
                ("a".to_string(), Annotation::Push),
                ("c".to_string(), Annotation::PopEq)
            ]
        );
    }

    const QUOTE_GRAMMAR: &str = r#"
    text <- (quote / word) (' '+ (quote / word))* !.
    quote <- 'Q' ' ' open ' ' (body ' ')* close
    open <- [a-z']+ #: PUSH
    body <- [a-z']+ #: PEEK-DIFF
    close <- [a-z']+ #: POP-EQ
    word <- [a-z']+
    "#;

    fn peg_accepts(input: &str) -> bool {
        let peg = crate::peg::grammar::Peg::new("text", QUOTE_GRAMMAR).unwrap();
        let r = peg.parse(input);
        r.2.is_ok() && r.1 == input.len()
    }

    #[test]
    fn peg_quote_needs_matching_delimiters() {
        assert!(peg_accepts("Q gy hello gy"));
        assert!(peg_accepts("Q gy hello world gy after"));
        assert!(peg_accepts("Q gy gy"));
        assert!(peg_accepts("w Q xu'u a b xuhu w"));
        assert!(!peg_accepts("Q gy hello dy"));
        assert!(!peg_accepts("Q gy hello"));
    }

    #[test]
    fn peg_quote_ends_at_first_delimiter() {
        // Without PEEK-DIFF the greedy body would swallow the closing `gy`.
        let peg = crate::peg::grammar::Peg::new("text", QUOTE_GRAMMAR).unwrap();
        use crate::peg::parsing::ParseNode;
        fn close_span(nodes: &[ParseNode]) -> Option<(usize, usize)> {
            nodes.iter().find_map(|node| match node {
                ParseNode::NonTerminal { name, span, .. } if name == "close" => {
                    Some((span.0, span.1))
                }
                ParseNode::NonTerminal { children, .. } => close_span(children),
                ParseNode::Terminal { .. } => None,
            })
        }
        let r = peg.parse("Q gy a gy x gy");
        assert_eq!(r.1, 14);
        let nodes = r.2.as_ref().as_ref().unwrap();
        assert_eq!(close_span(nodes), Some((7, 9)));
    }

    #[test]
    fn peg_stack_is_restored_after_failure() {
        // The first alternative pushes `gy` and then fails; the second one
        // must see an empty stack, so `gy` is not "equal to the top".
        let g = r#"
        s <- open ' ' 'X' / open ' ' close
        open <- [a-z]+ #: PUSH
        close <- [a-z]+ #: POP-EQ
        "#;
        let peg = crate::peg::grammar::Peg::new("s", g).unwrap();
        assert_eq!(peg.parse("gy gy").1, 5);
        assert!(peg.parse("gy dy").2.is_err());
        assert!(peg.delimiters.borrow().is_empty());
    }
}
