pub mod scanner;
pub mod environment;
mod syntax;
mod parser;
mod interpreter;

use interpreter::Interpreter;

use std::{env, process::exit};
use std::fs;
use std::io::stdin;

fn run(interpreter: &mut Interpreter, contents: &str) {
    let tokens = match scanner::scan_tokens(contents) {
        Ok(tokens) => tokens,
        Err(_) => { return; }
    };

    let statements = match parser::parse_tokens(tokens) {
        Ok(statements) => statements,
        Err(_) => { return; }
    };

    interpreter.interpret(&statements);
}

fn run_file(filename: &str) {
    println!("Running file {filename}");

    let mut interpreter = Interpreter::new(false);

    let contents = fs::read_to_string(filename).expect("Someting went wrong reading the file");
    run(&mut interpreter, &contents);
}

fn run_prompt() {
    println!("Running prompt");

    let mut interpreter = Interpreter::new(true);

    loop {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).expect("Failed to read line.");
        match buffer.as_str() {
            "\n" | "\r\n" => {
                println!("Exiting interactive prompt.");
                exit(0)
            },
            _ => run(&mut interpreter, &buffer),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {
            println!("Usage: jlox [script]");
            exit(64);
        }
    }
}
