use crate::scanner::{Literal, TokenType};
use crate::expressions::Expr;
use crate::error_reporter;

pub fn interpret(expr: &Expr) {
    let result = match evaluate(expr) {
        Some(value) => value,
        None => Value::Nil
    };

    if result == Value::Nil {
        println!("Failed to interpret expression.");
    } else {
        match result {
            Value::Number(value) => { println!("{}", value); },
            _ => {}
        }
    }
    
    error_reporter::reset_error();
}

fn evaluate(expr: &Expr) -> Option<Value> {
    match expr {
        Expr::Literal { value } => {
            return literal_to_value(value);
        },
        Expr::Grouping { expression } => {
            return evaluate(expression);
        },
        Expr::Unary { 
            operator, 
            right 
        } => {
            let right_object = evaluate(right)?;

            match operator.token_type {
                TokenType::Minus => {
                    match right_object {
                        Value::Number(value) => {
                            return Some(Value::Number(-value));
                        },
                        _ => { return None; }
                    }
                },
                TokenType::Bang => {
                    match right_object {
                        Value::Bool(value) => {
                            return Some(Value::Bool(!value));
                        },
                        _ => { return None; }
                    }
                }
                _ => { return None; }
            }
        },
        Expr::Binary { 
            left, 
            operator, 
            right 
        } => {
            let left_object = evaluate(left)?;
            let right_object = evaluate(right)?;

            match operator.token_type {
                TokenType::Minus => { 
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Number(left_value - right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::Plus => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Number(left_value + right_value));
                        },
                        (Value::Str(left_value), Value::Str(right_value)) => 
                        {
                            return Some(Value::Str(format!("{}{}", left_value, right_value)));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::Slash => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Number(left_value / right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::Star => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Number(left_value * right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::Greater => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Bool(left_value > right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::GreaterEqual => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Bool(left_value >= right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::Less => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Bool(left_value < right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::LessEqual => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Some(Value::Bool(left_value <= right_value));
                        },
                        (_, _) => { return None; }
                    }
                },
                TokenType::BangEqual => {
                    match is_equal(&left_object, &right_object) {
                        Some(result) => { return Some(Value::Bool(!result)); },
                        None => { return None; }
                    }
                }
                TokenType::EqualEqual => {
                    match is_equal(&left_object, &right_object) {
                        Some(result) => { return Some(Value::Bool(result)); },
                        None => { return None; }
                    }
                }
                _ => { return None; }
            }
        },
        _ => { return None; }
    }

    return None;
}

fn literal_to_value(literal: &Literal) -> Option<Value> {
    match literal {
        Literal::Identifier(text) => { Some(Value::Identifier(text.clone())) },
        Literal::Str(text) => { Some(Value::Str(text.clone())) },
        Literal::Number(number) => { Some(Value::Number(number.clone())) },
        Literal::Bool(value) => { Some(Value::Bool(value.clone())) },
        Literal::Nil => { Some(Value::Nil) },
    }
}

fn is_equal(left: &Value, right: &Value) -> Option<bool> {
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
        (_, _) => { return Some(false); }
    }
}

#[derive(Clone, PartialEq)]
enum Value {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Nil
}