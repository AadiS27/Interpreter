use crate::token::{Token, TokenType, TokenLiteral};
use crate::expr::{Expr, Binary, Unary, Literal, Grouping};
use std::sync::Arc;
use crate::stmt::Stmt;

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: &str) -> Self {
        ParseError {
            message: message.to_string(),
        }
    }
}
use std::fmt;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Parser {

   pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        Ok(self.equality())
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_tokens(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_tokens(&[
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            let operator = self.previous().clone();
            let right = self.term();
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_tokens(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right = self.factor();
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_tokens(&[TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.match_tokens(&[TokenType::FALSE]) {
            return Expr::Literal(Literal {
                value: Arc::new(false),
            });
        }
        if self.match_tokens(&[TokenType::TRUE]) {
            return Expr::Literal(Literal {
                value: Arc::new(true),
            });
        }
        if self.match_tokens(&[TokenType::NIL]) {
            return Expr::Literal(Literal {
                value: Arc::new(( )), // Using () as Rust's equivalent of `nil`
            });
        }
        if self.match_tokens(&[TokenType::NUMBER, TokenType::STRING]) {
            if let Some(value) = self.previous().literal.clone() {
                return Expr::Literal(Literal::new(value));
            }
            // Handle the case where the token has no literal value
            return Expr::Literal(Literal::new(TokenLiteral::Null));
        }
        
        
        
    
        if self.match_tokens(&[TokenType::LEFT_PAREN]) {
            let expr = self.expression().unwrap();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(Grouping {
                expression: Box::new(expr),
            });
        }
    
        panic!("{}", self.error(self.peek(), "Expect expression."));
    }
    
 

    pub fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for &token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }
    fn consume(&mut self, token_type: TokenType, message: &str) -> Token {
        if self.check(token_type) {
            return self.advance().clone();
        }
    
        panic!("{}", message); // Replace with proper error handling later
    }

    pub fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    pub fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    pub fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        ParseError::new(&format!(" Error at '{}': {}", token.lexeme, message))
    }
    fn report(&self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return;
            }

            match self.peek().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,

                _ => {}
            }

            self.advance();
        }
    }
    pub fn parse(&mut self) -> Option<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(stmt) = self.statement() {
                statements.push(stmt);
            } else {
                return None; // Return None if any statement fails
            }
        }
        Some(statements) // Return Some if parsing succeeds
    }
    

    fn statement(&mut self) -> Option<Stmt> {
        if self.match_tokens(&[TokenType::PRINT]) {
            return Some(self.print_statement());
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'print'."); // Require '('
    
        let value = match self.expression() {
            Ok(expr) => expr,
            Err(_) => {
                self.synchronize();
                return Stmt::Print { expression: Expr::Literal(Literal::new(TokenLiteral::Null)) };
            }
        };
    
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression."); // Require ')'
        self.consume(TokenType::SEMICOLON, "Expect ';' after print statement."); // Require ';'
    
        Stmt::Print { expression: value }
    }
    

    fn expression_statement(&mut self) -> Option<Stmt> {
        let expr = match self.expression() {
            Ok(expr) => expr,
            Err(err) => {
                self.synchronize();
                return None;
            }
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");
        Some(Stmt::Expression { expression: expr })
    }

    
}

