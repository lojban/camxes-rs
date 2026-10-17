//! Morphology regression over every headword of the lensisku English
//! dictionary (`tests/data/lensisku-en-words.tsv.gz`, refreshed with
//! `scripts/fetch_lensisku_corpus.py`).
//!
//! Only morphology is compared: the word split and each word's class. Parse
//! tree shapes are engine internals and are not compared.
//!
//! * Every headword the PEG engine accepts must get the same word split and
//!   classes from the Earley engine.
//! * The Earley engine must also accept the headwords that the PEG engine
//!   rejects (mostly "obsolete" CLL-dialect fu'ivla), except the short list
//!   of entries that are invalid under both camxes and CLL morphology.
//! * `wiki` entries are English page titles, not Lojban, and are skipped.
#![cfg(feature = "earley")]

use camxes_rs::morphology::{analyze, Dialect, Engine, Morphology, WordClass};
use flate2::read::GzDecoder;
use std::collections::BTreeSet;
use std::io::Read;

/// Headwords no Lojban morphology accepts (both engines reject them), with
/// the reason. Non-ASCII headwords (mojibake) are also treated as invalid.
const INVALID_EVERYWHERE: &[(&str, &str)] = &[
    ("cipnrbuteo", "vowel hiatus `eo` without apostrophe"),
    ("etreo", "vowel hiatus `eo` without apostrophe"),
    ("fiprpseudoskafirinku", "vowel hiatus `eu` without apostrophe"),
    ("fiprtetrapleurodo", "vowel hiatus `eu` without apostrophe"),
    ("freskeo", "vowel hiatus `eo` without apostrophe"),
    ("gaurkihu", "`h` is not a Lojban letter"),
    ("ku'urpicea", "vowel hiatus `ea` without apostrophe"),
    ("kuaidzi", "no consonant cluster in the first five letters (not a brivla)"),
    ("ma;u", "`;` is not Lojban orthography"),
    ("nuhempcr", "`h` is not a Lojban letter"),
    ("posytmo", "`y` inside a fu'ivla"),
    ("ricrbeaukarne'a", "vowel hiatus `ea` without apostrophe"),
    ("spareukomi", "vowel hiatus `eu` without apostrophe"),
    ("sparipeastru", "vowel hiatus `ea` without apostrophe"),
    ("sparnimfea", "vowel hiatus `ea` without apostrophe"),
    ("sparpersea", "vowel hiatus `ea` without apostrophe"),
    ("spatrleoxari", "vowel hiatus `eo` without apostrophe"),
    ("stagrleoxari", "vowel hiatus `eo` without apostrophe"),
    ("xa!", "`!` is not Lojban orthography"),
];

struct Entry {
    word: String,
    word_type: String,
}

fn corpus() -> Vec<Entry> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/lensisku-en-words.tsv.gz");
    let mut text = String::new();
    GzDecoder::new(std::fs::File::open(path).expect("corpus file"))
        .read_to_string(&mut text)
        .expect("gzip corpus");
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split('\t');
            Some(Entry {
                word: cols.next()?.to_string(),
                word_type: cols.next()?.to_string(),
            })
        })
        .collect()
}

type Outcome = (Option<Morphology>, Option<Morphology>);

/// Analyze all entries with both engines, spread over the available cores.
fn analyze_all(entries: &[Entry]) -> Vec<Outcome> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = entries.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = entries
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    part.iter()
                        .map(|e| {
                            (
                                analyze(&e.word, Engine::Peg).ok(),
                                analyze(&e.word, Engine::Earley).ok(),
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

#[test]
fn dictionary_headwords_morphology() {
    let entries = corpus();
    assert!(entries.len() > 30_000, "corpus too small: {}", entries.len());
    let outcomes = analyze_all(&entries);

    let invalid: BTreeSet<&str> = INVALID_EVERYWHERE.iter().map(|(w, _)| *w).collect();
    let (mut peg_ok, mut agree, mut earley_only, mut skipped) = (0, 0, 0, 0);
    let (mut obsolete, mut obsolete_earley, mut obsolete_peg) = (0, 0, 0);
    let mut disagreements = Vec::new();
    let mut unexpected_failures = Vec::new();
    let mut stale_invalid = Vec::new();

    for (e, (peg, earley)) in entries.iter().zip(&outcomes) {
        if e.word_type == "wiki" {
            skipped += 1;
            continue;
        }
        if e.word_type.starts_with("obsolete") {
            obsolete += 1;
            obsolete_peg += usize::from(peg.is_some());
            obsolete_earley += usize::from(earley.is_some());
        }
        match (peg, earley) {
            (Some(p), Some(q)) => {
                peg_ok += 1;
                if p.signature() == q.signature() {
                    agree += 1;
                } else {
                    disagreements.push(format!("{}: peg [{}] earley [{}]", e.word, p.render(), q.render()));
                }
            }
            (Some(p), None) => {
                peg_ok += 1;
                disagreements.push(format!("{}: peg [{}] earley rejected", e.word, p.render()));
            }
            (None, Some(_)) => {
                earley_only += 1;
                if invalid.contains(e.word.as_str()) {
                    stale_invalid.push(e.word.clone());
                }
            }
            (None, None) => {
                if !invalid.contains(e.word.as_str()) && e.word.is_ascii() {
                    unexpected_failures.push(format!("{} ({})", e.word, e.word_type));
                }
            }
        }
    }

    println!(
        "corpus {}: skipped wiki {skipped}; PEG accepted {peg_ok}, Earley agrees on {agree}; \
         Earley-only {earley_only}; obsolete {obsolete} (PEG {obsolete_peg}, Earley {obsolete_earley})",
        entries.len()
    );
    assert!(disagreements.is_empty(), "{} disagreements:\n{}", disagreements.len(), disagreements.join("\n"));
    assert!(
        unexpected_failures.is_empty(),
        "Earley rejected {} headwords:\n{}",
        unexpected_failures.len(),
        unexpected_failures.join("\n")
    );
    assert!(stale_invalid.is_empty(), "now accepted, drop from INVALID_EVERYWHERE: {stale_invalid:?}");
    assert_eq!(peg_ok, agree);
}

#[test]
fn cll_dialect_words_are_classified() {
    use WordClass::*;
    for (word, class) in [
        ("matriocka", Fuhivla),
        ("zarziei", Fuhivla),
        ("sargueia", Fuhivla),
        ("gondii'i", Fuhivla),
        ("pervu'ui", Fuhivla),
        ("gudjrati", Fuhivla),
        ("bangrlietuva", Fuhivla),
    ] {
        let m = analyze(word, Engine::Earley).unwrap_or_else(|e| panic!("{word}: {e}"));
        assert_eq!(m.signature(), vec![(word.to_string(), class)], "{word}");
        if analyze(word, Engine::Peg).is_err() {
            assert_eq!(m.dialect, Dialect::Cll, "{word}");
        }
    }
}
