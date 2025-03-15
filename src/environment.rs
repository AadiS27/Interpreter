use std::collections::HashMap;
use crate::token::Token;
use std::any::Any;
use crate::error::RuntimeError;
use crate::token::TokenLiteral; // Add this line to import TokenLiteral

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Box<dyn std::any::Any>>,
    enclosing: Option<Box<Environment>>, // For nested scopes
}


impl Clone for Environment {
    fn clone(&self) -> Self {
        let mut cloned_values = HashMap::new();
        for (key, value) in &self.values {
            let cloned_value = if let Some(v) = value.downcast_ref::<f64>() {
                Box::new(*v) as Box<dyn Any>
            } else if let Some(v) = value.downcast_ref::<String>() {
                Box::new(v.clone()) as Box<dyn Any>
            } else if let Some(v) = value.downcast_ref::<bool>() {
                Box::new(*v) as Box<dyn Any>
            } else if let Some(v) = value.downcast_ref::<TokenLiteral>() {
                Box::new(v.clone()) as Box<dyn Any>
            } else {
                panic!("Unsupported type in environment values");
            };
            cloned_values.insert(key.clone(), cloned_value);
        }
        Environment {
            values: cloned_values,
            enclosing: self.enclosing.clone(),
        }
    }
}
impl Environment {
    /// Creates a new, empty environment (global scope).
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }

    /// Creates a new environment with an enclosing scope (nested).
    pub fn with_enclosing(enclosing: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    ///  Defines a new variable or updates an existing one in the current scope.
    pub fn define(&mut self, name: String, value: Box<dyn Any>) {
        //  Ensure uninitialized variables store `nil`
        let stored_value: Box<dyn Any> = if value.downcast_ref::<TokenLiteral>() == Some(&TokenLiteral::Null) {
            Box::new(TokenLiteral::Null)
        } else {
            value
        };
    
        self.values.insert(name, stored_value);
    }
    
    

    ///  Retrieves the value of a variable.
    pub fn get(&self, name: &Token) -> Result<Box<dyn Any>, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            //  If variable exists but contains `None`, return `nil`
            if value.downcast_ref::<()>().is_some() {
                return Ok(Box::new(())); // Representing nil with an empty tuple
            }

            //  Clone the inner value and return a new `Box`
            if let Some(v) = value.downcast_ref::<f64>() {
                return Ok(Box::new(*v));
            } else if let Some(v) = value.downcast_ref::<String>() {
                return Ok(Box::new(v.clone()));
            } else if let Some(v) = value.downcast_ref::<bool>() {
                return Ok(Box::new(*v));
            }

            return Err(RuntimeError::new(
                name,
                format!("Unsupported type for '{}'.", name.lexeme),
            ));
        }

        //  Check enclosing scope (nested environments)
        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        //  If not found, return an "Undefined variable" error
        Err(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
    
    ///  Assigns a new value to an existing variable.
    pub fn assign(&mut self, name: &Token, value: Box<dyn std::any::Any>) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        // If not found in the current scope, try the enclosing scope
        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }

        Err(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }
}
