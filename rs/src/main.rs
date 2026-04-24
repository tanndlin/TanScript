use inkwell::context::Context;
use std::process::Command;
use std::{env::args, fs, path::PathBuf};

mod ast;
mod lexer;
mod llvm;
mod parser;
mod symbol_table;
mod types;

use crate::llvm::Compiler;
use crate::parser::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file: PathBuf = args()
        .nth(1)
        .expect("Expected .tan file as positional arg")
        .into();
    let file_as_string = fs::read_to_string(&file)?;

    let ast = parse(&file_as_string)?;
    println!("{ast}");

    let context = Context::create();
    let mut compiler = Compiler::new(&context);
    compiler.compile_program(&ast)?;
    let module = compiler.module;

    // Emit LLVM IR to stdout
    module.print_to_stderr();

    // Or write to a .ll file
    std::fs::create_dir_all("out").unwrap();
    module.print_to_file("out/output.ll").unwrap();

    Command::new("clang-17")
        .args(["out/output.ll", "-o", "out/output"])
        .status()
        .expect("failed to invoke clang");

    // Run the compiled binary
    let result = Command::new("./out/output")
        .status()
        .expect("failed to run the compiled binary");

    println!("result: {}", result.code().unwrap());

    Ok(())
}
