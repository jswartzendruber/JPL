use std::{collections::HashMap, fs::File, io::Write, process::Command};

use crate::{
    lexer::NumberContents,
    parser::{ParsedExpr, ParsedStatement, ParsedVarDecl},
};

struct Emitter {
    output_data: String,
    output_text: String,
    var_table: HashMap<String, ParsedExpr>,
    temporary_var_number: usize,
}

impl Emitter {
    fn new() -> Self {
        let mut emitter = Self {
            output_data: String::from("SECTION .data\n"),
            output_text: String::from("SECTION .text\n"),
            var_table: HashMap::new(),
            temporary_var_number: 0,
        };

        emitter.emit_textln("extern print_int");
        emitter.emit_textln("extern print_char");
        emitter.emit_textln("extern print_string");
        emitter.emit_textln("global _start");
        emitter.emit_textln("_start:");

        emitter
    }

    fn emit_dataln(&mut self, asm: &str) {
        self.output_data.push_str(&format!("\t{}\n", asm));
    }

    fn emit_textln(&mut self, asm: &str) {
        self.output_text.push_str(&format!("\t{}\n", asm));
    }

    fn emit_var_declaration(&mut self, var_name: ParsedVarDecl, expr: ParsedExpr) {
        match expr {
            ParsedExpr::NumericConstant(ref contents) => match contents {
                NumberContents::Integer(i) => {
                    self.emit_dataln(&format!("{} dd {}", var_name.name, i))
                }
                NumberContents::Floating(_) => todo!(),
            },
            ParsedExpr::BinaryOp(_, _, _) => {
                let string_val = ParsedExpr::evaluate_expr_to_string(&expr, &self.var_table);
                self.emit_dataln(&format!("{} db '{}', 0Ah", var_name.name, string_val));
            }
            ParsedExpr::QuotedString(ref str) => {
                self.emit_dataln(&format!("{} db '{}', 0Ah", var_name.name, str))
            }
            ParsedExpr::Var(_) => todo!(),
        }
        self.var_table.insert(var_name.name.to_string(), expr);
    }

    fn emit_print_expr(&mut self, expr: ParsedExpr) {
        match expr {
            ParsedExpr::NumericConstant(contents) => match contents {
                NumberContents::Integer(i) => self.emit_i32_print(i),
                NumberContents::Floating(_) => todo!(),
            },
            ParsedExpr::BinaryOp(_, _, _) => {
                match ParsedExpr::evaluate_expr(&expr, &self.var_table) {
                    NumberContents::Integer(i) => self.emit_i32_print(i),
                    NumberContents::Floating(_) => todo!(),
                }
            }
            ParsedExpr::QuotedString(str) => {
                self.emit_dataln(&format!("temp{} db '{}'", self.temporary_var_number, str));

                self.emit_textln(&format!("mov rdx, {}", str.len() + 1));
                self.emit_textln(&format!("mov rsi, temp{}", self.temporary_var_number));
                self.temporary_var_number += 1;

                self.emit_textln("call print_string");
                self.emit_textln("mov rdi, 10"); // newline
                self.emit_textln("call print_char");
            }
            ParsedExpr::Var(var_name) => {
                if let Some(var_value) = self.var_table.get(&var_name) {
                    match var_value {
                        ParsedExpr::NumericConstant(_) => todo!(),
                        ParsedExpr::BinaryOp(_, _, _) => todo!(),
                        ParsedExpr::QuotedString(_) => todo!(),
                        ParsedExpr::Var(_) => todo!(),
                    }
                }
            }
        }
    }

    fn emit_i32_print(&mut self, i: i64) {
        self.emit_textln(&format!("mov rdi, {}", i));
        self.emit_textln("call print_int");
        self.emit_textln("mov rdi, 10");
        self.emit_textln("call print_char");
    }
}

pub fn compile(statements: Vec<ParsedStatement>) {
    let mut emitter = Emitter::new();

    for statement in statements {
        match statement {
            ParsedStatement::VarDecl(var_name, expr) => {
                emitter.emit_var_declaration(var_name, expr);
            }
            ParsedStatement::Expression(_) => {}
            ParsedStatement::FunctionCall(name, exprs) => {
                if name.eq_ignore_ascii_case("print") {
                    for expr in exprs {
                        emitter.emit_print_expr(expr);
                    }
                }
            }
        }
    }

    emitter.emit_textln("mov rax, 60"); // sys_exit
    emitter.emit_textln("mov rdi, 0"); // return code
    emitter.emit_textln("syscall");

    write_asm_file(emitter);
    compile_asm_file();
    link_source();
    run_source();
    clean_up();
}

fn write_asm_file(emitter: Emitter) {
    let mut file = File::create("a.asm").expect("Failed to create output file.");
    file.write_all(
        &[
            emitter.output_data.as_bytes(),
            emitter.output_text.as_bytes(),
        ]
        .concat(),
    )
    .expect("Failed to write assembly to output file.");
}

fn compile_asm_file() {
    let compile_output = Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg("a.asm")
        .arg("-o")
        .arg("a.o")
        .output()
        .expect("Error assembling code.");
    print!(
        "{}{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let compile_lib = Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg("lib.asm")
        .arg("-o")
        .arg("lib.o")
        .output()
        .expect("Error assembling code.");
    print!(
        "{}{}",
        String::from_utf8_lossy(&compile_lib.stdout),
        String::from_utf8_lossy(&compile_lib.stderr)
    );
}

fn link_source() {
    let link_output = Command::new("ld")
        .arg("-m")
        .arg("elf_x86_64")
        .arg("a.o")
        .arg("lib.o")
        .arg("-o")
        .arg("a.out")
        .output()
        .expect("Error linking code.");

    print!(
        "{}{}",
        String::from_utf8_lossy(&link_output.stdout),
        String::from_utf8_lossy(&link_output.stderr)
    );
}

fn run_source() {
    let run_output = Command::new("./a.out")
        .output()
        .expect("Error running code.");
    print!(
        "{}{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
}

fn clean_up() {
    Command::new("rm")
        // .arg("a.asm")
        .arg("a.o")
        // .arg("a.out")
        .spawn()
        .expect("Failed to run cleanup.");
}
