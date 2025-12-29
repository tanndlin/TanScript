use std::{env::args, fs, path::PathBuf};

mod ast;
mod compile;
mod compile_scope;
mod lexer;
mod parser;
mod register_handler;
mod symbol_table;
mod types;

use crate::parser::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file: PathBuf = args()
        .nth(1)
        .expect("Expected .tan file as positional arg")
        .into();
    let file_as_string = fs::read_to_string(&file)?;

    let ast = parse(&file_as_string)?;
    println!("{ast}");
    let code = ast.compile()?;

    let asm_path = file.with_extension("asm");
    fs::write(asm_path, code)?;

    Ok(())
}
