use std::{
    env::args,
    fs,
    io::{IsTerminal, stderr},
    path::PathBuf,
};

mod ast;
mod codegen;
mod diagnostic;
mod error;
mod lex;
mod parser;
mod type_check;
mod types;

use crate::{
    diagnostic::Style, error::CompilerError, lex::Lexer, parser::parse, type_check::check_types,
};

fn main() {
    let style = Style::new(stderr().is_terminal());
    if let Err(e) = run(&style) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(style: &Style) -> Result<(), String> {
    let file: PathBuf = args()
        .nth(1)
        .expect("Expected .tan file as positional arg")
        .into();
    let file_as_string = fs::read_to_string(&file)
        .map_err(|e| style.message(format!("could not read {}: {e}", file.display())))?;

    let code = compile(&file_as_string).map_err(|e| style.render(&e, &file_as_string, &file))?;

    let asm_path = file.with_extension("asm");
    fs::write(&asm_path, code)
        .map_err(|e| style.message(format!("could not write {}: {e}", asm_path.display())))?;

    Ok(())
}

fn compile(source: &str) -> Result<String, CompilerError> {
    let lexer = Lexer::new(source)?;
    let mut ast = parse(lexer)?;
    check_types(&mut ast)?;

    println!("{ast}");
    Ok(ast.compile()?)
}
