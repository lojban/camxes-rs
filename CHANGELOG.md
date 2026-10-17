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
  `kuarka`; note that the dictionary also labels standard-form words such as `matriocka` obsolete) through the [tergenzalvi](https://github.com/lagleki/tergenzalvi)
  CLL morphology.
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

### Compatibility
- The PEG engine remains the default; existing APIs are unchanged and the
  `earley` feature is off by default.
