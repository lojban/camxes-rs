# camxes-rs — PEG and Earley parsers for Lojban

[![Crates.io](https://img.shields.io/crates/v/camxes-rs)](https://crates.io/crates/camxes-rs)
[![Documentation](https://docs.rs/camxes-rs/badge.svg)](https://docs.rs/camxes-rs)
[![License: MIT](https://img.shields.io/crates/l/camxes-rs)](LICENSE)

camxes-rs is a PEG parser generator with the camxes Lojban grammars built in,
plus an optional second engine (Cargo feature `earley`):

* **PEG engine** (default) — packrat-style PEG interpreter: ordered choice,
  greedy repetition, no backtracking into a committed choice.
* **Earley engine** (`earley`) — compiles the *same* PEG grammar text into a
  context-free grammar with lookahead symbols and parses it with an Earley
  chart, picking the PEG-preferred derivation when there is one. With it come
  the vendored CLL (Complete Lojban Language) morphology and Chapter 21
  grammar of [tergenzalvi](https://github.com/lagleki/tergenzalvi), which
  also accept the older CLL word forms (obsolete words such as `kuarka`).

Both engines are exposed through the same Lojban APIs: `morphology::analyze`
(word split + classes) and `syntax::parse` / `syntax::accepts` (whole texts).

API documentation: [docs.rs/camxes-rs](https://docs.rs/camxes-rs).

## Installation

```toml
[dependencies]
camxes-rs = "2.0.0"                                         # PEG engine only
# camxes-rs = { version = "2.0.0", features = ["earley"] }  # + Earley / CLL
```

| Cargo feature | Default | Adds |
|---------------|---------|------|
| *(none)*      | yes     | `peg`, `earley::EarleyGrammar` (generic engine), `grammars`, `morphology` with `Engine::Peg`, `syntax` with `Engine::Peg` |
| `earley`      | no      | `Engine::Earley` for `morphology` and `syntax`, the vendored `cll` module (CLL morphology, cmavo lexicon, Chapter 21 parser) |

The generic `earley::EarleyGrammar` is always compiled; the feature only
switches on the Lojban `Engine::Earley` and the `cll` module.

## Repository layout

| Path | What it is |
|------|------------|
| `src/peg/**` | PEG engine: `grammar/` (`Peg::new`, `Peg::parse`, bootstrap grammar of the PEG notation), `rule/` (rule tree and its matcher), `transformer/` (grammar text → rules), `parsing.rs` (`ParseResult`, `ParseNode`, `Span`, errors), `annotations.rs` (`#: PUSH`/`#: PEEK-DIFF`/`#: POP-EQ` handling) |
| `src/grammars.rs` | Embedded PEG grammars: `LOJBAN_MORPHOLOGY_GRAMMAR` (camxes word-level morphology, start `text`), `LOJBAN_GRAMMAR` (full camxes syntax from ilmentufa, start `text`) |
| `src/earley/mod.rs` | Generic Earley engine API: `EarleyGrammar::new`, `parse`, `parse_complete`, `analyze_ends`, `parse_to_json` |
| `src/earley/compile.rs` | PEG text → CFG compiler (reuses the PEG front end; adds lookahead and quote symbols, FIRST/nullable sets) |
| `src/earley/recognize.rs` | Earley chart, lookahead sub-recognitions, delimiter-matching quotes, PEG-preferred derivation extraction |
| `src/morphology/mod.rs` | Engine-independent morphology API and result types (`analyze`, `Morphology`, `MorphWord`, `WordClass`, `WordDialect`, `Dialect`, `Engine`) |
| `src/morphology/peg.rs` | PEG adapter: `LOJBAN_MORPHOLOGY_GRAMMAR` on the PEG engine, words read off the tree |
| `src/morphology/earley.rs` | Earley adapter: `LOJBAN_MORPHOLOGY_GRAMMAR` on the Earley engine, then CLL fallback, rule-based standard/obsolete decision |
| `src/syntax.rs` | Whole-text syntax check (`parse`, `accepts`, `earley_grammar`) over `LOJBAN_GRAMMAR` with either engine |
| `src/cll/**` | Vendored tergenzalvi (feature `earley`): `phonology/` (letters, diphthongs, consonant pair/triple tables, CLL ch. 3), `shapes/` (CV shape recognition), `cfg/` (morphology CFG built in `cfg/grammar.rs` + its Earley recognizer), `constraints/` (CLL ch. 4 filters: tosmabru, slinku'i, cmevla `la` ban, lujvo hyphens, rafsi/fu'ivla, stress, pauses, class exclusion), `lex/` (stream segmentation, tokens), `tables/` (rafsi data), `grammar/` (CLL ch. 21: cmavo lexicon, lexemes, EBNF compiler, Earley parser, elision, SI/SA/SU erasure) |
| `src/cll/data/cll21.ebnf` | CLL Chapter 21 EBNF grammar (compiled at runtime by `cll::grammar`) |
| `src/cll/data/cmavo_selmaho.tsv` | cmavo → selma'o lexicon (used for `MorphWord::selmaho` and the ch. 21 lexer) |
| `src/examples/cmaxes-test.rs` | Example: run `LOJBAN_MORPHOLOGY_GRAMMAR` over `examples/lujvo_tests.tsv` |
| `examples/morphology_corpus.rs` | Example: PEG vs Earley over the dictionary corpus, timings, per-word report |
| `tests/grammar_basics.rs` | Syntax test: basic constructions on both engines, `zoi`/`la'o` delimiters |
| `tests/dictionary_corpus.rs`, `tests/data/` | Dictionary corpus + committed morphology parity expectations (repository only, not in the crate package) |
| `benches/parser.rs` | Criterion benchmarks (grammar startup, parsing) |

### Where are the grammars?

* **PEG grammar and morphology:** `src/grammars.rs`
  (`LOJBAN_GRAMMAR`, `LOJBAN_MORPHOLOGY_GRAMMAR`), run by `src/peg/**`;
  morphology adapter `src/morphology/peg.rs`.
* **Earley grammar and morphology:** the Earley engine
  (`src/earley/{compile,recognize}.rs`) runs the *same* `src/grammars.rs`
  grammars compiled to a CFG; morphology adapter `src/morphology/earley.rs`.
  Its CLL fallback is `src/cll/**`: the morphology CFG in
  `src/cll/cfg/grammar.rs` plus the filters in `src/cll/constraints/`, and
  the CLL Chapter 21 syntax grammar `src/cll/data/cll21.ebnf`
  (`camxes_rs::cll::grammar`).

## Formalisms

### PEG notation (`peg`, `grammars`)

Grammars are plain text, one rule per line, loaded at runtime with
`Peg::new(start_rule, text)` (or `EarleyGrammar::new` with the same
arguments). The embedded grammars are `(start_rule, text)` tuples.

| Syntax | Meaning |
|--------|---------|
| `rule <- e` | definition |
| `e1 e2` | sequence |
| `e1 / e2` | **ordered** choice: `e2` is tried only if `e1` fails |
| `e*`, `e+`, `e?` | greedy repetition / option (never gives back input) |
| `&e`, `!e` | and/not predicates (zero-width lookahead) |
| `( … )` | grouping |
| `'abc'`, `"abc"` | literal |
| `[abc]`, `[a-z]` | character class / range |
| `.` | any character |
| `rule <- e #: PUSH` | back-reference annotation: `PUSH`, `PEEK-DIFF`, `POP-EQ` (see below) |

The grammar text has no comment syntax apart from these annotations.

**Back-references** (ilmentufa's annotations, used by camxes for
`zoi`/`la'o`): a `#: PUSH` rule remembers the word it matched, a
`#: PEEK-DIFF` rule fails if its match equals the remembered word, and a
`#: POP-EQ` rule matches only if its match equals it (then forgets it).
Words are compared after normalization (ASCII case, pauses `.`, commas and
spaces ignored, `h` ≡ `'`; `src/peg/annotations.rs`, shared by both
engines). The PEG engine keeps a delimiter stack that is restored when a
sequence fails and after predicates; PEEK-DIFF / POP-EQ rules are not
memoized. A PUSH rule must be closed by a POP-EQ rule in the same rule
body. So in camxes `(zoi_word spaces)* zoi_close` stops at the first word
equal to the opening delimiter instead of swallowing it.
`ParseResult(_, end, result)` gives the end of the match of the start rule
(the PEG matches a prefix) and the tree of `ParseNode::NonTerminal` /
`ParseNode::Terminal` nodes with byte spans.

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

### Earley engine (`earley`)

`EarleyGrammar::new` reuses the PEG front end, so it accepts exactly the
same text, and compiles it to a CFG:

* `a / b` becomes an *unordered* alternation; `e*`, `e+`, `e?` become
  right-recursive helper nonterminals; literals, classes, ranges and `.`
  become byte-set terminals.
* `&e` / `!e` become zero-width lookahead symbols, decided by a separate
  Earley recognition of `e` from the current position ("does `e` derive some
  prefix of the rest?"), memoized per (rule, position).
* `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ`: every `open … close` stretch of a rule, where
  `open` is a PUSH rule and `close` a POP-EQ rule, becomes one *quote*
  symbol. It matches only if the text of `close` equals the text of `open`
  (ASCII case, pauses `.`, commas and spaces ignored, `h` ≡ `'`), and it
  ends at the **first** such word, so no quoted (PEEK-DIFF) word can equal
  the delimiter. This gives the same accept/reject decisions and
  `zoi_open` / `zoi_word` / `zoi_close` nodes as the PEG delimiter stack.
* Charts are FIRST-set filtered and sized to the span a sub-recognition can
  reach; ε-completion is handled dynamically (Aycock–Horspool).

The chart holds every derivation. `parse` extracts the **PEG-preferred** one
(earlier alternatives first, greedy repetitions, and each nonterminal keeps
only its preferred match, i.e. no backtracking into a nonterminal that
already matched), which reproduces the PEG's own tree except where a
lookahead is decided differently (`&e`/`!e` ask whether `e` derives *some*
prefix); `parse_complete`
returns the most preferred derivation that covers the whole input, which
also exists where the PEG committed to a choice that later fails. So the
Earley engine accepts a superset of what the PEG engine accepts.

```rust
use camxes_rs::earley::EarleyGrammar;

// The PEG commits to `'a'` and stops after one byte; Earley finds "abc".
let g = EarleyGrammar::new("s", "s <- ('a' / 'ab') 'c'?").unwrap();
assert_eq!(g.parse("abc").1, 1);           // PEG-preferred match
assert_eq!(g.parse_complete("abc").1, 3);  // derivation covering everything
```

### CLL morphology and Chapter 21 EBNF (`cll`, feature `earley`)

The vendored tergenzalvi code follows the CLL text rather than camxes:

* **Morphology (CLL ch. 3–4):** a context-free *overgenerating* grammar of
  word shapes (cmavo, gismu, lujvo, fu'ivla, cmevla), built in
  `src/cll/cfg/grammar.rs` and recognized with its own Earley parser, is
  intersected with constraint filters that each cite a CLL section
  (`src/cll/constraints/`: tosmabru and slinku'i tests, the `la`/`doi` ban in
  cmevla, lujvo hyphenation, rafsi/fu'ivla shape rules, stress and pause
  handling, class exclusion). Continuous text is segmented by
  `cll::lex::stream`.
* **Syntax (CLL ch. 21):** `src/cll/data/cll21.ebnf` is compiled at runtime
  into a CFG over selma'o terminals (`cll::grammar::compile_ebnf`), with
  CLL's `...` repetition, `[ ]` options, `/X#/` elidable terminators and `&`
  selections; text is lexed into selma'o via `cmavo_selmaho.tsv`, SI/SA/SU
  erasure runs on the lexeme stream, and `zoi`/`la'o` quotes are cut out
  before morphology (the body is one opaque `FOREIGN` lexeme; the quote
  closes at the first word equal to the delimiter).

```rust
#[cfg(feature = "earley")]
{
    use camxes_rs::cll::grammar::parse_text;
    use camxes_rs::cll::shapes::{recognize, MorphClass};

    assert!(recognize("creka").classes.contains(&MorphClass::Gismu));
    assert!(parse_text("mi viska le gerku").accepted);
    assert!(!parse_text("zoi gy. hello .dy cu glico").accepted);
}
```

### Standard vs obsolete words

`Engine::Earley` decides each word's `WordDialect` **by rule, without word
lists**:

1. If the camxes morphology grammar (on the Earley engine) covers the input
   with its PEG-preferred derivation, the result is `Dialect::Camxes` and
   every word is `Standard` — the same answer the PEG engine gives.
2. Otherwise the CLL morphology analyses it (`Dialect::Cll`). Each word is
   then `Standard` if the camxes grammar alone reads that word as one word of
   the same class, and `Obsolete` otherwise.
3. Last resort: any camxes derivation covering the input
   (`Dialect::CamxesComplete`).

Typical obsolete forms: on-glide diphthongs `iV`/`uV` inside brivla and names
(`kuarka`, `tiuanJOUS`), `n` before an affricate (`djinundji`; allowed by CLL
§3.7, banned by camxes), diphthong + apostrophe + vowel (`gondii'i`) and
`CiV'V'V` cmavo (`cie'a'u`). Gismu rules are the same in both dialects, so no
gismu is ever obsolete; words the PEG engine accepts are always standard.

## Usage

### Morphology

`morphology::analyze(input, engine)` returns a normalized `Morphology`, the
same shape for both engines (internal trees are not exposed):

* `words: Vec<MorphWord>` — each with `text`, byte `span`, `class`
  (`WordClass::{Cmavo, Gismu, Lujvo, Fuhivla, Cmevla}`), `selmaho`
  (`Some("KOhA")` etc. for cmavo when the `earley` feature provides the
  lexicon, else `None`) and `word_dialect` (`Standard` / `Obsolete`);
  helpers `is_obsolete()` and `type_label()` (`"gismu"`,
  `"obsolete fu'ivla"`, …).
* `dialect: Dialect` — `Camxes`, `Cll` or `CamxesComplete` (see above).
* `signature()` (engine-comparable `(text, class)` list), `has_obsolete()`,
  `render()` (`le/cmavo nu/cmavo klama/gismu`).

```rust
use camxes_rs::morphology::{analyze, Dialect, Engine, WordClass};

let m = analyze("lenu klama", Engine::Peg).unwrap();
assert_eq!(m.render(), "le/cmavo nu/cmavo klama/gismu");
assert_eq!(m.dialect, Dialect::Camxes);
assert_eq!(m.words[2].class, WordClass::Gismu);
assert!(analyze("kuarka", Engine::Peg).is_err()); // obsolete form
```

```rust
#[cfg(feature = "earley")]
{
    use camxes_rs::morphology::{analyze, Dialect, Engine, WordDialect};

    // Same result as the PEG engine for standard words …
    let m = analyze("lenu klama", Engine::Earley).unwrap();
    assert_eq!(m.render(), "le/cmavo nu/cmavo klama/gismu");
    assert_eq!(m.words[0].selmaho.as_deref(), Some("LE"));

    // … and the older CLL dialect, flagged by rule.
    let m = analyze("lojbo zei kuarbli", Engine::Earley).unwrap();
    let labels: Vec<String> = m.words.iter().map(|w| w.type_label()).collect();
    assert_eq!(labels, ["gismu", "cmavo", "obsolete fu'ivla"]);
    assert_eq!(m.dialect, Dialect::Cll);
    assert_eq!(m.words[2].word_dialect, WordDialect::Obsolete);
    assert!(m.has_obsolete());
}
```

`Engine::available()` lists the engines compiled in; `"peg"` / `"earley"`
parse with `str::parse::<Engine>()`.

### Whole-text syntax

`syntax::parse(text, engine)` checks a complete text against
`LOJBAN_GRAMMAR` (letters are lowercased first; the start rule `text` must
cover the input) and returns the tree or a `SyntaxError` with the stop
position; `syntax::accepts` returns a bool. `syntax::earley_grammar()` gives
the compiled Earley grammar.

```rust
use camxes_rs::morphology::Engine;
use camxes_rs::syntax;

assert!(syntax::accepts("lo prenu poi klama cu tavla", Engine::Peg));
assert!(!syntax::accepts("lo cu klama", Engine::Peg));
let tree = syntax::parse("mi klama", Engine::Peg).unwrap();
assert!(!tree.is_empty());

// zoi/la'o quotes need the same delimiter word on both sides (both engines).
for &engine in Engine::available() {
    assert!(syntax::accepts("zoi gy. hello world .gy glico", engine));
    assert!(syntax::accepts("mi klama la'o dy. New York .dy", engine));
    assert!(!syntax::accepts("zoi gy. hello .dy glico", engine));
    assert!(!syntax::accepts("mi cusku zoi gy. hello", engine));
}

#[cfg(feature = "earley")]
assert!(syntax::accepts("lo prenu poi klama cu tavla", Engine::Earley));
```

The Earley engine first takes the PEG-preferred derivation and otherwise
any derivation covering the text. `tests/grammar_basics.rs` lists the
constructions tested on both engines.

### Examples and command line

```sh
# PEG morphology grammar over examples/lujvo_tests.tsv (or a TSV path)
cargo run --example cmaxes-test [path.tsv]

# PEG vs Earley morphology over the dictionary corpus, with timings;
# --raw times the bare grammar engines, --words w1,w2 checks single words
cargo run --release --features earley --example morphology_corpus -- [--raw] [--words w1,w2]
```

`morphology_corpus` also writes a per-word report (word, dictionary type,
PEG result, Earley result, status) to `target/corpus-results.tsv`.

### Tests, parity file and benchmarks

```sh
cargo test                                   # PEG engine
cargo test --release --features earley       # both engines + corpus parity
cargo bench [--features earley]              # criterion benchmarks
python3 scripts/fetch_lensisku_corpus.py     # refresh tests/data from lensisku
```

`tests/dictionary_corpus.rs` runs every lensisku English headword (`wiki`
page-title entries excluded; `tests/data/lensisku-en-words.tsv.gz`) through
both engines and compares with `tests/data/morphology-parity.tsv`, which
records for each headword the expected words, classes, obsolete flags and
selma'o, each engine's acceptance, the rule-based dialect, and whether the
dictionary's `obsolete` label agrees with it (`label_check`). After an
intended change, regenerate it and review the diff:

```sh
CAMXES_BLESS_PARITY=1 cargo test --release --features earley --test dictionary_corpus
```

`benches/parser.rs` measures grammar construction and parsing of short and
medium Lojban texts with `LOJBAN_GRAMMAR` (Earley benchmarks with
`--features earley`). The test data, corpus test, corpus example and
scripts live in the repository only and are excluded from the crates.io
package.

### Debugging

The PEG engine logs through the `log` crate:

```rust,no_run
env_logger::builder().filter_level(log::LevelFilter::Debug).init();
```

or `RUST_LOG=debug cargo run --example cmaxes-test`.

## Known limitations

* **Back-references:** delimiters are compared after normalization (case,
  pauses, commas, `h`/`'`). The Earley engine enforces `PEEK-DIFF` through
  "the quote ends at the first matching word" rather than as a separate
  check; the PEG engine checks it directly. Both require the PUSH and
  POP-EQ rules in the same rule body.
* **Trees:** when the PEG-preferred derivation covers the text the Earley
  tree equals the PEG tree (all 1,678 texts both engines accept in the
  `dwe` example corpus give identical trees); when only `parse_complete`
  succeeds it is the derivation the Earley engine picked. The parity test
  compares word split, classes and selma'o only.
* **Dictionary labels:** 226 headwords labelled obsolete in lensisku have a
  standard form (e.g. `matriocka`) and one standard-labelled word
  (`gudjrati`) is obsolete by rule; the parity file records these as
  `label_check` mismatches.
* The vendored CLL Chapter 21 layer drops words its morphology rejects
  (reported in `gaps`) and approximates some BU/ZEI interactions.
* `Peg` memoizes in a `RefCell` and is not `Sync`; use one parser per thread
  (as `syntax` does). `EarleyGrammar` is `Send + Sync`.

## Performance

Measured on the build machine (release build, single thread) over the 31,414
lensisku headwords with `examples/morphology_corpus.rs`:

| Workload | PEG | Earley |
|----------|-----|--------|
| `morphology::analyze`, all headwords | 1,244 words/s | 1,515 words/s |
| bare `LOJBAN_MORPHOLOGY_GRAMMAR` parse (`--raw`) | 1,340 inputs/s | 1,677 inputs/s |

Whole-text syntax with `LOJBAN_GRAMMAR` (`cargo bench --features earley`):

| Benchmark | PEG | Earley |
|-----------|-----|--------|
| compile the grammar (`Peg::new` / `EarleyGrammar::new`) | 196 ms | 221 ms |
| `mi prami do` (11 bytes) | 4.5 ms | 1.6 ms |
| `mi prami do .i do prami mi ` × 20 (540 bytes) | 102 ms | 222 ms |
| 1,699 example sentences (`dwe` corpus, `syntax::parse`) | 54–66 rows/s | 82–89 rows/s |

Compile a grammar once and reuse it (`syntax` caches both). The Earley
engine pays for lookahead sub-recognitions on long texts; the slowest
headwords are long cmavo clusters and CLL-fallback words (tens to a few
hundred ms). Numbers vary with machine load.

Results: PEG accepts 31,220 headwords, Earley 31,394; the engines never
disagree where both accept; 174 headwords are accepted only by Earley
(obsolete CLL forms) and 20 by neither.

## License

MIT — see [LICENSE](LICENSE). The camxes grammars are derived from
[ilmentufa](https://github.com/lojban/ilmentufa) / camxes.js
(© Masato Hagiwara, MIT); `src/cll` is vendored from
[tergenzalvi](https://github.com/lagleki/tergenzalvi) by the same author as
this crate. PEG: Bryan Ford, [Parsing Expression Grammars](https://bford.info/pub/lang/peg.pdf).
