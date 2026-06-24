use std::fs;
use std::path::Path;

use rustic_compiler::{tokenize, Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rcpp <file.rcpp>");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let source = fs::read_to_string(path).expect("Failed to read file");

    let tokens = tokenize(&source);

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(items) => {
            println!("Parsed {} items successfully", items.len());
            for item in items {
                println!("  {:?}", item);
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
