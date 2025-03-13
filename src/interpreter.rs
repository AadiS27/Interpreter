use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType, TokenLiteral};
use std::sync::Arc;

use std::any::Any;
use crate::environment::{self, Environment};
pub struct Interpreter {
    environment: Environment,
}
impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: environment::Environment::new(None),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Err(err) = self.visit_stmt(statement) {
                eprintln!("Runtime error: {}", err);
            }
        }
    }
    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        self.visit_stmt(stmt)
    }

    fn execute_block(&mut self, statements: &[Stmt], new_env: Environment) -> Result<(), String> {
    let previous = std::mem::replace(&mut self.environment, new_env); // Swap environments

    let result = statements.iter().try_for_each(|stmt| self.execute(stmt)); // Execute block

    self.environment = previous; // Restore old environment after execution
    result
}

    
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(statements) => {
                let enclosing = self.environment.as_ref().map(|env| env.clone()).unwrap_or_else(Environment::new);
                self.execute_block(statements, Environment::with_enclosing(enclosing))
            }
            
            
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Arc::new(())
                };
                let cloned_value = if let Some(v) = value.downcast_ref::<f64>() {
                    Box::new(*v) as Box<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<String>() {
                    Box::new(v.clone()) as Box<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<bool>() {
                    Box::new(*v) as Box<dyn Any + Send + Sync>
                } else {
                    Box::new(()) as Box<dyn Any + Send + Sync>
                };
                self.environment.define(name.clone(), cloned_value);
                Ok(())
            },
            Stmt::Expression { expression } => {
                let value = self.evaluate(expression)?;
                // println!("{}", self.stringify(&value)); // Print the result for debugging
                Ok(())
            }
            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", self.stringify(&value)); // Actual print statement
                Ok(())
            }
          
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Arc<dyn Any + Send + Sync>, String> {
        match expr {
          
            Expr::Variable(name) => {
                let token = Token::new(
                    TokenType::IDENTIFIER, 
                    name.name.lexeme.clone(), 
                    TokenLiteral::Identifier(name.name.lexeme.clone())
                );
            
                match self.environment.get(&token) {
                    Ok(value) => {
                        if let Some(v) = value.downcast_ref::<f64>() {
                            Ok(Arc::new(*v))
                        } else if let Some(v) = value.downcast_ref::<String>() {
                            Ok(Arc::new(v.clone()))
                        } else if let Some(v) = value.downcast_ref::<bool>() {
                            Ok(Arc::new(*v))
                        } else if value.is::<()>() {
                            Ok(Arc::new(TokenLiteral::Null)) // ✅ Return `nil` for uninitialized variables
                        } else {
                            Err("Unsupported type.".to_string())
                        }
                    }
                    Err(_) => Err(format!("Undefined variable '{}'.", name.name.lexeme)), // ✅ Return `nil` if variable is undefined
                }
            }
            
            Expr::Assign(name, value_expr) => {
                let value = self.evaluate(value_expr)?;
                let cloned_value = if let Some(v) = value.downcast_ref::<f64>() {
                    Box::new(*v) as Box<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<String>() {
                    Box::new(v.clone()) as Box<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<bool>() {
                    Box::new(*v) as Box<dyn Any + Send + Sync>
                } else {
                    Box::new(()) as Box<dyn Any + Send + Sync>
                };
                self.environment.assign(&Token::new(TokenType::IDENTIFIER, name.clone(), TokenLiteral::Identifier(name.clone())), cloned_value).map_err(|e| e.to_string())?;
                Ok(value)
            }
            Expr::Literal(lit) => {
                if let Some(token_literal) = lit.value.downcast_ref::<TokenLiteral>() {
                    match token_literal {
                        TokenLiteral::Number(n) => Ok(Arc::new(*n)),
                        TokenLiteral::String(s) => Ok(Arc::new(s.clone())),
                        TokenLiteral::Identifier(id) => Ok(Arc::new(id.clone())),
                        TokenLiteral::Boolean(b) => Ok(Arc::new(*b)),  
                        TokenLiteral::Null => Ok(Arc::new(())),
                    }
                } 
                else if let Some(b) = lit.value.downcast_ref::<bool>() {
                    Ok(Arc::new(*b))
                } 
              else {
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
