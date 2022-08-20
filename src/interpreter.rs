use crate::scanner::{Literal, TokenType};
use crate::syntax::{Expr, Stmt};
use crate::environment::Environment;

use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<Environment>,
    is_repl: bool,
}

impl Interpreter {
    pub fn new(is_repl: bool) -> Self {
        Self { 
            environment: Rc::new(Environment::new()),
            is_repl: is_repl,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), ()> {
        for statement in statements {
            match self.execute(&statement) {
                Ok(()) => {},
                Err(e) => {
                    println!("Failed to interpret statement.");
                    println!("{}", e);
                    return Err(());
                }
            }
        }

        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Expression { expression } => {
                match self.evaluate(expression) {
                    Ok(value) => { 
                        if self.is_repl {
                            println!("{}", self.stringify(&value));
                        }
                        Ok(())
                    },
                    Err(e) => { return Err(e); }
                }
            },
            Stmt::Print { expression } => {
                let value = self.evaluate(expression);
                match value {
                    Ok(value) => {
                        println!("{}", self.stringify(&value));
                        Ok(())
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            },
            Stmt::Variable { name, initializer } => {
                match initializer {
                    Some(expr) => {
                        let value = match self.evaluate(expr) {
                            Ok(value) => value,
                            Err(e) => { return Err(e); }
                        };
                        self.environment.define(name, value);
                    },
                    None => {}
                }

                Ok(())
            },
            Stmt::Block { statements } => {
                self.environment = Rc::new(Environment::from(Rc::clone(&self.environment)));
                for statement in statements {
                    match self.execute(statement) {
                        Ok(_) => {},
                        Err(e) => { 
                            self.environment = match &self.environment.enclosing {
                                Some(enclosing) => Rc::clone(&enclosing),
                                None => { return Err(format!("{}\n{}", "Enclosing environment not found.", e)); }
                            };

                            return Err(e);
                        }
                    }
                }

                self.environment = match &self.environment.enclosing {
                    Some(enclosing) => Rc::clone(&enclosing),
                    None => { return Err(String::from("Enclosing environment not found.")); }
                };

                Ok(())
            },
            Stmt::If { condition, then_branch, else_branch } => {
                if is_truthy(&self.evaluate(condition)?) {
                    self.execute(then_branch)
                } else if else_branch.is_some() {
                    // TODO: why?
                    match else_branch {
                        Some(statement) => self.execute(statement),
                        None => Err("This can literally never hit.".to_string())
                    }
                } else {
                    Ok(())
                }
            },
            Stmt::While { condition, body } => {
                while is_truthy(&self.evaluate(condition)?) {
                    match self.execute(body) {
                        Ok(_) => {},
                        Err(e) => {
                            if e == "break".to_string() {
                                return Ok(());
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }

                Ok(())
            },
            Stmt::Break {  } => {
                // TODO: this is super ghetto
                Err("break".to_string())
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal { value } => {
                self.literal_to_value(value)
            },
            Expr::Grouping { expression } => {
                self.evaluate(expression)
            },
            Expr::Variable { name } => {
                match self.environment.get(name) {
                    Some(value) => Ok(value.clone()),
                    None => Err(format!("Variable '{}' is undefined.", name.lexeme))
                }
            },
            Expr::Assign { name, value } => {
                let new_value = self.evaluate(value)?;
                self.environment.assign(name, new_value)
            },
            Expr::Logical { 
                left, 
                operator, 
                right 
            } => {
                let left_object = self.evaluate(left)?;

                if operator.token_type == TokenType::Or {
                    if is_truthy(&left_object) {
                        return Ok(left_object);
                    }
                } else {
                    if !is_truthy(&left_object) {
                        return Ok(left_object);
                    }
                }

                self.evaluate(right)
            },
            Expr::Unary { 
                operator, 
                right 
            } => {
                let right_object = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => {
                        match right_object {
                            Value::Number(value) => {
                                return Ok(Value::Number(-value));
                            },
                            _ => { return Err(self.generate_error(operator.line, "cannot apply '-' operator on a non-number.")); }
                        }
                    },
                    TokenType::Bang => {
                        match right_object {
                            Value::Bool(value) => {
                                return Ok(Value::Bool(!value));
                            },
                            _ => { return Err(self.generate_error(operator.line, "cannot apply '!' operator on a non-number.")); }
                        }
                    }
                    _ => { return Err(self.generate_error(operator.line, "unary operator must be '-' or '!'.")); }
                }
            },
            Expr::Binary { 
                left, 
                operator, 
                right 
            } => {
                let left_object = self.evaluate(left)?;
                let right_object = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => { 
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Number(left_value - right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "cannot apply '-' on non-numbers.")); }
                        }
                    },
                    TokenType::Plus => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Number(left_value + right_value));
                            },
                            (Value::Str(left_value), Value::Str(right_value)) => 
                            {
                                return Ok(Value::Str(format!("{}{}", left_value, right_value)));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'+' operator must be applied on numbers or strings.")); }
                        }
                    },
                    TokenType::Slash => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                if right_value == 0.0 {
                                    return Err(self.generate_error(operator.line, "cannot divide by 0."));
                                }
                                return Ok(Value::Number(left_value / right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'/' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::Star => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Number(left_value * right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'*' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::Greater => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Bool(left_value > right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'>' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::GreaterEqual => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Bool(left_value >= right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'>=' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::Less => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Bool(left_value < right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'<' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::LessEqual => {
                        match (left_object, right_object) {
                            (Value::Number(left_value), Value::Number(right_value)) => 
                            {
                                return Ok(Value::Bool(left_value <= right_value));
                            },
                            (_, _) => { return Err(self.generate_error(operator.line, "'<=' operator must be applied on numbers.")); }
                        }
                    },
                    TokenType::BangEqual => {
                        match self.is_equal(&left_object, &right_object) {
                            Some(result) => { return Ok(Value::Bool(!result)); },
                            // TODO: error should be reported in is_equal
                            None => { return Err(self.generate_error(operator.line, "'!=' operator must be applied on the same types.")); }
                        }
                    }
                    TokenType::EqualEqual => {
                        match self.is_equal(&left_object, &right_object) {
                            Some(result) => { return Ok(Value::Bool(result)); },
                            // TODO: error should be reported in is_equal
                            None => { return Err(self.generate_error(operator.line, "'==' operator must be applied on the same types.")); }
                        }
                    }
                    _ => { return Err(self.generate_error(operator.line, "unknown token found while parsing binary expression.")); }
                }
            },
        }
    }

    fn literal_to_value(&mut self, literal: &Literal) -> Result<Value, String> {
        match literal {
            Literal::Identifier(text) => { Ok(Value::Identifier(text.clone())) },
            Literal::Str(text) => { Ok(Value::Str(text.clone())) },
            Literal::Number(number) => { Ok(Value::Number(number.clone())) },
            Literal::Bool(value) => { Ok(Value::Bool(value.clone())) },
            Literal::Nil => { Ok(Value::Nil) },
        }
    }

    fn is_equal(&mut self, left: &Value, right: &Value) -> Option<bool> {
        match (left, right) {
            (Value::Identifier(left_value), Value::Identifier(right_value)) => 
            {
                return Some(left_value == right_value);
            },
            (Value::Str(left_value), Value::Str(right_value)) => 
            {
                return Some(left_value == right_value);
            },
            (Value::Number(left_value), Value::Number(right_value)) => 
            {
                return Some(left_value == right_value);
            },
            (Value::Bool(left_value), Value::Bool(right_value)) => 
            {
                return Some(left_value == right_value);
            },
            (Value::Nil, Value::Nil) => 
            {
                return Some(true);
            },
            (_, _) => { return None; }
        }
    }

    fn stringify(&mut self, value: &Value) -> String {
        match value {
            Value::Identifier(val) => { val.clone() },
            Value::Str(val) => { val.clone() },
            Value::Number(val) => { val.to_string() },
            Value::Bool(val) => { val.to_string() },
            Value::Nil => { String::from("nil") },
        }
    }

    fn generate_error(&mut self, line: i32, message: &str) -> String {
        format!("[line {line}] Error: {message}")
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(value) => *value,
        Value::Nil => false,
        _ => true,
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Nil
}