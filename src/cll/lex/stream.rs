//! Stream segmentation — CLL §4.1 + §4.9 obligatory pause contexts.
//!
//! # Normative status (CRITICAL)
//!
//! CLL §4.1: a Lojban sound stream can be uniquely broken into words and the
//! three classes have uniquely identifying properties, **but**:
//!
//! > a detailed algorithm for breaking sounds into words has not yet been
//! > fully debugged, and so is not presented in this book.
//!
//! This module therefore does **not** implement “the book’s algorithm” (absent).
//! It implements the strongest deterministic segmenter assemblable from
//! CLL-stated Ch. 3–4 rules, with every search/tie-break CLL does not dictate
//! marked **DERIVED**. Residual true underspecification remains **GAP**.
//!
//! See the repository README’s stream segmentation section.
//!
//! # Layers
//!
//! 1. **Orthographic** — spaces and explicit `.` pauses (§3.3 reminder).
//! 2. **Continuous** — DP over `morph_parse` prefixes with §3.3 pause inference,
//!    §4.9.4 cmevla adjacency, §4.9.5 marked final-stress before brivla, and
//!    `resolve_covers` DERIVED policy.
//! 3. **Streaming** — [`StreamingSegmenter`] for chunked / unbounded input
//!    with bounded buffer (see the repository README).
//!
//! # Compound cmavo (§4.2)
//!
//! A compound “is always identical in meaning and grammatical use to the
//! separated sequence of simple cmavo”. **DERIVED:** prefer covers that
//! decompose into simple cmavo over a single `CmavoCompound` token.
//!
//! # Non-Lojban quotes (§4.9.7 / §19.10)
//!
//! Only the Ch. 4 pause-surround requirement is in scope. `la'o` delimiter
//! internals are out of scope.

use crate::cll::constraints::pauses::cmevla_allowed_after;
use crate::cll::constraints::stress::final_syllable_stressed;
use crate::cll::constraints::{morph_parse, ConstrainedResult};
use crate::cll::lex::{Token, WordClass};
use crate::cll::phonology::{is_c, is_v, is_y};
use crate::cll::shapes::MorphClass;
use std::collections::BTreeSet;

/// DERIVED: CLL states no max word length; bounds continuous DP.
const MAX_WORD_BYTES: usize = 48;
/// DERIVED fan-out cap per byte index.
const MAX_PATHS_PER_INDEX: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamToken {
    pub text: String,
    pub result: ConstrainedResult,
    /// True when this boundary was forced by explicit `.` or whitespace.
    pub explicit_boundary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamResult {
    pub tokens: Vec<StreamToken>,
    // Byte indices in the raw input where a pause was recognized.
    pub pauses: Vec<usize>,
    /// Ambiguity / DERIVED / GAP notes (§4.1 algorithm absent from book).
    pub ambiguity: Vec<String>,
}

/// Segment orthography that already uses spaces and/or `.` pauses.
pub fn segment_orthographic(input: &str) -> StreamResult {
    let mut tokens = Vec::new();
    let mut pauses = Vec::new();
    let mut ambiguity = Vec::new();
    let mut byte_idx = 0usize;
    let mut pending_pause = false;

    for ws_piece in split_keeping_sep(input, |c| c.is_whitespace()) {
        if ws_piece.chars().all(|c| c.is_whitespace()) {
            byte_idx += ws_piece.len();
            continue;
        }
        let parts: Vec<&str> = ws_piece.split('.').collect();
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                if i > 0 || ws_piece.starts_with('.') {
                    pauses.push(byte_idx);
                }
                continue;
            }
            let has_following_dot =
                i + 1 < parts.len() || (i + 1 == parts.len() && ws_piece.ends_with('.'));
            // Keep written boundary pauses as orthographic evidence for the
            // constraint layer; the shape CFG sees only the word body.
            let r = parse_orthographic_piece(part, i > 0 || pending_pause, has_following_dot);
            pending_pause = has_following_dot;
            tokens.push(StreamToken {
                text: (*part).to_string(),
                result: r,
                explicit_boundary: true,
            });
            byte_idx += part.len();
            if has_following_dot {
                pauses.push(byte_idx);
                byte_idx += 1;
            }
        }
    }

    if tokens.is_empty() && !input.trim().is_empty() {
        ambiguity.push("C01 §4.1: orthographic segmenter produced no word tokens".into());
    }
    StreamResult {
        tokens,
        pauses,
        ambiguity,
    }
}

fn parse_orthographic_piece(
    body: &str,
    leading_pause: bool,
    trailing_pause: bool,
) -> ConstrainedResult {
    // Prefer the spelling carrying all available pause evidence. A pause after
    // an ordinary V-final cmavo is legal but need not be part of its word shape.
    for (lead, trail) in [
        (leading_pause, trailing_pause),
        (leading_pause, false),
        (false, trailing_pause),
    ] {
        if !lead && !trail {
            continue;
        }
        let mut spelling = String::with_capacity(body.len() + 2);
        if lead {
            spelling.push('.');
        }
        spelling.push_str(body);
        if trail {
            spelling.push('.');
        }
        let result = morph_parse_stream_piece(&spelling);
        if result.accepted {
            return result;
        }
    }
    morph_parse_stream_piece(body)
}

/// Continuous segmentation when pauses/spaces are omitted.
///
/// 1. Enumerate stream-valid morph prefixes at each index.
/// 2. Filter internal cuts before vowel-initial words (§4.9.3 has no pause
///    evidence in continuous input) and §4.9.4-illegal cmevla transitions.
/// 3. Detect §4.9.5: capital-marked final stress on W1 before brivla W2
///    requires a pause. Continuous text has no `.`, so such an adjacency is
///    illegal; if any such cut exists the whole continuous cover set fails
///    (no camxes-style cmevla escape). Cmevla left-words are exempt (§4.9.2
///    already forces a trailing pause). Unmarked text does not invent
///    default cmavo stress.
/// 4. DP all complete covers (fan-out capped — DERIVED).
/// 5. `resolve_covers` DERIVED policy; record residual ambiguity.
pub fn segment_continuous(input: &str) -> StreamResult {
    let raw: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if raw.contains('.') || input.chars().any(|c| c.is_whitespace()) {
        return segment_orthographic(input);
    }
    if raw.is_empty() {
        return StreamResult {
            tokens: vec![],
            pauses: vec![],
            ambiguity: vec![],
        };
    }

    let n = raw.len();
    let mut edges: Vec<Vec<Edge>> = vec![Vec::new(); n];
    for (i, _) in raw.char_indices() {
        let rest = &raw[i..];
        let max = rest.len().min(MAX_WORD_BYTES);
        for len in 1..=max {
            if !rest.is_char_boundary(len) || rest.as_bytes()[len - 1] == b'\'' {
                continue;
            }
            if let Some(parts) = classify_stream_word(&rest[..len]) {
                edges[i].push(Edge {
                    end: i + len,
                    unique: parts.unique,
                    compound_cmavo: parts.compound_cmavo,
                    cmevla: parts.cmevla,
                    brivla: parts.brivla,
                });
            }
        }
    }

    #[derive(Clone)]
    struct Reach {
        path: Vec<PathTok>,
    }

    let mut reaches: Vec<Vec<Reach>> = vec![Vec::new(); n + 1];
    reaches[0].push(Reach { path: vec![] });
    // Byte indices where §4.9.5 demands a pause (end of final-stressed W1).
    let mut stress_cuts: BTreeSet<usize> = BTreeSet::new();

    for i in 0..n {
        if reaches[i].is_empty() {
            continue;
        }
        // In continuous input, an internal cut supplies no written pause.
        // A V-initial word cannot begin there; e.g. au cannot be a|u.
        if i > 0 && is_v(raw.as_bytes()[i].to_ascii_lowercase()) {
            continue;
        }
        let parent_count = reaches[i].len();
        for edge in &edges[i] {
            for pi in 0..parent_count {
                let left = reaches[i][pi]
                    .path
                    .last()
                    .map(|t| &raw[t.start..t.end])
                    .unwrap_or("");
                if edge.cmevla && !cmevla_allowed_after(left) {
                    continue;
                }
                // §4.8 / §4.9.2: a cmevla must be followed by a pause. Continuous
                // input has none inside it, so a cmevla can only end the input.
                if edge.cmevla && edge.end != n {
                    continue;
                }
                // §4.9.5: marked final stress + following brivla needs pause.
                // Continuous input has no `.`; reject the adjacency and record
                // the cut so merges spanning it are also dropped. Cmevla left
                // words are exempt (§4.9.2 already forces a trailing pause).
                if edge.brivla {
                    if let Some(prev) = reaches[i][pi].path.last() {
                        let prev_text = &raw[prev.start..prev.end];
                        if final_syllable_stressed(prev_text) && !prev.cmevla {
                            stress_cuts.insert(i);
                            continue;
                        }
                    }
                }
                let mut path = reaches[i][pi].path.clone();
                path.push(PathTok {
                    start: i,
                    end: edge.end,
                    unique: edge.unique,
                    compound_cmavo: edge.compound_cmavo,
                    cmevla: edge.cmevla,
                    brivla: edge.brivla,
                });
                if reaches[edge.end].len() < MAX_PATHS_PER_INDEX {
                    reaches[edge.end].push(Reach { path });
                }
            }
        }
    }

    let mut ambiguity = Vec::new();
    let mut covers: Vec<Vec<PathTok>> = reaches[n].iter().map(|r| r.path.clone()).collect();
    if !stress_cuts.is_empty() {
        covers.retain(|path| !path_crosses_cuts(path, &stress_cuts));
        // Boundary-at-cut without pause is the filtered adjacency itself; any
        // remaining cover must not rely on that cut. If nothing remains, the
        // fused continuous reading fails.
        if covers.is_empty() {
            ambiguity.push(format!(
                "C11 §4.9.5: marked final stress before brivla in `{raw}` requires pause; continuous text has none — fused reading rejected (unmarked defaults do not trigger this)"
            ));
        } else {
            ambiguity.push(format!(
                "C11 §4.9.5: avoided final-stress|brivla cuts at bytes {stress_cuts:?} (pause required if that split is intended)"
            ));
        }
    }

    if covers.is_empty() {
        if stress_cuts.is_empty() {
            ambiguity.push(format!(
                "C01 §4.1 residual: no complete morph cover for `{raw}` (GAP if CLL-unique expected)"
            ));
        } else {
            ambiguity.push(format!(
                "C01 §4.1 residual: no legal continuous cover for `{raw}` after §4.9.5"
            ));
        }
        let r = morph_parse_stream_piece(&raw);
        return StreamResult {
            tokens: vec![StreamToken {
                text: raw,
                result: r,
                explicit_boundary: false,
            }],
            pauses: vec![],
            ambiguity,
        };
    }

    let (best, notes) = resolve_covers(&raw, &covers);
    ambiguity.extend(notes);
    if covers.len() > 1 {
        ambiguity.push(format!(
            "C01 §4.1: {} complete covers before DERIVED resolve; book algorithm not presented",
            covers.len()
        ));
    }

    let tokens: Vec<StreamToken> = best
        .iter()
        .map(|t| {
            let text = &raw[t.start..t.end];
            StreamToken {
                text: text.to_string(),
                result: morph_parse_stream_piece(text),
                explicit_boundary: false,
            }
        })
        .collect();

    StreamResult {
        tokens,
        pauses: vec![],
        ambiguity,
    }
}

/// Spaces / `.` → orthographic; else continuous DP.
pub fn segment_stream(input: &str) -> StreamResult {
    if input.chars().any(|c| c.is_whitespace()) || input.contains('.') {
        segment_orthographic(input)
    } else {
        segment_continuous(input)
    }
}

#[derive(Clone)]
struct Edge {
    end: usize,
    unique: bool,
    compound_cmavo: bool,
    cmevla: bool,
    brivla: bool,
}

#[derive(Clone)]
struct WordParts {
    unique: bool,
    compound_cmavo: bool,
    cmevla: bool,
    brivla: bool,
}

#[derive(Clone)]
struct PathTok {
    start: usize,
    end: usize,
    unique: bool,
    compound_cmavo: bool,
    cmevla: bool,
    brivla: bool,
}

fn classify_stream_word(cand: &str) -> Option<WordParts> {
    let r = morph_parse_stream_piece(cand);
    if !r.accepted {
        return None;
    }
    let word_classes: BTreeSet<MorphClass> = r
        .classes
        .iter()
        .copied()
        .filter(|c| c.to_word_class().is_some())
        .collect();
    if word_classes.is_empty() {
        return None;
    }
    let unique = word_classes.len() == 1
        || (word_classes.len() == 2
            && word_classes.contains(&MorphClass::Cmavo)
            && word_classes.contains(&MorphClass::CmavoCompound));
    let simple_cmavo = word_classes.contains(&MorphClass::Cmavo);
    let compound_cmavo = word_classes.contains(&MorphClass::CmavoCompound) && !simple_cmavo;
    let cmevla = word_classes.contains(&MorphClass::Cmevla);
    let brivla = word_classes.iter().any(|c| {
        matches!(
            c,
            MorphClass::Gismu | MorphClass::Lujvo | MorphClass::Fuhivla
        )
    });
    Some(WordParts {
        unique,
        compound_cmavo,
        cmevla,
        brivla,
    })
}

/// Morph-parse a stream piece with §3.3-inferred pause orthography.
///
/// Isolated `morph_parse` hard-rejects bare C-final cmevla (C09) and bare Cy
/// (C07). At a recognized stream boundary those pauses are implied.
pub fn morph_parse_stream_piece(text: &str) -> ConstrainedResult {
    let primary = morph_parse(text);
    // A consonant fragment (`xm` as a consonant pair) is not yet a word: try
    // the name reading with the implied pause.
    let is_word =
        |r: &ConstrainedResult| r.accepted && r.classes.iter().any(|c| c.to_word_class().is_some());
    if is_word(&primary) {
        return primary;
    }
    let core = text.trim_matches('.');
    if core.is_empty() {
        return primary;
    }
    let bytes = core.as_bytes();
    // Capitals only mark stress (§3.9); classify letters case-insensitively.
    let first = bytes
        .iter()
        .map(u8::to_ascii_lowercase)
        .find(|&c| is_c(c) || is_v(c) || is_y(c));
    let last = bytes
        .iter()
        .rev()
        .map(u8::to_ascii_lowercase)
        .find(|&c| is_c(c) || is_v(c) || is_y(c));
    let needs_lead = matches!(first, Some(c) if is_v(c)) && !text.starts_with('.');
    let is_cy = bytes.len() == 2
        && is_c(bytes[0].to_ascii_lowercase())
        && is_y(bytes[1].to_ascii_lowercase());
    let needs_trail = (matches!(last, Some(c) if is_c(c)) || is_cy) && !text.ends_with('.');
    if !needs_lead && !needs_trail {
        return primary;
    }
    let mut rebuilt = String::with_capacity(core.len() + 2);
    if needs_lead {
        rebuilt.push('.');
    }
    rebuilt.push_str(core);
    if needs_trail {
        rebuilt.push('.');
    }
    let alt = morph_parse(&rebuilt);
    if is_word(&alt) {
        alt
    } else {
        primary
    }
}

/// DERIVED cover resolution — **not** a CLL-published procedure.
///
/// Lexicographic score (larger better):
/// 1. Minimize cmevla count (anti-overgenerate names in continuous text).
/// 2. Minimize tokens with ambiguous word classes (§4.1 class uniqueness).
/// 3. Minimize the lower bound on constituent word shapes: one per token, plus
///    one for each compound cmavo (which has at least two pieces). Thus `au`
///    beats `a|u` without a diphthong exception, while `le|re` ties `lere`.
/// 4. On that tie, prefer separate simple cmavo over `CmavoCompound`.
/// 5. Maximize brivla tokens (gismu/lujvo/fu'ivla).
/// 6. Lexicographically smallest `join("|")` (deterministic).
fn resolve_covers(raw: &str, covers: &[Vec<PathTok>]) -> (Vec<PathTok>, Vec<String>) {
    let mut notes = vec![
        "C01 DERIVED resolve: min cmevla, min ambiguous tokens, min constituent \
         word shapes, prefer simple cmavo (§4.2), max brivla, lex tiebreak — not the (absent) book algorithm"
            .into(),
    ];

    type Score = (i32, i32, i32, i32, i32, String);
    let score = |path: &[PathTok]| -> Score {
        let cmevla_n = path.iter().filter(|t| t.cmevla).count() as i32;
        let ambiguous_n = path.iter().filter(|t| !t.unique).count() as i32;
        let compound_n = path.iter().filter(|t| t.compound_cmavo).count() as i32;
        let brivla_n = path.iter().filter(|t| t.brivla).count() as i32;
        let tok_n = path.len() as i32;
        let join = path
            .iter()
            .map(|t| &raw[t.start..t.end])
            .collect::<Vec<_>>()
            .join("|");
        let shape_n = tok_n + compound_n;
        (
            -cmevla_n,
            -ambiguous_n,
            -shape_n,
            -compound_n,
            brivla_n,
            join,
        )
    };

    let best = covers
        .iter()
        .max_by(|a, b| score(a).cmp(&score(b)))
        .cloned()
        .unwrap_or_default();

    if covers.len() > 1 {
        let best_s = score(&best);
        let tied = covers.iter().filter(|c| score(c) == best_s).count();
        if tied > 1 {
            notes.push(format!(
                "C01 GAP: {tied} covers tied under DERIVED score — emitting one deterministically"
            ));
        }
    }

    (best, notes)
}

fn path_crosses_cuts(path: &[PathTok], cuts: &BTreeSet<usize>) -> bool {
    for tok in path {
        for &cut in cuts {
            if tok.start < cut && cut < tok.end {
                return true;
            }
        }
    }
    false
}

fn split_keeping_sep<F>(s: &str, mut is_sep: F) -> Vec<&str>
where
    F: FnMut(char) -> bool,
{
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut in_sep: Option<bool> = None;
    for (idx, ch) in s.char_indices() {
        let sep = is_sep(ch);
        match in_sep {
            None => in_sep = Some(sep),
            Some(prev) if prev != sep => {
                out.push(&s[start..idx]);
                start = idx;
                in_sep = Some(sep);
            }
            _ => {}
        }
    }
    if start < s.len() || s.is_empty() {
        out.push(&s[start..]);
    }
    out
}

/// Map a constrained stream token to a lex [`Token`] when uniquely classified.
pub fn to_lex_token(st: &StreamToken) -> Option<Token> {
    let classes: Vec<WordClass> = st
        .result
        .classes
        .iter()
        .filter_map(|c| MorphClass::to_word_class(*c))
        .collect();
    if classes.len() == 1 {
        Some(Token::Word {
            class: classes[0],
            text: st.text.clone(),
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Streaming API — bounded memory for orthographic / regularly-committing text
// ---------------------------------------------------------------------------

/// Default continuous unresolved-buffer cap (bytes). DERIVED.
pub const DEFAULT_MAX_LOOKAHEAD: usize = 256;

/// Mode for [`StreamingSegmenter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    /// Emit on whitespace / `.` as soon as a word is complete. O(max word) buffer.
    Orthographic,
    /// Continuous DP with unique-prefix commit + max lookahead.
    Continuous,
    /// Auto: orthographic once any whitespace/`.` seen; else continuous.
    Auto,
}

/// Continuous overflow when unresolved buffer exceeds `max_lookahead`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    /// Apply DERIVED `resolve_covers` / `segment_continuous` to force a prefix
    /// commit (document as DERIVED).
    DerivedForceCommit,
    /// Refuse to grow; surface [`StreamPush::AmbiguousOverflow`].
    Refuse,
}

/// Result of a single [`StreamingSegmenter::push`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamPush {
    /// Zero or more tokens uniquely committed (or orthographically complete).
    Tokens(Vec<StreamToken>),
    /// Continuous buffer exceeded `max_lookahead` under [`OverflowPolicy::Refuse`].
    AmbiguousOverflow {
        buffered_bytes: usize,
        tokens: Vec<StreamToken>,
    },
}

/// Incremental stream segmenter — memory O(buffer), not O(total input).
///
/// - **Orthographic:** buffer at most one in-progress word. Happy path supports
///   effectively infinite spaced/dotted input.
/// - **Continuous:** keep unresolved suffix; emit when every surviving complete
///   cover of the current buffer agrees on the next cut (unique prefix
///   commitment). True unbounded unresolved ambiguity is a residual limit —
///   see [`OverflowPolicy`] and the repository README.
pub struct StreamingSegmenter {
    mode: StreamMode,
    effective: StreamMode,
    max_lookahead: usize,
    overflow: OverflowPolicy,
    buf: String,
    buf_origin: usize,
    leading_pause: bool,
    pauses: Vec<usize>,
    ambiguity: Vec<String>,
    saw_ortho_sep: bool,
    overflowed: bool,
}

impl Default for StreamingSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingSegmenter {
    pub fn new() -> Self {
        Self::with_options(
            StreamMode::Auto,
            DEFAULT_MAX_LOOKAHEAD,
            OverflowPolicy::DerivedForceCommit,
        )
    }

    pub fn with_options(mode: StreamMode, max_lookahead: usize, overflow: OverflowPolicy) -> Self {
        Self {
            mode,
            effective: mode,
            max_lookahead: max_lookahead.max(MAX_WORD_BYTES),
            overflow,
            buf: String::new(),
            buf_origin: 0,
            leading_pause: false,
            pauses: Vec::new(),
            ambiguity: Vec::new(),
            saw_ortho_sep: false,
            overflowed: false,
        }
    }

    /// Push a chunk; returns newly committed tokens (may be empty).
    pub fn push(&mut self, chunk: &str) -> StreamPush {
        if self.overflowed {
            return StreamPush::AmbiguousOverflow {
                buffered_bytes: self.buf.len(),
                tokens: vec![],
            };
        }
        if self.mode == StreamMode::Auto && !self.saw_ortho_sep {
            if chunk.chars().any(|c| c.is_whitespace()) || chunk.contains('.') {
                self.saw_ortho_sep = true;
                self.effective = StreamMode::Orthographic;
            } else {
                self.effective = StreamMode::Continuous;
            }
        } else if self.mode == StreamMode::Orthographic {
            self.effective = StreamMode::Orthographic;
        } else if self.mode == StreamMode::Continuous {
            self.effective = StreamMode::Continuous;
        }

        match self.effective {
            StreamMode::Orthographic | StreamMode::Auto => {
                StreamPush::Tokens(self.push_orthographic(chunk))
            }
            StreamMode::Continuous => self.push_continuous(chunk),
        }
    }

    /// Finish the stream: flush remaining buffer.
    pub fn finish(mut self) -> StreamResult {
        let mut tokens = Vec::new();
        match self.effective {
            StreamMode::Orthographic | StreamMode::Auto => {
                if !self.buf.is_empty() {
                    let piece = std::mem::take(&mut self.buf);
                    let r = parse_orthographic_piece(&piece, self.leading_pause, false);
                    tokens.push(StreamToken {
                        text: piece,
                        result: r,
                        explicit_boundary: true,
                    });
                }
            }
            StreamMode::Continuous => {
                if !self.buf.is_empty() {
                    let r = segment_continuous(&self.buf);
                    tokens.extend(r.tokens);
                    self.ambiguity.extend(r.ambiguity);
                    self.buf.clear();
                }
            }
        }
        StreamResult {
            tokens,
            pauses: self.pauses,
            ambiguity: self.ambiguity,
        }
    }

    fn push_orthographic(&mut self, chunk: &str) -> Vec<StreamToken> {
        let mut out = Vec::new();
        for ch in chunk.chars() {
            if ch.is_whitespace() {
                if !self.buf.is_empty() {
                    let piece = std::mem::take(&mut self.buf);
                    let r = parse_orthographic_piece(&piece, self.leading_pause, false);
                    out.push(StreamToken {
                        text: piece,
                        result: r,
                        explicit_boundary: true,
                    });
                    self.leading_pause = false;
                }
                self.buf_origin += ch.len_utf8();
                continue;
            }
            if ch == '.' {
                if !self.buf.is_empty() {
                    let piece = std::mem::take(&mut self.buf);
                    let r = parse_orthographic_piece(&piece, self.leading_pause, true);
                    let pause_at = self.buf_origin + piece.len();
                    out.push(StreamToken {
                        text: piece,
                        result: r,
                        explicit_boundary: true,
                    });
                    self.pauses.push(pause_at);
                    self.buf_origin = pause_at + 1;
                } else {
                    self.pauses.push(self.buf_origin);
                    self.buf_origin += 1;
                }
                self.leading_pause = true;
                continue;
            }
            self.buf.push(ch);
        }
        out
    }

    fn push_continuous(&mut self, chunk: &str) -> StreamPush {
        for ch in chunk.chars() {
            if ch.is_whitespace() {
                continue;
            }
            if ch == '.' {
                self.ambiguity.push(
                    "C01 streaming: `.` in continuous mode treated as orthographic pause".into(),
                );
                let mut emitted = Vec::new();
                if !self.buf.is_empty() {
                    let r = segment_continuous(&self.buf);
                    emitted.extend(r.tokens);
                    self.ambiguity.extend(r.ambiguity);
                    self.buf.clear();
                }
                self.effective = StreamMode::Orthographic;
                self.saw_ortho_sep = true;
                emitted.extend(self.push_orthographic("."));
                return StreamPush::Tokens(emitted);
            }
            self.buf.push(ch);
        }

        let mut emitted = Vec::new();
        while let Some(tok) = self.try_commit_unique_prefix() {
            let len = tok.text.len();
            emitted.push(tok);
            self.buf = self.buf[len..].to_string();
            self.buf_origin += len;
        }

        if self.buf.len() > self.max_lookahead {
            match self.overflow {
                OverflowPolicy::Refuse => {
                    self.overflowed = true;
                    self.ambiguity.push(format!(
                        "C01 streaming: continuous buffer {}B exceeds max_lookahead {} — AmbiguousOverflow",
                        self.buf.len(),
                        self.max_lookahead
                    ));
                    return StreamPush::AmbiguousOverflow {
                        buffered_bytes: self.buf.len(),
                        tokens: emitted,
                    };
                }
                OverflowPolicy::DerivedForceCommit => {
                    if let Some(tok) = self.force_derived_commit() {
                        self.ambiguity.push(
                            "C01 DERIVED streaming: forced prefix commit at max_lookahead".into(),
                        );
                        let len = tok.text.len();
                        self.buf = self.buf[len..].to_string();
                        self.buf_origin += len;
                        emitted.push(tok);
                    }
                }
            }
        }

        StreamPush::Tokens(emitted)
    }

    /// Unique-prefix commit: among complete covers of the current buffer, if
    /// every cover agrees on the first token end index, emit that token.
    /// Does not commit a whole-buffer single-token cover (wait for finish /
    /// more input) unless multi-token covers exist.
    fn try_commit_unique_prefix(&self) -> Option<StreamToken> {
        let raw = self.buf.as_str();
        if raw.is_empty() {
            return None;
        }
        let full = segment_continuous(raw);
        if full.tokens.len() < 2 {
            // Single token or failure fallback — wait for more input / finish.
            return None;
        }
        // Re-check uniqueness of first cut among all covers by re-running a
        // light agreement probe: if segment_continuous's DERIVED best starts
        // with T0, also verify that the greedy alternative of chopping T0
        // and re-segmenting the rest still yields a consistent reading —
        // stronger unique-commit: all tokens after the first must themselves
        // form a complete accepted cover of the suffix (always true for
        // segment_continuous's output). For true multi-cover agreement we
        // compare first-token ends via a second DP peek.
        let end = full.tokens[0].text.len();
        if !first_cut_unique(raw, end) {
            return None;
        }
        Some(StreamToken {
            text: raw[..end].to_string(),
            result: morph_parse_stream_piece(&raw[..end]),
            explicit_boundary: false,
        })
    }

    fn force_derived_commit(&self) -> Option<StreamToken> {
        let r = segment_continuous(&self.buf);
        r.tokens.into_iter().next()
    }
}

/// True when every complete cover of `raw` (under the same DP + DERIVED filters
/// as `segment_continuous`, before DERIVED resolve) shares first-token end `end`.
fn first_cut_unique(raw: &str, end: usize) -> bool {
    // Reuse segment_continuous's edge/cover construction via a dedicated helper.
    let covers = continuous_covers(raw);
    if covers.is_empty() {
        return false;
    }
    covers.iter().all(|c| !c.is_empty() && c[0].end == end)
}

fn continuous_covers(raw: &str) -> Vec<Vec<PathTok>> {
    let n = raw.len();
    if n == 0 {
        return vec![];
    }
    let mut edges: Vec<Vec<Edge>> = vec![Vec::new(); n];
    for (i, _) in raw.char_indices() {
        let rest = &raw[i..];
        let max = rest.len().min(MAX_WORD_BYTES);
        for len in 1..=max {
            if !rest.is_char_boundary(len) || rest.as_bytes()[len - 1] == b'\'' {
                continue;
            }
            if let Some(parts) = classify_stream_word(&rest[..len]) {
                edges[i].push(Edge {
                    end: i + len,
                    unique: parts.unique,
                    compound_cmavo: parts.compound_cmavo,
                    cmevla: parts.cmevla,
                    brivla: parts.brivla,
                });
            }
        }
    }

    #[derive(Clone)]
    struct Reach {
        path: Vec<PathTok>,
    }
    let mut reaches: Vec<Vec<Reach>> = vec![Vec::new(); n + 1];
    reaches[0].push(Reach { path: vec![] });
    let mut stress_cuts: BTreeSet<usize> = BTreeSet::new();

    for i in 0..n {
        let parent_count = reaches[i].len();
        if parent_count == 0 {
            continue;
        }
        if i > 0 && is_v(raw.as_bytes()[i].to_ascii_lowercase()) {
            continue;
        }
        for edge in &edges[i] {
            for pi in 0..parent_count {
                let left = reaches[i][pi]
                    .path
                    .last()
                    .map(|t| &raw[t.start..t.end])
                    .unwrap_or("");
                if edge.cmevla && !cmevla_allowed_after(left) {
                    continue;
                }
                // §4.8 / §4.9.2: a cmevla must be followed by a pause. Continuous
                // input has none inside it, so a cmevla can only end the input.
                if edge.cmevla && edge.end != n {
                    continue;
                }
                if edge.brivla {
                    if let Some(prev) = reaches[i][pi].path.last() {
                        let prev_text = &raw[prev.start..prev.end];
                        if final_syllable_stressed(prev_text) && !prev.cmevla {
                            stress_cuts.insert(i);
                            continue;
                        }
                    }
                }
                let mut path = reaches[i][pi].path.clone();
                path.push(PathTok {
                    start: i,
                    end: edge.end,
                    unique: edge.unique,
                    compound_cmavo: edge.compound_cmavo,
                    cmevla: edge.cmevla,
                    brivla: edge.brivla,
                });
                if reaches[edge.end].len() < MAX_PATHS_PER_INDEX {
                    reaches[edge.end].push(Reach { path });
                }
            }
        }
    }

    let mut covers: Vec<Vec<PathTok>> = reaches[n].iter().map(|r| r.path.clone()).collect();
    if !stress_cuts.is_empty() {
        covers.retain(|path| !path_crosses_cuts(path, &stress_cuts));
    }
    covers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(r: &StreamResult) -> Vec<&str> {
        r.tokens.iter().map(|t| t.text.as_str()).collect()
    }

    #[test]
    fn orthographic_spaces_and_dots() {
        let r = segment_stream("djan. braun.");
        assert!(r.tokens.len() >= 2, "{r:?}");
        assert!(r.tokens.iter().any(|t| t.text == "djan"));
        assert!(r.tokens.iter().any(|t| t.text == "braun"));
        assert!(
            r.tokens.iter().all(|t| t.result.accepted),
            "expected accepted cmevla with reattached pause: {r:?}"
        );
    }

    #[test]
    fn orthographic_cmavo_sequence() {
        let r = segment_stream(".i.ai.i.ai.o");
        assert!(r.tokens.len() >= 3, "{r:?}");
    }

    #[test]
    fn written_leading_pause_reaches_word_shape() {
        let r = segment_orthographic(".a .y. djan.a");
        assert_eq!(texts(&r), vec!["a", "y", "djan", "a"]);
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
        assert_eq!(r.tokens[0].result.normalized, ".a");
        assert_eq!(r.tokens[1].result.normalized, ".y.");
        assert_eq!(r.tokens[3].result.normalized, ".a");
        let spaced = segment_orthographic("djan. a");
        assert_eq!(spaced.tokens[1].result.normalized, ".a");

        let mut streaming = StreamingSegmenter::with_options(
            StreamMode::Orthographic,
            DEFAULT_MAX_LOOKAHEAD,
            OverflowPolicy::DerivedForceCommit,
        );
        let mut tokens = Vec::new();
        for chunk in [".", "a .", "y. djan.", "a"] {
            if let StreamPush::Tokens(mut emitted) = streaming.push(chunk) {
                tokens.append(&mut emitted);
            }
        }
        tokens.extend(streaming.finish().tokens);
        assert_eq!(
            tokens
                .iter()
                .map(|t| t.result.normalized.as_str())
                .collect::<Vec<_>>(),
            r.tokens
                .iter()
                .map(|t| t.result.normalized.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn unicode_preserves_text_without_slicing_panics() {
        for input in ["klama–do", "mi🙂", "é", "–"] {
            let result = segment_continuous(input);
            assert_eq!(
                result
                    .tokens
                    .iter()
                    .map(|t| t.text.as_str())
                    .collect::<String>(),
                input
            );
            for path in continuous_covers(input) {
                assert!(path
                    .iter()
                    .all(|t| input.is_char_boundary(t.start) && input.is_char_boundary(t.end)));
            }
        }
    }

    #[test]
    fn continuous_lerenanmu() {
        // CLL §4.2 examples use le re nanmu; continuous must recover it.
        let r = segment_stream("lerenanmu");
        assert_eq!(texts(&r), vec!["le", "re", "nanmu"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_diphthong_is_one_cmavo() {
        let r = segment_continuous("au");
        assert_eq!(texts(&r), vec!["au"], "{r:?}");
        assert_eq!(r.tokens[0].result.classes, [MorphClass::Cmavo].into());
        let covers = continuous_covers("au");
        assert!(
            covers.iter().all(|cover| cover.len() == 1),
            "a|u must not be a continuous cover"
        );
        assert_eq!(texts(&segment_orthographic("a.u")), vec!["a", "u"]);
        assert_eq!(texts(&segment_orthographic("a u")), vec!["a", "u"]);
    }

    #[test]
    fn continuous_laplas_breaks_to_la_plas() {
        // §4.8: invalid name laplas. would look like la plas.
        let r = segment_stream("laplas");
        assert_eq!(texts(&r), vec!["la", "plas"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_cy_cy_sequence() {
        // §4.9.6: Cy needs pause unless another Cy follows.
        let r = segment_stream("cydy");
        assert_eq!(texts(&r), vec!["cy", "dy"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_cy_then_brivla() {
        let r = segment_stream("cybroda");
        assert_eq!(texts(&r), vec!["cy", "broda"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn orthographic_c_final_before_v_initial() {
        // §4.9.2 + §4.9.3: written pause after C-final / before V-initial.
        let r = segment_stream("djan.a");
        assert_eq!(texts(&r), vec!["djan", "a"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_djana_prefers_gismu_cover() {
        // Continuous "djana" is uniquely a gismu shape; DERIVED prefers that
        // over cmevla+cmavo "djan|a" (which would require an inferred pause).
        let r = segment_stream("djana");
        assert_eq!(texts(&r), vec!["djana"], "{r:?}");
        assert!(
            r.tokens[0].result.classes.contains(&MorphClass::Gismu),
            "{r:?}"
        );
    }

    #[test]
    fn continuous_records_derived_note() {
        let r = segment_stream("lerenanmu");
        assert!(
            r.ambiguity
                .iter()
                .any(|a| a.contains("DERIVED") || a.contains("covers")),
            "{r:?}"
        );
    }

    #[test]
    fn continuous_marked_final_stress_before_brivla_rejected() {
        // §4.9.5: le RE nanmu with stress on RE demands pause; continuous
        // leREnanmu has none → fused reading rejected (no legal cover).
        let r = segment_stream("leREnanmu");
        assert!(
            r.ambiguity.iter().any(|a| a.contains("§4.9.5")),
            "expected §4.9.5 note: {r:?}"
        );
        assert!(
            !r.tokens.iter().all(|t| t.result.accepted)
                || r.tokens.len() == 1
                || texts(&r) != vec!["le", "RE", "nanmu"],
            "illegal RE|nanmu cover must not win cleanly: {r:?}"
        );
        // Fallback single-token (or non-accepted) — not the clean 3-word accept.
        assert_ne!(texts(&r).as_slice(), &["le", "RE", "nanmu"][..], "{r:?}");
    }

    #[test]
    fn continuous_unmarked_lerenanmu_still_ok() {
        // Unmarked: optional unstressed cmavo is valid; do not invent default
        // first-syllable cmavo stress as mandatory (§3.9 / §4.9.5).
        let r = segment_stream("lerenanmu");
        assert_eq!(texts(&r), vec!["le", "re", "nanmu"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_marked_nonfinal_stress_ok() {
        // Stress on LE (monosyllable) then re (unstressed) then nanmu —
        // LE before re (not brivla) is fine; re unmarked before nanmu is fine.
        let r = segment_stream("LErenanmu");
        assert_eq!(texts(&r), vec!["LE", "re", "nanmu"], "{r:?}");
        assert!(r.tokens.iter().all(|t| t.result.accepted), "{r:?}");
    }

    #[test]
    fn continuous_eu_bridi_final_stress_rejected() {
        let r = segment_stream("e'UBRIdi");
        assert!(
            r.ambiguity
                .iter()
                .any(|a| a.contains("§4.9.5") || a.contains("no complete")),
            "{r:?}"
        );
    }

    #[test]
    fn orthographic_pause_allows_final_stress() {
        let r = segment_stream("leRE.nanmu");
        assert_eq!(texts(&r), vec!["leRE", "nanmu"], "{r:?}");
        // Explicit pause satisfies §4.9.5 for the compound-or-cmavo before brivla.
        assert!(!r.pauses.is_empty(), "{r:?}");
    }

    #[test]
    fn streaming_orthographic_chunks_match_full() {
        let full = "djan. braun. le re nanmu";
        let expected = segment_stream(full);
        let mut s = StreamingSegmenter::with_options(
            StreamMode::Orthographic,
            DEFAULT_MAX_LOOKAHEAD,
            OverflowPolicy::DerivedForceCommit,
        );
        let mut got = Vec::new();
        for ch in full.chars() {
            let mut buf = [0u8; 4];
            let piece = ch.encode_utf8(&mut buf);
            match s.push(piece) {
                StreamPush::Tokens(toks) => got.extend(toks),
                StreamPush::AmbiguousOverflow { .. } => panic!("unexpected overflow"),
            }
        }
        let fin = s.finish();
        got.extend(fin.tokens);
        let got_texts: Vec<_> = got.iter().map(|t| t.text.as_str()).collect();
        let exp_texts: Vec<_> = expected.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(got_texts, exp_texts, "streamed={got:?} full={expected:?}");
    }

    #[test]
    fn streaming_continuous_unique_cuts() {
        // lerenanmu has unique DERIVED cuts le|re|nanmu — chunked push should
        // eventually emit the same tokens as segment_continuous.
        let full = "lerenanmu";
        let expected = segment_continuous(full);
        let mut s = StreamingSegmenter::with_options(
            StreamMode::Continuous,
            DEFAULT_MAX_LOOKAHEAD,
            OverflowPolicy::DerivedForceCommit,
        );
        let mut got = Vec::new();
        for ch in full.chars() {
            let mut buf = [0u8; 4];
            let piece = ch.encode_utf8(&mut buf);
            match s.push(piece) {
                StreamPush::Tokens(toks) => got.extend(toks),
                StreamPush::AmbiguousOverflow { .. } => panic!("unexpected overflow"),
            }
        }
        let fin = s.finish();
        got.extend(fin.tokens);
        let got_texts: Vec<_> = got.iter().map(|t| t.text.as_str()).collect();
        let exp_texts: Vec<_> = expected.tokens.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(got_texts, exp_texts, "streamed={got:?} full={expected:?}");
    }

    #[test]
    fn streaming_orthographic_tiny_chunks() {
        let mut s = StreamingSegmenter::new();
        let mut got = Vec::new();
        for piece in ["mi", " ", "kl", "ama", " ", "lo", " ", "mi", "x", "u"] {
            match s.push(piece) {
                StreamPush::Tokens(toks) => got.extend(toks),
                other => panic!("unexpected {other:?}"),
            }
        }
        let fin = s.finish();
        got.extend(fin.tokens);
        let texts: Vec<_> = got.iter().map(|t| t.text.as_str()).collect();
        assert_eq!(texts, vec!["mi", "klama", "lo", "mixu"]);
        assert!(got.iter().all(|t| t.explicit_boundary));
    }
}
