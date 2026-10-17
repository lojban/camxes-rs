//! Cmavo → selma'o lexicon (jbovlaste-style), embedded under `grammar`.
//!
//! Subtypes like `UI1`, `KOhA3`, `FAhA4` map to Ch.21 terminals `UI`, `KOhA`, `FAhA`.

use std::collections::HashMap;
use std::sync::OnceLock;

const CMAVO_TSV: &str = include_str!("../data/cmavo_selmaho.tsv");

static LEXICON: OnceLock<CmavoLexicon> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct CmavoLexicon {
    /// normalized form (lowercase, no leading/trailing `.`) → Ch.21 selma'o
    by_form: HashMap<String, String>,
    /// raw subtype retained for diagnostics
    raw_selmaho: HashMap<String, String>,
}

impl CmavoLexicon {
    pub fn load() -> Self {
        let mut by_form = HashMap::new();
        let mut raw_selmaho = HashMap::new();
        for line in CMAVO_TSV.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split('\t');
            let Some(word) = parts.next() else { continue };
            let Some(base) = parts.next() else { continue };
            let raw = parts.next().unwrap_or(base);
            let key = normalize_cmavo(word);
            by_form.insert(key.clone(), base.to_string());
            raw_selmaho.insert(key, raw.to_string());
        }
        Self {
            by_form,
            raw_selmaho,
        }
    }

    pub fn global() -> &'static CmavoLexicon {
        LEXICON.get_or_init(Self::load)
    }

    pub fn selmaho(&self, word: &str) -> Option<&str> {
        self.by_form
            .get(&normalize_cmavo(word))
            .map(|selmaho| selmaho.as_str())
    }

    pub fn raw_selmaho(&self, word: &str) -> Option<&str> {
        self.raw_selmaho
            .get(&normalize_cmavo(word))
            .map(|selmaho| selmaho.as_str())
    }

    pub fn len(&self) -> usize {
        self.by_form.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_form.is_empty()
    }

    /// Greedy left-to-right split of a compound cmavo orthographic string into
    /// known simple cmavo (longest match). DERIVED helper when morphology emits
    /// a single compound token.
    pub fn split_compound(&self, text: &str) -> Option<Vec<(String, String)>> {
        let normalized = normalize_cmavo(text);
        if normalized.is_empty() {
            return None;
        }
        let mut byte_index = 0;
        let bytes = normalized.as_bytes();
        let mut pieces = Vec::new();
        while byte_index < bytes.len() {
            let mut matched_piece = None;
            for piece_len in (1..=bytes.len() - byte_index).rev() {
                let piece = std::str::from_utf8(&bytes[byte_index..byte_index + piece_len]).ok()?;
                if let Some(selmaho) = self.selmaho(piece) {
                    matched_piece = Some((piece.to_string(), selmaho.to_string()));
                    byte_index += piece_len;
                    break;
                }
            }
            pieces.push(matched_piece?);
        }
        if pieces.len() <= 1 {
            // single piece — not a useful split
            None
        } else {
            Some(pieces)
        }
    }
}

pub fn normalize_cmavo(word: &str) -> String {
    let mut normalized = word.trim().to_lowercase();
    // strip surrounding pauses/dots often present orthographically
    while normalized.starts_with('.') {
        normalized.remove(0);
    }
    while normalized.ends_with('.') {
        normalized.pop();
    }
    // drop commas (syllable marks)
    normalized.replace(',', "")
}

/// Map jbovlaste subtype selma'o to Ch.21 terminal (UI1→UI, etc.).
pub fn map_selmaho_subtype(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut end = bytes.len();
    // strip trailing digits and optional letter subtype (UI3a, BY2)
    while end > 0 && bytes[end - 1].is_ascii_lowercase() {
        // only strip lowercase after digits
        let before = end - 1;
        if before > 0 && bytes[before - 1].is_ascii_digit() {
            end = before;
        } else {
            break;
        }
    }
    while end > 0 && bytes[end - 1].is_ascii_digit() {
        end -= 1;
    }
    if end == 0 {
        raw.to_string()
    } else {
        raw[..end].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicon_smoke() {
        let lex = CmavoLexicon::global();
        assert!(lex.len() > 500);
        assert_eq!(lex.selmaho("mi"), Some("KOhA"));
        assert_eq!(lex.selmaho("le"), Some("LE"));
        assert_eq!(lex.selmaho("cu"), Some("CU"));
        assert_eq!(lex.selmaho("ui"), Some("UI"));
        assert_eq!(lex.selmaho(".a'a"), Some("UI"));
    }

    #[test]
    fn subtype_map() {
        assert_eq!(map_selmaho_subtype("UI1"), "UI");
        assert_eq!(map_selmaho_subtype("UI3a"), "UI");
        assert_eq!(map_selmaho_subtype("KOhA3"), "KOhA");
        assert_eq!(map_selmaho_subtype("FAhA"), "FAhA");
    }
}
