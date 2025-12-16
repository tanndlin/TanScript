use std::{fs, path::Path};

use glob::glob;

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
    let dir = std::env::current_dir()?;

    let paths = glob(&format!("{}/**/*.tan", dir.display()))?;
    let file = paths.into_iter().next().ok_or("No file found")??;
    let file_as_string = fs::read_to_string(file)?;

    let ast = parse(&file_as_string).map_err(|e| {
        eprintln!("Error during parsing: {e}");
        e
    })?;

    println!("{ast}");

    let code = ast.compile().map_err(|e| {
        eprintln!("Error during compilation: {e}");
        e
    })?;

    fs::write(Path::new("script.asm"), code)?;

    Ok(())
}
