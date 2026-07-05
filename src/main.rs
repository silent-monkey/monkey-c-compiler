mod ast;
mod codegen;
mod lexer;
mod parser;

use std::env;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mcc <input.mc> [-o <output.s>]");
        eprintln!();
        eprintln!("Compiles Monkey C source to RISC-V 64 assembly.");
        eprintln!();
        eprintln!("To compile and run a Monkey C program:");
        eprintln!("  mcc prog.mc -o prog.s");
        eprintln!("  llvm-mc -triple=riscv64 -filetype=obj prog.s -o prog.o --mattr=+d,+m");
        eprintln!("  clang --target=riscv64-linux-gnu --sysroot=/usr/riscv64-linux-gnu \\");
        eprintln!("      -static prog.o -o prog");
        eprintln!("  qemu-riscv64-static prog");
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        args[3].clone()
    } else {
        let p = Path::new(input_path);
        p.with_extension("s").to_string_lossy().to_string()
    };

    // Read source
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // Lex
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();

    if !parser.errors.is_empty() {
        eprintln!("Parse errors:");
        for err in &parser.errors {
            eprintln!("  {}", err);
        }
        process::exit(1);
    }

    // Generate code
    let mut codegen = codegen::CodeGen::new();
    let assembly = codegen.generate(&program);

    // Write output
    if let Err(e) = fs::write(&output_path, &assembly) {
        eprintln!("Error writing '{}': {}", output_path, e);
        process::exit(1);
    }

    eprintln!(
        "Compiled '{}' -> '{}' ({} bytes)",
        input_path,
        output_path,
        assembly.len()
    );
}
