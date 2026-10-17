//! ilmentufa-style rule annotations (`rule <- body #: PUSH`).
//!
//! camxes grammars mark the `zoi`/`la'o` delimiter rules with `#: PUSH`
//! (opening delimiter), `#: PEEK-DIFF` (quoted word, must differ from the
//! delimiter) and `#: POP-EQ` (closing delimiter, must equal it). The PEG
//! front end ignores them; the Earley compiler turns each PUSH … POP-EQ pair
//! into a delimiter-matching quote symbol.

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
}
