//! No public entry point may panic, whatever the input or grammar text:
//! arbitrary / adversarial strings (non-ASCII, emoji, control characters,
//! empty, long, random code points, malformed grammars) must give `Ok` or
//! `Err`. A small deterministic generator replaces a property-testing crate.

#[cfg(feature = "earley")]
use camxes_rs::earley::EarleyGrammar;
use camxes_rs::grammars::LOJBAN_MORPHOLOGY_GRAMMAR;
use camxes_rs::morphology::{self, Engine};
#[cfg(feature = "peg")]
use camxes_rs::peg::grammar::Peg;
use camxes_rs::syntax;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// xorshift64* — deterministic, dependency-free.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

const POOL: &[char] = &[
    'a',
    'e',
    'i',
    'o',
    'u',
    'y',
    'b',
    'c',
    'd',
    'g',
    'k',
    'l',
    'm',
    'n',
    'r',
    's',
    't',
    'z',
    'x',
    'j',
    'f',
    'v',
    'p',
    'h',
    'A',
    'U',
    'Y',
    '\'',
    ',',
    '.',
    ' ',
    ' ',
    ' ',
    '\t',
    '\n',
    '\r',
    '\0',
    '\u{7}',
    '\u{1b}',
    '0',
    '9',
    '#',
    ':',
    '<',
    '-',
    '/',
    '(',
    ')',
    '[',
    ']',
    '*',
    '+',
    '?',
    '!',
    '&',
    '"',
    '\\',
    '{',
    '}',
    'п',
    'у',
    'ж',
    'é',
    'ß',
    'ʼ',
    '’',
    '中',
    '😀',
    '\u{200b}',
    '\u{feff}',
    '\u{301}',
    '\u{10ffff}',
];

fn random_text(rng: &mut Rng, max_len: usize) -> String {
    let len = rng.below(max_len + 1);
    (0..len)
        .map(|_| {
            if rng.below(8) == 0 {
                // Any valid code point.
                loop {
                    if let Some(c) = char::from_u32((rng.next() % 0x11_0000) as u32) {
                        break c;
                    }
                }
            } else {
                POOL[rng.below(POOL.len())]
            }
        })
        .collect()
}

/// Lojban-looking text with occasional corruption.
fn lojbanish(rng: &mut Rng, words: usize) -> String {
    const WORDS: &[&str] = &[
        "mi", "do", "klama", "lo", "le", "cu", "zarci", "zoi", "gy.", ".gy", "la'o", "zo", "lu",
        "li'u", "lo'u", "le'u", "si", "sa", "su", "bu", "zei", ".i", "kuarka", "ku", "kei", "nu",
        "pa", "re", "fa'o", "ba'e", "ui", "nai", "сlama", "😀", "la", ".djan.", "y", "ga", "gi",
    ];
    let mut out = String::new();
    for _ in 0..words {
        out.push_str(WORDS[rng.below(WORDS.len())]);
        if rng.below(10) == 0 {
            out.push(POOL[rng.below(POOL.len())]);
        }
        out.push(' ');
    }
    out
}

fn adversarial_inputs() -> Vec<String> {
    let mut v: Vec<String> = [
        "",
        " ",
        ".",
        "..",
        "'",
        ",",
        "\0",
        "\u{feff}",
        "😀",
        "п",
        "mi п",
        "mi klama lу nu",
        "zoi",
        "zoi gy.",
        "zoi gy. п .gy",
        "zoi 😀. x .😀",
        "la'o",
        "zo",
        "lo'u",
        "le'u",
        "si si si",
        "y'y'y'",
        "ńćź",
        "a\u{301}",
        "\u{10ffff}",
        "mi\u{0}klama",
        "\r\n\t",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    v.push("a".repeat(5000));
    v.push("'".repeat(300));
    v.push("п".repeat(300));
    v.push("mi klama ".repeat(60));
    v.push("zoi gy. ".repeat(40));
    // Deep nesting.
    v.push(format!("{}broda", "lo nu ".repeat(300)));
    v.push(format!("{}mi", "lu ".repeat(300)));
    v.push("mi klama le zarci .i ".repeat(150));
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..120 {
        v.push(random_text(&mut rng, 40));
    }
    for _ in 0..60 {
        let n = 1 + rng.below(12);
        v.push(lojbanish(&mut rng, n));
    }
    v
}

fn malformed_grammars() -> Vec<String> {
    let mut v: Vec<String> = [
        "",
        " ",
        "s",
        "s <-",
        "<- 'a'",
        "s <- 'a",
        "s <- \"a",
        "s <- [a-",
        "s <- [",
        "s <- (",
        "s <- )",
        "s <- 'п'",
        "s <- [а-я]+",
        "s <- [é-ü] / '😀'",
        "п <- 'a'",
        "s <- 'a' #: PUSH",
        "s <- a\na <- 'x' #: POP-EQ",
        "s <- a b\na <- 'x' #: PUSH\nb <- 'y' #: PEEK-DIFF",
        "s <- s",
        "s <- s 'a' / 'a'",
        "s <- !s",
        "s <- &&&'a'",
        "s <- 'a'**",
        "s <- 'a' / / 'b'",
        "s <- undefined",
        "s <- '\\",
        "s <- [\\]",
        "s <- '\0'",
        "s <- .\n\n\n",
        "\u{feff}s <- 'a'",
        "s <- ('a'?)*",
        "s <- ('a'*)+ 'b'",
        "s <- [z-a]",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    v.push(format!(
        "s <- {}'a'{}",
        "(".repeat(20_000),
        ")".repeat(20_000)
    ));
    v.push(format!("s <- {}'a'", "!".repeat(20_000)));
    v.push(
        (0..3000)
            .map(|i| format!("r{i} <- r{} / 'a'\n", i + 1))
            .collect::<String>()
            + "r3000 <- 'b'",
    );
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for _ in 0..150 {
        v.push(format!("s <- {}", random_text(&mut rng, 20)));
    }
    for _ in 0..50 {
        v.push(random_text(&mut rng, 30));
    }
    v
}

/// Run `f`, recording a panic instead of aborting the test.
fn no_panic(failures: &mut Vec<String>, what: &str, input: &str, f: impl FnOnce()) {
    if catch_unwind(AssertUnwindSafe(f)).is_err() {
        failures.push(format!("{what} panicked on {input:?}"));
    }
}

fn assert_none(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} panics:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn grammar_constructors_never_panic() {
    let mut failures = Vec::new();
    for g in malformed_grammars() {
        for start in ["s", "", "missing", "п"] {
            #[cfg(feature = "peg")]
            no_panic(&mut failures, "Peg::new", &g, || {
                if let Ok(peg) = Peg::new(start, &g) {
                    for input in ["", "a", "п", "ab😀", "x\0y"] {
                        let _ = peg.parse(input);
                        let _ = peg.parse_to_json(input);
                    }
                }
            });
            #[cfg(feature = "earley")]
            no_panic(&mut failures, "EarleyGrammar::new", &g, || {
                if let Ok(earley) = EarleyGrammar::new(start, &g) {
                    for input in ["", "a", "п", "ab😀", "x\0y"] {
                        let _ = earley.parse(input);
                        let _ = earley.parse_complete(input);
                        let _ = earley.analyze_ends(input);
                        let _ = earley.parse_to_json(input);
                    }
                }
            });
        }
    }
    assert_none(failures);
}

#[test]
fn embedded_grammars_never_panic() {
    let (start, grammar) = LOJBAN_MORPHOLOGY_GRAMMAR;
    #[cfg(feature = "peg")]
    let peg = Peg::new(start, grammar).unwrap();
    #[cfg(feature = "earley")]
    let earley = EarleyGrammar::new(start, grammar).unwrap();
    let mut failures = Vec::new();
    for input in adversarial_inputs() {
        #[cfg(feature = "peg")]
        no_panic(&mut failures, "morphology Peg::parse", &input, || {
            let _ = peg.parse(&input);
        });
        #[cfg(feature = "earley")]
        if input.chars().count() <= 400 {
            no_panic(&mut failures, "morphology EarleyGrammar", &input, || {
                let _ = earley.parse(&input);
                let _ = earley.parse_complete(&input);
            });
        }
    }
    assert_none(failures);
}

#[test]
fn lojban_apis_never_panic() {
    let mut failures = Vec::new();
    for input in adversarial_inputs() {
        // The Earley engine is cubic in the worst case: keep its inputs and
        // the whole-text syntax check to moderate lengths.
        let len = input.chars().count();
        for &engine in Engine::available() {
            let earley = format!("{engine:?}") == "Earley";
            if !earley || len <= 400 {
                no_panic(&mut failures, "morphology::analyze", &input, || {
                    let _ = morphology::analyze(&input, engine);
                });
            }
            if len <= 4000 && (!earley || len <= 400) {
                no_panic(&mut failures, "syntax::parse", &input, || {
                    let _ = syntax::parse(&input, engine);
                });
            }
        }
    }
    assert_none(failures);
}

#[cfg(feature = "earley")]
#[test]
fn vendored_cll_never_panics() {
    use camxes_rs::cll;
    let mut failures = Vec::new();
    for input in adversarial_inputs() {
        let s = input.as_str();
        // Word-level analysis is cubic in the word length.
        let short = input.chars().count() <= 300;
        if short {
            no_panic(&mut failures, "cll shapes/constraints", s, || {
                let _ = cll::shapes::recognize(s);
                let _ = cll::shapes::classify(s);
                let _ = cll::shapes::normalize_orthography(s);
                let _ = cll::constraints::morph_parse(s);
                let _ = cll::constraints::morph_parse_with(
                    s,
                    cll::constraints::MorphOptions::default(),
                );
                let _ = cll::pipeline::shape_parse(s);
                let _ = cll::pipeline::shape_forest(s);
                let _ = cll::pipeline::unique_word_class(s);
                let _ = cll::constraints::rafsi_fuhivla::try_parse_experimental_lujvo(s);
                let _ = cll::constraints::rafsi_fuhivla::cultural_stem(s);
                let _ = cll::constraints::lujvo_hyphen::check_lujvo(s);
                let _ = cll::constraints::cmevla_ban::banned_substring(s);
                let _ = cll::constraints::slinkui::failing_cv_prefix(s);
                let _ = cll::constraints::tosmabru::check_partial(s);
                let _ = cll::constraints::stress::check_phrase(s);
                let _ = cll::constraints::pauses::obligatory_pause_between(s, s);
                let _ = cll::tables::classify_rafsi_shape(s);
                let _ = cll::tables::RafsiLexicon::global().segment_lujvo(s);
                let _ = cll::phonology::classify_diphthong(s);
            });
        }
        // Lexing a long run of letters (one huge "word") is slow too.
        if input.chars().count() > 1000 {
            continue;
        }
        no_panic(&mut failures, "cll lex", s, || {
            let _ = cll::lex::stream::segment_stream(s);
            let _ = cll::lex::stream::segment_orthographic(s);
            let _ = cll::lex::stream::segment_continuous(s);
            let _ = cll::lex::stream::morph_parse_stream_piece(s);
        });
        no_panic(&mut failures, "cll grammar lexemes", s, || {
            let _ = cll::grammar::lexeme_stream(s);
            let _ = cll::grammar::split_quote_regions(s);
            let _ = cll::grammar::quote_word_key(s);
            let _ = cll::grammar::terminals_only(s);
        });
        if short {
            no_panic(&mut failures, "cll grammar parse", s, || {
                let _ = cll::grammar::parse_text(s);
                let _ = cll::grammar::parse_as(s, "sumti");
                let _ = cll::grammar::parse_as(s, "no-such-rule");
                let _ = cll::grammar::parse_as_with_tree(s, "text");
            });
        }
    }
    for g in malformed_grammars() {
        no_panic(&mut failures, "cll compile_ebnf", &g, || {
            let _ = cll::grammar::compile_ebnf(&g, "s");
            let _ = cll::grammar::compile_ebnf(&g, "");
        });
    }
    assert_none(failures);
}

/// Deep recursion fails with `RecursionLimit` instead of overflowing the
/// stack, also on a thread with a small stack.
#[test]
fn deep_recursion_is_an_error() {
    let run = || {
        #[cfg(feature = "peg")]
        peg_deep();
        #[cfg(feature = "earley")]
        earley_deep();
    };
    run();
    std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(feature = "peg")]
fn peg_deep() {
    use camxes_rs::peg::parsing::ErrorKind;
    {
        let left = Peg::new("s", "s <- s 'a' / 'a'").unwrap();
        let r = left.parse("aaa");
        assert!(matches!(r.2.as_ref(), Err(e) if matches!(e.error, ErrorKind::RecursionLimit)));
        let right = Peg::new("s", "s <- 'a' s / 'a'").unwrap();
        assert_eq!(right.parse("aaaa").1, 4);
        let r = right.parse(&"a".repeat(200_000));
        assert!(matches!(r.2.as_ref(), Err(e) if matches!(e.error, ErrorKind::RecursionLimit)));
        // The same Peg still works afterwards.
        assert_eq!(right.parse("aa").1, 2);
    }
}

#[cfg(feature = "earley")]
fn earley_deep() {
    use camxes_rs::peg::parsing::ErrorKind;
    {
        let earley = EarleyGrammar::new("s", "s <- '(' s ')' / 'a'").unwrap();
        assert_eq!(earley.parse("((a))").1, 5);
        let deep = format!("{}a{}", "(".repeat(20_000), ")".repeat(20_000));
        let r = earley.parse(&deep);
        assert!(matches!(r.2.as_ref(), Err(e) if matches!(e.error, ErrorKind::RecursionLimit)));
        assert_eq!(earley.analyze_ends(&deep), (None, false));
        assert_eq!(earley.parse("(a)").1, 3);
    }
}
