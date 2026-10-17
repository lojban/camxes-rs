//! Cmevla substring ban — CLL §4.8.

use super::{Constraint, ConstraintMeta, ConstraintStatus, Reject};
use crate::cll::cfg::CandidateSpan;
use crate::cll::phonology::is_c;

pub struct CmevlaBan;

impl Constraint for CmevlaBan {
    fn meta(&self) -> ConstraintMeta {
        ConstraintMeta {
            id: "C19",
            cll: "§4.8",
            status: ConstraintStatus::Stated,
        }
    }

    fn check(&self, cand: &CandidateSpan) -> Result<(), Reject> {
        if cand.proposed_class != crate::cll::lex::WordClass::Cmevla {
            return Ok(());
        }
        if let Some(hit) = banned_substring(&cand.text) {
            return Err(Reject {
                id: "C19",
                cll: "§4.8",
                detail: format!("banned substring `{hit}` not after consonant"),
            });
        }
        Ok(())
    }
}

pub fn banned_substring(word: &str) -> Option<&'static str> {
    let mut letters = word.bytes().filter(|&byte| byte != b'.' && byte != b',');
    let mut previous = None;

    while let Some(first) = letters.next() {
        let mut lookahead = letters.clone();
        let substring = match first {
            b'd' if lookahead.next() == Some(b'o') && lookahead.next() == Some(b'i') => Some("doi"),
            b'l' if lookahead.next() == Some(b'a') => Some(if lookahead.next() == Some(b'i') {
                "lai"
            } else {
                "la"
            }),
            _ => None,
        };

        if let Some(substring) = substring {
            if previous.is_none_or(|byte| !is_c(byte)) {
                return Some(substring);
            }
        }
        previous = Some(first);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn examples() {
        assert_eq!(banned_substring("laplas."), Some("la"));
        assert_eq!(banned_substring("ilanas."), Some("la"));
        assert_eq!(banned_substring("klaplas."), None);
    }
}
