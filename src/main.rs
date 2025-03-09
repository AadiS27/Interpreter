mod error;
mod token;
mod astprinter;
mod expr;
mod interpreter;
mod parser;

use std::fs;
use std::env;
use token::Tokensizer;
use interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run <filename>");
        return;
    }
    
    let filename = &args[1];
    let source = fs::read_to_string(filename).expect("Failed to read file");
    
    let mut tokenizer = Tokensizer::new(source);
    let tokens = tokenizer.tokenize();
    
    let mut parser = parser::Parser::new(tokens);
    
    match parser.parse() {
        Some(expression) => {
            let ast_output = astprinter::AstPrinter::new().print(&expression);
            println!("AST Output:\n{}", ast_output);

            let interpreter = Interpreter::new();
            let result = interpreter.interpret(&expression);
            println!("{:?}", result);
        }
        None => eprintln!("Parsing failed due to syntax errors."),
    }
}
