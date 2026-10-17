# Changelog

## 2.0.0 — unreleased

A new major version: the latest `camxes-rs` on crates.io is 1.1.1, published
from a different code base (lojban/tersmu), so this release supersedes it
with a major bump rather than claiming compatibility with the 1.x API.

### Added
- `morphology` module: engine-independent Lojban morphology API
  (`morphology::analyze(input, Engine)`) returning a normalized `Morphology`
  (word segmentation, `WordClass` — cmavo / gismu / lujvo / fu'ivla / cmevla —
  and cmavo selma'o), plus the `Dialect` that produced it.
- `earley` module: an Earley recognizer that compiles the same PEG grammar text
  (`EarleyGrammar::new`) into a CFG with lookahead symbols for `&` / `!`, and
  extracts the PEG-preferred derivation (ordered choice, greedy repetition).
- Optional Cargo feature `earley` enabling `Engine::Earley`. It accepts both
  dialects: the camxes dialect (same result as the PEG engine for every word
  the PEG accepts) and the older CLL dialect (e.g. obsolete words such as
  `kuarka`; note that the dictionary also labels standard-form words such as `matriocka` obsolete) through the CLL morphology of
  [tergenzalvi](https://github.com/lagleki/tergenzalvi) (vendored as `cll`).
- Rule-based obsolete-dialect flag per word: `MorphWord::word_dialect`
  (`WordDialect::Standard` / `WordDialect::Obsolete`), `MorphWord::is_obsolete`,
  `MorphWord::type_label` (e.g. `obsolete fu'ivla`) and
  `Morphology::has_obsolete`. A word is obsolete when the standard camxes
  morphology rules reject it (or give another class) but the CLL Chapter 3–4
  rules accept it — no word lists are involved, so new words are classified
  too (on-glide diphthongs in brivla/names such as `kuarka`, `n` before an
  affricate as in `djinundji`, `CiV'V'V` cmavo such as `cia'o'e`, ...). In a
  zei-lujvo only the obsolete component is flagged. The PEG engine only
  produces standard words.
- `syntax` module and `grammars::LOJBAN_GRAMMAR` (the full camxes Lojban
  grammar from ilmentufa / camxes.js, MIT): `syntax::parse(text, Engine)` /
  `syntax::accepts` check whole texts with either engine.
  `tests/grammar_basics.rs` covers basic constructions (bridi, LE/KU, FA/BAI,
  tenses, NU/DU'U, POI/NOI/GOI, JE/.e/GIhA/GA, .i, lu/li'u, zo, numbers and
  MEX, vocatives, UI, lujvo/fu'ivla/cmevla in sentences, ...) and
  ungrammatical texts.
- Delimiter back-references in the Earley engine: ilmentufa-style rule
  annotations `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ` are compiled into a
  quote symbol, so `zoi X. … .X` / `la'o X. … .X` is accepted only when the
  closing word equals the opening one (case, pauses, commas ignored, `h` ≡
  `'`) and the quote ends at the first such word (the quoted text never
  contains the delimiter as a separate word). `zoi gy. hello .dy glico` is
  rejected. `LOJBAN_GRAMMAR` carries the annotations on `zoi_open` /
  `zoi_word` / `zoi_close`; the PEG front end accepts and ignores them (the
  PEG engine still rejects `zoi`/`la'o` quotes because its greedy repetition
  swallows the closing delimiter).
- `cll` module (feature `earley`): vendored copy of tergenzalvi (commit
  e07f176, same author, included under this crate's MIT license) providing the CLL morphology and cmavo lexicon,
  so the crate has no git dependency and can be published.
- `grammars::LOJBAN_MORPHOLOGY_GRAMMAR` (previously only in the example).
- README rewritten to document both engines (layout, formalisms, usage,
  limitations, performance); its code examples run as doctests.
- Dictionary corpus test (`tests/dictionary_corpus.rs`) over all lensisku
  English headwords, excluding `wiki` page-title entries
  (`tests/data/lensisku-en-words.tsv.gz`, refreshed with
  `scripts/fetch_lensisku_corpus.py`) and `examples/morphology_corpus.rs` for
  agreement reports and benchmarks (`--raw` times the bare grammar engines);
  it also writes a per-word report to `target/corpus-results.tsv`.
- `tests/data/morphology-parity.tsv`: committed expected morphology (words,
  classes, obsolete flags, selma'o), rule-based dialect, a check of the
  dictionary's `obsolete` label against it, and PEG/Earley acceptance for every headword; the corpus
  test fails if either engine deviates. CI workflow `.github/workflows/test.yml`
  runs it with `--features earley`.

### Fixed
- `Peg::new` panicked on grammar text with leading whitespace.
- Clippy warnings in the PEG engine and the `cmaxes-test` example.

### Removed
- Unused `once_cell` dependency.
- Stale commented-out "examples" module from the crate docs and the
  commented-out README example (replaced by a tested one).
- Private doc-only `ParseNode` alias in `earley`, redundant `use serde_json` /
  `use env_logger` imports, and the `[profile.release]` of the
  `loglan-wasm-app` workspace member (Cargo ignored it with a warning).
- **Public API:** `grammars::LOGLAN_GRAMMAR` and the `loglan` example; this
  crate is now Lojban-only (the `loglan` keyword is dropped as well).
- The `loglan-wasm-app` workspace member (including its committed `pkg/`
  build output), the `[workspace]` section and the GitHub Pages workflow
  `.github/workflows/deploy-wasm-app.yml`.
- Separate `src/cll/LICENSE-*.tergenzalvi` files: the crate has a single MIT
  `LICENSE` at the repository root.

### Changed
- Benchmarks use `std::hint::black_box` instead of `criterion::black_box`
  and measure the camxes Lojban grammar (both engines with `--features
  earley`) instead of the removed Loglan grammar.
- The vendored CLL Chapter 21 layer (`cll::grammar`) now cuts `zoi`/`la'o`
  quotes out before morphology (CLL 19.10): the body is one opaque `FOREIGN`
  lexeme, the quote closes at the first word equal to the delimiter, and an
  unclosed or mismatched quote is a lexical gap (rejected).

### Packaging
- Single MIT `LICENSE` file at the repository root, shipped in the package
  (the README badge and license section link to it). The camxes grammar
  attribution (ilmentufa / camxes.js, © Masato Hagiwara, MIT) is in the
  README.
- Test fixtures (`tests/data/`, corpus test, fetch script) stay in the
  repository but are excluded from the crates.io package.

### Compatibility
- The PEG engine remains the default and the `earley` feature is off by
  default. Apart from the removals listed above, existing APIs are
  unchanged.
