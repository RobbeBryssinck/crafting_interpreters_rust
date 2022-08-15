use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::interpreter::Value;
use crate::scanner::Token;

#[derive(Clone)]
pub struct Environment {
    pub enclosing: Option<Rc<Environment>>,
    values: RefCell<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn from(enclosing: Rc<Environment>) -> Self {
        Self {
            enclosing: Some(enclosing),
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn get(&self, name: &Token) -> Option<Value> {
        match self.values.borrow().get(&name.lexeme) {
            Some(value) => Some(value.clone()),
            None => {
                match &self.enclosing {
                    Some(enclosing) => enclosing.get(name),
                    None => None
                }
            }
        }
    }

    pub fn define(&self, name: &Token, value: Value) {
        self.values.borrow_mut().insert(name.lexeme.to_string(), value);
    }

    pub fn assign(&self, name: &Token, value: Value) -> Result<Value, String> {
        if self.values.borrow().contains_key(&name.lexeme) {
            self.values.borrow_mut().entry(name.lexeme.clone()).or_insert(value.clone());
            Ok(value)
        } else {
            match &self.enclosing {
                Some(enclosing) => enclosing.assign(name, value),
                None => Err(format!("Variable '{}' does not exist.", name.lexeme))
            }
        }
    }
}