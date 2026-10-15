use clap::{Parser, ValueEnum};
use std::{
    fs,
    io::{IsTerminal, stderr},
    path::Path,
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
struct CliArgs {
    file: String,
    #[arg(short, long, ignore_case = true)]
    emit: Option<Emit>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, ValueEnum)]
enum Emit {
    Tokens,
    AST,
}

fn main() {
    let style = Style::new(stderr().is_terminal());
    if let Err(e) = run(&style) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(style: &Style) -> Result<(), String> {
    let CliArgs { file, emit } = CliArgs::parse();

    let file_as_string = fs::read_to_string(&file)
        .map_err(|e| style.message(format!("could not read {file}: {e}")))?;

    let path: &Path = file.as_ref();
    let code = compile(&file_as_string, emit.as_ref())
        .map_err(|e| style.render(&e, &file_as_string, path))?;

    if emit.is_some() {
        print!("{code}");
        return Ok(());
    }

    let asm_path = path.with_extension("asm");
    fs::write(&asm_path, code)
        .map_err(|e| style.message(format!("could not write {}: {e}", asm_path.display())))?;

    Ok(())
}

fn compile(source: &str, emit: Option<&Emit>) -> Result<String, CompilerError> {
    let lexer = Lexer::new(source)?;
    if let Some(Emit::Tokens) = emit {
        let tokens: Vec<String> = lexer.tokens.iter().rev().map(ToString::to_string).collect();
        return Ok(tokens.join("\n") + "\n");
    }

    let mut ast = parse(lexer)?;
    if let Some(Emit::AST) = emit {
        return Ok(format!("{ast}\n"));
    }

    check_types(&mut ast)?;

    Ok(ast.compile()?)
}
