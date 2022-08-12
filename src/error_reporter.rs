use crate::scanner::{Token, TokenType};

static mut IS_ERROR: bool = false;

pub fn error(line: i32, message: &str) {
	println!("[line {line}] Error: {message}");
	unsafe { IS_ERROR = true; }
}

pub fn token_error(token: &Token, message: &str) {
	/*
	if token.token_type == TokenType::EOF {
		error(token.line, message);
	}
	*/
	error(token.line, message);
}