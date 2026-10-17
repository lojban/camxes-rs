//! Morphology regression over every headword of the lensisku English
//! dictionary (`tests/data/lensisku-en-words.tsv.gz`, refreshed with
//! `scripts/fetch_lensisku_corpus.py`).
//!
//! Only morphology is compared: the word split and each word's class. Parse
//! tree shapes are engine internals and are not compared.
//!
//! * Every headword the PEG engine accepts must get the same word split and
//!   classes from the Earley engine (the vendored tergenzalvi CLL pipeline
//!   with its camxes-compatibility rules, see `src/cll/camxes.rs`), except
//!   the documented known differences (`tests/data/known-differences.tsv`,
//!   categories in [`DIFFERENCE_CATEGORIES`]), where camxes and CLL morphology
//!   genuinely disagree.
//! * The Earley engine must also accept the headwords that the PEG engine
//!   rejects (mostly "obsolete" CLL-dialect fu'ivla), except the short list
//!   of entries that are invalid under both camxes and CLL morphology.
//! * `wiki` entries (English page titles, not Lojban) are not part of the
//!   corpus: the fetch script drops them and the loader filters them out.
//! * `tests/data/morphology-parity.tsv` records, for every headword, the
//!   expected normalized morphology (words, classes, cmavo selma'o) and
//!   whether each engine accepts it. Both engines must reproduce it exactly.
//!   After an intended change, regenerate it with
//!   `CAMXES_BLESS_PARITY=1 cargo test --release --features earley --test dictionary_corpus`
//!   and review the diff.
#![cfg(all(feature = "peg", feature = "earley"))]

use camxes_rs::morphology::{analyze, Dialect, Engine, Morphology, WordClass};
use flate2::read::GzDecoder;
use std::collections::BTreeSet;
use std::io::Read;
use std::sync::OnceLock;

const PARITY_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/morphology-parity.tsv"
);
const PARITY_HEADER: &str =
    "word\tword_type\texpected_morphology\tpeg\tearley\tearley_source\tdialect\tlabel_check";

/// Headwords no Lojban morphology accepts (both engines reject them), with
/// the reason. Non-ASCII headwords (mojibake) are also treated as invalid.
const INVALID_EVERYWHERE: &[(&str, &str)] = &[
    ("cipnrbuteo", "vowel hiatus `eo` without apostrophe"),
    ("etreo", "vowel hiatus `eo` without apostrophe"),
    (
        "fiprpseudoskafirinku",
        "vowel hiatus `eu` without apostrophe",
    ),
    ("fiprtetrapleurodo", "vowel hiatus `eu` without apostrophe"),
    ("freskeo", "vowel hiatus `eo` without apostrophe"),
    ("gaurkihu", "`h` is not a Lojban letter"),
    ("ku'urpicea", "vowel hiatus `ea` without apostrophe"),
    (
        "kuaidzi",
        "no consonant cluster in the first five letters (not a brivla)",
    ),
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

/// Categories of PEG-accepted headwords on which the Earley engine (CLL
/// pipeline) gives a different split or class, or rejects: `(code, kind,
/// reason)`. `kind` says who is right: `camxes bug` (camxes accepts what
/// explicit CLL text forbids), `CLL rule` (CLL text decides it; camxes
/// follows later usage), `accepted difference` (both readings are defensible,
/// the user chose CLL) or `camxes extension` (camxes has a form that CLL does
/// not describe). The README section "Differences between the PEG (camxes)
/// and Earley (CLL) engines" gives the same reasons with examples.
const DIFFERENCE_CATEGORIES: &[(&str, &str, &str)] = &[
    (
        "medial-pair",
        "camxes bug",
        "medial consonant pair forbidden by CLL §3.7 (`zt`, `fg`, `mz`); camxes checks only some clusters",
    ),
    (
        "hesitation",
        "CLL rule",
        "`y` is a hesitation that CLL §19.14 requires to have pauses before and after; camxes reads it as a cmavo",
    ),
    (
        "vowel-initial-pause",
        "camxes bug",
        "CLL §4.9 rule 3: a word starting with a vowel must be preceded by a pause, so the camxes split before a vowel-initial word is invalid",
    ),
    (
        "comma",
        "accepted difference",
        "CLL §3.3: a comma is a syllable break, not a word boundary; camxes splits words at commas",
    ),
    (
        "name-la",
        "CLL rule",
        "CLL §4.8: a name must not contain `la`, `lai` or `doi` (except after a consonant); Earley rejects it or reads the cmavo out of it",
    ),
    (
        "consonant-name",
        "camxes extension",
        "consonant-only names (camxes `zifcme`, used for lerfu); CLL §4.8 names are syllabic words and CLL has no consonant-only form",
    ),
    (
        "vowel-grouping",
        "CLL rule",
        "CLL §3.4: vowels other than the diphthongs need an apostrophe or comma between them (\"a comma must be used\" for other groupings); camxes allows on-glides after a vowel",
    ),
    (
        "fuhivla-cluster",
        "CLL rule",
        "CLL §4.7 rule 1: a fu'ivla must have a consonant cluster in the first five letters; camxes allows a later cluster",
    ),
    (
        "hyphen-not-required",
        "CLL rule",
        "CLL §4.6: \"It is illegal to add a hyphen at a place that is not required\"; an r/n after a non-initial CVV rafsi is not a hyphen, so the word is not a lujvo (Earley: fu'ivla)",
    ),
    (
        "non-cll-rafsi",
        "CLL rule",
        "camxes lujvo built from pieces that are not CLL §4.5 rafsi forms (vowel-initial pieces, CV'V/CVV + `y`, `'` after `y`, CVCV finals, 5-letter rafsi not at the end, 4-letter rafsi without `y`, CVV rafsi with a non-diphthong); per §4.6/§4.7 Earley rejects the word or breaks it into the words it falls apart into",
    ),
];

const KNOWN_DIFFERENCES_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/known-differences.tsv"
);

/// `tests/data/known-differences.tsv`: `word`, category code (from
/// [`DIFFERENCE_CATEGORIES`]) and the Earley result (`rejected` or the
/// rendered morphology). A headword whose engines disagree must be listed
/// with its exact Earley result; an entry that no longer differs fails too.
fn known_differences() -> &'static std::collections::BTreeMap<String, (String, String)> {
    static KNOWN: OnceLock<std::collections::BTreeMap<String, (String, String)>> = OnceLock::new();
    KNOWN.get_or_init(|| {
        let text = std::fs::read_to_string(KNOWN_DIFFERENCES_PATH).expect("known-differences.tsv");
        let mut map = std::collections::BTreeMap::new();
        for line in text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
        {
            let cols: Vec<&str> = line.split('\t').collect();
            assert_eq!(cols.len(), 3, "bad known-differences row: {line}");
            assert!(
                DIFFERENCE_CATEGORIES.iter().any(|(c, _, _)| *c == cols[1]),
                "unknown category in known-differences row: {line}"
            );
            let old = map.insert(
                cols[0].to_string(),
                (cols[1].to_string(), cols[2].to_string()),
            );
            assert!(old.is_none(), "duplicate known-differences row: {line}");
        }
        map
    })
}

struct Entry {
    word: String,
    word_type: String,
}

fn corpus() -> Vec<Entry> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/lensisku-en-words.tsv.gz"
    );
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
        .filter(|e| e.word_type != "wiki")
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
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

/// Corpus and both engines' results, computed once and shared by the tests.
fn corpus_outcomes() -> &'static (Vec<Entry>, Vec<Outcome>) {
    static CELL: OnceLock<(Vec<Entry>, Vec<Outcome>)> = OnceLock::new();
    CELL.get_or_init(|| {
        let entries = corpus();
        let outcomes = analyze_all(&entries);
        (entries, outcomes)
    })
}

/// Words with class, `+obsolete` for obsolete-dialect words and (for cmavo)
/// selma'o, e.g. `lo/cmavo:LE nu/cmavo:NU` or `kuarka/fu'ivla+obsolete`.
fn render_full(m: &Morphology) -> String {
    m.words
        .iter()
        .map(|w| {
            let mut s = format!("{}/{}", w.text, w.class);
            if w.is_obsolete() {
                s.push_str("+obsolete");
            }
            if let Some(sel) = &w.selmaho {
                s.push(':');
                s.push_str(sel);
            }
            s
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `standard` / `obsolete` (any obsolete word) for an accepted result.
fn dialect_of(m: &Morphology) -> &'static str {
    if m.has_obsolete() {
        "obsolete"
    } else {
        "standard"
    }
}

/// Dictionary label (`obsolete …` or not) against the rule-based dialect.
fn label_check(word_type: &str, dialect: Option<&str>) -> &'static str {
    match (word_type.starts_with("obsolete"), dialect) {
        (_, None) => "-",
        (true, Some("obsolete")) | (false, Some("standard")) => "ok",
        (true, Some(_)) => "labeled-obsolete-form-standard",
        (false, Some(_)) => "labeled-standard-form-obsolete",
    }
}

/// One parity row. The expected morphology is the PEG result when the PEG
/// accepts (else the Earley result, else `-`). The Earley column is `ok`
/// only when its result equals the expected morphology, `differs` when it
/// accepts with a different one, and `reject` otherwise.
fn parity_row(e: &Entry, (peg, earley): &Outcome) -> String {
    let expected = peg.as_ref().or(earley.as_ref()).map(render_full);
    let earley_status = match (earley, &expected) {
        (None, _) => "reject",
        (Some(m), Some(x)) if &render_full(m) == x => "ok",
        (Some(_), _) => "differs",
    };
    let source = earley
        .as_ref()
        .map(|m| format!("{:?}", m.dialect))
        .unwrap_or_else(|| "-".into());
    let dialect = earley.as_ref().map(dialect_of);
    format!(
        "{}\t{}\t{}\t{}\t{earley_status}\t{source}\t{}\t{}",
        e.word,
        e.word_type,
        expected.as_deref().unwrap_or("-"),
        if peg.is_some() { "ok" } else { "reject" },
        dialect.unwrap_or("-"),
        label_check(&e.word_type, dialect),
    )
}

#[test]
fn morphology_parity_matches_committed_expectations() {
    let (entries, outcomes) = corpus_outcomes();
    let actual: Vec<String> = entries
        .iter()
        .zip(outcomes)
        .map(|(e, o)| parity_row(e, o))
        .collect();
    if std::env::var_os("CAMXES_BLESS_PARITY").is_some() {
        let mut text = String::from(PARITY_HEADER);
        for row in &actual {
            text.push('\n');
            text.push_str(row);
        }
        text.push('\n');
        std::fs::write(PARITY_PATH, text).expect("write parity file");
        return;
    }
    let committed = std::fs::read_to_string(PARITY_PATH).expect("tests/data/morphology-parity.tsv");
    let mut lines = committed.lines();
    assert_eq!(lines.next(), Some(PARITY_HEADER));
    let expected: Vec<&str> = lines.collect();
    assert_eq!(
        expected.len(),
        actual.len(),
        "parity file and corpus differ in size"
    );
    let mismatches: Vec<String> = expected
        .iter()
        .zip(&actual)
        .filter(|(x, a)| *x != a)
        .map(|(x, a)| format!("expected {x}\n  actual {a}"))
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} parity mismatches (first 50):\n{}",
        mismatches.len(),
        mismatches
            .iter()
            .take(50)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    let known = known_differences();
    let unexpected: Vec<&String> = actual
        .iter()
        .filter(|r| r.contains("\tdiffers\t"))
        .filter(|r| !known.contains_key(r.split('\t').next().unwrap_or("")))
        .collect();
    assert!(
        unexpected.is_empty(),
        "unexpected differences: {unexpected:?}"
    );
}

/// Rule-based dialect: a result is obsolete exactly when the standard
/// (PEG/camxes) rules reject it; dictionary labels are only reported.
#[test]
fn obsolete_dialect_is_rule_based() {
    let (entries, outcomes) = corpus_outcomes();
    let mut bad = Vec::new();
    let (mut obs_ok, mut obs_std, mut std_obs) = (Vec::new(), Vec::new(), Vec::new());
    for (e, (peg, earley)) in entries.iter().zip(outcomes) {
        let Some(m) = earley else { continue };
        // Known differences are other words (another split), so the
        // camxes rules do not describe them.
        if peg.is_some() == m.has_obsolete() && !known_differences().contains_key(&e.word) {
            bad.push(format!(
                "{} (peg ok: {}): {}",
                e.word,
                peg.is_some(),
                render_full(m)
            ));
        }
        match label_check(&e.word_type, Some(dialect_of(m))) {
            "labeled-obsolete-form-standard" => obs_std.push(e.word.as_str()),
            "labeled-standard-form-obsolete" => std_obs.push(e.word.as_str()),
            _ if m.has_obsolete() => obs_ok.push(e.word.as_str()),
            _ => {}
        }
    }
    println!(
        "obsolete by rule and label: {}; labeled obsolete but standard form: {}; \
         labeled standard but obsolete form: {} {:?}",
        obs_ok.len(),
        obs_std.len(),
        std_obs.len(),
        std_obs
    );
    assert!(
        bad.is_empty(),
        "dialect does not follow the standard rules:\n{}",
        bad.join("\n")
    );
}

#[test]
fn dictionary_headwords_morphology() {
    let (entries, outcomes) = corpus_outcomes();
    assert!(
        entries.len() > 30_000,
        "corpus too small: {}",
        entries.len()
    );

    let invalid: BTreeSet<&str> = INVALID_EVERYWHERE.iter().map(|(w, _)| *w).collect();
    let known = known_differences();
    let mut known_seen = BTreeSet::new();
    let (mut peg_ok, mut agree, mut earley_only) = (0, 0, 0);
    let (mut obsolete, mut obsolete_earley, mut obsolete_peg) = (0, 0, 0);
    let mut disagreements = Vec::new();
    let mut unexpected_failures = Vec::new();
    let mut stale_invalid = Vec::new();
    let mut per_category: std::collections::BTreeMap<&str, usize> = Default::default();

    for (e, (peg, earley)) in entries.iter().zip(outcomes) {
        if e.word_type.starts_with("obsolete") {
            obsolete += 1;
            obsolete_peg += usize::from(peg.is_some());
            obsolete_earley += usize::from(earley.is_some());
        }
        match (peg, earley) {
            (Some(p), q) => {
                peg_ok += 1;
                if q.as_ref().is_some_and(|q| p.signature() == q.signature()) {
                    agree += 1;
                    continue;
                }
                let got = q
                    .as_ref()
                    .map_or_else(|| "rejected".to_string(), |q| q.render());
                match known.get(&e.word) {
                    Some((category, recorded)) if *recorded == got => {
                        known_seen.insert(e.word.as_str());
                        *per_category.entry(category.as_str()).or_default() += 1;
                        println!("known difference [{category}] {}: peg [{}] earley [{got}]", e.word, p.render());
                    }
                    Some((category, recorded)) => disagreements.push(format!(
                        "{}: peg [{}] earley [{got}], but known-differences.tsv records [{recorded}] ({category})",
                        e.word,
                        p.render()
                    )),
                    None => disagreements.push(format!(
                        "{}: peg [{}] earley [{got}]  (new row: {}\t<category>\t{got})",
                        e.word,
                        p.render(),
                        e.word
                    )),
                }
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
        "corpus {}: PEG accepted {peg_ok}, Earley agrees on {agree}; \
         Earley-only {earley_only}; obsolete {obsolete} (PEG {obsolete_peg}, Earley {obsolete_earley})",
        entries.len()
    );
    assert!(
        disagreements.is_empty(),
        "{} disagreements:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
    assert!(
        unexpected_failures.is_empty(),
        "Earley rejected {} headwords:\n{}",
        unexpected_failures.len(),
        unexpected_failures.join("\n")
    );
    assert!(
        stale_invalid.is_empty(),
        "now accepted, drop from INVALID_EVERYWHERE: {stale_invalid:?}"
    );
    let stale_known: Vec<&String> = known
        .keys()
        .filter(|w| !known_seen.contains(w.as_str()))
        .collect();
    assert!(
        stale_known.is_empty(),
        "no longer differ, drop from known-differences.tsv: {stale_known:?}"
    );
    println!("known differences by category: {per_category:?}");
    assert_eq!(peg_ok, agree + known.len());
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

/// Made-up (non-dictionary) words: obsolete classes are found by rule.
#[test]
fn made_up_obsolete_words_are_flagged() {
    use WordClass::*;
    for (input, expect) in [
        // on-glide diphthongs (`ia`, `ua`) inside a fu'ivla
        ("spatrkuandi", vec![("spatrkuandi", Fuhivla, true)]),
        ("tiadrome", vec![("tiadrome", Fuhivla, true)]),
        // `n` before an affricate (`ndj`), allowed by CLL §3.7, banned by camxes
        ("djinundji", vec![("djinundji", Lujvo, true)]),
        // CVV'V'V cmavo with an on-glide
        ("cie'a'u", vec![("cie'a'u", Cmavo, true)]),
        // on-glide diphthong in a name
        ("tiuanJOUS", vec![("tiuanJOUS", Cmevla, true)]),
        // zei-lujvo: only the obsolete component is flagged
        (
            "lojbo zei kuarbli",
            vec![
                ("lojbo", Gismu, false),
                ("zei", Cmavo, false),
                ("kuarbli", Fuhivla, true),
            ],
        ),
        // standard made-up words stay standard
        ("pluviobasu", vec![("pluviobasu", Fuhivla, false)]),
        ("zbanunkla", vec![("zbanunkla", Lujvo, false)]),
        ("blaxi", vec![("blaxi", Gismu, false)]),
    ] {
        let m = analyze(input, Engine::Earley).unwrap_or_else(|e| panic!("{input}: {e}"));
        let got: Vec<(&str, WordClass, bool)> = m
            .words
            .iter()
            .map(|w| (w.text.as_str(), w.class, w.is_obsolete()))
            .collect();
        assert_eq!(got, expect, "{input}");
        let peg = analyze(input, Engine::Peg);
        assert_eq!(
            peg.is_ok(),
            !m.has_obsolete(),
            "{input}: PEG acceptance vs obsolete flag"
        );
    }
}
