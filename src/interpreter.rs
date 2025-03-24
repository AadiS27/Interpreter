use crate::environment::{self, Environment};
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Token, TokenLiteral, TokenType};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
struct Function {
    name: String,
    params: Vec<Token>,
    body: Vec<Box<Stmt>>,
    closure: Rc<RefCell<Environment>>, // Captures the defining environment
}

// Implement Send and Sync for Function to satisfy Arc<dyn Any + Send + Sync>
// This is safe because we only use Function for serialization/deserialization
unsafe impl Send for Function {}
unsafe impl Sync for Function {}
impl Function {
    fn new(
        name: String,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Function {
            name,
            params,
            body,
            closure,
        }
    }
}
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
        let previous = self.environment.clone(); //  Save old environment
        self.environment = environment.clone();

        let result = statements.iter().try_for_each(|stmt| self.execute(stmt));

        self.environment = previous; // Restore previous environment after execution
        result
    }

    
    fn call_function(
        &mut self,
        function: &Function,
        arguments: Vec<Arc<dyn Any + Send + Sync>>,
    ) -> Result<Arc<dyn Any + Send + Sync>, String> {
        let environment = Rc::new(RefCell::new(Environment::new(Some(function.closure.clone()))));
    
        // Bind function parameters to arguments
        for (param, arg) in function.params.iter().zip(arguments.iter()) {
            environment.borrow_mut().define(param.lexeme.clone(), arg.clone());
        }
    
        // Store previous environment and switch to function's environment
        let previous_environment = self.environment.clone();
        self.environment = environment.clone();
    
        let mut return_value: Option<Arc<dyn Any + Send + Sync>> = None;
    
        for stmt in &function.body {
            match self.execute(&**stmt) {
                Err(e) if e.starts_with("Return:") => {
                    // Extract the value from the error message
                    let value_str = e.strip_prefix("Return: ").unwrap_or("");
    
                    // Try to convert common types
                    if value_str == "true" {
                        return_value = Some(Arc::new(true));
                    } else if value_str == "false" {
                        return_value = Some(Arc::new(false));
                    } else if let Ok(num) = value_str.parse::<i64>() {
                        return_value = Some(Arc::new(num));
                    } else if let Ok(num) = value_str.parse::<f64>() {
                        return_value = Some(Arc::new(num));
                    } else {
                        // Prevent issues with large numbers or incorrect values
                        // Checking for extremely large values and handling them appropriately
                        if value_str.len() > 100 {
                            return Err("Value too large to process.".to_string());
                        }
    
                        // Remove the debug formatting artifacts from the string
                        let cleaned_str = value_str.trim()
                            .trim_start_matches("Some(")
                            .trim_end_matches(")")
                            .trim_matches('"');
                        return_value = Some(Arc::new(cleaned_str.to_string()));
                    }
                    break;
                }
                Err(e) => {
                    self.environment = previous_environment;
                    return Err(e);
                }
                Ok(_) => continue,
            }
        }
    
        // Restore previous environment
        self.environment = previous_environment;
    
        Ok(return_value.unwrap_or_else(|| Arc::new(0_i64)))
    }
    

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Return { value, .. } => {
                let value = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    Arc::new(())
                };
                
                // Create a cleaner string representation for the return value
                let return_str = if let Some(v) = value.downcast_ref::<i64>() {
                    format!("Return: {}", v)
                } else if let Some(v) = value.downcast_ref::<f64>() {
                    format!("Return: {}", v)
                } else if let Some(v) = value.downcast_ref::<bool>() {
                    format!("Return: {}", v)
                } else if let Some(v) = value.downcast_ref::<String>() {
                    format!("Return: {}", v)
                } else if value.is::<()>() {
                    String::from("Return: nil")
                } else {
                    format!("Return: {}", self.stringify(&value))
                };
                
                Err(return_str)
            }
            Stmt::Function { name, params, body } => {
                let function = Arc::new(Function::new(
                    name.lexeme.clone(),
                    params.clone(),
                    body.iter().map(|stmt| Box::new(stmt.clone())).collect(),
                    self.environment.clone(),
                ));
            
                self.environment.borrow_mut().define(
                    name.lexeme.clone(),
                    function.clone() as Arc<dyn Any + Send + Sync>, // Ensure it's stored as a dynamic type
                );
            
                Ok(())
            }
            
            Stmt::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                if let Some(init) = initializer {
                    self.execute(init)?;
                }
                while {
                    if let Some(cond) = condition {
                        let result = self.evaluate(cond)?;
                        self.is_truthy(&result)
                    } else {
                        true
                    }
                } {
                    self.execute(body)?;
                    if let Some(inc) = increment {
                        self.evaluate(inc)?;
                    }
                }
                Ok(())
            }
            Stmt::Input { name } => {
                // Read user input from the console
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input"); // Read user input from the console
                let input = input.trim().to_string(); // Remove whitespace

                // Try parsing as number, otherwise store as string
                let value: Arc<dyn Any + Send + Sync> = if let Ok(num) = input.parse::<f64>() {
                    Arc::new(num) // Store as number if possible
                } else {
                    Arc::new(input) // Store as string otherwise
                };

                self.environment
                    .borrow_mut()
                    .assign(name, value)
                    .map_err(|e| e.to_string())
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

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
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
                self.environment
                    .borrow_mut()
                    .define(name.clone(), cloned_value);
                Ok(())
            }
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

            Expr::Call { callee, arguments } => {
                let function_value = self.evaluate(callee)?;
            
                let function = function_value
                    .downcast_ref::<Function>()
                    .ok_or_else(|| "Runtime error: Expected function, found unsupported type.".to_string())?;
            
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate(arg)?);
                }
            
                let result = self.call_function(function, args)?;
                
                //  Ensure result is properly unwrapped
                if let Some(value) = result.downcast_ref::<i64>() {
                    return Ok(Arc::new(*value)); //  Convert back to expected type
                }
                
                Ok(result) //  Ensure correct type is returned
            }
            
            Expr::Logical {
                left,
                operator,
                right,
            } => {
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
                    _ => {
                        return Err(format!(
                            "Unsupported logical operator: {:?}",
                            operator.token_type
                        ))
                    }
                }

                let right_val = self.evaluate(right)?;
                Ok(Arc::new(self.is_truthy(&right_val))) // Ensure boolean result
            }

            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = self.evaluate(condition)?;
            
                // Ensure the condition is treated as a boolean
                if let Some(condition_bool) = condition_value.downcast_ref::<bool>() {
                    if *condition_bool {
                        return self.evaluate(then_branch);
                    } else if let Some(else_expr) = else_branch {
                        return self.evaluate(else_expr);
                    } else {
                        return Ok(Arc::new(())); // Ensure that the if-expression always returns a value (avoid nil issues)
                    }
                } else {
                    return Err("Runtime error: Condition must be a boolean.".to_string());
                }
            }
            
            Expr::Variable(name) => {
                let token = Token::new(
                    TokenType::IDENTIFIER,
                    name.name.lexeme.clone(),
                    TokenLiteral::Identifier(name.name.lexeme.clone()),
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
                            Ok(Arc::new(TokenLiteral::Null)) //  Return `nil` for uninitialized variables
                        } else if let Some(func) = value.downcast_ref::<Function>() {
                            Ok(Arc::new(func.clone())) //  Return the function reference
                        } else {
                            Err("Unsupported type.".to_string())
                        }
                    }
                    Err(_) => Err(format!("Undefined variable '{}'.", name.name.lexeme)), //  Return `nil` if variable is undefined
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
                self.environment
                    .borrow_mut()
                    .assign(
                        &Token::new(
                            TokenType::IDENTIFIER,
                            name.clone(),
                            TokenLiteral::Identifier(name.clone()),
                        ),
                        cloned_value,
                    )
                    .map_err(|e| e.to_string())?;
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
                } else if let Some(b) = lit.value.downcast_ref::<bool>() {
                    Ok(Arc::new(*b))
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
                    // In your binary operation handler for PLUS
                TokenType::PLUS => {
                    // First, try to handle the case where both are f64
                    if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                        return Ok(Arc::new(l + r));
                    }
                    
                    // Add this new case to handle i64 + i64
                    if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<i64>()) {
                        return Ok(Arc::new(l + r));
                    }
                    
                    // Add conversion between i64 and f64
                    if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<f64>()) {
                        return Ok(Arc::new(*l as f64 + r));
                    }
                    
                    if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<i64>()) {
                        return Ok(Arc::new(l + *r as f64));
                    }
                    
                    // String handling remains the same...
                    if let (Some(l), Some(r)) = (
                        left.downcast_ref::<String>(),
                        right.downcast_ref::<String>(),
                    ) {
                        return Ok(Arc::new(format!("{}{}", l, r)));
                    }
                    
                    // Mixed string handling remains the same...
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
                        
                        // Add this new case to handle i64 + i64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l - r));
                        }
                        
                        // Add conversion between i64 and f64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(*l as f64 - r));
                        }
                        
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l - *r as f64));
                        }if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l - r));
                        }
                        
                        // Add this new case to handle i64 + i64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l * r));
                        }
                        
                        // Add conversion between i64 and f64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(*l as f64 * r));
                        }
                        
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l * *r as f64));
                        }
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l - r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::STAR => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l * r));
                        }
                        
                        // Add this new case to handle i64 + i64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l * r));
                        }
                        
                        // Add conversion between i64 and f64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(*l as f64 * r));
                        }
                        
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l * *r as f64));
                        }
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l * r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::SLASH => {
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(l / r));
                        }
                        
                        // Add this new case to handle i64 + i64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l / r));
                        }
                        
                        // Add conversion between i64 and f64
                        if let (Some(l), Some(r)) = (left.downcast_ref::<i64>(), right.downcast_ref::<f64>()) {
                            return Ok(Arc::new(*l as f64 / r));
                        }
                        
                        if let (Some(l), Some(r)) = (left.downcast_ref::<f64>(), right.downcast_ref::<i64>()) {
                            return Ok(Arc::new(l / *r as f64));
                        }
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            if *r == 0.0 {
                                return Err("Division by zero.".to_string());
                            }
                            return Ok(Arc::new(l / r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }

                    TokenType::EQUAL_EQUAL => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l == r));
                        }
                        if let (Some(l), Some(r)) = (
                            left.downcast_ref::<String>(),
                            right.downcast_ref::<String>(),
                        ) {
                            return Ok(Arc::new(l == r));
                        }
                        Err("Operands must be two numbers or two strings.".to_string())
                    }
                    TokenType::BANG_EQUAL => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l != r));
                        }
                        if let (Some(l), Some(r)) = (
                            left.downcast_ref::<String>(),
                            right.downcast_ref::<String>(),
                        ) {
                            return Ok(Arc::new(l != r));
                        }
                        Err("Operands must be two numbers or two strings.".to_string())
                    }

                    TokenType::GREATER => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l > r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::GREATER_EQUAL => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l >= r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::LESS => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
                            return Ok(Arc::new(l < r));
                        }
                        Err("Operands must be numbers.".to_string())
                    }
                    TokenType::LESS_EQUAL => {
                        if let (Some(l), Some(r)) =
                            (left.downcast_ref::<f64>(), right.downcast_ref::<f64>())
                        {
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
        if let Some(v) = value.downcast_ref::<i64>() {
            return v.to_string();
        } else if let Some(v) = value.downcast_ref::<f64>() {
            return v.to_string();
        } else if let Some(v) = value.downcast_ref::<String>() {
            return v.clone();
        } else if value.downcast_ref::<()>().is_some() {
            return "nil".to_string();
        }
        "(Unknown type)".to_string() // Default case
    }
    
}
