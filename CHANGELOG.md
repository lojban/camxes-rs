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
  (`word_class` in `src/morphology/camxes.rs`). Syntax: the CLL Chapter 21 EBNF
  (`src/cll/data/cll21.ebnf`) over selma'o tokens from `cmavo_selmaho.tsv`,
  with elidable terminators; trees use the Chapter 21 rule names. It gives
  the same word split and classes as the PEG engine on every PEG-accepted
  dictionary headword except 230 documented camxes/CLL differences
  (`tests/data/known-differences.tsv`, README "Differences between the PEG
  (camxes) and Earley (CLL) engines"), and also accepts the
  older CLL dialect (obsolete words such as `kuarka`; note that the
  dictionary also labels standard-form words such as `matriocka` obsolete).
- Rule-based obsolete-dialect flag per word: `MorphWord::word_dialect`
  (`WordDialect::Standard` / `WordDialect::Obsolete`), `MorphWord::is_obsolete`,
  `MorphWord::type_label` (e.g. `obsolete fu'ivla`) and
  `Morphology::has_obsolete`. A word is obsolete when the CLL Chapter 3–4
  rules accept it but it breaks the camxes syllable rules (a port of camxes
  `syllables` / `zifcme`, `is_obsolete_form` in `src/morphology/camxes.rs`) — no word lists
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
- `cll` module (feature `earley`): vendored copy of tergenzalvi (kept
  identical to its `src/`, see `scripts/check_cll_sync.sh`; same author, included under this crate's MIT license) providing the CLL morphology and cmavo lexicon,
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

- Earley dialect switch (feature `earley`): `morphology::MorphOptions`
  (re-export of the shared tergenzalvi type) has one dialect field,
  `camxes_dialect`, next to the §4.16 flag `rafsi_fuhivla`. Strict CLL
  (`false`) and the camxes dialect (`true`) differ in exactly two ways that
  always change together: (1) lujvo built from rafsi outside the CLL-era
  list are recognised by §4.5 rafsi shape (CVC, CVV/CV'V, CCV, 4-letter +
  `y`, final 5-letter), still subject to the §4.6 hyphen and §4.7
  tosmabru/slinku'i rules; strict CLL accepts only list rafsi (C13); and
  (2) names may contain `la`/`lai`/`doi` (strict CLL applies the §4.8 ban,
  C19). Presets `default()` (strict CLL + §4.16), `settled()` (strict CLL,
  no §4.16) and `camxes()`; `with_camxes_dialect`. `Engine::Earley` uses
  `morphology::earley_options()` = `camxes()`; `morphology::analyze_with_options`
  and `syntax::parse_with_options` take any options. `WordDialect` stays
  rule-based. `volapuk`, `laus`, `lat`, `lauDZIS`, `katajinadelazariRAdek`,
  `kuualalumpur`, `tigbalan`, `doitclan` now agree with the PEG engine.
- Consonant-only names in the shared CLL code (both dialects): a name made
  only of consonants is accepted when every adjacent pair and triple is
  permissible (§3.7): `.d.`, `.c.`, `.t.`, `.rl.`, `.xm.` are names, `.mz.`
  and `.ndj.` are not. `t`, `xm`, `ng bu`, `b zei p zei f zei k`,
  `l zei l zei g` and the `dwe` texts ending in `zo d` / `la c` now agree
  with the PEG engine.

### Fixed
- Earley (shared CLL code): vowel runs in fu'ivla are grouped from the left
  as CLL §3.5 says ("When more than two vowels occur together in Lojban,
  the normal pronunciation pairs vowels from the left into syllables"):
  a stretch of diphthongs optionally ending in one vowel, across
  apostrophes too. A fu'ivla may consist of one such run of two or more
  syllables (`blaia`, `gruue`, `jdaio`, `sriuu`, `tsauai`, `traii`). A
  non-diphthong adjacent pair (`eu`, `ou`, `ea`) still needs a comma or
  apostrophe; gismu/lujvo diphthong restrictions are unchanged. 17 dictionary
  headwords now agree with the PEG engine (`blaia`, `briii`,
  `cmaiueiorze'aiuei`, `gondiii'i`, `gruue`, `jboia`, `jboiu`, `jdaio`,
  `jmaia`, `mla'aiana`, `onmatopo'eia`, `sriuu`, `tciuaua`, `traii`,
  `tsauai`, `vluuo`, `xruii`).
- `grammars::LOJBAN_MORPHOLOGY_GRAMMAR` (PEG) accepted words with a medial
  consonant pair mixing voiced and unvoiced consonants (CLL §3.7). A word is
  now rejected when a voicing clash occurs before the next pause; on the
  dictionary corpus this rejects exactly `ciztolnei` (`zt`) and `refgau`
  (`fg`), now invalid for both engines. The forbidden pair `mz` is still not
  checked (`vamzma` remains a known difference).
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
- Known differences (`tests/data/known-differences.tsv`,
  `DIFFERENCE_CATEGORIES`): 262 → 230. `name-la` and `consonant-name` are
  gone; `non-cll-rafsi` (171) is split by the specific CLL rule each word
  breaks: `n-hyphen` (2), `rafsi-vowel-initial` (98), `y-after-vowel` (16),
  `rafsi-shape` (55); `medial-pair` is now `vamzma` only.
  After the vowel-grouping fix (see Fixed) `vowel-grouping` has 15 words (runs
  whose left grouping starts with the non-diphthong `eu`/`ou`) and
  `fuhivla-cluster` 13 (`nijnanidouaga` moved to `vowel-grouping`;
  `mla'aiana` now agrees). Total 230.
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
- The vendored CLL code in `src/cll` is identical to tergenzalvi's `src/`
  (apart from crate paths and data-file locations; checked by
  `scripts/check_cll_sync.sh`) and has no camxes-specific behaviour: earlier
  camxes extensions that contradict the CLL text were removed (no
  `la`/`lai`/`doi` ban exemption, fu'ivla need a cluster in the first five
  letters, no `CVVr` hyphen after a non-initial CVV rafsi, no on-glide right
  after a vowel in fu'ivla, no comma pauses or `y`-hyphen lujvo, no
  consonant-only names). Gap fills where the CLL is silent (experimental
  vowel-chain cmavo, apostrophe chains, form-based lujvo segmentation with
  the §4.5 restrictions, the literal slinku'i test, the §4.6/§4.7 cmavo
  break-up rule, digits as `PA`) are shared with tergenzalvi. The mapping
  to camxes word types and the obsolete-form rule moved to
  `src/morphology/camxes.rs`. The 230 PEG-accepted headwords on which the
  engines now differ are listed by category in
  `tests/data/known-differences.tsv` and in the README.
- The Chapter 21 lexer no longer splits an unknown cmavo before a vowel
  (`co'oi` was read as `co'o` + `i`; vowel-initial cmavo need a pause).
- `syntax::parse` with `Engine::Earley` rejects a text containing a word
  the CLL morphology cannot analyse (the Chapter 21 lexer used to drop such
  words and parse the rest; `cll::grammar::parse_text` now refuses such
  texts itself, and skips punctuation-only tokens such as `-`), and a character that is not Lojban
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
