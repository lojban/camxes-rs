//! Benchmarks: grammar startup and whole-text parse with both engines.
//!
//! `cargo bench` (PEG only) or `cargo bench --features earley`.

use camxes_rs::grammars::LOJBAN_GRAMMAR;
use camxes_rs::peg::grammar::Peg;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

const SHORT: &str = "mi prami do";

fn medium() -> String {
    "mi prami do .i do prami mi ".repeat(20)
}

fn bench_grammar_startup(c: &mut Criterion) {
    c.bench_function("peg_new_lojban_grammar", |b| {
        b.iter(|| black_box(Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap()))
    });
    #[cfg(feature = "earley")]
    c.bench_function("earley_new_lojban_grammar", |b| {
        b.iter(|| {
            black_box(
                camxes_rs::earley::EarleyGrammar::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap(),
            )
        })
    });
}

fn bench_parse(c: &mut Criterion) {
    let peg = Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap();
    let medium = medium();
    c.bench_function("peg_parse_short", |b| {
        b.iter(|| black_box(peg.parse(black_box(SHORT))))
    });
    c.bench_function("peg_parse_medium", |b| {
        b.iter(|| black_box(peg.parse(black_box(medium.as_str()))))
    });
    #[cfg(feature = "earley")]
    {
        let earley =
            camxes_rs::earley::EarleyGrammar::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap();
        c.bench_function("earley_parse_short", |b| {
            b.iter(|| black_box(earley.parse(black_box(SHORT))))
        });
        c.bench_function("earley_parse_medium", |b| {
            b.iter(|| black_box(earley.parse(black_box(medium.as_str()))))
        });
    }
}

criterion_group!(benches, bench_grammar_startup, bench_parse);
criterion_main!(benches);
