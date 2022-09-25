use std::{env, fs, process};

use parser::Parser;

mod lexer;
mod parser;

pub struct JPLError {
    message: String,
}

impl JPLError {
    fn new(message: String) -> Self {
        Self { message }
    }

    pub fn print_error(&self) {
        println!("Error: {}", self.message);
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 0 {
        eprintln!("fatal error: no input files");
        process::exit(1);
    }

    let source = match fs::read_to_string(&args[0]) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("failed to read input file {}", args[0]);
            process::exit(1);
        }
    };

    let tokens = match lexer::lex(source.as_bytes()) {
        Ok(t) => t,
        Err(e) => {
            e.print_error();
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(_) => {
            println!("parsing successful!");

            for statement in parser.statements {
                println!("{:?}", statement);
            }
        }
        Err(e) => e.print_error(),
    }
}
