use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use tanscript::{compile, source_map::SourceMap};
use tanscript_emulator::{Options, run};

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/golden");

fn read_dir_files(dir: &Path, keep: impl Fn(&Path) -> bool) -> Vec<(String, Vec<u8>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file() && keep(p))
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read(&p).unwrap(),
            )
        })
        .collect()
}

fn data_files(tan: &Path) -> HashMap<String, Vec<u8>> {
    let mut files = read_dir_files(tan.parent().unwrap(), |p| {
        p.extension().is_some_and(|e| e == "tan")
    });
    files.extend(read_dir_files(&tan.with_extension(""), |_| true));
    files.into_iter().collect()
}

#[test]
fn golden_scripts_match_native_output() {
    let mut scripts: Vec<PathBuf> = fs::read_dir(GOLDEN_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "tan"))
        .collect();
    scripts.sort();
    if let Ok(filter) = std::env::var("GOLDEN_FILTER") {
        scripts.retain(|p| p.to_string_lossy().contains(&filter));
    }

    let mut failures = Vec::new();
    let mut skipped = Vec::new();
    for tan in &scripts {
        let name = tan.file_name().unwrap().to_string_lossy().into_owned();
        let Ok(expected) = fs::read_to_string(tan.with_extension("out")) else {
            continue;
        };
        if !read_dir_files(&tan.with_extension(""), |p| {
            p.extension().is_some_and(|e| e == "c")
        })
        .is_empty()
        {
            skipped.push(name);
            continue;
        }

        let mut sources = SourceMap::default();
        let file = sources.add(tan.clone(), fs::read_to_string(tan).unwrap());
        let Ok(asm) = compile(file, &mut sources, None) else {
            failures.push(format!("[{name}] compile failed"));
            continue;
        };

        let result = run(
            &asm,
            Options {
                files: data_files(tan),
                ..Options::default()
            },
        );
        let actual = result.stdout_lossy().replace('\r', "");
        let expected = expected.replace('\r', "");
        match result.result {
            Err(e) => failures.push(format!("[{name}] {e}\n--- stdout so far ---\n{actual}")),
            Ok(_) if actual != expected => failures.push(format!(
                "[{name}] mismatch\n--- expected ---\n{expected}\n--- actual ---\n{actual}"
            )),
            Ok(_) => {}
        }
    }

    eprintln!("skipped (custom C externs): {skipped:?}");
    assert!(
        failures.is_empty(),
        "{} of {} golden script(s) failed:\n\n{}",
        failures.len(),
        scripts.len(),
        failures.join("\n\n")
    );
}
