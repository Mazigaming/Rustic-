use std::env;
use std::fs;
use std::path::Path;

use rustic_compiler::{tokenize, Parser};
use rustic_interp::{eval_expr, eval_item, Env};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rustic-interp <file.rcpp>");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let source = fs::read_to_string(path).expect("Failed to read file");

    let tokens = tokenize(&source);
    let mut parser = Parser::new(tokens);
    let items = parser.parse().expect("Parse error");

    let mut env: Env = Vec::new();
    env.push(std::collections::HashMap::new());

    for item in &items {
        match item {
            rustic_compiler::Item::Fn { name, body, .. } if name == "main" => {
                match eval_expr(body, &mut env) {
                    Ok(v) => println!("{:?}", v),
                    Err(e) => eprintln!("Runtime error: {}", e),
                }
            }
            _ => {
                eval_item(item, &mut env).expect("Runtime error");
            }
        }
    }
}
