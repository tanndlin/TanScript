use std::{fs, path::Path};

use glob::glob;

mod ast;
mod compile;
mod compile_scope;
mod lexer;
mod parser;
mod types;

use crate::parser::parse;

fn main() {
    let dir = std::env::current_dir().unwrap();
    println!("Current dir: {}", dir.display());

    let paths =
        glob(format!("{}/**/*.tan", dir.display()).as_str()).expect("Couldn't find script file");
    let file = paths.into_iter().next().expect("No file found").unwrap();
    let file_as_string = std::fs::read_to_string(file).unwrap();

    let ast = parse(&file_as_string);
    println!("{}", ast);

    match ast.compile() {
        Ok(code) => fs::write(Path::new("script.asm"), code).expect("Failed to write to file"),
        Err(e) => eprintln!("Error during compilation: {}", e),
    }
}
