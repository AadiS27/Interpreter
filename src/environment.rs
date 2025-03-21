use std::collections::HashMap;
use std::any::Any;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use crate::token::Token;
use crate::error::RuntimeError;
use crate::native_fn::NativeFunction; // Import NativeFunction trait
use crate::native_fn::ClockFunction; // Import ClockFunction

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Arc<dyn Any + Send + Sync>>, // Stores variables & functions
    enclosing: Option<Rc<RefCell<Environment>>>, // For nested scopes
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        let mut cloned_values = HashMap::new();
        for (key, value) in &self.values {
            let cloned_value = if let Some(v) = value.downcast_ref::<f64>() {
                Arc::new(*v) as Arc<dyn Any + Send + Sync>
            } else if let Some(v) = value.downcast_ref::<String>() {
                Arc::new(v.clone()) as Arc<dyn Any + Send + Sync>
            } else if let Some(v) = value.downcast_ref::<bool>() {
                Arc::new(*v) as Arc<dyn Any + Send + Sync>
            } else if value.downcast_ref::<Arc<dyn NativeFunction>>().is_some() {
                value.clone()
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
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing,
        }
    }
    pub fn new_global() -> Rc<RefCell<Self>> {
        let env = Rc::new(RefCell::new(Environment::new(None)));

        // Example of defining a native function (assuming `ClockFunction` implements `NativeFunction`)
        env.borrow_mut().define_native("clock", ClockFunction {});

        env
    }

    /// Creates a new environment with an enclosing scope (nested scope).
    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    /// Defines a new variable or updates an existing one in the current scope.
    pub fn define(&mut self, name: String, value: Arc<dyn Any + Send + Sync>) {
        self.values.insert(name, value);
        
    }

    /// Defines a built-in (native) function in the environment.
    pub fn define_native<T: NativeFunction + 'static>(&mut self, name: &str, function: T) {
        self.values.insert(name.to_string(), Arc::new(function));
    }
    /// Retrieves the value of a variable.
    pub fn get(&self, name: &Token) -> Result<Arc<dyn Any + Send + Sync>, RuntimeError> {
        if let Some(value) = self.values.get(&name.lexeme) {
            return Ok(value.clone());
        }
    
        // If not found, check parent environment
        if let Some(ref enclosing) = self.enclosing {
            return enclosing.borrow().get(name);
        }
    
        Err(RuntimeError::new(name, format!("Undefined variable '{}'.", name.lexeme)))
    }

    /// Assigns a new value to an existing variable.
    pub fn assign(&mut self, name: &Token, value: Arc<dyn Any + Send + Sync>) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
    
        // If variable isn't found, assign in parent environment
        if let Some(ref mut enclosing) = self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        
        Err(RuntimeError::new(name, format!("Undefined variable '{}'.", name.lexeme)))
    }
}
