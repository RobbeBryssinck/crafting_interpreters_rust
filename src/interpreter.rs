use crate::scanner::{Literal, TokenType};
use crate::expressions::Expr;
use crate::error_reporter;

pub fn interpret(expr: &Expr) {
    let result = match evaluate(expr) {
        Ok(value) => value,
        Err(_e) => Value::Nil
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

fn evaluate(expr: &Expr) -> Result<Value, String> {
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
                            return Ok(Value::Number(-value));
                        },
                        _ => { return Err(generate_error(operator.line, "cannot apply '-' operator on a non-number.")); }
                    }
                },
                TokenType::Bang => {
                    match right_object {
                        Value::Bool(value) => {
                            return Ok(Value::Bool(!value));
                        },
                        _ => { return Err(generate_error(operator.line, "cannot apply '!' operator on a non-number.")); }
                    }
                }
                _ => { return Err(generate_error(operator.line, "unary operator must be '-' or '!'.")); }
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
                            return Ok(Value::Number(left_value - right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "cannot apply '-' on non-numbers.")); }
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
                        (_, _) => { return Err(generate_error(operator.line, "'+' operator must be applied on numbers or strings.")); }
                    }
                },
                TokenType::Slash => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            if right_value == 0.0 {
                                return Err(generate_error(operator.line, "cannot divide by 0."));
                            }
                            return Ok(Value::Number(left_value / right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'/' operator must be applied on numbers.")); }
                    }
                },
                TokenType::Star => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Ok(Value::Number(left_value * right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'*' operator must be applied on numbers.")); }
                    }
                },
                TokenType::Greater => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Ok(Value::Bool(left_value > right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'>' operator must be applied on numbers.")); }
                    }
                },
                TokenType::GreaterEqual => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Ok(Value::Bool(left_value >= right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'>=' operator must be applied on numbers.")); }
                    }
                },
                TokenType::Less => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Ok(Value::Bool(left_value < right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'<' operator must be applied on numbers.")); }
                    }
                },
                TokenType::LessEqual => {
                    match (left_object, right_object) {
                        (Value::Number(left_value), Value::Number(right_value)) => 
                        {
                            return Ok(Value::Bool(left_value <= right_value));
                        },
                        (_, _) => { return Err(generate_error(operator.line, "'<=' operator must be applied on numbers.")); }
                    }
                },
                TokenType::BangEqual => {
                    match is_equal(&left_object, &right_object) {
                        Some(result) => { return Ok(Value::Bool(!result)); },
                        // TODO: error should be reported in is_equal
                        None => { return Err(generate_error(operator.line, "'!=' operator must be applied on the same types.")); }
                    }
                }
                TokenType::EqualEqual => {
                    match is_equal(&left_object, &right_object) {
                        Some(result) => { return Ok(Value::Bool(result)); },
                        // TODO: error should be reported in is_equal
                        None => { return Err(generate_error(operator.line, "'==' operator must be applied on the same types.")); }
                    }
                }
                _ => { return Err(generate_error(operator.line, "unknown token found while parsing binary expression.")); }
            }
        },
        _ => { return Err(String::from("Unknown expression type found.")); }
    }
}

fn literal_to_value(literal: &Literal) -> Result<Value, String> {
    match literal {
        Literal::Identifier(text) => { Ok(Value::Identifier(text.clone())) },
        Literal::Str(text) => { Ok(Value::Str(text.clone())) },
        Literal::Number(number) => { Ok(Value::Number(number.clone())) },
        Literal::Bool(value) => { Ok(Value::Bool(value.clone())) },
        Literal::Nil => { Ok(Value::Nil) },
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
        (_, _) => { return None; }
    }
}

fn generate_error(line: i32, message: &str) -> String {
	format!("[line {line}] Error: {message}")
}

#[derive(Clone, PartialEq)]
enum Value {
    Identifier(String),
    Str(String),
    Number(f64),
    Bool(bool),
    Nil
}