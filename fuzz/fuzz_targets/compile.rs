#![no_main]

use std::path::PathBuf;

use libfuzzer_sys::fuzz_target;
use tanscript::{compile, source_map::SourceMap};

fuzz_target!(|source: &str| {
    let mut sources = SourceMap::default();
    let file = sources.add(PathBuf::from("fuzz.tan"), source.to_string());
    let _ = compile(file, &mut sources, None);
});
