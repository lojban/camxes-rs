# Performance Plan: Parser & Build Acceleration

This document outlines a structured plan to accelerate the camxes.rs PEG parser (runtime) and the project build/startup, from both Rust and code-architecture perspectives. Each section is ordered by impact vs. effort where practical.

---

## 1. Executive Summary

| Area | Main bottlenecks | Suggested focus |
|------|------------------|-----------------|
| **Parser runtime** | Memo key cloning, `ParseResult`/`ParseNode` cloning, `Rule` cloning on error paths, `calculate_line_column` on every error | Memo key representation, reduce cloning, lazy errors |
| **Startup / grammar load** | Full bootstrap parse + transform of ~1000-line grammar on every run | Precompiled grammar (codegen) or cached `Peg` |
| **Rust build** | Already optimized (LTO, codegen-units=1); dev build not tuned for fast iteration | Add dev profile and optional “fast release” |

---

## 2. Parser Runtime Optimizations

### 2.1 Memoization cache (high impact)

**Current behavior**

- Key: `(String, usize)` — every NonTerminal does `name.clone()` to form the key.
- Value: full `ParseResult` (with `Vec<ParseNode>` or `ParseError`) is cloned on cache hit and before insert.

**Recommendations**

1. **Rule name interning**  
   - Introduce a string interner (e.g. `Symbol` or `RuleId(u32)`) so rule names are stored once; memo key becomes `(RuleId, usize)` or `(u32, usize)` with no string clone per lookup/insert.
2. **Avoid cloning `ParseResult` on cache hit**  
   - Store something that allows reuse without cloning the whole tree (e.g. store “position + success” and rebuild only the `ParseNode` for that nonterminal from the current parse stack, or use `Arc<ParseResult>` if sharing is acceptable).
3. **Faster hash map**  
   - Use `ahash` or `rustc_hash` for `HashMap` (e.g. `HashMap<(u32, usize), _>`) to reduce hashing cost in the hot path.
4. **Memo key without alloc**  
   - If keeping `String` keys, consider `(Cow<'_, str>, usize)` or `(&str, usize)` with a borrowed name from the rule map so the hot path doesn’t allocate (lifetime design may require refactor).

**Files:** `src/peg/grammar/types.rs` (memo type), `src/peg/rule/core.rs` (NonTerminal branch: key construction, cache get/insert, clone).

---

### 2.2 Reducing cloning on success and error paths (high impact)

**Current behavior**

- On every cache hit: `return cached_result.clone()` (clones entire `ParseResult` and nested `Vec<ParseNode>` / error).
- On every cache insert: `result.clone()` before `insert`.
- On many error branches: `expression: self.clone()` or `(*expr).as_ref().clone()` (cloning potentially large `Rule` trees).

**Recommendations**

1. **Cache:** Store `ParseResult` behind `Arc` (or similar) so cache hit returns `Arc::clone` only; no clone of the result payload. Consider same for insert (store `Arc<ParseResult>`).
2. **Error payload:**  
   - Lazy `calculate_line_column`: compute line/column only when formatting or serializing the error, not at construction.  
   - Avoid storing full `Rule` in `ParseError` for the hot path: e.g. store `RuleId` or rule name only; resolve to `Rule` only when displaying/serializing.  
   - This removes most `self.clone()` and `(*expr).as_ref().clone()` from error paths.

**Files:** `src/peg/parsing.rs` (ParseError, ParseResult), `src/peg/rule/core.rs` (all branches that construct `ParseError` or return cached result).

---

### 2.3 Literal and character matching (medium impact)

**Current behavior**

- `Rule::Literal(pattern)`: `String`, `input[position..].starts_with(pattern)`.
- `Rule::Class(symbols)`: `HashSet<String>`, linear scan with `find(|s| input[position..].starts_with(s.as_str()))`.

**Recommendations**

1. **Single-byte / single-codepoint literals:** Fast path: if `pattern.len() == 1` (or single ASCII char), compare one byte and advance by 1; avoid `starts_with` and string iteration where possible.
2. **Multi-byte literals:** Keep `starts_with`; ensure `pattern` is stored in a way that doesn’t force extra indirection (e.g. `Arc<str>` or `Box<str>` if you need to share, to avoid large `Rule` size).
3. **Rule::Class:**  
   - Prefer longest match: sort or iterate by length descending so the first match is longest.  
   - Consider a small trie or byte-based lookup for character classes used very often (e.g. “any of these chars”) to avoid scanning many strings.

**Files:** `src/peg/rule/core.rs` (Literal, Class), `src/peg/rule/types.rs` (Rule definition).

---

### 2.4 Allocations in sequences and repetitions (medium impact)

**Current behavior**

- `Rule::Sequence`: `let mut captures = vec![]`; then `captures.append(&mut m)` per element.
- `Rule::ZeroOrMore` / `OneOrMore`: same pattern; repeated `append` can cause reallocations.

**Recommendations**

1. **Reserve capacity:** If the grammar or typical parse gives a known small upper bound for sequence length, use `Vec::with_capacity(n)` before the loop.
2. **Reuse buffer:** For “parse many” loops, consider reusing a single growable buffer and only building the final `Vec<ParseNode>` once (reduces number of small allocs and moves).

**Files:** `src/peg/rule/core.rs` (Sequence, ZeroOrMore, OneOrMore).

---

### 2.5 Line/column calculation (low–medium impact)

**Current behavior**

- `calculate_line_column(input, position)` is called on every error construction and walks from the start of `input` up to `position` (O(position)).

**Recommendations**

1. **Lazy computation:** Store only `position` in `ParseError`; compute `(line, column)` in `Display::fmt` or in a getter when needed (and optionally cache in the error struct if you need it multiple times).
2. **Cached line starts:** For long inputs and multiple errors, maintain a small cache of “byte offset → (line, column)” (e.g. at line boundaries) and binary-search or scan from nearest cached point.

**Files:** `src/peg/parsing.rs` (`calculate_line_column`, `ParseError`), `src/peg/rule/core.rs` (all call sites).

---

### 2.6 Debug and logging (low impact in release)

**Current behavior**

- `log::debug!` and `"│".repeat(depth)` in the hot path.

**Recommendations**

1. Ensure `release` builds do not enable `log` at debug level (no-op macros).
2. Avoid `"│".repeat(depth)` in hot path when debug is disabled (e.g. wrap in `if log_enabled!(Debug)` or use a lazy message).

**Files:** `src/peg/rule/core.rs` (NonTerminal and any other branches that log).

---

## 3. Startup & Grammar Load

### 3.1 Precompiled grammar (codegen) (high impact for startup)

**Current behavior**

- `Peg::new(start_rule, grammar_str)` parses the grammar string with the bootstrap PEG, then runs the Transformer to build the `Rule` tree. For Lojban this is a large string (~1000 lines) parsed and transformed on every process start (e.g. in examples or apps).

**Recommendations**

1. **Build-time codegen:** Add a build script or offline tool that:  
   - Reads `lojban.peg` (or the in-crate grammar string),  
   - Runs the existing bootstrap parser + transformer once,  
   - Emits Rust code that constructs the same `Peg` (e.g. `fn lojban_grammar() -> Peg { ... }`) using `Rule` constructors directly.  
   - The runtime then calls `lojban_grammar()` instead of `Peg::new(..., grammar_str)`, eliminating bootstrap parse and transform at startup.
2. **Cached binary:** Alternatively, serialize the built `Peg` (e.g. bincode) to a file and load it at startup; avoid re-parsing the grammar text. Less portable than codegen but easier to add initially.

**Files:** New: `build.rs` or `tools/codegen_grammar.rs`; `src/grammars.rs` or a new `grammar_compiled.rs` that uses the generated code; `src/peg/grammar/core.rs` (keep `Peg::new` for dynamic grammars).

---

### 3.2 Lazy / static grammar instance (medium impact)

**Current behavior**

- Each example or app that uses the Lojban grammar calls `Peg::new(...)` and pays the cost every time.

**Recommendations**

1. **Lazy static:** Use `once_cell::sync::Lazy` (or `std::sync::OnceLock`) to build the `Peg` once and reuse (e.g. `pub static LOJBAN_PEG: Lazy<Peg> = Lazy::new(|| Peg::new("text", LOJBAN_GRAMMAR_STR).unwrap());`).  
   - Combines well with precompiled grammar: then the “build” is just constructing the struct, no parse.
2. **Document:** Recommend that long-running apps or benchmarks hold a single `Peg` and reuse it for many inputs.

**Files:** `src/grammars.rs`, `src/lib.rs`, example binaries.

---

## 4. Rust Build & Compile Time

### 4.1 Dev profile (fast iteration)

**Current behavior**

- `profile.release` has `lto = true`, `codegen-units = 1` (slow release builds); dev builds use default (multiple codegen units). No explicit dev tuning.

**Recommendations**

1. **Explicit dev profile:** In `Cargo.toml`, set `[profile.dev]` with `opt-level = 1` (or 0) and `codegen-units = 16` (or higher) for faster compile during development. Optionally enable minimal debuginfo.
2. **Optional “fast release”:** e.g. `[profile.release-fast]` with `inherits = "release"`, `lto = false`, `codegen-units = 16` for quicker release builds when profiling or benchmarking iteration (document in README or this doc).

**Files:** `Cargo.toml`.

---

### 4.2 Release profile (already strong)

- Current `release`: `lto = true`, `codegen-units = 1`, `strip = true`, `panic = 'abort'` is already good for maximum runtime performance.  
- No change required unless you add the “release-fast” variant above for convenience.

---

### 4.3 Dependencies and macros

- Keep dependencies minimal; `serde`/`serde_json` are used for JSON output — ensure they’re not pulled in for “parse-only” code paths if you later split a minimal parser API.  
- No heavy proc-macros observed; if you add codegen, prefer build scripts or external tools over proc-macros for compile-time impact.

---

## 5. Measurement & Validation

### 5.1 Benchmarks

1. **Add criterion (or std bench):**  
   - Benchmark: `Peg::new(start, grammar)` for the full Lojban grammar (startup).  
   - Benchmark: `peg.parse(input)` for several inputs (short sentence, long text, failing input).  
   - Store the `Peg` in a lazy static or once per benchmark run so that “parse only” isn’t dominated by grammar build.
2. **Track:** Parse time vs. input length, memo hit rate (if you add metrics), and allocation count (e.g. with `#[global_allocator]` and a simple counter in dev).

**Files:** New `benches/` (e.g. `criterion` in `[dev-dependencies]`), or use `#[bench]` and `cargo bench` if you prefer.

---

### 5.2 Profiling

1. **CPU:** `cargo build --release && perf record -g ./target/release/examples/lojban` (or equivalent); inspect hotspots in `Rule::parse`, memo lookup, and cloning.  
2. **Allocations:** Instruments or a custom allocator to see where allocations come from (e.g. `ParseNode` vecs, `String` clones, `ParseResult` clone).  
3. **Cache behavior:** Temporarily log or count memo hits/misses to see how effective the cache is and whether key design (e.g. interning) is worth it.

---

## 6. Implementation Priority (Suggested Order)

| Priority | Item | Effort | Impact |
|----------|------|--------|--------|
| 1 | Add benchmarks (startup + parse) | Low | Enables measuring all other work |
| 2 | Lazy/static `Peg` for default grammar | Low | Big startup win for multi-parse scenarios |
| 3 | Memo: avoid cloning `ParseResult` (e.g. `Arc`) | Medium | High impact in hot path |
| 4 | Memo key: rule name interning + `(RuleId, usize)` | Medium | Reduces allocs and hashing cost |
| 5 | Error path: lazy line/column, no full `Rule` in error | Medium | Reduces cloning and work on failure |
| 6 | Precompiled grammar (codegen or cached load) | High | Largest startup improvement |
| 7 | Literal/Class fast paths and small alloc wins | Low–Medium | Steady gains in parse hot path |
| 8 | Dev profile + optional release-fast | Low | Better DX and profiling iteration |

---

## 7. Summary

- **Parser:** Focus on memo representation (no clone on hit/insert, interned rule names), lighter error representation (lazy line/column, no `Rule` clone), and literal/class fast paths.  
- **Startup:** Precompile the Lojban grammar (codegen or cached `Peg`) and expose it via a lazy/static instance so normal runs don’t re-parse the grammar.  
- **Build:** Add an explicit dev profile and optional “release-fast” for iteration; keep current release settings for production.  
- **Validation:** Add benchmarks and use profiling to confirm improvements and avoid regressions.

This plan should give both immediate wins (lazy static, Arc in memo, lazy errors) and a clear path to larger gains (codegen, interning, and targeted hot-path tuning).
