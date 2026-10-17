//! Benchmarks: grammar startup and whole-text parse with both engines.
//!
//! `cargo bench` (PEG only) or `cargo bench --features earley`.
//!
//! The Earley engine is the vendored CLL pipeline (`src/cll`): startup is
//! compiling `src/cll/data/cll21.ebnf`; parsing goes through
//! `syntax::parse(.., Engine::Earley)`. The CLL word analysis keeps a
//! per-thread cache, so repeated iterations measure warm-cache parsing.

#[cfg(feature = "peg")]
use camxes_rs::grammars::LOJBAN_GRAMMAR;
#[cfg(feature = "peg")]
use camxes_rs::peg::grammar::Peg;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

const SHORT: &str = "mi prami do";

fn medium() -> String {
    "mi prami do .i do prami mi ".repeat(20)
}

fn bench_grammar_startup(c: &mut Criterion) {
    #[cfg(feature = "peg")]
    c.bench_function("peg_new_lojban_grammar", |b| {
        b.iter(|| black_box(Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap()))
    });
    #[cfg(feature = "earley")]
    c.bench_function("earley_compile_cll21_ebnf", |b| {
        const CLL21: &str = include_str!("../src/cll/data/cll21.ebnf");
        b.iter(|| black_box(camxes_rs::cll::grammar::compile_ebnf(CLL21, "text").unwrap()))
    });
}

fn bench_parse(c: &mut Criterion) {
    let medium = medium();
    #[cfg(feature = "peg")]
    {
        let peg = Peg::new(LOJBAN_GRAMMAR.0, LOJBAN_GRAMMAR.1).unwrap();
        c.bench_function("peg_parse_short", |b| {
            b.iter(|| black_box(peg.parse(black_box(SHORT))))
        });
        c.bench_function("peg_parse_medium", |b| {
            b.iter(|| black_box(peg.parse(black_box(medium.as_str()))))
        });
    }
    #[cfg(feature = "earley")]
    {
        use camxes_rs::{morphology::Engine, syntax};
        c.bench_function("earley_parse_short", |b| {
            b.iter(|| black_box(syntax::parse(black_box(SHORT), Engine::Earley)))
        });
        c.bench_function("earley_parse_medium", |b| {
            b.iter(|| black_box(syntax::parse(black_box(medium.as_str()), Engine::Earley)))
        });
    }
}

criterion_group!(benches, bench_grammar_startup, bench_parse);
criterion_main!(benches);
