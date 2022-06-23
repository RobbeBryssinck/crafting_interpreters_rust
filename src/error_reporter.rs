static mut IS_ERROR: bool = false;

pub fn error(line: i32, message: &str) {
	println!("[line {line}] Error: {message}");
	unsafe { IS_ERROR = true; }
}