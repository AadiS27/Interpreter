use crate::token::Token;
#[warn(dead_code)]

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
    pub _token: Token,
    pub _message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, _message: String) -> Self {
        RuntimeError {
            _token: token.clone(),
            _message,
        }
    }
}
use std::fmt;

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Runtime error")
    }
}
