use clap::Parser;
use std::{
    env::args,
    fs,
    io::{IsTerminal, stderr},
    path::{Path, PathBuf},
};

mod ast;
mod codegen;
mod diagnostic;
mod error;
mod lex;
mod parse;
mod type_check;
mod types;

use crate::{
    diagnostic::Style, error::CompilerError, lex::Lexer, parse::parse, type_check::check_types,
};

#[derive(Parser)]
struct Args {
    file: String,
}

fn main() {
    let style = Style::new(stderr().is_terminal());
    if let Err(e) = run(&style) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(style: &Style) -> Result<(), String> {
    let Args { file } = Args::parse();

    let file_as_string = fs::read_to_string(&file)
        .map_err(|e| style.message(format!("could not read {file}: {e}")))?;

    let path: &Path = file.as_ref();
    let code = compile(&file_as_string).map_err(|e| style.render(&e, &file_as_string, path))?;

    let asm_path = path.with_extension("asm");
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
