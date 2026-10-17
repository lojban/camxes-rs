//! DERIVED metalinguistic eraser for SI / SA / SU (CLL Ch.19).
//!
//! Ch.21 marks `null = any-word SI | utterance SA | text SU` as **non-formal**.
//! This module realizes erasure on the **lexeme stream** after morph→selma'o
//! lexing and the Ch.21 word-layer (UI/BAhE strip, cmavo re-fuse), **before**
//! Earley. The formal CFG is unchanged.
//!
//! ## Normative behaviour (CLL Ch.19 §Erasure)
//!
//! - **SI**: erase the preceding *word* (as if never spoken). Multiple SI stack.
//! - **ZO** quote: the word after `zo` is quoted (even if it is `si`/`sa`/`su`);
//!   `zo` + quoted word are **two** SI-words (CLL ex. 19.77, 19.80).
//! - **ZOI** quote: for SI, the construct is **four** words —
//!   `zoi` + open-delim + body + close-delim — with the body counting as **one**
//!   word even if it spans several lexemes (CLL ex. 19.79).
//! - **SA**: erase back to and including the last **starting marker of the same
//!   kind** as the word following `sa` (CLL ex. 19.81 `.i`, 19.83 `le`).
//! - **SU**: erase the entire stream (speaker-scoped in conversation; single-
//!   stream suite → clear all).
//!
//! ## DERIVED approximations (GAP→DERIVED)
//!
//! - **Starting-marker “kind”** for SA: identical Ch.21 terminal / selma'o as
//!   the word after `sa` (CLL “same kind”). Starter inventory is not exhaustively
//!   listed in CLL; we match any prior lexeme with that terminal. Cite GAP if a
//!   construct start is not a single selma'o-tagged word.
//! - **LOhU…LEhU** as a multi-word SI unit is not specially coalesced here
//!   (no suite target); words erase individually.
//! - Multi-speaker **su** / **susu** discourse scope is out of scope for the
//!   single-stream recognizer.

use super::lexeme::Lexeme;

/// One SI-erasable unit. May expand to several CFG lexemes (ZOI body).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Unit {
    /// Single lexeme = one SI-word.
    Atom(Lexeme),
    /// ZOI foreign body: one SI-word, N Earley terminals.
    ZoiBody(Vec<Lexeme>),
}

impl Unit {
    fn marker_terminal(&self) -> Option<&str> {
        match self {
            Unit::Atom(l) => Some(l.terminal.as_str()),
            Unit::ZoiBody(_) => None,
        }
    }

    fn emit(self, out: &mut Vec<Lexeme>) {
        match self {
            Unit::Atom(l) => out.push(l),
            Unit::ZoiBody(body) => out.extend(body),
        }
    }

    fn describe(&self) -> String {
        match self {
            Unit::Atom(l) => format!("{}:{}", l.terminal, l.surface),
            Unit::ZoiBody(body) => {
                let parts: Vec<_> = body.iter().map(|l| l.surface.as_str()).collect();
                format!("ZOI-body[{}]", parts.join(" "))
            }
        }
    }
}

/// Apply SI/SA/SU erasure. Returns the surviving lexemes and DERIVED notes.
pub fn apply_erasure(raw: Vec<Lexeme>) -> (Vec<Lexeme>, Vec<String>) {
    let mut stack: Vec<Unit> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut lexeme_index = 0;

    while lexeme_index < raw.len() {
        let term = raw[lexeme_index].terminal.as_str();
        match term {
            "SI" => {
                match stack.pop() {
                    Some(unit) => notes.push(format!(
                        "DERIVED Ch.19 SI: erase preceding word {}",
                        unit.describe()
                    )),
                    None => notes.push(format!(
                        "DERIVED Ch.19 SI: {:?} with empty stream (no-op)",
                        raw[lexeme_index].surface
                    )),
                }
                lexeme_index += 1;
            }
            "SA" => {
                if lexeme_index + 1 >= raw.len() {
                    notes.push(format!(
                        "DERIVED Ch.19 SA: {:?} at end of stream (dropped)",
                        raw[lexeme_index].surface
                    ));
                    lexeme_index += 1;
                    continue;
                }
                let follow = &raw[lexeme_index + 1];
                let kind = follow.terminal.as_str();
                // Erase back to and including last starting marker of same kind.
                let pos = stack
                    .iter()
                    .rposition(|unit| unit.marker_terminal() == Some(kind));
                match pos {
                    Some(p) => {
                        let erased: Vec<_> = stack.drain(p..).map(|unit| unit.describe()).collect();
                        notes.push(format!(
                            "DERIVED Ch.19 SA: erase back through last {kind} marker \
                             (follow={:?}); removed [{}]",
                            follow.surface,
                            erased.join(", ")
                        ));
                    }
                    None => {
                        let erased: Vec<_> = stack.drain(..).map(|unit| unit.describe()).collect();
                        notes.push(format!(
                            "DERIVED Ch.19 SA: no prior {kind} marker \
                             (follow={:?}); clear stream [{}]",
                            follow.surface,
                            erased.join(", ")
                        ));
                    }
                }
                // Consume SA only; the following marker is processed next.
                lexeme_index += 1;
            }
            "SU" => {
                let word_unit_count = stack.len();
                stack.clear();
                notes.push(format!(
                    "DERIVED Ch.19 SU: erase entire text ({word_unit_count} word-units)"
                ));
                lexeme_index += 1;
            }
            "ZO" => {
                // Quote the following word even if it is SI/SA/SU (CLL 19.77).
                stack.push(Unit::Atom(raw[lexeme_index].clone()));
                lexeme_index += 1;
                if lexeme_index < raw.len() {
                    notes.push(format!(
                        "DERIVED Ch.19 ZO: quote word {}:{} as SI-word",
                        raw[lexeme_index].terminal, raw[lexeme_index].surface
                    ));
                    stack.push(Unit::Atom(raw[lexeme_index].clone()));
                    lexeme_index += 1;
                } else {
                    notes.push("DERIVED Ch.19 ZO: marker without following word".into());
                }
            }
            "ZOI" => {
                // Four SI-words: ZOI + open + body(one) + close (CLL 19.79).
                let zoi = raw[lexeme_index].clone();
                lexeme_index += 1;
                if lexeme_index >= raw.len() {
                    notes.push("DERIVED Ch.19 ZOI: marker without delimiter".into());
                    stack.push(Unit::Atom(zoi));
                    continue;
                }
                let open = raw[lexeme_index].clone();
                let delim = open.surface.clone();
                lexeme_index += 1;
                let mut body: Vec<Lexeme> = Vec::new();
                while lexeme_index < raw.len() && raw[lexeme_index].surface != delim {
                    // Do not let SI/SA/SU inside an unclosed ZOI act as erasers;
                    // they are foreign-body text until the close delim (CLL:
                    // delimiting word chosen; body is “anything”).
                    body.push(raw[lexeme_index].clone());
                    lexeme_index += 1;
                }
                stack.push(Unit::Atom(zoi));
                stack.push(Unit::Atom(open));
                stack.push(Unit::ZoiBody(body));
                if lexeme_index < raw.len() && raw[lexeme_index].surface == delim {
                    stack.push(Unit::Atom(raw[lexeme_index].clone()));
                    lexeme_index += 1;
                    notes.push(format!(
                        "DERIVED Ch.19 ZOI: closed with delim {delim:?} \
                         (4 SI-words; body is one word)"
                    ));
                } else {
                    notes.push(format!(
                        "DERIVED Ch.19 ZOI: unclosed delim {delim:?} (GAP partial)"
                    ));
                }
            }
            _ => {
                stack.push(Unit::Atom(raw[lexeme_index].clone()));
                lexeme_index += 1;
            }
        }
    }

    let mut out = Vec::new();
    for unit in stack {
        unit.emit(&mut out);
    }
    (out, notes)
}

#[cfg(test)]
mod tests {
    use crate::cll::grammar::lexeme::lexeme_stream;

    fn terms(input: &str) -> Vec<String> {
        lexeme_stream(input)
            .lexemes
            .into_iter()
            .map(|l| l.terminal)
            .collect()
    }

    fn surfaces(input: &str) -> Vec<String> {
        lexeme_stream(input)
            .lexemes
            .into_iter()
            .map(|l| l.surface)
            .collect()
    }

    #[test]
    fn si_erases_one_word_19_75() {
        // ti gerku si mlatu → ti mlatu
        assert_eq!(surfaces("ti gerku si mlatu"), vec!["ti", "mlatu"]);
        assert_eq!(terms("ti gerku si mlatu"), vec!["KOhA", "BRIVLA"]);
    }

    #[test]
    fn si_stacks_19_76() {
        // ta blanu zdani si si xekri zdani → ta xekri zdani
        assert_eq!(
            surfaces("ta blanu zdani si si xekri zdani"),
            vec!["ta", "xekri", "zdani"]
        );
    }

    #[test]
    fn zo_si_needs_three_si_19_77() {
        // zo .bab. se cmene zo si si si la bab. → zo bab se cmevla la bab
        assert_eq!(
            surfaces("zo .bab. se cmene zo si si si la bab."),
            vec!["zo", "bab", "se", "cmene", "la", "bab"]
        );
        assert_eq!(
            terms("zo .bab. se cmene zo si si si la bab."),
            vec!["ZO", "CMENE", "SE", "BRIVLA", "LA", "CMENE"]
        );
    }

    #[test]
    fn zoi_body_is_one_si_word_19_79() {
        // mi cusku zoi fy. gy. .fy. si si si si zo .djan
        // → mi cusku zo djan  (four SI erase zoi+fy+body+fy)
        assert_eq!(
            surfaces("mi cusku zoi fy. gy. .fy. si si si si zo .djan"),
            vec!["mi", "cusku", "zo", "djan"]
        );
        assert_eq!(
            terms("mi cusku zoi fy. gy. .fy. si si si si zo .djan"),
            vec!["KOhA", "BRIVLA", "ZO", "CMENE"]
        );
    }

    #[test]
    fn zo_quote_two_si_19_80() {
        // mi se cmene zo .djan. si si zo .djordj.
        // → mi se cmevla zo djordj
        assert_eq!(
            surfaces("mi se cmene zo .djan. si si zo .djordj."),
            vec!["mi", "se", "cmene", "zo", "djordj"]
        );
    }

    #[test]
    fn sa_to_i_19_81() {
        // mi viska le sa .i mi cusku zo .djan.
        // → .i mi cusku zo djan  (no prior I → clear, keep following I)
        assert_eq!(
            surfaces("mi viska le sa .i mi cusku zo .djan."),
            vec!["i", "mi", "cusku", "zo", "djan"]
        );
        assert_eq!(
            terms("mi viska le sa .i mi cusku zo .djan."),
            vec!["I", "KOhA", "BRIVLA", "ZO", "CMENE"]
        );
    }

    #[test]
    fn sa_to_le_19_83() {
        // mi viska le blanu .zdan. sa le xekri zdani
        // → mi viska le xekri zdani
        assert_eq!(
            surfaces("mi viska le blanu .zdan. sa le xekri zdani"),
            vec!["mi", "viska", "le", "xekri", "zdani"]
        );
    }

    #[test]
    fn su_clears_stream() {
        let r = lexeme_stream("mi viska le gerku su do klama");
        let surf: Vec<_> = r.lexemes.iter().map(|l| l.surface.as_str()).collect();
        assert_eq!(surf, vec!["do", "klama"]);
        assert!(
            r.morph_notes.iter().any(|n| n.contains("Ch.19 SU")),
            "{:?}",
            r.morph_notes
        );
    }

    #[test]
    fn erasure_notes_present() {
        let r = lexeme_stream("ti gerku si mlatu");
        assert!(
            r.morph_notes.iter().any(|n| n.contains("Ch.19 SI")),
            "{:?}",
            r.morph_notes
        );
    }
}
