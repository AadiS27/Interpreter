mod error;
mod token;
mod astprinter;
mod expr;
use std::fs;
use std::env;
use token::Tokensizer;
mod parser;


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
    
   
   

        let mut parser = parser::Parser::new(_tokens);
        match parser.parse() {
            Some(expression) => println!("{}", astprinter::AstPrinter::new().print(&expression)),
            None => eprintln!("Parsing failed due to syntax errors."),
        }
    
}