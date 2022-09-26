use std::{fs::File, io::Write, process::Command};

use crate::parser::{ParsedExpr, ParsedStatement};

pub fn compile(statements: Vec<ParsedStatement>) {
    let mut output = String::from("; Program compiled using JPL compiler\n\n");
    output.push_str("SECTION .data\n");

    for statement in &statements {
        match statement {
            ParsedStatement::Expression(_) => {}
            ParsedStatement::VarDecl(var_name, expr) => match expr {
                ParsedExpr::NumericConstant(_) => todo!(),
                ParsedExpr::BinaryOp(_, _, _) => todo!(),
                ParsedExpr::QuotedString(str) => {
                    output.push_str(&format!("\t{} db '{}', 0Ah\n", var_name.name, str))
                }
                ParsedExpr::Var(_) => todo!(),
            },
            ParsedStatement::FunctionCall(_, _) => todo!(),
        }
    }
    output.push_str("\n");
    output.push_str("SECTION .text\n");
    output.push_str("global _start\n\n");
    output.push_str("_start:\n");

    for statement in &statements {
        match statement {
            ParsedStatement::Expression(_) => {}
            ParsedStatement::VarDecl(_, _) => {},
            ParsedStatement::FunctionCall(name, exprs) => {
                for expr in exprs {
                    match expr {
                        ParsedExpr::NumericConstant(_) => todo!(),
                        ParsedExpr::BinaryOp(_, _, _) => todo!(),
                        ParsedExpr::QuotedString(_) => todo!(),
                        ParsedExpr::Var(var_name) => {
                            output.push_str(&format!("\tmov edx, {}\n", var_name.len() + 1));
                            output.push_str(&format!("\tmov ecx, {}\n", var_name));
                        }
                    }
                }
            }
        }
    }
    output.push_str("\tmov ebx, 1\n");
    output.push_str("\tmov eax, 4\n");
    output.push_str("\tint 80h\n");
    output.push_str("\tmov ebx, 0\n");
    output.push_str("\tmov eax, 1\n");
    output.push_str("\tint 80h\n");

    let mut file = File::create("a.asm").expect("Failed to create output file.");
    file.write_all(output.as_bytes())
        .expect("Failed to write assembly to output file.");

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

    link_source();
    run_source();
    clean_up();
}

fn link_source() {
    let link_output = Command::new("ld")
        .arg("-m")
        .arg("elf_x86_64")
        .arg("a.o")
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
