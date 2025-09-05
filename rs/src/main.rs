use glob::glob;

use crate::lexer::Lexer;

mod ast;
mod lexer;
mod parser;
mod types;

fn main() {
    let dir = std::env::current_dir().unwrap();
    println!("Current dir: {}", dir.display());

    let paths =
        glob(format!("{}/*.tan", dir.display()).as_str()).expect("Couldn't find script file");
    let file = paths.into_iter().next().expect("No file found").unwrap();
    let chars = std::fs::read_to_string(file)
        .unwrap()
        .chars()
        .collect::<Vec<char>>();

    let mut lexer = Lexer::new(chars.as_slice());
    let tokens = lexer.tokenize();

    dbg!(tokens);
}
