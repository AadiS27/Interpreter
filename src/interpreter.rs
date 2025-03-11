use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{TokenType, TokenLiteral};
use std::sync::Arc;
use std::any::Any;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    pub fn interpret(&self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(err) = self.visit_stmt(statement) {
                eprintln!("Runtime error: {}", err);
            }
        }
    }

    fn visit_stmt(&self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", self.stringify(&value)); // Print the result for debugging
                Ok(())
            }
            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", self.stringify(&value)); // Actual print statement
                Ok(())
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Arc<dyn Any + Send + Sync>, String> {
        match expr {
            Expr::Literal(lit) => {
                if let Some(token_literal) = lit.value.downcast_ref::<TokenLiteral>() {
                    match token_literal {
                        TokenLiteral::Number(n) => Ok(Arc::new(*n)),
                        TokenLiteral::String(s) => Ok(Arc::new(s.clone())),
                        TokenLiteral::Identifier(id) => Ok(Arc::new(id.clone())),
                        TokenLiteral::Boolean(b) => Ok(Arc::new(*b)),  
                        TokenLiteral::Null => Ok(Arc::new(())),
                    }
                } else {
                    Err("Unknown literal type.".to_string())
                }
            }
            Expr::Grouping(group) => self.evaluate(&group.expression),
            Expr::Unary(unary) => {
                let right = self.evaluate(&unary.right)?;

                match unary.operator.token_type {
                    TokenType::MINUS => {
                        if let Some(n) = right.downcast_ref::<f64>() {
                            return Ok(Arc::new(-*n));
                        }
                        Err("Operand must be a number.".to_string())
                    }
                    TokenType::BANG => {
                        if let Some(b) = right.downcast_ref::<bool>() {
                            return Ok(Arc::new(!b));
                        }
                        Ok(Arc::new(false))
                    }
                    TokenType::FALSE => Ok(Arc::new(false)),
                    TokenType::TRUE => Ok(Arc::new(true)),
                    _ => Err("Unknown unary operator.".to_string()),
                }
            }
            Expr::Binary(binary) => {
                let left = self.evaluate(&binary.left)?;
                let right = self.evaluate(&binary.right)?;

                match binary.operator.token_type {
                    TokenType::PLUS => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l + r));
                        }
                        if let (Some(l), Some(r)) = (left.downcast_ref::<String>(), right.downcast_ref::<String>()) {
                            return Ok(Arc::new(format!("{}{}", l, r)));
                        }
                        // NEW: Allow string + any other type by converting to string
                        if let Some(l) = left.downcast_ref::<String>() {
                            return Ok(Arc::new(format!("{}{}", l, self.stringify(&right))));
                        }
                        if let Some(r) = right.downcast_ref::<String>() {
                            return Ok(Arc::new(format!("{}{}", self.stringify(&left), r)));
                        }
                        Err("Operands must be two numbers or two strings.".to_string())
                    }
                    
                    TokenType::MINUS => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l - r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::STAR => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l * r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::SLASH => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            if *r == 0.0 {
                                return Err("Division by zero.".to_string());
                            }
                            return Ok(Arc::new(l / r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }

                    TokenType::EQUAL_EQUAL => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l == r));
                        }
                        if let (Some(l), Some(r)) = (left.downcast_ref::<String>(), right.downcast_ref::<String>()) {
                            return Ok(Arc::new(l == r));
                        }
                        Err("Operands must be two numbers or two strings.".to_string())
                    }
                    TokenType::BANG_EQUAL => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l != r));
                        }
                        if let (Some(l), Some(r)) = (left.downcast_ref::<String>(), right.downcast_ref::<String>()) {
                            return Ok(Arc::new(l != r));
                        }
                        Err("Operands must be two numbers or two strings.".to_string())
                    }

                    TokenType::GREATER => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l > r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::GREATER_EQUAL => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l >= r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::LESS => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l < r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::LESS_EQUAL => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l <= r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    _ => Err("Unknown binary operator.".to_string()),
                }
            }
        }
    }

    fn stringify(&self, value: &Arc<dyn Any + Send + Sync>) -> String {
        if let Some(n) = value.downcast_ref::<f64>() {
            return n.to_string();
        }
        if let Some(b) = value.downcast_ref::<bool>() {
            return b.to_string();
        }
        if let Some(s) = value.downcast_ref::<String>() {
            return s.clone();
        }
        if value.downcast_ref::<()>().is_some() {
            return "nil".to_string();
        }
        "nil".to_string()
    }
}
