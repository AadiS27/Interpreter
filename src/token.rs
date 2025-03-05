use std::fmt::Display;
use crate::error;
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum TokenLiteral {
    String(String),
    Number(f64),
    Identifier(String),
    Null,
}
impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::String(s) => write!(f, "{}", s),
            TokenLiteral::Number(n) => write!(f, "{}", n),
            TokenLiteral::Identifier(s) => write!(f, "{}", s),
            TokenLiteral::Null => write!(f, "null"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: TokenLiteral,
    line: usize,
}
impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: TokenLiteral, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {}", self.token_type, self.lexeme, self.literal)
    }
}
#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    // One or two character tokens.
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,
    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
}
pub struct Tokensizer {
    src: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}
impl Tokensizer {
    pub fn new(src: String) -> Self {
        Self {
            src,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }
    fn add_token(&mut self, token_type: TokenType, literal: TokenLiteral) {
        let text = self.src[self.start..self.current].to_string();
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }
    fn advance(&mut self) -> char {
        self.current += 1;
        self.src.chars().nth(self.current - 1).unwrap()
    }
    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LEFT_PAREN, TokenLiteral::Null),
            ')' => self.add_token(TokenType::RIGHT_PAREN, TokenLiteral::Null),
            '{' => self.add_token(TokenType::LEFT_BRACE, TokenLiteral::Null),
            '}' => self.add_token(TokenType::RIGHT_BRACE, TokenLiteral::Null),
            ',' => self.add_token(TokenType::COMMA, TokenLiteral::Null),
            '.' => self.add_token(TokenType::DOT, TokenLiteral::Null),
            '-' => self.add_token(TokenType::MINUS, TokenLiteral::Null),
            '+' => self.add_token(TokenType::PLUS, TokenLiteral::Null),
            ';' => self.add_token(TokenType::SEMICOLON, TokenLiteral::Null),
            '*' => self.add_token(TokenType::STAR, TokenLiteral::Null),
            _ => {
      
            }
        }
    }
    pub fn tokenize(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            "".into(),
            TokenLiteral::Null,
            self.line,
        ));
        self.tokens.clone()
    }
    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{}", token);
        }
    }
}