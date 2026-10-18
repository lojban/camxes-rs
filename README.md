# camxes-rs — PEG and Earley parsers for Lojban

[![Crates.io](https://img.shields.io/crates/v/camxes-rs)](https://crates.io/crates/camxes-rs)
[![Documentation](https://docs.rs/camxes-rs/badge.svg)](https://docs.rs/camxes-rs)
[![License: MIT](https://img.shields.io/crates/l/camxes-rs)](LICENSE)

camxes-rs is a PEG parser generator with the camxes Lojban grammars built in,
plus an optional second engine. Each engine is a Cargo feature (`peg`, on by
default, and `earley`); at least one must be enabled:

* **PEG engine** (`peg`, default) — packrat-style PEG interpreter: ordered choice,
  greedy repetition, no backtracking into a committed choice.
* **Earley engine** (`earley`) — the vendored CLL (Complete Lojban Language)
  pipeline of [tergenzalvi](https://github.com/lagleki/tergenzalvi) in
  `src/cll`: Earley-parsed CFG word shapes + CLL constraint filters + stream
  segmentation for morphology, and the CLL Chapter 21 EBNF grammar
  (`src/cll/data/cll21.ebnf`) over selma'o tokens for syntax. It does not use
  the camxes PEG grammars. It also accepts the older CLL word forms
  (obsolete words such as `kuarka`), flagged by rule.

Both engines are exposed through the same Lojban APIs: `morphology::analyze`
(word split + classes) and `syntax::parse` / `syntax::accepts` (whole texts).

API documentation: [docs.rs/camxes-rs](https://docs.rs/camxes-rs).

## Installation

```toml
[dependencies]
camxes-rs = "2.0.0"                                         # PEG engine only
# camxes-rs = { version = "2.0.0", features = ["earley"] }  # PEG + Earley / CLL
# camxes-rs = { version = "2.0.0", default-features = false, features = ["earley"] }  # Earley only
```

| Cargo feature | Default | Adds |
|---------------|---------|------|
| `peg`         | yes     | the PEG engine (`peg::grammar::Peg`), the camxes grammar texts (`grammars`), the grammar AST (`peg::rule`), the stack guard (`peg::stack`), `Engine::Peg` for `morphology` and `syntax` |
| `earley`      | no      | the vendored `cll` module (CLL morphology, cmavo lexicon, Chapter 21 parser) and `Engine::Earley` for `morphology` and `syntax` |

Always compiled: the engine-independent `morphology` and `syntax` APIs and
the parse-tree types (`peg::parsing`). Building with neither feature is a
compile error. `Engine::default()` is `Engine::Peg` when `peg` is enabled,
else `Engine::Earley`.

## Repository layout

| Path | What it is |
|------|------------|
| `src/peg/**` | PEG engine: `grammar/` (`Peg::new`, `Peg::parse`, bootstrap grammar of the PEG notation), `rule/` (rule tree and its matcher), `transformer/` (grammar text → rules), `parsing.rs` (`ParseResult`, `ParseNode`, `Span`, errors), `annotations.rs` (`#: PUSH`/`#: PEEK-DIFF`/`#: POP-EQ` handling), `stack.rs` (stack guard). Everything except `parsing.rs` needs feature `peg` |
| `src/grammars.rs` | Embedded PEG grammars (feature `peg`): `LOJBAN_MORPHOLOGY_GRAMMAR` (camxes word-level morphology, start `text`), `LOJBAN_GRAMMAR` (full camxes syntax from ilmentufa, start `text`) |
| `src/morphology/mod.rs` | Engine-independent morphology API and result types (`analyze`, `Morphology`, `MorphWord`, `WordClass`, `WordDialect`, `Dialect`, `Engine`) |
| `src/morphology/peg.rs` | PEG adapter (feature `peg`): `LOJBAN_MORPHOLOGY_GRAMMAR` on the PEG engine |
| `src/morphology/tree.rs` | Words read off a camxes morphology parse tree (PEG adapter) |
| `src/morphology/earley.rs` | Earley adapter (feature `earley`): CLL word analysis and segmentation from `src/cll`, tergenzalvi `MorphClass` → camxes word type, rule-based standard/obsolete decision |
| `src/syntax.rs` | Whole-text syntax check (`parse`, `accepts`): `LOJBAN_GRAMMAR` on the PEG engine, `cll21.ebnf` on the Earley engine |
| `src/cll/**` | Vendored tergenzalvi (feature `earley`): `phonology/` (letters, diphthongs, consonant pair/triple tables, CLL ch. 3), `shapes/` (CV shape recognition), `cfg/` (morphology CFG built in `cfg/grammar.rs` + its Earley recognizer), `constraints/` (CLL ch. 4 filters: tosmabru, slinku'i, lujvo hyphens, rafsi/fu'ivla, stress, pauses, class exclusion), `lex/` (stream segmentation, tokens), `tables/` (rafsi data), `grammar/` (CLL ch. 21: cmavo lexicon, lexemes, EBNF compiler, Earley parser, elision, SI/SA/SU erasure), `forms.rs` (cmavo break-up rule, CLL §4.6/§4.7) |
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
* **Earley grammar and morphology:** only the vendored tergenzalvi code in
  `src/cll/**`. Morphology: the word-shape CFG in `src/cll/cfg/grammar.rs`,
  the filters in `src/cll/constraints/`, segmentation in `src/cll/lex/`,
  camxes-compatibility rules in `src/cll/camxes.rs`; adapter
  `src/morphology/earley.rs`. Syntax: the CLL Chapter 21 grammar
  `src/cll/data/cll21.ebnf` over the selma'o of
  `src/cll/data/cmavo_selmaho.tsv` (`camxes_rs::cll::grammar`).

## Formalisms

### PEG notation (`peg`, `grammars`)

Grammars are plain text, one rule per line, loaded at runtime with
`Peg::new(start_rule, text)`. The embedded grammars are `(start_rule, text)`
tuples.

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
spaces ignored, `h` ≡ `'`; `src/peg/annotations.rs`). The PEG engine keeps a delimiter stack that is restored when a
sequence fails and after predicates; PEEK-DIFF / POP-EQ rules are not
memoized. A PUSH rule must be closed by a POP-EQ rule in the same rule
body. So in camxes `(zoi_word spaces)* zoi_close` stops at the first word
equal to the opening delimiter instead of swallowing it.
`ParseResult(_, end, result)` gives the end of the match of the start rule
(the PEG matches a prefix) and the tree of `ParseNode::NonTerminal` /
`ParseNode::Terminal` nodes with byte spans.

```rust
#[cfg(feature = "peg")]
{
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
}
```

### Earley engine: CLL morphology and Chapter 21 EBNF (`cll`, feature `earley`)

The vendored tergenzalvi code follows the CLL text rather than camxes:

* **Morphology (CLL ch. 3–4):** a context-free *overgenerating* grammar of
  word shapes (cmavo, gismu, lujvo, fu'ivla, cmevla), built in
  `src/cll/cfg/grammar.rs` and recognized with its own Earley parser, is
  intersected with constraint filters that each cite a CLL section
  (`src/cll/constraints/`: tosmabru and slinku'i tests, lujvo hyphenation,
  rafsi/fu'ivla shape rules, stress and pause handling, class exclusion).
  Continuous text is segmented by `cll::lex::stream`.
* **Syntax (CLL ch. 21):** `src/cll/data/cll21.ebnf` is compiled at runtime
  into a CFG over selma'o terminals (`cll::grammar::compile_ebnf`), with
  CLL's `...` repetition, `[ ]` options, `/X#/` elidable terminators and `&`
  selections; text is lexed into selma'o via `cmavo_selmaho.tsv`, SI/SA/SU
  erasure runs on the lexeme stream, and `zoi`/`la'o` quotes are cut out
  before morphology (the body is one opaque `FOREIGN` lexeme; the quote
  closes at the first word equal to the delimiter). `syntax::parse` returns
  the Chapter 21 derivation as `ParseNode`s named after the EBNF rules and
  selma'o (`sumti`, `KOhA`, `BRIVLA`, `FOREIGN`, …); a text with a word the
  CLL morphology rejects is a syntax error.

**No local changes.** `src/cll` is identical to tergenzalvi's `src/` apart
from crate paths and the data-file location; `scripts/check_cll_sync.sh
[PATH_TO_TERGENZALVI]` diffs the two trees with that glue normalised and
prints nothing but an "in sync" line when they match. Where CLL is silent,
both repositories fill the gap the same way (each case cites the CLL text
it relies on): experimental cmavo with three or more vowels and `y`
apostrophe chains (§4.2), apostrophe vowel chains in brivla (§3.4), `y`
chains in names (§4.8), in the camxes dialect (see [Dialect
options](#dialect-options-earley)) lujvo recognized by rafsi *form* with the
§4.5 restrictions, consonant-only names such as
`.d.` or `.rl.` when every adjacent pair and triple is permissible (§3.7;
`mz`, `ndj` are not), the literal slinku'i test of §4.7 rule 3, the §4.6/§4.7 rule
that a word breaking up into a cmavo followed by cmavo/gismu/lujvo is not a
brivla (`lerenanmu` = `le` + `re` + `nanmu`), digits as `PA` cmavo, and
punctuation-only tokens (`-`) skipped. A text with a word the morphology
rejects has no derivation. tergenzalvi's `MorphClass` is mapped to the camxes
word types outside `src/cll`, in `src/morphology/camxes.rs`; the
words on which the two engines still disagree are listed in
[Differences between the PEG (camxes) and Earley (CLL)
engines](#differences-between-the-peg-camxes-and-earley-cll-engines).

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

### Dialect options (Earley)

The CLL pipeline takes a `MorphOptions` (re-exported as
`camxes_rs::morphology::MorphOptions`) with **one dialect switch**,
`camxes_dialect`, plus the separate §4.16 proposal flag `rafsi_fuhivla`:

| Setting | Strict CLL (`camxes_dialect: false`) | camxes dialect (`camxes_dialect: true`) |
|---|---|---|
| Lujvo rafsi | only rafsi of the CLL-era (1994) list | also rafsi outside the list, recognised by §4.5 shape (CVC, CVV/CV'V, CCV, 4-letter + `y`, final 5-letter); the §4.6 hyphen and §4.7 tosmabru/slinku'i rules still apply |
| Brivla-stem rafsi | none | as in camxes: a consonant-final brivla/fu'ivla stem + `y` (`afxanyru'u`), a vowel-final stem + `'y` (`baxmasi'ygu'e`), `y'` before a vowel-initial piece (`pedy'utka`), a brivla-shaped last piece (`bemymantodi`); pieces keep §3.7 clusters and §3.4–3.5 vowel groups |
| Names | §4.8: no `la`, `lai`, `doi` inside a name (`volapuk` → `vo la puk`) | `la`, `lai`, `doi` allowed (`volapuk`, `laus`, `tigbalan`) |

They always change together: there is no way to get one without the
other. Presets: `MorphOptions::default()` (strict CLL + §4.16, tergenzalvi's
default), `MorphOptions::settled()` (strict CLL, no §4.16) and
`MorphOptions::camxes()` (camxes dialect + §4.16); `with_camxes_dialect(bool)`
sets the switch on any of them. `Engine::Earley` uses
`morphology::earley_options()`, which is `camxes()`, so it agrees with the
PEG engine on names, post-1994 rafsi and brivla-stem rafsi; `morphology::analyze_with_options`
and `syntax::parse_with_options` take strict CLL or any other set. The
switch only decides which words are accepted and how text splits;
`WordDialect` (next section) stays rule-based on the camxes syllable rules,
so `volapuk` is a standard cmevla, and a word is never `Obsolete` merely
because the camxes dialect admitted it. Consonant-only names (below),
vowel grouping (§3.4–3.5, with on-glides) and commas as syllable breaks
(§3.3) are the same in both dialects.

```rust
#[cfg(feature = "earley")]
{
    use camxes_rs::morphology::{analyze, analyze_with_options, Engine, MorphOptions};

    // Engine::Earley (camxes dialect): a name with `la`, a lujvo with the
    // post-CLL rafsi `gam`.
    assert_eq!(analyze("volapuk", Engine::Earley).unwrap().render(), "volapuk/cmevla");
    assert_eq!(analyze("gamkemdau", Engine::Earley).unwrap().render(), "gamkemdau/lujvo");
    // Strict CLL turns off both at once.
    let strict = MorphOptions::default();
    assert!(analyze_with_options("volapuk", strict).map_or(true, |m| m.render() != "volapuk/cmevla"));
    assert!(analyze_with_options("gamkemdau", strict).map_or(true, |m| m.render() != "gamkemdau/lujvo"));
}
```

### Standard vs obsolete words

`Engine::Earley` decides each word's `WordDialect` **by rule, without word
lists**:

the CLL pipeline analyses the input (`Dialect::Cll`), and each word is
`Obsolete` when it breaks the camxes syllable rules
(`is_obsolete_form` in `src/morphology/camxes.rs`, a port of the camxes `syllables` /
`zifcme` rules: names must be camxes `zifcme`, other words must start with a
camxes onset and split into camxes syllables), `Standard` otherwise. On the
dictionary corpus a word is obsolete exactly when the PEG engine rejects it.

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
* `dialect: Dialect` — `Camxes` (PEG engine) or `Cll` (Earley engine).
* `signature()` (engine-comparable `(text, class)` list), `has_obsolete()`,
  `render()` (`le/cmavo nu/cmavo klama/gismu`).

```rust
#[cfg(feature = "peg")]
{
    use camxes_rs::morphology::{analyze, Dialect, Engine, WordClass};

    let m = analyze("lenu klama", Engine::Peg).unwrap();
    assert_eq!(m.render(), "le/cmavo nu/cmavo klama/gismu");
    assert_eq!(m.dialect, Dialect::Camxes);
    assert_eq!(m.words[2].class, WordClass::Gismu);
    assert!(analyze("kuarka", Engine::Peg).is_err()); // obsolete form
}
```

```rust
#[cfg(feature = "earley")]
{
    use camxes_rs::morphology::{analyze, Dialect, Engine, WordDialect};

    // Same words and classes as the PEG engine for standard words …
    let m = analyze("lenu klama", Engine::Earley).unwrap();
    assert_eq!(m.render(), "le/cmavo nu/cmavo klama/gismu");
    assert_eq!(m.words[0].selmaho.as_deref(), Some("LE"));
    assert_eq!(m.dialect, Dialect::Cll);

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

`syntax::parse(text, engine)` checks a complete text and returns the tree or
a `SyntaxError` with a position; `syntax::accepts` returns a bool. The PEG
engine runs `LOJBAN_GRAMMAR` (letters are lowercased first; the start rule
`text` must cover the input); the Earley engine runs the CLL Chapter 21
grammar `src/cll/data/cll21.ebnf` (start `text`) over selma'o tokens, with
elidable terminators. The two engines' trees differ (camxes vs Chapter 21
rule names).

```rust
use camxes_rs::morphology::Engine;
use camxes_rs::syntax;

let engine = Engine::default(); // `Engine::Peg` with the default features
assert!(syntax::accepts("lo prenu poi klama cu tavla", engine));
assert!(!syntax::accepts("lo cu klama", engine));
let tree = syntax::parse("mi klama", engine).unwrap();
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

`tests/grammar_basics.rs` lists the constructions tested on both engines.

### Examples and command line

```sh
# PEG morphology grammar over examples/lujvo_tests.tsv (or a TSV path)
cargo run --example cmaxes-test [path.tsv]

# PEG vs Earley morphology over the dictionary corpus, with timings;
# --raw times the bare word recognisers (camxes PEG grammar vs CLL
# morph_parse), --words w1,w2 checks single words
cargo run --release --features earley --example morphology_corpus -- [--raw] [--words w1,w2]
```

`morphology_corpus` also writes a per-word report (word, dictionary type,
PEG result, Earley result, status) to `target/corpus-results.tsv`.

### Tests, parity file and benchmarks

```sh
cargo test                                   # PEG engine
cargo test --release --features earley       # both engines + corpus parity (release: the CLL pipeline is slow in debug)
cargo bench [--features earley]              # criterion benchmarks
cargo test --no-default-features --features earley   # Earley-only build
python3 scripts/fetch_lensisku_corpus.py     # refresh tests/data from lensisku
```

`tests/dictionary_corpus.rs` runs every lensisku English headword (`wiki`
page-title entries excluded; `tests/data/lensisku-en-words.tsv.gz`) through
both engines and compares with `tests/data/morphology-parity.tsv`, which
records for each headword the expected words, classes, obsolete flags and
selma'o, each engine's acceptance, the rule-based dialect, and whether the
dictionary's `obsolete` label agrees with it (`label_check`). The engines
must agree on every PEG-accepted headword except the documented known
differences in `tests/data/known-differences.tsv` (word, category, exact
Earley result; see [Differences between the PEG (camxes) and Earley (CLL)
engines](#differences-between-the-peg-camxes-and-earley-cll-engines)); a
new difference, a changed Earley result or a listed word that now agrees
fails the test. After an
intended change, regenerate it and review the diff:

```sh
CAMXES_BLESS_PARITY=1 cargo test --release --features earley --test dictionary_corpus
```

`benches/parser.rs` measures grammar construction and parsing of short and
medium Lojban texts: `LOJBAN_GRAMMAR` on the PEG engine; with
`--features earley`, compiling `cll21.ebnf` and `syntax::parse` on the
Earley engine. The test data, corpus test, corpus example and
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
  pauses, commas, `h`/`'`). The PEG engine checks `PEEK-DIFF` / `POP-EQ`
  with its delimiter stack (PUSH and POP-EQ rules in the same rule body); the
  Earley engine cuts `zoi`/`la'o` quotes out before lexing, closing at the
  first word equal to the delimiter.
* **Word types:** on the dictionary corpus the engines disagree on 37 of
  the 31,219 PEG-accepted headwords; each is a documented camxes/CLL
  difference (next section).
* **Trees:** the PEG tree uses camxes rule names, the Earley tree CLL
  Chapter 21 rule names; they are not comparable. The parity test compares
  word split, classes and selma'o only.
* **Dictionary labels:** 226 headwords labelled obsolete in lensisku have a
  standard form (e.g. `matriocka`) and one standard-labelled word
  (`gudjrati`) is obsolete by rule; the parity file records these as
  `label_check` mismatches.
* The vendored CLL Chapter 21 layer approximates some BU/ZEI interactions;
  texts that the Chapter 21 grammar and camxes judge differently are listed
  by the `dwe` comparison (see Performance).
* **Errors, not panics:** malformed grammar text (including non-ASCII) is a
  `GrammarError` / `CompileError`, and no input string makes a parse panic
  (`tests/robustness.rs`). Recursion deeper than the current thread's stack
  allows (very deep nesting, left-recursive rules) fails with
  `ErrorKind::RecursionLimit`; the stack size is queried on Linux and macOS,
  elsewhere 1 MiB is assumed (`peg::stack`, `Peg::set_stack_budget`).
  Worst-case cost is still high: the vendored CLL word analysis is cubic in
  the length of one unbroken word, so cap untrusted input length.
* `Peg` memoizes in a `RefCell` and is not `Sync`; use one parser per thread
  (as `syntax` does). The CLL pipeline caches word analyses per thread.

## Differences between the PEG (camxes) and Earley (CLL) engines

The PEG engine implements camxes; the Earley engine implements the text of
the CLL (Complete Lojban Language) and fills gaps only where the CLL is
silent. Where they disagree, the Earley engine is not changed to match
camxes. Each row of `tests/data/known-differences.tsv` has one of the
categories below (the same reasons as `DIFFERENCE_CATEGORIES` in
`tests/dictionary_corpus.rs`). Counts are PEG-accepted dictionary headwords
(37 in total).

| Category (count) | Kind | Examples (PEG → Earley) | CLL |
|---|---|---|---|
| `medial-pair` (1) | camxes bug | `vamzma`: lujvo → rejected | §3.7: "the specific pairs cx, kx, xc, xk, and mz are forbidden"; the camxes morphology grammar does not check `mz` (the voicing clashes `zt`, `fg` of `ciztolnei`, `refgau` are now rejected by both engines, see below) |
| `hesitation` (2) | CLL rule | `y` → rejected; `yyy`: `y` + `yy` → rejected | §19.14: `.y.` "differs from [English uh] in the requirement for pauses before and after" |
| `vowel-initial-pause` (1) | camxes bug | `aierne`: `a` + `ierne` → one fu'ivla `aierne` | §4.9 rule 3: a word beginning with a vowel must be preceded by a pause, so `a` + `ierne` is not a valid split |
| `fuhivla-cluster` (13) | CLL rule (kept) | `ku'aidzi`, `fau'arma`, `ri'oirkai`, `xai'otso` → rejected | §4.7 rule 1: every fu'ivla "must contain a consonant cluster in the first five letters of the word". The apostrophe is not a consonant (§3.6) and fu'ivla cannot contain `y` (rule 4), so in `ku'aidzi` the first five letters are `k u ' a i` and the only cluster, `dz`, is at letters 6–7: not a fu'ivla. camxes accepts a cluster anywhere; this stays a documented difference |
| `hyphen-not-required` (20) | CLL rule | `geirzdi`, `toirsti`, `selda'ergau`, `relpi'irna'u`: lujvo → fu'ivla | §4.6: "It is illegal to add a hyphen at a place that is not required by this algorithm"; an r/n-hyphen follows only a CVV rafsi that needs it |

Resolved categories (both engines now agree):

* **Brivla/fu'ivla-stem rafsi** (former `rafsi-vowel-initial` 98,
  `rafsi-shape` 55, `y-after-vowel` 16, `n-hyphen` 2). In the camxes dialect
  (`Engine::Earley`) a lujvo may also use the stem of any brivla as a rafsi,
  as the camxes grammar does (`fuhivla_rafsi`, `hy_rafsi`, `y h?`, a final
  `fuhivla`): a consonant-final stem + `y` (`afxan`-y-`ru'u`,
  `kexmair`-y-`bau`), a vowel-final stem + `'y` (`baxmasi`-'y-`gu'e`), `y'`
  before a vowel-initial piece (`ped`-y'-`utka`) and a brivla-shaped last
  piece (`bem`-y-`mantodi`). Each piece must have §3.7 clusters, §3.4–3.5
  vowel groups and apostrophes only between vowels; a vowel-initial piece
  needs a word start, an apostrophe or an on-glide (`batky`-`uidje`); short
  rafsi between stems keep the §4.6 `r`/`n` hyphen rule. Strict CLL
  (`MorphOptions::default()`) still rejects all of them
  (`src/cll/constraints/camxes_lujvo.rs`).
* **Vowel grouping** (former `vowel-grouping` 15). Vowels group from the
  left (§3.5), and `i`/`u` before a vowel is an on-glide starting the next
  syllable (§3.4: on-glides are "used only as stand-alone words and in
  Lojbanized names and borrowings"), so `gleua` = `gle·ua`, `proteuu`,
  `paxtoua`, `iuctcouo` are fu'ivla in both dialects. A pair that is neither
  a diphthong nor vowel + glide (`ea`) still needs a comma or apostrophe.
* **Comma** (former `comma` 7). The camxes morphology grammar treated a
  comma as a pause (`pause_0 <- [ ,.]+`), so `lijdrba'a,i` was
  `lijdrba'a` + `i`. It now treats a comma as a syllable break inside a word
  (CLL §3.3): before a vowel it separates syllables (`h <- [,'] &nucleus`),
  before a consonant or an on-glide it is skipped, and only a comma next to
  a space or the end of the input is a pause. `lijdrba'a,i`, `OTa,uas`,
  `bodga,ias`, `nu,IORK`, `tai,UAN`, `xali,uyd`, `klanrniut,ni` are now one
  word in both engines (the Earley engine also retries a word without the
  commas that do not separate vowels). The only other dictionary change is
  `bie,uaRUC`, now accepted by the PEG engine (a standard name).
* Earlier: `name-la` (8) and `consonant-name` (5) — the camxes dialect
  allows `la` in names, and consonant-only names with permissible clusters
  (§3.7) are accepted; the voicing clashes `ciztolnei`, `refgau` are
  rejected by both engines.

The other direction: 173 dictionary headwords are accepted only by the
Earley engine. They are the older CLL dialect ("obsolete" words such as
`kuarka`, `tiuanJOUS`, `djinundji`, `gondii'i`), flagged
`WordDialect::Obsolete` by rule (see Standard vs obsolete words).

**Whole texts (dwe corpus).** On the 1,699-text `dwe` comparison corpus
the PEG engine accepts 1,678 texts and the Earley engine 1,676 (all of them
also PEG-accepted). The two texts only the PEG engine accepts (reported, not
fixed):

* `ti marce ge le se vecnu na'o ku gi le remna ca'a ku` — Earley: "no CLL
  Chapter 21 derivation of `text`". A forethought termset (`ge … gi …`
  joining terms) needs `nu'i` in the CLL grammar (Ch. 14/21 `termset`);
  camxes allows it without.
* `zo co'oi poi do pu cusku cu ranxi le ka se smuni zo coi vau le ka se smuni zo co'o`
  — Earley: `cmavo "co'oi" not in lexicon`. `co'oi` is not in the CLL cmavo
  table (`cmavo_selmaho.tsv`); camxes accepts any cmavo-shaped word.

The texts ending in `zo d` and `la c` are now accepted by both engines
(consonant-only names, CLL §3.7).

## Performance

Measured on the build machine (release build, single thread) over the 31,414
lensisku headwords with `examples/morphology_corpus.rs`:

| Workload | PEG | Earley (CLL pipeline) |
|----------|-----|--------|
| `morphology::analyze`, all headwords | 1,029 words/s | 343 words/s |
| bare word recogniser (`--raw`: camxes grammar / `cll::constraints::morph_parse`) | 1,118 inputs/s | 296 inputs/s |

Whole-text syntax (`cargo bench --features earley`; the Earley numbers are
warm-cache: the CLL word analysis is cached per thread):

| Benchmark | PEG | Earley |
|-----------|-----|--------|
| grammar startup (`Peg::new(LOJBAN_GRAMMAR)` / `compile_ebnf(cll21.ebnf)`) | 195 ms | 1.2 ms |
| `mi prami do` (11 bytes) | 5.6 ms | 0.47 ms |
| `mi prami do .i do prami mi ` × 20 (540 bytes) | 100 ms | 0.58 ms |
| 1,699 example sentences (`dwe` corpus, `syntax::parse`, cold cache) | 53 rows/s | 433 rows/s |

Compile a grammar once and reuse it (`syntax` caches both). The slowest
headwords for the CLL pipeline are very long cmavo clusters (0.5 s); long
camxes lujvo, which the CLL pipeline used to break up word by word (up to
3.5 s), are now read whole as brivla-stem lujvo, which also raised the
Earley `analyze` rate from 214 to 343 words/s on the same machine. Numbers
vary with machine load.

Results: PEG accepts 31,219 headwords, Earley 31,376; the engines agree on
word split and class on 31,182, the other 37 PEG-accepted headwords are the
documented known differences; 173 headwords are accepted only by Earley
(obsolete CLL forms) and 22 by neither.

## License

MIT — see [LICENSE](LICENSE). The camxes grammars are derived from
[ilmentufa](https://github.com/lojban/ilmentufa) / camxes.js
(© Masato Hagiwara, MIT); `src/cll` is vendored from
[tergenzalvi](https://github.com/lagleki/tergenzalvi) by the same author as
this crate. PEG: Bryan Ford, [Parsing Expression Grammars](https://bford.info/pub/lang/peg.pdf).
