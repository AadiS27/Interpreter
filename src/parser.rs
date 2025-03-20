use crate::token::{Token, TokenType, TokenLiteral};
use crate::expr::{Expr, Binary, Unary, Literal, Grouping};
use std::sync::Arc;
use crate::stmt::Stmt;
use crate::expr::Variable;

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
   
        self.assignment() // Instead of self.equality()
    }
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?; // This should be `or()`, not `equality()`
    
        if self.match_tokens(&[TokenType::EQUAL]) {
            let value = self.assignment()?; // Recursively parse the RHS
    
            if let Expr::Variable(var) = expr { // Ensure LHS is a variable
                return Ok(Expr::Assign(var.name.lexeme.clone(), Box::new(value)));
            } 
    
            return Err(ParseError::new("Invalid assignment target."));
        }
        
        Ok(expr)
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
       

        match self.primary() {
            Ok(expr) => expr,
            Err(err) => {
                // Handle the error appropriately, for example by returning a default expression
                return Expr::Literal(Literal::new(TokenLiteral::Null));
            },
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_tokens(&[TokenType::FALSE]) {
            return Ok(Expr::Literal(Literal {
                value: Arc::new(false),
            }));
        }
        if self.match_tokens(&[TokenType::TRUE]) {
            return Ok(Expr::Literal(Literal {
                value: Arc::new(true),
            }));
        }
        if self.match_tokens(&[TokenType::NIL]) {
            return Ok(Expr::Literal(Literal {
                value: Arc::new(()), // Using () as Rust's equivalent of `nil`
            }));
        }
        if self.match_tokens(&[TokenType::NUMBER, TokenType::STRING]) {
            if let Some(value) = self.previous().literal.clone() {
                return Ok(Expr::Literal(Literal::new(value)));
            }
            // Handle the case where the token has no literal value
            return Ok(Expr::Literal(Literal::new(TokenLiteral::Null)));
        }
    
        if self.match_tokens(&[TokenType::LEFT_BRACKET]) {
            let mut elements = Vec::new();
    
            if !self.check(TokenType::RIGHT_BRACKET) { // Not an empty array
                loop {
                    elements.push(self.expression()?);
                    if !self.match_tokens(&[TokenType::COMMA]) {
                        break;
                    }
                }
            }
    
            self.consume(TokenType::RIGHT_BRACKET, "Expect ']' after array elements.");
            return Ok(Expr::Array(elements));
        }
        // ✅ NEW: Handle identifiers (variable names)
        if self.match_tokens(&[TokenType::IDENTIFIER]) {
            let var_name = self.previous().clone();
    
            // ✅ Handle array indexing: `arr[1]`
            if self.match_tokens(&[TokenType::LEFT_BRACKET]) {
                let index = self.expression()?;  // Parse the index expression
                self.consume(TokenType::RIGHT_BRACKET, "Expect ']' after index.");
                return Ok(Expr::Indexing {
                    array: Box::new(Expr::Variable(Variable { name: var_name })),
                    index: Box::new(index),
                });
            }
    
            return Ok(Expr::Variable(Variable { name: var_name }));
        }
         
        // ✅ Fix: Return error instead of panicking
        if self.match_tokens(&[TokenType::LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Ok(Expr::Grouping(Grouping {
                expression: Box::new(expr),
            }));
        }
        
    
        Err(ParseError::new(&format!("Error at '{}': Expect expression.", self.peek().lexeme)))
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
        if self.match_tokens(&[TokenType::VAR]) {  
            return Some(self.variable_declaration());
        }
        if self.match_tokens(&[TokenType::PRINT]) {
            return Some(self.print_statement());
        }
        if self.match_tokens(&[TokenType::LEFT_BRACE]) { 
            return Some(Stmt::Block(self.block())); // ✅ NEW: Handle block statements
        }
        if self.match_tokens(&[TokenType::IF]) {
            return Some(self.if_statement());
        }
        if self.match_tokens(&[TokenType::WHILE]) {
            return match self.while_statement() {
                Ok(stmt) => Some(stmt),
                Err(err) => {
                    self.synchronize();
                    None
                }
            };
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

    
    fn variable_declaration(&mut self) -> Stmt {
        let name_token = self.consume(TokenType::IDENTIFIER, "Expect variable name.");
        let name = name_token.lexeme.clone();
    
        let initializer = if self.match_tokens(&[TokenType::EQUAL]) {
            match self.expression() {
                Ok(expr) => Some(expr),
                Err(_) => None,
            }
        } else {
            None
        };
    
        self.consume(TokenType::SEMICOLON, "Expect ';' after variable declaration.");
    
        Stmt::Var { name, initializer }
    }
    
    
    
    fn block(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
    
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            if let Some(stmt) = self.statement() {
                statements.push(stmt);
            }
        }
    
        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");
        statements
    }
    
    fn if_statement(&mut self) -> Stmt {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression().unwrap();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after if condition.");
    
        let then_branch = Box::new(self.statement().unwrap());
        let else_branch = if self.match_tokens(&[TokenType::ELSE]) {
            Some(Box::new(self.statement().unwrap()))
        } else {
            None
        };
    
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }
    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality(); // Parse left-hand side
    
        while self.match_tokens(&[TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality(); // Parse right-hand side
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?; // Parse left-hand side
    
        while self.match_tokens(&[TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.and()?; // Parse right-hand side
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
    
        Ok(expr)
    }
    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LEFT_PAREN, "Expected '(' after 'while'.");
        let condition = self.expression()?; // Parse condition
        self.consume(TokenType::RIGHT_PAREN, "Expected ')' after condition.");
        let body = match self.statement() {
            Some(stmt) => stmt,
            None => return Err(ParseError::new("Expected statement for while body")),
        }; // Parse loop body
    
        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }
    
    
}

