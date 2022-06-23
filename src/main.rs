mod scanner;
mod error_reporter;

use std::{env, process::exit};
use std::fs;
use std::io::stdin;

fn run(contents: &str) {
    let tokens = scanner::scan_tokens(contents);

    for token in tokens.iter() {
    }
}

fn run_file(filename: &str) {
    println!("Running file {filename}");

    let contents = fs::read_to_string(filename).expect("Someting went wrong reading the file");
    run(&contents);
}

fn run_prompt() {
    println!("Running prompt");

    loop {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).expect("Failed to read line.");
        match buffer.as_str() {
            "\n" | "\r\n" => {
                println!("Exiting interactive prompt.");
                exit(0)
            },
            _ => run(&buffer),
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
