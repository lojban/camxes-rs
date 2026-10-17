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
                    Ok(m) => println!("{:7} {w:20} {:?} {}", e.name(), m.dialect, m.render()),
                    Err(err) => println!("{:7} {w:20} ERR {err}", e.name()),
                }
            }
        }
        return;
    }
    let raw = args.iter().any(|a| a == "--raw");
    let args: Vec<String> = args.into_iter().filter(|a| a != "--raw").collect();
    let path = args
        .first()
        .cloned()
        .unwrap_or_else(|| concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/lensisku-en-words.tsv.gz").into());
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
                disagree.push(format!("{word}\t{ty}\tpeg: {}\tearley: {}", pm.render(), em.render()));
            }
            (Err(_), Ok(em)) => {
                st[5] += 1;
                peg_only_rej.push(format!("{word}\t{ty}\t{:?}\tearley: {}", em.dialect, em.render()));
            }
            _ => {}
        }
        if let Err(err) = &e {
            earley_fail.push(format!(
                "{word}\t{ty}\tpeg: {}\t{err}",
                p.as_ref().map(|m| m.render()).unwrap_or_else(|_| "ERR".into())
            ));
        }
    }
    let n = entries.len();
    println!("entries: {n}");
    println!("{:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}", "word_type", "total", "pegOK", "earOK", "agree", "differ", "pegRej+");
    let mut tot = [0usize; 6];
    for (ty, s) in &stats {
        println!("{ty:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}", s[0], s[1], s[2], s[3], s[4], s[5]);
        for i in 0..6 {
            tot[i] += s[i];
        }
    }
    println!("{:24} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8}", "TOTAL", tot[0], tot[1], tot[2], tot[3], tot[4], tot[5]);
    println!(
        "time: peg {:?} ({:.0} words/s), earley {:?} ({:.0} words/s)",
        tp,
        n as f64 / tp.as_secs_f64(),
        te,
        n as f64 / te.as_secs_f64()
    );
    slow_p.sort_by(|a, b| b.0.cmp(&a.0));
    slow_e.sort_by(|a, b| b.0.cmp(&a.0));
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
}

/// Same grammar, both engines, no adapters or fallbacks.
fn raw_grammar_timing(entries: &[(String, String)]) {
    use camxes_rs::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
    use camxes_rs::morphology::camxes_morphology_grammar;
    use camxes_rs::peg::grammar::Peg;
    let peg = Peg::new(LOJBAN_MORPHOLOGY_GRAMMAR.0, LOJBAN_MORPHOLOGY_GRAMMAR.1).unwrap();
    let earley = camxes_morphology_grammar();
    let inputs: Vec<String> = entries
        .iter()
        .map(|(w, _)| format!("{} ", w.to_ascii_lowercase()))
        .collect();
    let bytes: usize = inputs.iter().map(|s| s.len()).sum();
    let mut same_end = 0usize;
    let t = Instant::now();
    let pe: Vec<Option<usize>> = inputs
        .iter()
        .map(|i| {
            let r = peg.parse(i);
            r.2.is_ok().then_some(r.1)
        })
        .collect();
    let tp = t.elapsed();
    let t = Instant::now();
    let ee: Vec<Option<usize>> = inputs
        .iter()
        .map(|i| {
            let r = earley.parse(i);
            r.2.is_ok().then_some(r.1)
        })
        .collect();
    let te = t.elapsed();
    for (a, b) in pe.iter().zip(&ee) {
        if a == b {
            same_end += 1;
        }
    }
    let mut worst: Vec<(Duration, &str)> = inputs
        .iter()
        .map(|i| {
            let t = Instant::now();
            let _ = earley.parse(i);
            (t.elapsed(), i.as_str())
        })
        .collect();
    worst.sort_by(|a, b| b.0.cmp(&a.0));
    println!("raw grammar on {} inputs ({} bytes)", inputs.len(), bytes);
    println!("  peg    {:?}  ({:.0} inputs/s, {:.0} KB/s)", tp, inputs.len() as f64 / tp.as_secs_f64(), bytes as f64 / 1024.0 / tp.as_secs_f64());
    println!("  earley {:?}  ({:.0} inputs/s, {:.0} KB/s)", te, inputs.len() as f64 / te.as_secs_f64(), bytes as f64 / 1024.0 / te.as_secs_f64());
    println!("  same match end: {same_end}/{}", inputs.len());
    for (d, i) in worst.iter().take(5) {
        println!("  slow earley {:?} {:.60}", d, i);
    }
}
