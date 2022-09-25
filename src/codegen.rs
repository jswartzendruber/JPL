use std::{fs::File, io::Write, process::Command};

use crate::parser::ParsedStatement;

pub fn compile(statements: Vec<ParsedStatement>) {
    let output = String::from(
        "; Program compiled using JPL compiler

SECTION .data
    message db 'Hello JPL!', 0Ah

SECTION .text
global _start

_start:
    mov edx, 11
    mov ecx, message
    mov ebx, 1
    mov eax, 4
    int 80h
    mov ebx, 0
    mov eax, 1
    int 80h",
    );

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
