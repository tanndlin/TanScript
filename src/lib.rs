#![allow(clippy::missing_errors_doc)]

use clap::ValueEnum;

mod ast;
mod codegen;
pub mod diagnostic;
pub mod error;
mod lex;
mod parse;
mod preprocess;
pub mod source_map;
mod type_check;
mod types;

use crate::{
    error::CompilerError, lex::Lexer, parse::parse, preprocess::resolve_imports,
    source_map::SourceMap, type_check::check_types,
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, ValueEnum)]
pub enum Emit {
    Tokens,
    AST,
}

pub fn compile(
    file: u32,
    sources: &mut SourceMap,
    emit: Option<&Emit>,
) -> Result<String, CompilerError> {
    let lexer = Lexer::for_file(&sources.get(file).text, file)?;
    if let Some(Emit::Tokens) = emit {
        let tokens: Vec<String> = lexer.tokens.iter().rev().map(ToString::to_string).collect();
        return Ok(tokens.join("\n") + "\n");
    }

    let mut ast = parse(lexer)?;
    resolve_imports(&mut ast, file, sources)?;
    if let Some(Emit::AST) = emit {
        return Ok(format!("{ast}\n"));
    }

    check_types(&mut ast)?;

    Ok(ast.compile()?)
}
