use crate::error;
use std::fmt::Display;
#[derive(Debug, Clone, PartialEq)] //	Allows println!("{:?}", obj); for debugging.  Allows obj.clone(); for copying data.
#[allow(dead_code)] //Prevents warnings for unused code.
pub enum TokenLiteral {
    String(String),
    Number(f64),
    Identifier(String),
    Boolean(bool),
    Null,
}
impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenLiteral::String(s) => write!(f, "{}", s),
            TokenLiteral::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{:.1}", n) // If integer, show as 12.0
                } else {
                    write!(f, "{}", n) // If decimal, show normally (e.g., 12.34)
                }
            }
            TokenLiteral::Boolean(b) => write!(f, "{}", b),
            TokenLiteral::Identifier(s) => write!(f, "{}", s),
            TokenLiteral::Null => write!(f, "null"),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<TokenLiteral>,
}
impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: TokenLiteral) -> Self {
        Self {
            token_type,
            lexeme,
            literal: Some(literal),
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}
#[allow(dead_code, non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    RIGHT_BRACKET,
    LEFT_BRACKET,
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
    SCAN,
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
        self.tokens.push(Token::new(token_type, text, literal));
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.src.chars().nth(self.current - 1).unwrap()
    }

    fn peek(&self) -> Option<char> {
        self.src.chars().nth(self.current)
    }

    //this function is used to scan the string
    fn string(&mut self) {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error::error(self.line, "Unterminated string", "");
            return;
        }
        self.advance();

        // Extract the string value without the surrounding quotes
        let value = self.src[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenType::STRING, TokenLiteral::String(value));
    }

    fn isdigit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn number(&mut self) {
        while self.peek().map_or(false, Self::isdigit) {
            self.advance();
        }

        // Look for a fractional part
        let mut _has_fraction = false;
        if self.peek() == Some('.') {
            // Ensure the next character is a digit before consuming the dot
            if self.peek_next().map_or(false, Self::isdigit) {
                _has_fraction = true;
                self.advance(); // Consume the '.'

                while self.peek().map_or(false, Self::isdigit) {
                    self.advance();
                }
            }
        }

        // Convert the lexeme to a floating-point number

        // If no fractional part exists, ensure .0 is appended
        // Convert the lexeme to a floating-point number
        let lexeme = &self.src[self.start..self.current];
        let float_lexeme = if lexeme.contains('.') {
            lexeme.to_string()
        } else {
            format!("{}.0", lexeme) // Append .0 if it's an integer
        };

        if let Ok(value) = float_lexeme.parse::<f64>() {
            self.add_token(TokenType::NUMBER, TokenLiteral::Number(value));
        } else {
            error::error(self.line, &format!("Invalid number format: {}", lexeme), "");
        }
    }

    fn peek_next(&self) -> Option<char> {
        self.src.chars().nth(self.current + 1)
    }

    fn isalpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn identifier(&mut self) {
        while self.peek().map_or(false, Self::isalpha) {
            self.advance();
        }

        let text = &self.src[self.start..self.current];
        let token_type = match text {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "for" => TokenType::FOR,
            "fun" => TokenType::FUN,
            "if" => TokenType::IF,
            "nil" => TokenType::NIL,
            "or" => TokenType::OR,
            "write" => TokenType::PRINT,
            "return" => TokenType::RETURN,
            "super" => TokenType::SUPER,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "var" => TokenType::VAR,
            "while" => TokenType::WHILE,
            "scan" => TokenType::SCAN,
            _ => TokenType::IDENTIFIER,
        };

        self.add_token(token_type, TokenLiteral::Null);
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '[' => self.add_token(TokenType::LEFT_BRACKET, TokenLiteral::Null),
            ']' => self.add_token(TokenType::RIGHT_BRACKET, TokenLiteral::Null),
            '(' => self.add_token(TokenType::LEFT_PAREN, TokenLiteral::Null),
            ')' => self.add_token(TokenType::RIGHT_PAREN, TokenLiteral::Null),
            '{' => self.add_token(TokenType::LEFT_BRACE, TokenLiteral::Null),
            '}' => self.add_token(TokenType::RIGHT_BRACE, TokenLiteral::Null),
            ',' => self.add_token(TokenType::COMMA, TokenLiteral::Null),
            '.' => {
                if self.peek().map_or(false, |c| c.is_digit(10)) {
                    self.advance(); // Consume '.'
                    while self.peek().map_or(false, |c| c.is_digit(10)) {
                        self.advance();
                    }

                    let value: f64 = self.src[self.start..self.current].parse().unwrap();
                    self.add_token(TokenType::NUMBER, TokenLiteral::Number(value));
                } else {
                    self.add_token(TokenType::DOT, TokenLiteral::Null);
                }
            }

            '-' => self.add_token(TokenType::MINUS, TokenLiteral::Null),
            '+' => self.add_token(TokenType::PLUS, TokenLiteral::Null),
            ';' => self.add_token(TokenType::SEMICOLON, TokenLiteral::Null),
            '*' => self.add_token(TokenType::STAR, TokenLiteral::Null),
            '=' => {
                if self.src.chars().nth(self.current).unwrap() == '=' {
                    self.current += 1;
                    self.add_token(TokenType::EQUAL_EQUAL, TokenLiteral::Null);
                } else {
                    self.add_token(TokenType::EQUAL, TokenLiteral::Null);
                }
            }
            '!' => {
                if self.src.chars().nth(self.current).unwrap() == '=' {
                    //check the next one if it is equal then add the token
                    self.current += 1;
                    self.add_token(TokenType::BANG_EQUAL, TokenLiteral::Null);
                } else {
                    //else add Bang token
                    self.add_token(TokenType::BANG, TokenLiteral::Null);
                }
            }
            '<' => {
                if self.src.chars().nth(self.current) == Some('=') {
                    // ✅ Safe check
                    self.current += 1;
                    self.add_token(TokenType::LESS_EQUAL, TokenLiteral::Null);
                } else {
                    self.add_token(TokenType::LESS, TokenLiteral::Null);
                }
            }

            '>' => {
                if self.src.chars().nth(self.current) == Some('=') {
                    //check the next one if it is equal then add the token
                    self.current += 1;
                    self.add_token(TokenType::GREATER_EQUAL, TokenLiteral::Null);
                } else {
                    //else add Greater token
                    self.add_token(TokenType::GREATER, TokenLiteral::Null);
                }
            }
            '/' => {
                if self.src.chars().nth(self.current) == Some('/') {
                    //check the next one if it is equal then add the token used some because it returns an option and we are getting initially none as unwrap cant be none
                    while self.src.chars().nth(self.current) != Some('\n') && !self.is_at_end() {
                        self.current += 1;
                    }
                } else {
                    self.add_token(TokenType::SLASH, TokenLiteral::Null);
                }
            }

            ' ' | '\r' | '\t' => {
                // Ignore whitespace
            }
            '\n' => {
                self.line += 1; // Track line numbers correctly
            }

            '"' => self.string(), //here we are calling the string function

            '0'..='9' => self.number(), // Call number() when encountering a digit

            _ if Self::isalpha(c) => self.identifier(),

            _ => {
                let line_content: String = self
                    .src
                    .lines()
                    .nth(self.line - 1)
                    .unwrap_or("")
                    .to_string();

                error::error(
                    self.line,
                    &format!("Unexpected character: '{}'", c),
                    &line_content,
                );
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(TokenType::EOF, "".into(), TokenLiteral::Null));
        self.tokens.clone()
    }
    // pub fn print_tokens(&self) {
    //     for token in &self.tokens {
    //         println!("{}", token);
    //     }
    // }
}
