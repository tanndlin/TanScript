use std::{env::args, fs, path::PathBuf};

mod ast;
mod codegen;
mod error;
mod lex;
mod parser;
mod type_check;
mod types;

use crate::{error::CompilerError, lex::Lexer, parser::parse, type_check::check_types};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let file: PathBuf = args()
        .nth(1)
        .expect("Expected .tan file as positional arg")
        .into();
    let file_as_string = fs::read_to_string(&file)?;

    let code = compile(&file_as_string)?;

    let asm_path = file.with_extension("asm");
    fs::write(asm_path, code)?;

    Ok(())
}

fn compile(source: &str) -> Result<String, CompilerError> {
    let lexer = Lexer::new(source)?;
    let mut ast = parse(lexer)?;
    check_types(&mut ast)?;

    println!("{ast}");
    Ok(ast.compile()?)
}
