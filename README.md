# 🚀 Rust PEG Parser Generator

[![Crates.io](https://img.shields.io/crates/v/camxes-rs)](https://crates.io/crates/camxes-rs)
[![Documentation](https://docs.rs/camxes-rs/badge.svg)](https://docs.rs/camxes-rs)
[![License](https://img.shields.io/crates/l/camxes-rs)](LICENSE)

A lightning-fast Parsing Expression Grammar (PEG) parser generator implemented in Rust. This tool helps you create parsers from grammar definitions with minimal hassle.

## 📚 Documentation

Full API documentation is available on [docs.rs](https://docs.rs/camxes-rs)

## ✨ Features

- **Zero-Copy Parsing**: Efficient parsing without unnecessary string allocations
- **Rich Debugging**: Detailed logging for grammar validation and parsing process
- **Flexible Grammar Syntax**: Supports all standard PEG operators and extensions
- **Error Recovery**: Robust error handling with detailed diagnostic messages
- **Thread-Safe**: Designed for concurrent use with thread-safe grammar structures

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
camxes-rs = "2.0.0"
```

## 🚦 Quick Start

### Local testing

You must have latest Rust installed.

Go to the project directory and run `cargo run --example cmaxes-test`.
It will run the PEG grammar specified in `src/examples/cmaxes-test.rs` file against a sample input.


### Library example

```rust
use camxes_rs::peg::grammar::Peg;

let grammar = r#"
expression <- term (('+' / '-') term)*
term <- factor (('*' / '/') factor)*
factor <- number / '(' expression ')'
number <- [0-9]+
"#;
let parser = Peg::new("expression", grammar).unwrap();
let result = parser.parse("2+3*4");
assert!(result.2.is_ok());
assert_eq!(result.1, 5); // bytes matched
```

## 🔧 Grammar Syntax

The grammar supports these PEG operators:

| Operator | Description | Example |
|----------|-------------|---------|
| `<-`     | Definition | `rule <- expression` |
| `/`      | Ordered choice | `a / b` |
| `*`      | Zero or more | `[0-9]*` |
| `+`      | One or more | `[a-z]+` |
| `?`      | Optional | `[A-Z]?` |
| `&`      | And-predicate | `&[a-z]` |
| `!`      | Not-predicate | `![0-9]` |
| `()`     | Grouping | `(a / b)` |
| `[]`     | Character range / class | `[abd]` and `[a-zA-Z]` |
| `.`      | Any character | `.` |

## 🧬 Lojban morphology and the Earley engine

`camxes_rs::morphology::analyze` splits a word or cmavo cluster into words and
classifies them (cmavo with selma'o, gismu, lujvo, fu'ivla, cmevla):

```rust
use camxes_rs::morphology::{analyze, Engine};

let m = analyze("lojbo", Engine::Peg).unwrap();
println!("{:?}", m.words);
```

With the optional `earley` feature an alternative engine is available:

```toml
[dependencies]
camxes-rs = { version = "2.0.0", features = ["earley"] }
```

`Engine::Earley` runs the same grammar on an Earley chart parser (with PEG
lookahead and ordered-choice preference) and falls back to the CLL morphology
of [tergenzalvi](https://github.com/lagleki/tergenzalvi) (vendored in the
crate as `camxes_rs::cll`), so it accepts both
the camxes dialect and the older CLL dialect (obsolete dictionary words such as
`kuarka` that the PEG engine rejects). For words the PEG accepts, both
engines return the same segmentation and word classes.

### Whole-text syntax

`camxes_rs::syntax` checks complete Lojban texts against the camxes grammar
(`grammars::LOJBAN_GRAMMAR`) with either engine:

```rust
use camxes_rs::morphology::Engine;
use camxes_rs::syntax;

assert!(syntax::accepts("lo prenu poi klama cu tavla", Engine::Peg));
assert!(!syntax::accepts("lo cu klama", Engine::Peg));
```

The Earley engine first takes the PEG-preferred derivation and otherwise any
derivation covering the text. `zoi`/`la'o` quotes need delimiter
back-references neither engine implements: the PEG engine rejects them, the
Earley engine accepts them without checking that the delimiters match.
`tests/grammar_basics.rs` lists the constructions tested.

### Standard vs obsolete words

Each word of a result carries a rule-based `word_dialect`: `Standard` when the
standard camxes morphology rules accept the word with that class, `Obsolete`
when only the older CLL (Chapters 3–4) rules accept it. No dictionary lookup is
involved, so made-up words are classified the same way:

```rust
use camxes_rs::morphology::{analyze, Engine};

let m = analyze("lojbo zei kuarbli", Engine::Earley).unwrap();
let labels: Vec<String> = m.words.iter().map(|w| w.type_label()).collect();
assert_eq!(labels, ["gismu", "cmavo", "obsolete fu'ivla"]);
```

Typical obsolete forms: on-glide diphthongs `iV`/`uV` inside brivla and names
(`kuarka`, `tiuanJOUS`), `n` before an affricate (`djinundji`; allowed by CLL
§3.7, banned by camxes), diphthong + apostrophe + vowel (`gondii'i`) and
`CiV'V'V` cmavo (`cie'a'u`). Gismu rules are the same in both dialects, so no
gismu is ever obsolete. Words the PEG engine accepts are always standard.

Corpus check and benchmark over all lensisku headwords (`wiki` page-title
entries are excluded):

```sh
cargo test --release --features earley --test dictionary_corpus
cargo run --release --features earley --example morphology_corpus -- --raw
python3 scripts/fetch_lensisku_corpus.py   # refresh tests/data
```

`tests/data/morphology-parity.tsv` documents the expected morphology (with
obsolete flags), each engine's acceptance, the rule-based dialect, and whether
the dictionary's `obsolete` label agrees with it (`label_check`) for every
headword; the test asserts both engines match it.
After an intended change, regenerate it with
`CAMXES_BLESS_PARITY=1 cargo test --release --features earley --test dictionary_corpus`
and review the diff. The example also writes a per-word report to
`target/corpus-results.tsv`.

## 🔍 Debugging

Enable debug logging to see detailed parsing information:

```rust
// Initialize logging (using env_logger)
env_logger::builder()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

Or set the environment variable:
```bash
RUST_LOG=debug cargo run
```

## 📚 Examples

### Simple Calculator Grammar
```rust
let calc_grammar = (
    "calc",
    r#"
    calc <- additive
    additive <- multiplicative (('+' / '-') multiplicative)*
    multiplicative <- primary (('*' / '/') primary)*
    primary <- number / '(' additive ')'
    number <- [0-9]+ 
    "#
);
```

### JSON Parser Grammar
```rust
let json_grammar = (
    "json",
    r#"
    json <- spacing value spacing
    value <- object / array / string / number / true / false / null
    object <- '{' spacing (pair (',' pair)*)? '}' 
    pair <- string spacing ':' spacing value
    array <- '[' spacing (value (',' value)*)? ']'
    string <- '"' (!'"' .)* '"'
    number <- '-'? ([0-9] / [1-9][0-9]*) ('.' [0-9]+)? ([eE][-+]?[0-9]+)?
    true <- 'true'
    false <- 'false'
    null <- 'null'
    spacing <- [ \t\n\r]*
    "#
);
```

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Based on Bryan Ford's [Parsing Expression Grammars](https://bford.info/pub/lang/peg.pdf) paper
- Inspired by various PEG implementations in the Rust ecosystem

---

## 🌐 Loglan WASM Parser

A web-based interface for the Loglan parser is available in the `loglan-wasm-app` directory. See the [Loglan WASM App README](loglan-wasm-app/README.md) for instructions on how to build and run it.

---

Built with ❤️ for the Rust community
