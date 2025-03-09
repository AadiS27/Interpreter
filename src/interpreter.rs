use crate::expr::{Expr, Binary, Grouping, Literal, Unary};
use crate::token::{Token, TokenType, TokenLiteral};
use std::sync::Arc;
use std::any::Any;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter
    }

    pub fn interpret(&self, expr: &Expr) -> Result<String, String> {
        let value = self.evaluate(expr)?;
        Ok(self.stringify(&value))
    }

    fn evaluate(&self, expr: &Expr) -> Result<Arc<dyn Any + Send + Sync>, String> {
        match expr {
            Expr::Literal(lit) => {
                if let Some(token_literal) = lit.value.downcast_ref::<TokenLiteral>() {
                    match token_literal {
                        TokenLiteral::Number(n) => Ok(Arc::new(*n)),
                        TokenLiteral::String(s) => Ok(Arc::new(s.clone())),
                        TokenLiteral::Identifier(id) => Ok(Arc::new(id.clone())),
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
                        if let (Some(l), Some(r)) = (left.downcast_ref::<bool>(), right.downcast_ref::<bool>()) {
                            return Ok(Arc::new(l == r));
                        }
                        if left.downcast_ref::<()>().is_some() && right.downcast_ref::<()>().is_some() {
                            return Ok(Arc::new(true)); // Both are `nil`
                        }
                        Ok(Arc::new(false)) // Different types are always false
                    }
                    
                    TokenType::BANG_EQUAL => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l != r));
                        }
                        if let (Some(l), Some(r)) = (left.downcast_ref::<String>(), right.downcast_ref::<String>()) {
                            return Ok(Arc::new(l != r));
                        }
                        if let (Some(l), Some(r)) = (left.downcast_ref::<bool>(), right.downcast_ref::<bool>()) {
                            return Ok(Arc::new(l != r));
                        }
                        if left.downcast_ref::<()>().is_some() && right.downcast_ref::<()>().is_some() {
                            return Ok(Arc::new(false)); // Both are `nil`, so they are equal
                        }
                        Ok(Arc::new(true)) // Different types are always true
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
