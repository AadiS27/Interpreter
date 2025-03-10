use std::collections::HashMap;
use crate::token::Token;
use crate::error::RuntimeError;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Box<dyn std::any::Any>>,
    enclosing: Option<Box<Environment>>, // For nested scopes
}

impl Environment {
    /// Creates a new, empty environment (global scope).
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    /// Creates a new environment with an enclosing scope (nested).
    pub fn with_enclosing(enclosing: Environment) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    /// ✅ Defines a new variable or updates an existing one in the current scope.
    pub fn define(&mut self, name: String, value: Box<dyn std::any::Any>) {
        self.values.insert(name, value);
    }

    /// ✅ Retrieves the value of a variable.
    pub fn get(&self, name: &Token) -> Result<&Box<dyn std::any::Any>, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value);
        }
        // If not found in the current scope, check the enclosing scope
        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name);
        }

        Err(RuntimeError::new(
            name,
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    /// ✅ Assigns a new value to an existing variable.
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
