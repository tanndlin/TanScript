use glob::glob;

use crate::{lexer::Lexer, parser::expr};

mod ast;
mod lexer;
mod parser;
mod types;

fn main() {
    let dir = std::env::current_dir().unwrap();
    println!("Current dir: {}", dir.display());

    let paths =
        glob(format!("{}/**/*.tan", dir.display()).as_str()).expect("Couldn't find script file");
    let file = paths.into_iter().next().expect("No file found").unwrap();
    let file_as_string = std::fs::read_to_string(file).unwrap();

    let ast = expr(&file_as_string);
    println!("{}", ast);
}
