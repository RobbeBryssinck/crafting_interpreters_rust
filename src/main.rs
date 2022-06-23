use std::{env, process::exit};

fn run_file(filename: &str) {
    println!("Running file {filename}");
}

fn run_prompt() {
    println!("Running prompt");
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
