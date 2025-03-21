use std::sync::Arc;
use std::any::Any;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::RuntimeError; // Import RuntimeError from your error module


/// Trait for all built-in native functions in the interpreter

pub trait NativeFunction: Any + Send + Sync {
    fn call(&self, arguments: Vec<Box<dyn Any + Send + Sync>>) -> Result<Box<dyn Any + Send + Sync>, RuntimeError>;
    fn arity(&self) -> usize;
    fn to_string(&self) -> String;
}

/// `ClockFunction` - Returns the system time in seconds
pub struct ClockFunction;

impl NativeFunction for ClockFunction {
    fn arity(&self) -> usize {
        0 // No arguments required
    }

        fn call(&self, _args: Vec<Box<dyn Any + Send + Sync>>) -> Result<Box<dyn Any + Send + Sync>, RuntimeError> {
            let duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            Ok(Box::new(duration.as_secs_f64())) // Return current timestamp
        }
        
        fn to_string(&self) -> String {
            "native fn clock()".to_string()
        }
    }
