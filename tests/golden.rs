use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

fn run_compiler(tan: &Path, work_dir: &Path) -> Result<PathBuf, String> {
    for entry in fs::read_dir(tan.parent().unwrap()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "tan") {
            fs::copy(&path, work_dir.join(path.file_name().unwrap())).unwrap();
        }
    }
    let copied = work_dir.join(tan.file_name().unwrap());

    let data_dir = tan.with_extension("");
    if data_dir.is_dir() {
        for entry in fs::read_dir(&data_dir).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), work_dir.join(entry.file_name())).unwrap();
        }
    }

    let output = Command::new(env!("CARGO_BIN_EXE_tanscript"))
        .arg(&copied)
        .output()
        .unwrap();

    if output.status.success() {
        Ok(copied.with_extension("asm"))
    } else {
        Err(format!(
            "compile failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn run_tool(cmd: &str, args: &[&Path]) -> Result<(), String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("failed to launch {cmd}: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{cmd} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn assemble_and_run(asm: &Path) -> Result<String, String> {
    let obj = asm.with_extension("o");
    let exe = asm.with_extension("exe");
    run_tool(
        "nasm",
        &[
            Path::new("-f"),
            Path::new("win64"),
            asm,
            Path::new("-o"),
            &obj,
        ],
    )?;
    run_tool("gcc", &[&obj, Path::new("-o"), &exe])?;

    let output = Command::new(&exe)
        .current_dir(asm.parent().unwrap())
        .output()
        .map_err(|e| format!("failed to run {}: {e}", exe.display()))?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_golden(tan: &Path, work_dir: &Path) -> Result<Option<String>, String> {
    let asm = run_compiler(tan, work_dir)?;
    if cfg!(windows) {
        assemble_and_run(&asm).map(Some)
    } else {
        Ok(None)
    }
}

fn normalize(s: &str) -> String {
    s.replace('\r', "")
}

fn check(tan: &Path, stdout: Option<String>, bless: bool) -> Result<(), String> {
    let Some(actual) = stdout else {
        return Ok(());
    };
    let path = tan.with_extension("out");

    if bless {
        fs::write(&path, &actual).unwrap();
        return Ok(());
    }

    let expected = fs::read_to_string(&path).map_err(|_| {
        format!(
            "missing {} (run with BLESS=1)\nactual:\n{actual}",
            path.display()
        )
    })?;

    if normalize(&expected) == normalize(&actual) {
        Ok(())
    } else {
        Err(format!(
            "mismatch in {}\n--- expected ---\n{expected}\n--- actual ---\n{actual}",
            path.display()
        ))
    }
}

#[test]
fn golden_scripts() {
    let bless = env::var_os("BLESS").is_some();
    let work_root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("golden")
        .join(std::process::id().to_string());

    let mut scripts: Vec<PathBuf> = fs::read_dir(GOLDEN_DIR)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "tan"))
        .collect();
    scripts.sort();
    assert!(
        !scripts.is_empty(),
        "no golden scripts found in {GOLDEN_DIR}"
    );

    let failures: Vec<String> = scripts
        .iter()
        .filter_map(|tan| {
            let work_dir = work_root.join(tan.file_stem().unwrap());
            let _ = fs::remove_dir_all(&work_dir);
            fs::create_dir_all(&work_dir).unwrap();

            run_golden(tan, &work_dir)
                .and_then(|stdout| check(tan, stdout, bless))
                .err()
                .map(|e| format!("[{}] {e}", tan.file_name().unwrap().display()))
        })
        .collect();

    let _ = fs::remove_dir_all(&work_root);

    assert!(
        failures.is_empty(),
        "{} golden script(s) failed:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
