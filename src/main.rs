mod error;
mod token;
mod astprinter;
mod expr;
mod interpreter;
mod parser;
mod stmt;
mod environment;

use std::env;
use std::fs;
use token::Tokensizer;
use interpreter::Interpreter;

use axum::{
    body::Bytes,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

async fn run_handler(bytes: Bytes) -> impl IntoResponse {
    let code = String::from_utf8(bytes.to_vec()).unwrap();
    let result = run_code(&code);
    (StatusCode::OK, result)
}



fn run_code(source: &str) -> String {
    // Disallow "scan" keyword
    if source.contains("scan") {
        return "Error: Usage of 'scan' keyword is not allowed.".to_string();
    }

    // Disallow numeric values > 148
    let number_check = regex::Regex::new(r"\b\d+\b").unwrap();
    for cap in number_check.captures_iter(source) {
        if let Ok(n) = cap[0].parse::<u32>() {
            if n > 148 {
                return format!("Error: Numeric value '{}' exceeds the limit of 148.", n);
            }
        }
    }

    let mut tokenizer = Tokensizer::new(source.to_string());
    let tokens = tokenizer.tokenize();
    let mut parser = parser::Parser::new(tokens);

    match parser.parse() {
        Some(statements) => {
            let mut interpreter = Interpreter::new();
            let result = interpreter.interpret(&statements);
            result
        }
        None => "Parsing failed due to syntax errors.".to_string(),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Note: binary name is args[0], first argument is args[1]
    if args.len() >= 2 && args[1] == "server" {
        println!("Aoi interpreter server running on http://localhost:8080");

        let app = Router::new()
            .route("/", axum::routing::get(|| async { "Aoi interpreter server is running" }))
            .route("/run", post(run_handler));

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    } else if args.len() >= 2 {
        // CLI mode
        let filename = &args[1];
        let source = fs::read_to_string(filename).expect("Failed to read file");

        let result = run_code(&source);
        println!("{}", result);
    } else {
        eprintln!("Usage:");
        eprintln!("  ./server <filename>       # CLI mode");
        eprintln!("  ./server server           # Start web server");
    }
}

