use camxes_rs::peg::grammar::Peg;
use camxes_rs::peg::parsing::ParseNode;

use camxes_rs::grammars::LOJBAN_MORPHOLOGY_GRAMMAR as CMAXES_GRAMMAR;

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

        match parse_result.2.as_ref() {
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
