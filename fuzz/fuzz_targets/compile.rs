#![no_main]

use std::{
    cell::RefCell,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    panic,
    path::{Path, PathBuf},
    sync::Once,
};

use libfuzzer_sys::fuzz_target;
use tanscript::{compile, source_map::SourceMap};

thread_local! {
    static INPUT: RefCell<String> = const { RefCell::new(String::new()) };
}

static HOOK: Once = Once::new();

fuzz_target!(|source: &str| {
    if cfg!(windows) {
        HOOK.call_once(|| {
            let previous = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                INPUT.with_borrow(|input| save_crash(input));
                previous(info);
            }));
        });
        INPUT.with_borrow_mut(|input| {
            input.clear();
            input.push_str(source);
        });
    }

    let mut sources = SourceMap::default();
    let file = sources.add(PathBuf::from("fuzz.tan"), source.to_string());
    let _ = compile(file, &mut sources, None);
});

fn save_crash(source: &str) {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);

    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("compile");
    let path = dir.join(format!("crash-{:016x}.tan", hasher.finish()));
    let _ = fs::create_dir_all(&dir);
    if fs::write(&path, source).is_ok() {
        eprintln!("saved crashing input to {}", path.display());
    }
}
