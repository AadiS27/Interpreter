mod error;
mod token;

use std::env;
use std::fs;
use token::Tokensizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run <filename>");
        return;
    }
    
    let filename = &args[1];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    
    let mut tokenizer = Tokensizer::new(source);
    let _tokens = tokenizer.tokenize();
    
   
        tokenizer.print_tokens();
    
}