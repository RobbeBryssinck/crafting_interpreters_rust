use std::collections::HashMap;
use crate::interpreter::Value;
use crate::scanner::Token;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn get(&self, name: &Token) -> Option<&Value> {
        self.values.get(&name.lexeme)
    }

    pub fn define(&mut self, name: &Token, value: Value) {
        self.values.insert(name.lexeme.to_string(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value, String> {
        if self.values.contains_key(&name.lexeme) {
            self.values.entry(name.lexeme.clone()).or_insert(value.clone());
            Ok(value)
        } else {
            Err(format!("Variable '{}' does not exist.", name.lexeme))
        }
    }
}