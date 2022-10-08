use std::{fs::File, io::Write, process::Command};

use crate::parser::{BinaryOperator, ParsedExpr, ParsedStatement};

struct Emitter {
    output_data: String,
    output_text: String,
}

impl Emitter {
    fn new() -> Self {
        let mut emitter = Self {
            output_data: String::from("SECTION .data\n"),
            output_text: String::from("SECTION .text\n"),
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

    fn emit_expr(&mut self, expr: &ParsedExpr) {
        match expr {
            ParsedExpr::IntegerConstant(i) => {
                self.emit_textln(&format!("push {}", i));
            }
            ParsedExpr::FloatConstant(_) => todo!(),
            ParsedExpr::BinaryOp(expr1, op, expr2) => {
                self.emit_expr(&*expr1);
                self.emit_expr(&*expr2);
                match op {
                    BinaryOperator::Add => {
                        self.emit_textln("pop rax");
                        self.emit_textln("pop rbx");
                        self.emit_textln("add rax, rbx");
                        self.emit_textln("push rax");
                    }
                    BinaryOperator::Subtract => {
                        self.emit_textln("pop rax");
                        self.emit_textln("pop rbx");
                        self.emit_textln("sub rbx, rax");
                        self.emit_textln("push rbx");
                    },
                    BinaryOperator::Multiply => {
                        self.emit_textln("pop rax");
                        self.emit_textln("pop rbx");
                        self.emit_textln("imul rax, rbx");
                        self.emit_textln("push rax");
                    },
                    BinaryOperator::Divide => {
                        self.emit_textln("pop rbx");
                        self.emit_textln("pop rax");
                        self.emit_textln("xor rdx, rdx");
                        self.emit_textln("idiv rbx");
                        self.emit_textln("push rax");
                    },
                }
            }
            ParsedExpr::QuotedString(_) => todo!(),
            ParsedExpr::Var(name) => self.emit_textln(&format!("push QWORD [{}]", name)),
        }
    }
}

pub fn compile(statements: Vec<ParsedStatement>) {
    let mut emitter = Emitter::new();

    for statement in statements {
        match statement {
            ParsedStatement::VarDecl(decl, expr) => match expr {
                ParsedExpr::IntegerConstant(i) => {
                    emitter.emit_dataln(&format!("{} dq {}", decl.name, i))
                }
                ParsedExpr::FloatConstant(_) => todo!(),
                ParsedExpr::BinaryOp(_, _, _) => {
                    emitter.emit_dataln(&format!("{} dq 0", decl.name));
                    emitter.emit_expr(&expr);
                    emitter.emit_textln("pop rdi");
                    emitter.emit_textln(&format!("mov [{}], rdi", decl.name));
                }
                ParsedExpr::QuotedString(_) => todo!(),
                ParsedExpr::Var(_) => todo!(),
            },
            ParsedStatement::FunctionCall(function, args) => {
                if function == "print".to_string() {
                    emitter.emit_expr(&args[0]);

                    emitter.emit_textln("pop rdi");
                    emitter.emit_textln("call print_int");
                    emitter.emit_textln("mov rdi, 10"); // newline
                    emitter.emit_textln("call print_char");
                } else {
                    todo!()
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
