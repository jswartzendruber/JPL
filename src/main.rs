use std::{env, fs, process};

use parser::Parser;

use crate::codegen::compile;

mod codegen;
mod lexer;
mod parser;

#[derive(Debug)]
pub struct JPLError {
    message: String,
    line: usize,
}

impl JPLError {
    fn new(message: String, line: usize) -> Self {
        Self { message, line }
    }

    pub fn print_error(&self) {
        eprintln!("Error on line {}: {}", self.line, self.message);
    }
}

fn main() -> Result<(), JPLError> {
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
    let statements = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            e.print_error();
            process::exit(2);
        }
    };
    compile(statements);

    Ok(())
}
