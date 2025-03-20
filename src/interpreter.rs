use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType, TokenLiteral};
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use crate::environment::{self, Environment};
pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}
impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Rc::new(RefCell::new(environment::Environment::new(None))),
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
    fn is_truthy(&self, value: &Arc<dyn Any + Send + Sync>) -> bool {
        if let Some(b) = value.downcast_ref::<bool>() {
            *b
        } else if let Some(n) = value.downcast_ref::<f64>() {
            *n != 0.0
        } else if let Some(s) = value.downcast_ref::<String>() {
            !s.is_empty()
        } else {
            false
        }
    }
    

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), String> {
        let previous = self.environment.clone();  // ✅ Save old environment
        self.environment = environment.clone(); 

        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));

        self.environment = previous; // Restore previous environment after execution
        result
    }

    
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {

            Stmt::Input { name } => {
                // Read user input from the console
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read input");
                let input = input.trim().to_string(); // Remove whitespace
            
                // Try parsing as number, otherwise store as string
                let value: Arc<dyn Any + Send + Sync> = if let Ok(num) = input.parse::<f64>() {
                    Arc::new(num) // Store as number if possible
                } else {
                    Arc::new(input) // Store as string otherwise
                };
            
                self.environment.borrow_mut().assign(name, value).map_err(|e| e.to_string())
            }
            
            Stmt::While { condition, body } => {
              
                while {
                    
                    let result = self.evaluate(condition)?;
                    self.is_truthy(&result)
                } {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Block(statements) => {
                let enclosing = self.environment.clone();
                let new_env = Environment::new(Some(enclosing));
                self.execute_block(statements, Rc::new(RefCell::new(new_env)))
            }
            
            Stmt::If { condition, then_branch, else_branch } => {
                let condition = self.evaluate(condition)?;
                if let Some(b) = condition.downcast_ref::<bool>() {
                    if *b {
                        self.execute(then_branch)
                    } else if let Some(else_branch) = else_branch {
                        self.execute(else_branch)
                    } else {
                        Ok(())
                    }
                } else {
                    Err("Condition must be a boolean.".to_string())
                }
            }
            
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Arc::new(())
                };
                let cloned_value = if let Some(v) = value.downcast_ref::<f64>() {
                    Arc::new(*v) as Arc<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<String>() {
                    Arc::new(v.clone()) as Arc<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<bool>() {
                    Arc::new(*v) as Arc<dyn Any + Send + Sync>
                } else {
                    Arc::new(()) as Arc<dyn Any + Send + Sync>
                };
                self.environment.borrow_mut().define(name.clone(), cloned_value);
                Ok(())
            },
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
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
          
            
            Expr::Logical { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let left_truthy = self.is_truthy(&left_val);
            
                match operator.token_type {
                    TokenType::OR => {
                        if left_truthy {
                            return Ok(Arc::new(true)); // Ensuring a boolean result
                        }
                    }
                    TokenType::AND => {
                        if !left_truthy {
                            return Ok(Arc::new(false)); // Ensuring a boolean result
                        }
                    }
                    _ => return Err(format!("Unsupported logical operator: {:?}", operator.token_type)),
                }
            
                let right_val = self.evaluate(right)?;
                Ok(Arc::new(self.is_truthy(&right_val))) // Ensure boolean result
            }
            
    
            Expr::If { condition, then_branch, else_branch } => {
                let condition = self.evaluate(condition)?;
                if let Some(b) = condition.downcast_ref::<bool>() {
                    if *b {
                        self.evaluate(then_branch)
                    } else if let Some(else_branch) = else_branch {
                        self.evaluate(else_branch)
                    } else {
                        Ok(Arc::new(()))
                    }
                } else {
                    Err("Condition must be a boolean.".to_string())
                }
            }
            Expr::Variable(name) => {
                let token = Token::new(
                    TokenType::IDENTIFIER, 
                    name.name.lexeme.clone(), 
                    TokenLiteral::Identifier(name.name.lexeme.clone())
                );
            
                match self.environment.borrow().get(&token) {
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
                    Arc::new(*v) as Arc<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<String>() {
                    Arc::new(v.clone()) as Arc<dyn Any + Send + Sync>
                } else if let Some(v) = value.downcast_ref::<bool>() {
                    Arc::new(*v) as Arc<dyn Any + Send + Sync>
                } else {
                    Arc::new(()) as Arc<dyn Any + Send + Sync>
                };
                self.environment.borrow_mut().assign(&Token::new(TokenType::IDENTIFIER, name.clone(), TokenLiteral::Identifier(name.clone())), cloned_value).map_err(|e| e.to_string())?;
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
        if let Some(s) = value.downcast_ref::<String>() {
            return s.clone();
        }
        if let Some(num) = value.downcast_ref::<f64>() {
            return num.to_string();
        }
        if let Some(b) = value.downcast_ref::<bool>() {
            return b.to_string();
        }
        if value.is::<()>() {
            return "nil".to_string();
        }
        "Unknown".to_string()
    }
    
}
