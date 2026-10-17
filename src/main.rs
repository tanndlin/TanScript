use clap::Parser;
use std::{
    fs,
    io::{IsTerminal, stderr},
    path::Path,
};

use tanscript::{Emit, compile, diagnostic::Style, source_map::SourceMap};

#[derive(Parser)]
struct CliArgs {
    file: String,
    #[arg(short, long, ignore_case = true)]
    emit: Option<Emit>,
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
    let mut sources = SourceMap::default();
    let file_id = sources.add(path.to_path_buf(), file_as_string);
    let code =
        compile(file_id, &mut sources, emit.as_ref()).map_err(|e| style.render(&e, &sources))?;

    if emit.is_some() {
        print!("{code}");
        return Ok(());
    }

    let asm_path = path.with_extension("asm");
    fs::write(&asm_path, code)
        .map_err(|e| style.message(format!("could not write {}: {e}", asm_path.display())))?;

    Ok(())
}
