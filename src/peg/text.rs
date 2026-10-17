//! Grammar-text reader shared by both engines.
//!
//! Turns PEG grammar text (`name <- expr` rules, after
//! [`strip_annotations`](crate::peg::annotations::strip_annotations)) into
//! [`Rule`] trees. It is a hand-written recursive-descent version of the
//! bootstrap grammar the PEG engine uses for the same job, so an Earley-only
//! build does not need the PEG engine; with both engines enabled a test
//! checks that the two readers agree.
//!
//! Syntax: identifiers `[a-zA-Z_][a-zA-Z0-9_]*`; `<-`, `/`, `&`, `!`, `?`,
//! `*`, `+`, `(`, `)`, `.`; literals in `'…'` or `"…"`; classes `[…]` with
//! `a-z` ranges; escapes `\n \r \t \' \" \[ \] \\` and `\123;` are kept as
//! written except `\'`, `\"`, `\[`, `\]` and `\\`, which stand for the
//! character. Spacing is space, `\n` or `\r\n`.

use crate::peg::rule::Rule;
use std::collections::{HashMap, HashSet};

/// Deepest `( … )` nesting accepted (keeps the recursion bounded).
const MAX_NESTING: usize = 256;

/// Parse grammar text into named rules. Errors name the byte offset where
/// the text stops being understood, or the rules referenced but not defined.
pub(crate) fn parse_rules(text: &str) -> Result<HashMap<String, Rule>, String> {
    let mut p = Reader {
        s: text,
        pos: 0,
        depth: 0,
        refs: HashSet::new(),
    };
    p.spacing();
    let mut rules = HashMap::new();
    loop {
        let save = p.pos;
        match p.definition() {
            Some((name, rule)) => {
                rules.insert(name, rule);
            }
            None => {
                p.pos = save;
                break;
            }
        }
    }
    if p.depth > MAX_NESTING {
        return Err(format!("grammar nests deeper than {MAX_NESTING} levels"));
    }
    if p.pos != text.len() {
        return Err(format!("grammar text not understood at byte {}", p.pos));
    }
    let mut undefined: Vec<String> = p
        .refs
        .iter()
        .filter(|r| !rules.contains_key(*r))
        .cloned()
        .collect();
    if !undefined.is_empty() {
        undefined.sort();
        return Err(format!("Missing rules: [{}]!", undefined.join(", ")));
    }
    Ok(rules)
}

/// Same unescaping as the PEG engine's transformer.
fn unescape(raw: &str) -> String {
    raw.replace("\\'", "'")
        .replace("\\\"", "\"")
        .replace("\\[", "[")
        .replace("\\]", "]")
        .replace("\\\\", "\\")
}

struct Reader<'a> {
    s: &'a str,
    pos: usize,
    /// Current `( … )` nesting; set above [`MAX_NESTING`] to abort.
    depth: usize,
    refs: HashSet<String>,
}

impl Reader<'_> {
    fn rest(&self) -> &str {
        self.s.get(self.pos..).unwrap_or_default()
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn eat(&mut self, lit: &str) -> bool {
        if self.rest().starts_with(lit) {
            self.pos += lit.len();
            true
        } else {
            false
        }
    }

    fn spacing(&mut self) {
        while self.eat(" ") || self.eat("\n") || self.eat("\r\n") {}
    }

    /// An operator token followed by spacing.
    fn op(&mut self, lit: &str) -> bool {
        if self.eat(lit) {
            self.spacing();
            true
        } else {
            false
        }
    }

    fn definition(&mut self) -> Option<(String, Rule)> {
        let name = self.identifier()?;
        if !self.op("<-") {
            return None;
        }
        let rule = self.expression()?;
        Some((name, rule))
    }

    fn identifier(&mut self) -> Option<String> {
        let start = self.pos;
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.pos += 1,
            _ => return None,
        }
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == '_') {
            self.pos += 1;
        }
        let name = self.s[start..self.pos].to_string();
        self.spacing();
        Some(name)
    }

    fn expression(&mut self) -> Option<Rule> {
        let mut alts = vec![self.sequence()?];
        while self.op("/") {
            alts.push(self.sequence()?);
        }
        Some(if alts.len() == 1 {
            alts.remove(0)
        } else {
            Rule::Choice(alts)
        })
    }

    fn sequence(&mut self) -> Option<Rule> {
        let mut items = Vec::new();
        loop {
            let save = self.pos;
            match self.item() {
                Some(rule) => items.push(rule),
                None => {
                    self.pos = save;
                    break;
                }
            }
            if self.depth > MAX_NESTING {
                return None;
            }
        }
        Some(match items.len() {
            0 => Rule::Empty,
            1 => items.remove(0),
            _ => Rule::Sequence(items),
        })
    }

    /// `prefix? primary suffix?`
    fn item(&mut self) -> Option<Rule> {
        let prefix = if self.op("&") {
            Some(true)
        } else if self.op("!") {
            Some(false)
        } else {
            None
        };
        let mut rule = self.primary()?;
        if self.op("?") {
            rule = Rule::Optional(rule.boxed());
        } else if self.op("*") {
            rule = Rule::ZeroOrMore(rule.boxed());
        } else if self.op("+") {
            rule = Rule::OneOrMore(rule.boxed());
        }
        Some(match prefix {
            Some(true) => Rule::And(rule.boxed()),
            Some(false) => Rule::Not(rule.boxed()),
            None => rule,
        })
    }

    fn primary(&mut self) -> Option<Rule> {
        let save = self.pos;
        if let Some(name) = self.identifier() {
            if !self.rest().starts_with("<-") {
                self.refs.insert(name.clone());
                return Some(Rule::NonTerminal(name));
            }
        }
        self.pos = save;
        if self.op("(") {
            self.depth += 1;
            if self.depth > MAX_NESTING {
                return None;
            }
            let inner = self.expression();
            self.depth -= 1;
            if let Some(inner) = inner {
                if self.op(")") {
                    return Some(inner);
                }
            }
            if self.depth > MAX_NESTING {
                return None;
            }
        }
        self.pos = save;
        for quote in ["'", "\""] {
            if let Some(rule) = self.literal(quote) {
                return Some(rule);
            }
            self.pos = save;
        }
        if let Some(rule) = self.class() {
            return Some(rule);
        }
        self.pos = save;
        if self.op(".") {
            return Some(Rule::Any);
        }
        None
    }

    /// One (possibly escaped) character, as written.
    fn char_raw(&mut self) -> Option<&str> {
        let start = self.pos;
        let rest = self.rest();
        if let Some(after) = rest.strip_prefix('\\') {
            let mut chars = after.chars();
            match chars.next() {
                Some('n' | 'r' | 't' | '\'' | '"' | '[' | ']' | '\\') => self.pos += 2,
                Some(c) if c.is_ascii_digit() => {
                    let digits = after.bytes().take_while(u8::is_ascii_digit).count();
                    if after.as_bytes().get(digits) != Some(&b';') {
                        return None;
                    }
                    self.pos += 1 + digits + 1;
                }
                _ => return None,
            }
        } else {
            self.pos += rest.chars().next()?.len_utf8();
        }
        self.s.get(start..self.pos)
    }

    fn literal(&mut self, quote: &str) -> Option<Rule> {
        if !self.eat(quote) {
            return None;
        }
        let mut content = String::new();
        while !self.rest().starts_with(quote) {
            let save = self.pos;
            match self.char_raw() {
                Some(raw) => content.push_str(&unescape(raw)),
                None => {
                    self.pos = save;
                    break;
                }
            }
        }
        if !self.eat(quote) {
            return None;
        }
        self.spacing();
        Some(Rule::Literal(content))
    }

    fn class(&mut self) -> Option<Rule> {
        if !self.eat("[") {
            return None;
        }
        let mut parts = Vec::new();
        let mut symbols = HashSet::new();
        while !self.rest().starts_with(']') {
            let save = self.pos;
            let Some(first) = self.char_raw().map(unescape) else {
                self.pos = save;
                break;
            };
            let before_dash = self.pos;
            if self.eat("-") {
                if let Some(last) = self.char_raw().map(unescape) {
                    parts.push(Rule::Range(first, last));
                    continue;
                }
            }
            self.pos = before_dash;
            symbols.insert(first);
        }
        if !self.eat("]") {
            return None;
        }
        self.spacing();
        if !symbols.is_empty() {
            parts.push(Rule::Class(symbols));
        }
        Some(match parts.len() {
            0 => Rule::Empty,
            1 => parts.remove(0),
            _ => Rule::Choice(parts),
        })
    }
}

/// With both readers compiled in, they must agree on every grammar.
#[cfg(all(test, feature = "peg"))]
mod tests {
    use super::*;
    use crate::peg::annotations::strip_annotations;
    use crate::peg::grammar::Peg;

    /// Deterministic text form (class members sorted).
    fn canon(rule: &Rule) -> String {
        match rule {
            Rule::Empty => "ε".into(),
            Rule::Any => ".".into(),
            Rule::Literal(s) => format!("{s:?}"),
            Rule::NonTerminal(n) => n.clone(),
            Rule::Range(a, b) => format!("[{a:?}-{b:?}]"),
            Rule::Class(set) => {
                let mut v: Vec<&String> = set.iter().collect();
                v.sort();
                format!("{v:?}")
            }
            Rule::Group(r) => format!("({})", canon(r)),
            Rule::ZeroOrMore(r) => format!("{}*", canon(r)),
            Rule::OneOrMore(r) => format!("{}+", canon(r)),
            Rule::Optional(r) => format!("{}?", canon(r)),
            Rule::And(r) => format!("&{}", canon(r)),
            Rule::Not(r) => format!("!{}", canon(r)),
            Rule::Choice(v) => format!("<{}>", v.iter().map(canon).collect::<Vec<_>>().join(" / ")),
            Rule::Sequence(v) => format!("<{}>", v.iter().map(canon).collect::<Vec<_>>().join(" ")),
        }
    }

    fn canon_map(rules: &HashMap<String, Rule>) -> Vec<String> {
        let mut v: Vec<String> = rules
            .iter()
            .map(|(k, r)| format!("{k} <- {}", canon(r)))
            .collect();
        v.sort();
        v
    }

    fn agree(grammar: &str) {
        let (plain, _) = strip_annotations(grammar);
        let ours = parse_rules(plain.trim());
        let theirs = Peg::new("s", grammar);
        match (&ours, &theirs) {
            (Ok(a), Ok(b)) => assert_eq!(canon_map(a), canon_map(&b.rules), "{grammar:?}"),
            (Err(_), Err(_)) => {}
            _ => panic!(
                "readers disagree on {grammar:?}: text.rs {:?} vs Peg {:?}",
                ours.is_ok(),
                theirs.is_ok()
            ),
        }
    }

    #[test]
    fn readers_agree() {
        use crate::grammars::{LOJBAN_GRAMMAR, LOJBAN_MORPHOLOGY_GRAMMAR};
        agree(LOJBAN_GRAMMAR.1);
        agree(LOJBAN_MORPHOLOGY_GRAMMAR.1);
        for g in [
            "",
            "s",
            "s <-",
            "s <- 'a",
            "s <- [a-",
            "s <- (",
            "s <- )",
            "s <- 'п' [а-я]+ .",
            "s <- [é-ü] / '😀'",
            "п <- 'a'",
            "s <- 'a' / / 'b'",
            "s <- undefined",
            "s <- '\\",
            "s <- [\\]",
            "s <- [\\]]",
            "s <- '\\n\\t\\123;\\''",
            "s <- \"x'y\" [a-]",
            "s <- [-a] [a-z0-9_]",
            "s <- &&'a' !b?\nb <- 'b'*",
            "s <- ('a' / ('b' 'c')+)?",
            "s <- a\n  a <- 'x'",
            "s <- 'a'\r\nt <- s",
            "s <- 'a'\rt <- s",
            "s <- 'a'\tt",
            "s <- a <- b",
            "s <- ()",
            "s <- []",
            "s <- [ab]",
            "s<-'a'",
            "_x1 <- 'a'\ns <- _x1",
            "s <- '\\x'",
            "s <- [a-\\]]",
            "s <- 'a' #: PUSH",
        ] {
            agree(g);
        }
        // Pseudo-random grammar texts from grammar characters.
        let pool: Vec<char> = "sab'\"[]()-/!&?*+.<- \n\\_п1".chars().collect();
        let mut x = 0x2545_F491_4F6C_DD1Du64;
        for _ in 0..3000 {
            let mut g = String::from("s <- ");
            for _ in 0..(x % 12) {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                g.push(pool[(x % pool.len() as u64) as usize]);
            }
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            agree(&g);
        }
    }
}
