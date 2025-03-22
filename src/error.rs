use crate::token::Token;

pub fn error(line: usize, message: &str, context: &str) {
    eprintln!(
        "[line {}] Error: {}\n{}\n{}^",
        line,
        message,
        context,
        " ".repeat(context.len())
    );
}
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: String) -> Self {
        RuntimeError {
            token: token.clone(),
            message,
        }
    }
}
use std::fmt;

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error")
    }
}
