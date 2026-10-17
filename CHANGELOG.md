# Changelog

## 2.0.0 — unreleased

A new major version: the latest `camxes-rs` on crates.io is 1.1.1, published
from a different code base (lojban/tersmu), so this release supersedes it
with a major bump rather than claiming compatibility with the 1.x API.

### Added
- Cargo features per engine: `peg` (default) and `earley` (off by default).
  A disabled engine is not compiled in: without `peg` there is no PEG engine
  (`peg::grammar`, `peg::transformer`, `peg::rule`, `peg::stack`), no camxes
  grammar texts (`grammars`) and no `Engine::Peg`; without `earley` there is
  no `cll` module (incl. the embedded `cll21.ebnf` and `cmavo_selmaho.tsv`)
  and no `Engine::Earley`. At least one feature is required
  (`compile_error!` otherwise). `Engine::default()` is `Engine::Peg`, or
  `Engine::Earley` in an Earley-only build. Only the tree types
  (`peg::parsing`) and the `morphology` / `syntax` APIs are shared.
- `morphology` module: engine-independent Lojban morphology API
  (`morphology::analyze(input, Engine)`) returning a normalized `Morphology`
  (word segmentation, `WordClass` — cmavo / gismu / lujvo / fu'ivla / cmevla —
  and cmavo selma'o), plus the `Dialect` that produced it.
- Optional Cargo feature `earley` enabling `Engine::Earley`: the vendored
  CLL pipeline of [tergenzalvi](https://github.com/lagleki/tergenzalvi)
  (`cll`) and nothing else. Morphology: Earley-parsed CFG word shapes
  (`src/cll/cfg/grammar.rs`) + CLL constraint filters + stream segmentation,
  with tergenzalvi's `MorphClass` mapped to the camxes word types
  (`cll::camxes::word_class`). Syntax: the CLL Chapter 21 EBNF
  (`src/cll/data/cll21.ebnf`) over selma'o tokens from `cmavo_selmaho.tsv`,
  with elidable terminators; trees use the Chapter 21 rule names. It gives
  the same word split and classes as the PEG engine on every PEG-accepted
  dictionary headword except 7 documented camxes/CLL differences
  (`KNOWN_DIFFERENCES` in `tests/dictionary_corpus.rs`), and also accepts the
  older CLL dialect (obsolete words such as `kuarka`; note that the
  dictionary also labels standard-form words such as `matriocka` obsolete).
- Rule-based obsolete-dialect flag per word: `MorphWord::word_dialect`
  (`WordDialect::Standard` / `WordDialect::Obsolete`), `MorphWord::is_obsolete`,
  `MorphWord::type_label` (e.g. `obsolete fu'ivla`) and
  `Morphology::has_obsolete`. A word is obsolete when the CLL Chapter 3–4
  rules accept it but it breaks the camxes syllable rules (a port of camxes
  `syllables` / `zifcme`, `cll::camxes::is_obsolete_form`) — no word lists
  are involved, so new words are classified too (on-glide diphthongs in brivla/names such as `kuarka`, `n` before an
  affricate as in `djinundji`, `CiV'V'V` cmavo such as `cia'o'e`, ...). In a
  zei-lujvo only the obsolete component is flagged. The PEG engine only
  produces standard words.
- `syntax` module and `grammars::LOJBAN_GRAMMAR` (the full camxes Lojban
  grammar from ilmentufa / camxes.js, MIT): `syntax::parse(text, Engine)` /
  `syntax::accepts` check whole texts with either engine (PEG:
  `LOJBAN_GRAMMAR`; Earley: CLL Chapter 21 `cll21.ebnf`).
  `tests/grammar_basics.rs` covers basic constructions (bridi, LE/KU, FA/BAI,
  tenses, NU/DU'U, POI/NOI/GOI, JE/.e/GIhA/GA, .i, lu/li'u, zo, numbers and
  MEX, vocatives, UI, lujvo/fu'ivla/cmevla in sentences, ...) and
  ungrammatical texts.
- Delimiter back-references in the PEG engine: ilmentufa-style rule
  annotations `#: PUSH` / `#: PEEK-DIFF` / `#: POP-EQ`. The PEG engine keeps
  a delimiter stack (restored on failed sequences and after predicates;
  PEEK-DIFF / POP-EQ rules are not memoized); the Earley engine (CLL
  Chapter 21) cuts quotes out before lexing (see Changed). So `zoi X. … .X` / `la'o X. … .X` is accepted only when the
  closing word equals the opening one (case, pauses, commas ignored, `h` ≡
  `'`) and the quote ends at the first such word (the quoted text never
  contains the delimiter as a separate word). `zoi gy. hello .dy glico` is
  rejected, and la'o names parse. `LOJBAN_GRAMMAR` carries the annotations
  on `zoi_open` / `zoi_word` / `zoi_close`; both engines agree on every
  `zoi`/`la'o` case in `tests/grammar_basics.rs` (the Earley tree has the
  delimiter words around one `FOREIGN` node). (Before
  this, the PEG engine's greedy repetition swallowed the closing delimiter
  and every `zoi` quote was rejected.)
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
  agreement reports and benchmarks (`--raw` times the bare word recognisers);
  it also writes a per-word report to `target/corpus-results.tsv`.
- `tests/data/morphology-parity.tsv`: committed expected morphology (words,
  classes, obsolete flags, selma'o), rule-based dialect, a check of the
  dictionary's `obsolete` label against it, and PEG/Earley acceptance for every headword; the corpus
  test fails if either engine deviates. CI workflow `.github/workflows/test.yml`
  runs it with `--features earley`.

### Fixed
- Public entry points return errors instead of panicking on arbitrary input
  or grammar text (new `tests/robustness.rs` fuzz-style test):
  - `Peg::new` panicked on non-ASCII grammar text (`'п'`, `[а-я]`); `.`
    now matches one whole character, ranges advance by the character's
    length. Grammar text the
    bootstrap parser cannot fully consume is now a `GrammarError`.
  - Deep recursion (deeply nested input, left-recursive rules such as
    `s <- s 'a'`) overflowed the stack and aborted the process; it now fails with the new `ErrorKind::RecursionLimit`
    (stack guard in `peg::stack`, `Peg::set_stack_budget`). The PEG parser's
    stack frames were split per rule kind, cutting stack use per nesting
    level ~8x in debug and ~20% in release.
  - `e*` / `e+` where `e` can match empty looped forever; repetition now
    stops at an empty match.
  - `cll::grammar::compile_ebnf` with an unknown start rule panicked; it is
    now an `Err`. `TokenGrammar::from_parts` returns `Result` and checks
    symbol ids. `cll::grammar` `ParseNode::structure` no longer panics on a tree from
    another grammar.
- The PEG engine panicked on non-ASCII input (e.g. a Cyrillic letter in a
  Lojban text) when `.` had consumed one byte of a multi-byte character;
  literals and classes now compare bytes and ranges require a character
  boundary.
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
- `Peg` gained crate-private fields (annotations, delimiter stack), so it can
  no longer be built with a struct literal outside the crate; use
  `Peg::new` (the public fields `rules`, `start`, `memo` are unchanged).
- Benchmarks use `std::hint::black_box` instead of `criterion::black_box`
  and measure the camxes Lojban grammar instead of the removed Loglan
  grammar; with `--features earley` they also measure compiling
  `cll21.ebnf` and `syntax::parse` on the Earley engine.
- The vendored CLL Chapter 21 layer (`cll::grammar`) now cuts `zoi`/`la'o`
  quotes out before morphology (CLL 19.10): the body is one opaque `FOREIGN`
  lexeme, the quote closes at the first word equal to the delimiter, and an
  unclosed or mismatched quote is a lexical gap (rejected).
- The vendored CLL morphology has marked *camxes extensions (not CLL)* so
  that its word types agree with camxes (listed in `src/cll/mod.rs`):
  camxes cmavo shapes (vowel and `y` chains, `i`/`u` glides), the camxes
  slinku'i test, form-based lujvo segmentation with the `CVVr` hyphen, no
  `la`/`lai`/`doi` ban in names (C19 not applied), consonant-only names,
  fu'ivla with a late cluster, "a cmavo followed by words is not a brivla",
  comma pauses and `y`-hyphen lujvo (`src/cll/camxes.rs`); in the Chapter 21
  lexer a digit is a `PA` cmavo.
- The Chapter 21 lexer no longer splits an unknown cmavo before a vowel
  (`co'oi` was read as `co'o` + `i`; vowel-initial cmavo need a pause).
- `syntax::parse` with `Engine::Earley` rejects a text containing a word
  the CLL morphology cannot analyse (the Chapter 21 lexer used to drop such
  words and parse the rest), and a character that is not Lojban
  orthography outside a `zoi`/`la'o` body (the CLL normalizer used to drop
  it, reading `valsi---` as `valsi`).
- During 2.0.0 development `Engine::Earley` first ran the camxes PEG
  grammars through a PEG→CFG converter (`earley::EarleyGrammar`,
  `peg::text`, `syntax::earley_grammar`,
  `morphology::camxes_morphology_grammar`, `Dialect::CamxesComplete`); that
  converter was removed before release in favour of the CLL pipeline.

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
