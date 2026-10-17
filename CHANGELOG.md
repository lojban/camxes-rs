# Changelog

## 0.1.6 — unreleased

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
  `matriocka`) through the [tergenzalvi](https://github.com/lagleki/tergenzalvi)
  CLL morphology.
- `grammars::LOJBAN_MORPHOLOGY_GRAMMAR` (previously only in the example).
- Dictionary corpus test (`tests/dictionary_corpus.rs`) over all lensisku
  English headwords (`tests/data/lensisku-en-words.tsv.gz`, refreshed with
  `scripts/fetch_lensisku_corpus.py`) and `examples/morphology_corpus.rs` for
  agreement reports and benchmarks (`--raw` times the bare grammar engines).

### Fixed
- `Peg::new` panicked on grammar text with leading whitespace.

### Compatibility
- The PEG engine remains the default; existing APIs are unchanged and the
  `earley` feature is off by default.
