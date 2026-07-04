mod token;
mod lexer;
mod ast;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<String> = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                } else {
                    eprintln!("mcc: -o requires an argument");
                    process::exit(1);
                }
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            arg if !arg.starts_with('-') => {
                if input_path.is_none() {
                    input_path = Some(arg);
                } else {
                    eprintln!("mcc: unexpected argument: {}", arg);
                    process::exit(1);
                }
            }
            unknown => {
                eprintln!("mcc: unknown flag: {}", unknown);
                process::exit(1);
            }
        }
        i += 1;
    }

    let input_path = match input_path {
        Some(p) => p,
        None => {
            eprintln!("Usage: mcc [-o <output>] <input.c>");
            eprintln!("Try 'mcc --help' for more information.");
            process::exit(1);
        }
    };

    let output_path = output_path.unwrap_or_else(|| input_path.replace(".c", ".s"));

    // Read source
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("mcc: error reading '{}': {}", input_path, e);
            process::exit(1);
        }
    };

    // Lex
    let mut lex = lexer::Lexer::new(&source);
    let tokens = match lex.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("mcc: lexer error: {}", e);
            process::exit(1);
        }
    };

    // Parse
    let mut parser = parser::Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("mcc: parse error: {}", e);
            process::exit(1);
        }
    };

    // Codegen
    let mut cg = codegen::CodeGen::new();
    let asm = cg.generate(&program);

    // Write output
    if let Err(e) = fs::write(&output_path, &asm) {
        eprintln!("mcc: error writing '{}': {}", output_path, e);
        process::exit(1);
    }

    eprintln!("Compiled '{}' -> '{}'", input_path, output_path);
}

fn print_help() {
    println!("mcc - a tiny C compiler targeting RISC-V 64");
    println!();
    println!("Usage: mcc [-o <output>] <input.c>");
    println!();
    println!("Options:");
    println!("  -o <output>    Write assembly to <output> (default: input.s)");
    println!("  -h, --help     Show this help message");
}
