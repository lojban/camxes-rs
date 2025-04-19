use camxes_rs::{grammars::LOGLAN_GRAMMAR, peg::grammar::Peg};
use std::time::Instant;

fn main() {
    use env_logger;
    env_logger::builder().init();

    let (start, grammar) = LOGLAN_GRAMMAR;
    let p = Peg::new(start, grammar).unwrap();
    println!("{:#?}", "go...");
    let input = "mi cluva";
    println!("Parsing input: '{}'", input);

    let start_time_debug = Instant::now();
    let result_debug = p.parse(input);
    let duration_debug = start_time_debug.elapsed();
    println!("\n--- Debug Output ---");
    println!("{:#?}", result_debug);
    println!("Debug parsing took: {:?}", duration_debug);

    let start_time_json = Instant::now();
    match p.parse_to_json(input) {
        Ok(json_output) => {
            let duration_json = start_time_json.elapsed();
            println!("\n--- JSON Output ---");
            println!("{}", json_output);
            println!("JSON parsing & serialization took: {:?}", duration_json);
        }
        Err(e) => {
            eprintln!("\nError generating JSON output: {}", e);
        }
    }
}
