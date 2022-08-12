use crate::scanner::{Token, TokenType};

// TODO: this whole module is bad
static mut IS_ERROR: bool = false;

pub fn reset_error() {
	unsafe { IS_ERROR = false; }
}

pub fn is_error() -> bool {
	unsafe { return IS_ERROR; }
}

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