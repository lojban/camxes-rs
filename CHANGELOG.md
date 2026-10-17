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
  ungrammatical texts. Limitation: `zoi`/`la'o` delimiters are not
  back-referenced, so the PEG engine rejects `zoi` quotes and the Earley
  engine accepts them without checking that the delimiters match.
- `cll` module (feature `earley`): vendored copy of tergenzalvi (commit
  6eaea40, MIT OR Apache-2.0) providing the CLL morphology and cmavo lexicon,
  so the crate has no git dependency and can be published.
- `grammars::LOJBAN_MORPHOLOGY_GRAMMAR` (previously only in the example).
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
- No public API was removed.

### Changed
- Benchmarks use `std::hint::black_box` instead of `criterion::black_box`.

### Packaging
- Test fixtures (`tests/data/`, corpus test, fetch script) stay in the
  repository but are excluded from the crates.io package.

### Compatibility
- The PEG engine remains the default; existing APIs are unchanged and the
  `earley` feature is off by default.
