use camxes_rs::peg::grammar::Peg;
use camxes_rs::peg::parsing::ParseNode;

const CMAXES_GRAMMAR: (&str, &str) = (
    "text",
    "text <- any_word+

any_word <- jbovla

jbovla <- pause_0 / cmevla / (cmavo) / (gismu) / (!gismu !fuhivla !cmavo !(ccv h y onset) lujvo_core) / (fuhivla)

cmevla <- ((&zifcme any_syllable+ &pause) / zifcme)

gismu <- long_rafsi &stress &final_syllable pa_zei_karsna &post_word

fuhivla_head <- !rafsi_string !cmavo !(!rafsi_string zunsna rafsi_string) !h &onset unstressed_syllable*

fuhivla_trim <- fuhivla_head slaka &stress consonantal_syllable*

generic_fuhivla <- fuhivla_trim final_syllable

fuhivla <- &generic_fuhivla (ccv / cvv / cvc) (r / n / l) (unstressed_syllable* slaka &stress consonantal_syllable* final_syllable) / &generic_fuhivla (long_rafsi) (r / n / l) (unstressed_syllable* slaka &stress consonantal_syllable* final_syllable / final_syllable) / generic_fuhivla

cmavo <- !cmevla !(cvc !stress y h? lujvo_core / cvc &stress y short_final_rafsi) (!h !(zunsna zunsna+) onset (nucleus h)* nucleus / y+) &post_word

lujvo_core <- ((hy_rafsi / fuhivla_rafsi / y_rafsi / !any_fuhivla_rafsi y_less_rafsi !any_fuhivla_rafsi)*) ((fuhivla / gismu_cvv_final_rafsi) / (((stressed_hy_rafsi / stressed_fuhivla_rafsi / stressed_y_rafsi / cvc_ccv_cvv &stress)) (short_final_rafsi)))

any_fuhivla_rafsi <- fuhivla / fuhivla_rafsi / stressed_fuhivla_rafsi

rafsi_string <- y_less_rafsi* (gismu_cvv_final_rafsi / cvc_ccv_cvv &stress !y short_final_rafsi / y_rafsi / stressed_y_rafsi / (cvc_ccv_cvv &stress !y)? initial_pair y / hy_rafsi / stressed_hy_rafsi)

zifcme <- !h (nucleus / glaide / h / zunsna !pause)* zunsna &pause

stressed_fuhivla_rafsi <- (fuhivla_trim) (h y) / (fuhivla_trim onset) (y)

fuhivla_rafsi <- &unstressed_syllable (fuhivla_head) (h y) / (fuhivla_head onset) (y h?)

stressed_y_rafsi <- (long_rafsi / cvc) &stress (y)

y_rafsi <- ((long_rafsi / cvc) !stress) (y h?)

y_less_rafsi <- !y_rafsi !stressed_y_rafsi !hy_rafsi !stressed_hy_rafsi cvc_ccv_cvv !stress !y !h

stressed_hy_rafsi <- ((long_rafsi pa_zei_karsna / cvc_ccv_cvv)) &stress (h y)

hy_rafsi <- ((long_rafsi pa_zei_karsna / cvc_ccv_cvv)) (!stress h y h?)

cvc <- cv zunsna

cvc_ccv <- cvc / ccv

ccv <- initial_pair pa_zei_karsna

cvv <- zunsna re_zei_karsna

cvc_ccv_cvv <- cvc_ccv / cvvr

cvvr <- (zunsna pa_zei_karsna !stress h pa_zei_karsna / cvv) ((r &zunsna / n &r)?)

gismu_cvv_final_rafsi <- gismu / cv &stress h &final_syllable pa_zei_karsna &post_word

short_final_rafsi <- &final_syllable (zunsna re_zei_karsna / ccv) &post_word

unstressed_syllable <- slaka !stress / consonantal_syllable

long_rafsi <- cvc_ccv zunsna

cv <- zunsna pa_zei_karsna

final_syllable <- onset !y nucleus !cmevla &post_word

stress <- (zunsna / glaide)* h? y? slaka pause

any_syllable <- onset nucleus coda? / consonantal_syllable

slaka <- onset !y nucleus coda?

consonantal_syllable <- zunsna &syllabic coda

coda <- !any_syllable zunsna &any_syllable / syllabic? zunsna? &pause

onset <- (h / glaide / affricate / (cs !x / jz !(n / l / r))? (pfbgvkx / (t / d / n !r) !l / m)? (l / r)?) !zunsna !glaide

nucleus <- pa_zei_karsna / re_zei_karsna / y !nucleus

glaide <- (ii / w) &nucleus

re_zei_karsna <- ([a] w !u / [aeo] ii !i) !nucleus

pa_zei_karsna <- [aeiou] !nucleus

i <- [i]

u <- [u]

y <- [y] !(!y nucleus)

ii <- [i]

w <- [uw]

initial_pair <- &onset zunsna zunsna !zunsna

affricate <- t cs / d jz

zunsna <- pfbgvkx / d / jz / cs / t / syllabic

syllabic <- l / m / n / r

l <- [l]

m <- [m]

n <- [n] !affricate

r <- [r]

pfbgvkx <- [pfbgvkx]

d <- [d]

jz <- [jz]

cs <- [cs]

x <- [x]

t <- [t]

h <- [,'] &nucleus

post_word <- pause / !nucleus jbovla

pause <- pause_0 / !.

pause_0 <- [ ,.]+
");

/// Collects lujvo segment strings from the parse tree for readable output.
/// Expands stressed_fuhivla_rafsi into "rafsi + 'y" so you see e.g. "arba + 'y + mla".
fn lujvo_segments(input: &str, nodes: &[ParseNode]) -> Option<Vec<String>> {
    fn find_lujvo_core(nodes: &[ParseNode]) -> Option<&ParseNode> {
        for node in nodes {
            match node {
                ParseNode::NonTerminal { name, children, .. } => {
                    if name == "lujvo_core" {
                        return Some(node);
                    }
                    if let Some(found) = find_lujvo_core(children) {
                        return Some(found);
                    }
                }
                ParseNode::Terminal { .. } => {}
            }
        }
        None
    }

    let lujvo_core = find_lujvo_core(nodes)?;
    let ParseNode::NonTerminal { children, .. } = lujvo_core else {
        return None;
    };

    let mut parts: Vec<(usize, String)> = Vec::new();

    for node in children {
        match node {
            ParseNode::NonTerminal { name, span, children: sub } => {
                let (s, e) = (span.0, span.1);
                if s >= e || e > input.len() {
                    continue;
                }
                let text = input[s..e].to_string();
                if name == "stressed_fuhivla_rafsi" {
                    // Split into rafsi + 'y or rafsi + y (e.g. arba + 'y, or arb + y for arby)
                    // Grammar: (fuhivla_trim)(h y) | (fuhivla_trim onset)(y) — include onset in rafsi so "arb" not "ar"
                    let mut rafsi_end = s;
                    let mut hy_start = e;
                    for n in sub {
                        if let ParseNode::NonTerminal { name: nname, span: nspan, .. } = n {
                            let (ns, ne) = (nspan.0, nspan.1);
                            if nname == "fuhivla_trim" {
                                rafsi_end = ne;
                            }
                            if nname == "onset" && ne <= e {
                                // (fuhivla_trim onset)(y) form: include onset in rafsi (e.g. arb + y)
                                rafsi_end = rafsi_end.max(ne);
                            }
                            if nname == "h" || nname == "y" {
                                if ns < hy_start {
                                    hy_start = ns;
                                }
                            }
                        }
                    }
                    if rafsi_end > s {
                        parts.push((s, input[s..rafsi_end].to_string()));
                    }
                    if hy_start < e {
                        parts.push((hy_start, input[hy_start..e].to_string()));
                    }
                } else if name == "stressed_hy_rafsi" || name == "stressed_y_rafsi" {
                    // Split into rafsi + 'y or rafsi + y (e.g. zerba + 'y)
                    let mut rafsi_end = s;
                    let mut hy_start = e;
                    for n in sub {
                        if let ParseNode::NonTerminal { name: nname, span: nspan, .. } = n {
                            let (ns, ne) = (nspan.0, nspan.1);
                            if nname != "h" && nname != "y" {
                                if ne > rafsi_end {
                                    rafsi_end = ne;
                                }
                            }
                            if nname == "h" || nname == "y" {
                                if ns < hy_start {
                                    hy_start = ns;
                                }
                            }
                        }
                    }
                    if rafsi_end > s {
                        parts.push((s, input[s..rafsi_end].to_string()));
                    }
                    if hy_start < e {
                        parts.push((hy_start, input[hy_start..e].to_string()));
                    }
                } else {
                    parts.push((s, text));
                }
            }
            ParseNode::Terminal { .. } => {}
        }
    }

    parts.sort_by_key(|(start, _)| *start);
    Some(parts.into_iter().map(|(_, s)| s).collect())
}

/// Returns the span (start, end) of the first any_word in the parse tree.
fn first_word_span(nodes: &[ParseNode]) -> Option<(usize, usize)> {
    for node in nodes {
        match node {
            ParseNode::NonTerminal { name, span, children } => {
                if name == "any_word" {
                    return Some((span.0, span.1));
                }
                if let Some(sp) = first_word_span(children) {
                    return Some(sp);
                }
            }
            ParseNode::Terminal { .. } => {}
        }
    }
    None
}

/// Segments for the first word: lujvo split (e.g. arb + y + mla) or single word (e.g. klama).
fn get_segments(input: &str, nodes: &[ParseNode]) -> Option<Vec<String>> {
    if let Some(segments) = lujvo_segments(input, nodes) {
        return Some(segments);
    }
    let (s, e) = first_word_span(nodes)?;
    if e <= input.len() && s < e {
        return Some(vec![input[s..e].to_string()]);
    }
    None
}

fn main() {
    use std::env;
    use std::fs;
    use std::path::Path;

    env_logger::builder().init();

    let (start, grammar) = CMAXES_GRAMMAR;
    let p = Peg::new(start, grammar).unwrap();

    // TSV path: first arg, or default examples/lujvo_tests.tsv next to Cargo.toml
    let tsv_path = env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples")
                .join("lujvo_tests.tsv")
        });

    let tsv_content = match fs::read_to_string(&tsv_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", tsv_path.display(), e);
            std::process::exit(1);
        }
    };

    let mut passed = 0usize;
    let mut failed = 0usize;

    for (line_no, line) in tsv_content.lines().enumerate() {
        let line_no = line_no + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.splitn(2, '\t');
        let lujvo = match cols.next() {
            Some(s) => s.trim(),
            None => continue,
        };
        let expected_str = match cols.next() {
            Some(s) => s.trim(),
            None => {
                eprintln!("{}:{}: missing expected column", tsv_path.display(), line_no);
                failed += 1;
                continue;
            }
        };

        if lujvo == "lujvo" && expected_str == "expected" {
            continue; // header
        }

        let expected: Vec<String> = expected_str.split('+').map(|s| s.trim().to_string()).collect();

        // Parse with a trailing space so text <- any_word+ can end with pause
        let input = format!("{} ", lujvo);
        let parse_result = p.parse(&input);

        match &parse_result.2 {
            Ok(nodes) => {
                let actual = match get_segments(&input, nodes) {
                    Some(segments) => segments,
                    None => {
                        println!("FAIL {}:{}  {}  (no segments)", tsv_path.display(), line_no, lujvo);
                        failed += 1;
                        continue;
                    }
                };

                if actual == expected {
                    println!("PASS {}:{}  {}  =>  {}", tsv_path.display(), line_no, lujvo, expected_str);
                    passed += 1;
                } else {
                    println!(
                        "FAIL {}:{}  {}  expected [{}]  got [{}]",
                        tsv_path.display(),
                        line_no,
                        lujvo,
                        expected.join(" + "),
                        actual.join(" + ")
                    );
                    failed += 1;
                }
            }
            Err(e) => {
                println!("FAIL {}:{}  {}  parse error: {}", tsv_path.display(), line_no, lujvo, e);
                failed += 1;
            }
        }
    }

    println!("\n--- {} passed, {} failed ---", passed, failed);
    if failed > 0 {
        std::process::exit(1);
    }
}
