# 🚀 Rust PEG Parser Generator

[![Crates.io](https://img.shields.io/crates/v/camxes-rs)](https://crates.io/crates/camxes-rs)
[![Documentation](https://docs.rs/camxes-rs/badge.svg)](https://docs.rs/camxes-rs)
[![License](https://img.shields.io/crates/l/camxes-rs)](LICENSE)

A lightning-fast Parsing Expression Grammar (PEG) parser generator implemented in Rust. This tool helps you create parsers from grammar definitions with minimal hassle.

## 📚 Documentation

Full API documentation is available on [docs.rs](https://docs.rs/camxes-rs)

## ✨ Features

- **Zero-Copy Parsing**: Efficient parsing without unnecessary string allocations
- **Rich Debugging**: Detailed logging for grammar validation and parsing process
- **Flexible Grammar Syntax**: Supports all standard PEG operators and extensions
- **Error Recovery**: Robust error handling with detailed diagnostic messages
- **Thread-Safe**: Designed for concurrent use with thread-safe grammar structures

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
camxes-rs = "0.1.1"
```

## 🚦 Quick Start

### Local testing

You must have latest Rust installed.

Go to the project directory and run `cargo run --example cmaxes-test`.
It will run the PEG grammar specified in `src/examples/cmaxes-test.rs` file against a sample input.


<!-- Here's a simple example that parses a basic grammar:

```rust
use camxes_rs::peg::grammar::PEG;

fn main() {
    // Define your grammar
    let grammar = (
        "expression",  // Start rule
        r#"
        expression <- term (('+' / '-') term)*
        term <- factor (('*' / '/') factor)*
        factor <- number / '(' expression ')'
        number <- [0-9]+ 
        "#
    );

    // Create parser
    let parser = PEG::new(grammar.0, grammar.1).unwrap();
    
    // Parse input
    let result = parser.parse("2+3*4");
    println!("{:#?}", result);
}
``` -->

## 🔧 Grammar Syntax

The grammar supports these PEG operators:

| Operator | Description | Example |
|----------|-------------|---------|
| `<-`     | Definition | `rule <- expression` |
| `/`      | Ordered choice | `a / b` |
| `*`      | Zero or more | `[0-9]*` |
| `+`      | One or more | `[a-z]+` |
| `?`      | Optional | `[A-Z]?` |
| `&`      | And-predicate | `&[a-z]` |
| `!`      | Not-predicate | `![0-9]` |
| `()`     | Grouping | `(a / b)` |
| `[]`     | Character range / class | `[abd]` and `[a-zA-Z]` |
| `.`      | Any character | `.` |

## 🔍 Debugging

Enable debug logging to see detailed parsing information:

```rust
// Initialize logging (using env_logger)
env_logger::builder()
    .filter_level(log::LevelFilter::Debug)
    .init();
```

Or set the environment variable:
```bash
RUST_LOG=debug cargo run
```

## 📚 Examples

### Simple Calculator Grammar
```rust
let calc_grammar = (
    "calc",
    r#"
    calc <- additive
    additive <- multiplicative (('+' / '-') multiplicative)*
    multiplicative <- primary (('*' / '/') primary)*
    primary <- number / '(' additive ')'
    number <- [0-9]+ 
    "#
);
```

### JSON Parser Grammar
```rust
let json_grammar = (
    "json",
    r#"
    json <- spacing value spacing
    value <- object / array / string / number / true / false / null
    object <- '{' spacing (pair (',' pair)*)? '}' 
    pair <- string spacing ':' spacing value
    array <- '[' spacing (value (',' value)*)? ']'
    string <- '"' (!'"' .)* '"'
    number <- '-'? ([0-9] / [1-9][0-9]*) ('.' [0-9]+)? ([eE][-+]?[0-9]+)?
    true <- 'true'
    false <- 'false'
    null <- 'null'
    spacing <- [ \t\n\r]*
    "#
);
```

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Based on Bryan Ford's [Parsing Expression Grammars](https://bford.info/pub/lang/peg.pdf) paper
- Inspired by various PEG implementations in the Rust ecosystem

---

## 🌐 Loglan WASM Parser

A web-based interface for the Loglan parser is available in the `loglan-wasm-app` directory. See the [Loglan WASM App README](loglan-wasm-app/README.md) for instructions on how to build and run it.

---

Built with ❤️ for the Rust community
