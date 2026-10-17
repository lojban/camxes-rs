//! Whole-text syntax: both engines must accept basic Lojban constructions
//! and reject clearly ungrammatical texts. Tree shapes are not compared.

use camxes_rs::morphology::Engine;
use camxes_rs::syntax::accepts;

/// Expected acceptance per engine.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Expect {
    /// Both engines accept.
    Ok,
    /// Both engines reject.
    Bad,
    /// Only the Earley engine accepts: `zoi` needs delimiter back-references
    /// the PEG engine lacks (see `camxes_rs::syntax`).
    EarleyOnly,
}
use Expect::*;

const CASES: &[(&str, &str, Expect)] = &[
    ("simple bridi", "mi klama", Ok),
    ("simple bridi, 3 sumti", "mi klama lo zarci lo zdani", Ok),
    ("LE ... KU, cu", "lo gerku ku cu batci lo mlatu ku", Ok),
    ("le / la with cmevla", "le prenu cu tavla la .alis.", Ok),
    ("tanru selbri", "mi sutra klama", Ok),
    ("place tags FA", "fe lo zarci fa mi klama", Ok),
    ("BAI tag", "mi klama bai lo nu do cusku", Ok),
    ("FIhO tag", "mi klama fi'o tutci lo karce", Ok),
    ("tense PU", "mi pu klama", Ok),
    ("tense compound ZAhO CAhA", "mi ba'o ca'o klama", Ok),
    ("tense FAhA VA", "mi va klama", Ok),
    ("NU abstraction", "mi nelci lo nu do klama kei", Ok),
    ("DU'U abstraction", "mi djuno lo du'u do klama", Ok),
    ("relative clause POI", "lo prenu poi klama cu tavla", Ok),
    (
        "relative clause NOI with ku'o",
        "la .djan. noi pendo mi ku'o cu klama",
        Ok,
    ),
    ("GOI", "lo gerku goi ko'a cu batci", Ok),
    ("JE in tanru", "mi melbi je barda", Ok),
    ("sumti connective .e", "mi .e do klama", Ok),
    ("GIhA bridi-tail", "mi klama gi'e tavla", Ok),
    ("GA forethought", "ga mi klama gi do klama", Ok),
    (".i sentences", "mi klama .i do tavla", Ok),
    (".i with ijek", "mi klama .ije do tavla", Ok),
    ("lu ... li'u quote", "lu mi klama li'u jufra", Ok),
    ("zo quote", "zo klama valsi", Ok),
    ("zoi quote", "zoi gy. hello .gy glico", EarleyOnly),
    ("number PA", "mi viska ci gerku", Ok),
    ("number with MOI", "mi pamoi", Ok),
    ("MEX li ... du", "li re su'i re du li vo", Ok),
    ("vocative COI", "coi la .djan.", Ok),
    ("vocative DOI + ko", "doi .alis. ko klama", Ok),
    ("lujvo in sentence", "mi klamrkalci", Ok),
    ("lujvo, CVV hyphen", "do zbasaigau", Ok),
    ("fu'ivla in sentence", "mi citka lo spageti", Ok),
    ("cmevla in sentence", "mi tavla la .alis.", Ok),
    ("UI attitudinal, start", ".ui mi klama", Ok),
    ("UI + NAI", "mi klama .uinai", Ok),
    ("UI + CAI", "do pendo mi sai", Ok),
    ("NA negation", "mi na klama", Ok),
    ("SE conversion", "lo zarci cu se klama mi", Ok),
    ("question ma", "do klama ma", Ok),
    ("question xu", "xu do klama", Ok),
    ("prenex zo'u", "ro da zo'u da klama", Ok),
    ("ME", "mi me la .djan.", Ok),
    ("ZEI lujvo", "mi klama zei sutra", Ok),
    ("BU letteral", "by. bu", Ok),
    ("SEI discursive", "mi klama sei do djuno", Ok),
    ("TO parenthesis", "mi to do klama toi tavla", Ok),
    // clearly ungrammatical
    ("LE without selbri", "lo cu klama", Bad),
    ("dangling KU", "mi ku ku klama", Bad),
    ("only LE", "le le le", Bad),
    ("cu without sumti", "cu klama", Bad),
    ("unmatched li'u", "mi klama li'u", Bad),
    ("POI without bridi", "mi klama poi", Bad),
    ("GI without GA", "gi mi klama", Bad),
    ("non-Lojban word", "mi klamq", Bad),
];

fn engines() -> &'static [Engine] {
    Engine::available()
}

#[test]
fn basic_grammar_both_engines() {
    let mut failures = Vec::new();
    println!("{:34} {:40} {:>6} {:>6}", "case", "text", "peg", "earley");
    for &(name, text, expect) in CASES {
        let mut row = format!("{name:34} {text:40}");
        for &engine in engines() {
            let got = accepts(text, engine);
            let want = match expect {
                Ok => true,
                Bad => false,
                EarleyOnly => engine != Engine::Peg,
            };
            row.push_str(&format!(" {:>6}", if got { "ok" } else { "reject" }));
            if got != want {
                failures.push(format!(
                    "{} on {name:?} ({text:?}): got {got}, want {want}",
                    engine.name()
                ));
            }
        }
        println!("{row}");
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
