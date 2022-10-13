use std::{fs::File, io::Write, process::Command};

use crate::parser::{BinaryOperator, ParsedExpr, ParsedStatement, Scope};

#[derive(Clone, Copy)]
enum EmitLocation {
    Data,
    Text,
    Main,
}

struct Emitter {
    output_data: String,
    output_text: String,
    output_main: String,
}

impl Emitter {
    fn new() -> Self {
        let mut emitter = Self {
            output_data: String::from("SECTION .data\n"),
            output_text: String::from("SECTION .text\n"),
            output_main: String::from("\n"),
        };

        emitter.emit(EmitLocation::Text, "extern print_int");
        emitter.emit(EmitLocation::Text, "extern print_char");
        emitter.emit(EmitLocation::Text, "extern print_string");
        emitter.emit(EmitLocation::Text, "global _start");
        emitter.emit(EmitLocation::Main, "_start:");

        emitter
    }

    fn emit(&mut self, loc: EmitLocation, asm: &str) {
        match loc {
            EmitLocation::Data => self.output_data.push_str(&format!("\t{}\n", asm)),
            EmitLocation::Text => self.output_text.push_str(&format!("\t{}\n", asm)),
            EmitLocation::Main => self.output_main.push_str(&format!("\t{}\n", asm)),
        }
    }

    fn emit_expr(&mut self, expr: &ParsedExpr) {
        match expr {
            ParsedExpr::IntegerConstant(i) => {
                self.emit(EmitLocation::Text, &format!("push {}", i));
            }
            ParsedExpr::FloatConstant(_) => todo!(),
            ParsedExpr::BinaryOp(expr1, op, expr2) => {
                self.emit_expr(&*expr1);
                self.emit_expr(&*expr2);
                match op {
                    BinaryOperator::Add => {
                        self.emit(EmitLocation::Text, "pop rax");
                        self.emit(EmitLocation::Text, "pop rbx");
                        self.emit(EmitLocation::Text, "add rax, rbx");
                        self.emit(EmitLocation::Text, "push rax");
                    }
                    BinaryOperator::Subtract => {
                        self.emit(EmitLocation::Text, "pop rax");
                        self.emit(EmitLocation::Text, "pop rbx");
                        self.emit(EmitLocation::Text, "sub rbx, rax");
                        self.emit(EmitLocation::Text, "push rbx");
                    }
                    BinaryOperator::Multiply => {
                        self.emit(EmitLocation::Text, "pop rax");
                        self.emit(EmitLocation::Text, "pop rbx");
                        self.emit(EmitLocation::Text, "imul rax, rbx");
                        self.emit(EmitLocation::Text, "push rax");
                    }
                    BinaryOperator::Divide => {
                        self.emit(EmitLocation::Text, "pop rbx");
                        self.emit(EmitLocation::Text, "pop rax");
                        self.emit(EmitLocation::Text, "xor rdx, rdx");
                        self.emit(EmitLocation::Text, "idiv rbx");
                        self.emit(EmitLocation::Text, "push rax");
                    }
                }
            }
            ParsedExpr::QuotedString(_) => todo!(),
            ParsedExpr::Var(name) => {
                self.emit(EmitLocation::Text, &format!("push QWORD [{}]", name))
            }
            ParsedExpr::FunctionCall(function_name, args) => {
                self.emit_function_call(function_name, args, Scope::empty())
            }
        }
    }

    fn emit_function_call(&mut self, function_name: &String, args: &Vec<ParsedExpr>, scope: Scope) {
        println!("scope: {:?}", scope);
        let emit_location = if scope.caller == "" {
            EmitLocation::Main
        } else {
            EmitLocation::Text
        };

        if function_name == "print" {
            self.emit_expr(&args[0]);

            self.emit(emit_location, "pop rdi");
            self.emit(emit_location, "call print_int");
            self.emit(emit_location, "mov rdi, 10"); // newline
            self.emit(emit_location, "call print_char");
        } else {
            let len = args.len();

            // load first arg into first call register
            if len >= 1 {
                self.emit_expr(&args[0]);
                self.emit(emit_location, "pop rdi");
            }

            self.emit(emit_location, &format!("call {}", function_name));
        }
    }

    fn emit_binary_operation_declaration(&mut self, var_name: &str, var_expr: &ParsedExpr) {
        self.emit(EmitLocation::Data, &format!("{} dq 0", var_name));
        self.emit_expr(var_expr);
        self.emit(EmitLocation::Text, "pop rdi");
        self.emit(EmitLocation::Text, &format!("mov [{}], rdi", var_name));
    }
}

pub fn compile(statements: Vec<ParsedStatement>) {
    let mut emitter = Emitter::new();

    for statement in statements {
        match statement {
            ParsedStatement::VarDecl(name, expr) => match expr {
                ParsedExpr::IntegerConstant(i) => {
                    emitter.emit(EmitLocation::Data, &format!("{} dq {}", name, i))
                }
                ParsedExpr::FloatConstant(_) => todo!(),
                ParsedExpr::BinaryOp(_, _, _) => {
                    emitter.emit_binary_operation_declaration(&name, &expr)
                }
                ParsedExpr::QuotedString(_) => todo!(),
                ParsedExpr::Var(_) => todo!(),
                ParsedExpr::FunctionCall(function_name, args) => {
                    emitter.emit_function_call(&function_name, &args, Scope::empty())
                }
            },
            ParsedStatement::FunctionCall(function_name, args, scope) => {
                emitter.emit_function_call(&function_name, &args, scope)
            }
            ParsedStatement::ReturnStatement(_, _) => todo!(),
            ParsedStatement::FunctionDeclaration(name, arg, stmts) => {
                emitter.emit(EmitLocation::Text, &format!("{}:", name));
                for stmt in &stmts {
                    match stmt {
                        ParsedStatement::VarDecl(_, _) => todo!(),
                        ParsedStatement::FunctionCall(_, _, _) => todo!(),
                        ParsedStatement::FunctionDeclaration(_, _, _) => todo!(),
                        ParsedStatement::ReturnStatement(expr, scope) => {
                            emitter.emit_expr(expr);
                        },
                    }
                }
            }
        }
    }

    emitter.emit(EmitLocation::Main, "mov rax, 60"); // sys_exit
    emitter.emit(EmitLocation::Main, "mov rdi, 0"); // return code
    emitter.emit(EmitLocation::Main, "syscall");

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
            emitter.output_main.as_bytes(),
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
