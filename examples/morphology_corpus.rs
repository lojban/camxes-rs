//! Compare PEG and Earley morphology over the lensisku dictionary headwords.
//!
//! ```bash
//! cargo run --release --features earley --example morphology_corpus -- [corpus.tsv.gz] [--words w1,w2]
//! ```
//! Prints agreement counts, disagreements, PEG-only rejections, Earley
//! failures, and timing for both engines.

use camxes_rs::morphology::{analyze, Engine, Morphology, MorphologyError};
use flate2::read::GzDecoder;
use std::collections::BTreeMap;
use std::io::Read;
use std::time::{Duration, Instant};

fn load(path: &str) -> Vec<(String, String)> {
    let mut s = String::new();
    GzDecoder::new(std::fs::File::open(path).expect("corpus"))
        .read_to_string(&mut s)
        .expect("gzip");
    s.lines()
        .skip(1)
        .filter_map(|l| {
            let mut c = l.split('\t');
            Some((c.next()?.to_string(), c.next()?.to_string()))
        })
        // `wiki` entries are English page titles, not Lojban headwords.
        .filter(|(_, t)| t != "wiki")
        .collect()
}

fn timed(word: &str, e: Engine) -> (Result<Morphology, MorphologyError>, Duration) {
    let t = Instant::now();
    let r = analyze(word, e);
    (r, t.elapsed())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(i) = args.iter().position(|a| a == "--words") {
        for w in args[i + 1].split(',') {
            for e in Engine::available() {
                match analyze(w, *e) {
                    Ok(m) => println!(
                        "{:7} {w:20} {:?} {}",
                        e.name(),
                        m.dialect,
                        m.words
                            .iter()
                            .map(|x| format!("{}/{}", x.text, x.type_label()))
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                    Err(err) => println!("{:7} {w:20} ERR {err}", e.name()),
                }
            }
        }
        return;
    }
    let raw = args.iter().any(|a| a == "--raw");
    let args: Vec<String> = args.into_iter().filter(|a| a != "--raw").collect();
    let path = args.first().cloned().unwrap_or_else(|| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/data/lensisku-en-words.tsv.gz"
        )
        .into()
    });
    let entries = load(&path);
    if raw {
        raw_grammar_timing(&entries);
        return;
    }
    let mut stats: BTreeMap<String, [usize; 6]> = BTreeMap::new(); // total, peg ok, earley ok, agree, disagree, peg-rej earley-ok
    let (mut tp, mut te) = (Duration::ZERO, Duration::ZERO);
    let mut slow_p: Vec<(Duration, String)> = vec![];
    let mut slow_e: Vec<(Duration, String)> = vec![];
    let mut disagree = vec![];
    let mut earley_fail = vec![];
    let mut peg_only_rej = vec![];
    // Full per-word report: one row per headword.
    let mut results =
        vec!["word\tword_type\tpeg\tearley\tstatus\tearley_source\tdialect".to_string()];
    let cell = |r: &Result<Morphology, MorphologyError>| match r {
        Ok(m) => m
            .words
            .iter()
            .map(|w| format!("{}/{}", w.text, w.type_label()))
            .collect::<Vec<_>>()
            .join(" "),
        Err(err) => format!("ERR {err}").replace(['\t', '\n'], " "),
    };
    for (word, ty) in &entries {
        let (p, dp) = timed(word, Engine::Peg);
        let (e, de) = timed(word, Engine::Earley);
        tp += dp;
        te += de;
        slow_p.push((dp, word.clone()));
        slow_e.push((de, word.clone()));
        let st = stats.entry(ty.clone()).or_default();
        st[0] += 1;
        if p.is_ok() {
            st[1] += 1;
        }
        if e.is_ok() {
            st[2] += 1;
        }
        match (&p, &e) {
            (Ok(pm), Ok(em)) if pm.signature() == em.signature() => st[3] += 1,
            (Ok(pm), Ok(em)) => {
                st[4] += 1;
                disagree.push(format!(
                    "{word}\t{ty}\tpeg: {}\tearley: {}",
                    pm.render(),
                    em.render()
                ));
            }
            (Err(_), Ok(em)) => {
                st[5] += 1;
                peg_only_rej.push(format!(
                    "{word}\t{ty}\t{:?}\tearley: {}",
                    em.dialect,
                    em.render()
                ));
            }
            _ => {}
        }
        let status = match (&p, &e) {
            (Ok(pm), Ok(em)) if pm.signature() == em.signature() => "agree",
            (Ok(_), _) => "disagree",
            (Err(_), Ok(_)) => "earley-only",
            (Err(_), Err(_)) => "both-reject",
        };
        results.push(format!(
            "{word}\t{ty}\t{}\t{}\t{status}\t{}\t{}",
            cell(&p),
            cell(&e),
            e.as_ref()
                .map(|m| format!("{:?}", m.dialect))
                .unwrap_or_default(),
            match &e {
                Ok(m) if m.has_obsolete() => "obsolete",
                Ok(_) => "standard",
                Err(_) => "-",
            }
        ));
        if let Err(err) = &e {
            earley_fail.push(format!(
                "{word}\t{ty}\tpeg: {}\t{err}",
                p.as_ref()
                    .map(|m| m.render())
                    .unwrap_or_else(|_| "ERR".into())
            ));
        }
    }
    let n = entries.len();
    println!("entries: {n}");
    println!(
        "{:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}",
        "word_type", "total", "pegOK", "earOK", "agree", "differ", "pegRej+"
    );
    let mut tot = [0usize; 6];
    for (ty, s) in &stats {
        println!(
            "{ty:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}",
            s[0], s[1], s[2], s[3], s[4], s[5]
        );
        for i in 0..6 {
            tot[i] += s[i];
        }
    }
    println!(
        "{:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}",
        "TOTAL", tot[0], tot[1], tot[2], tot[3], tot[4], tot[5]
    );
    println!(
        "time: peg {:?} ({:.0} words/s), earley {:?} ({:.0} words/s)",
        tp,
        n as f64 / tp.as_secs_f64(),
        te,
        n as f64 / te.as_secs_f64()
    );
    slow_p.sort_by_key(|a| std::cmp::Reverse(a.0));
    slow_e.sort_by_key(|a| std::cmp::Reverse(a.0));
    println!("slowest peg: {:?}", &slow_p[..5.min(n)]);
    println!("slowest earley: {:?}", &slow_e[..5.min(n)]);
    let dump = |name: &str, v: &Vec<String>| {
        let out = format!("{}/target/corpus-{name}.tsv", env!("CARGO_MANIFEST_DIR"));
        std::fs::write(&out, v.join("\n")).unwrap();
        println!("{name}: {} (written to {out})", v.len());
    };
    dump("disagree", &disagree);
    dump("earley-fail", &earley_fail);
    dump("peg-rejected-earley-ok", &peg_only_rej);
    let out = format!("{}/target/corpus-results.tsv", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(&out, results.join("\n") + "\n").unwrap();
    println!(
        "results: {} headwords (written to {out})",
        results.len() - 1
    );
}

/// Each engine's core word recogniser, without the `morphology` adapters:
/// the camxes PEG morphology grammar versus the vendored CLL single-word
/// pipeline (`cll::constraints::morph_parse`: Earley shape grammar plus
/// constraint filters). They are not the same grammar, so the acceptance
/// counts differ (the PEG grammar also splits cmavo clusters, the CLL
/// recogniser accepts a single word only).
fn raw_grammar_timing(entries: &[(String, String)]) {
    use camxes_rs::cll::constraints::morph_parse;
    use camxes_rs::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
    use camxes_rs::peg::grammar::Peg;
    let peg = Peg::new(LOJBAN_MORPHOLOGY_GRAMMAR.0, LOJBAN_MORPHOLOGY_GRAMMAR.1).unwrap();
    let inputs: Vec<String> = entries
        .iter()
        .map(|(w, _)| w.to_ascii_lowercase())
        .collect();
    let bytes: usize = inputs.iter().map(|s| s.len()).sum();
    let t = Instant::now();
    let peg_ok = inputs
        .iter()
        .filter(|i| {
            let input = format!("{i} ");
            let r = peg.parse(&input);
            r.2.is_ok() && r.1 == input.len()
        })
        .count();
    let tp = t.elapsed();
    let mut worst: Vec<(Duration, &str)> = Vec::with_capacity(inputs.len());
    let mut cll_ok = 0usize;
    let t = Instant::now();
    for i in &inputs {
        let t1 = Instant::now();
        if morph_parse(i).accepted {
            cll_ok += 1;
        }
        worst.push((t1.elapsed(), i.as_str()));
    }
    let te = t.elapsed();
    worst.sort_by_key(|a| std::cmp::Reverse(a.0));
    println!(
        "raw recognisers on {} inputs ({} bytes)",
        inputs.len(),
        bytes
    );
    println!(
        "  peg grammar     {:?}  ({:.0} inputs/s, {:.0} KB/s), accepted {peg_ok}",
        tp,
        inputs.len() as f64 / tp.as_secs_f64(),
        bytes as f64 / 1024.0 / tp.as_secs_f64()
    );
    println!(
        "  cll morph_parse {:?}  ({:.0} inputs/s, {:.0} KB/s), accepted {cll_ok}",
        te,
        inputs.len() as f64 / te.as_secs_f64(),
        bytes as f64 / 1024.0 / te.as_secs_f64()
    );
    for (d, i) in worst.iter().take(5) {
        println!("  slow cll {:?} {:.60}", d, i);
    }
}
