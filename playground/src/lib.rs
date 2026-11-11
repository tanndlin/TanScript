//! Minimal WebAssembly interface to the `TanScript` compiler.
//!
//! No wasm-bindgen: the host writes UTF-8 source into a buffer from [`alloc`],
//! calls [`compile`], then reads the result back with [`output_ptr`] and
//! [`output_len`]. `compile` returns 1 on success (output is assembly) and 0 on
//! a compile error (output is the rendered diagnostic).
//!
//! [`run`] takes assembly the same way and executes it in the emulator, with
//! `time` (Unix seconds) as the clock the program sees through `time()`. It
//! returns 1 if the program exited normally (output is its stdout, exit code
//! from [`exit_code`]) and 0 on a runtime error (output is stdout so far
//! followed by the error message).

use std::{cell::RefCell, fmt::Write, path::PathBuf};

use tanscript::{compile as compile_source, diagnostic::Style, source_map::SourceMap};
use tanscript_emulator::{Options, run as emulate};

thread_local! {
    static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
    static EXIT_CODE: RefCell<i32> = const { RefCell::new(0) };
}

/// Allocates `len` bytes for the host to write source text into.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Frees a buffer previously returned by [`alloc`].
///
/// # Safety
/// `ptr` and `len` must come from a single prior call to [`alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
}

/// Compiles the source in `ptr[..len]`; see the module docs for the protocol.
///
/// # Safety
/// `ptr[..len]` must be initialized memory obtained from [`alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn compile(ptr: *const u8, len: usize) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let source = String::from_utf8_lossy(bytes).into_owned();

    let mut sources = SourceMap::default();
    let file = sources.add(PathBuf::from("playground.tan"), source);
    let (ok, text) = match compile_source(file, &mut sources, None) {
        Ok(asm) => (1, asm),
        Err(e) => (0, Style::new(false).render(&e, &sources)),
    };

    OUTPUT.with_borrow_mut(|out| *out = text);
    ok
}

#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> *const u8 {
    OUTPUT.with_borrow(|out| out.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn output_len() -> usize {
    OUTPUT.with_borrow(String::len)
}

/// Runs the assembly in `ptr[..len]` in the emulator; see the module docs.
///
/// # Safety
/// `ptr[..len]` must be initialized memory obtained from [`alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn run(ptr: *const u8, len: usize, time: u32) -> u32 {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let asm = String::from_utf8_lossy(bytes);

    let options = Options {
        time: u64::from(time),
        ..Options::default()
    };
    let result = emulate(&asm, options);
    let mut text = result.stdout_lossy();
    let ok = match result.result {
        Ok(code) => {
            EXIT_CODE.with_borrow_mut(|c| *c = code);
            1
        }
        Err(e) => {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            write!(text, "runtime error: {e}").unwrap();
            0
        }
    };

    OUTPUT.with_borrow_mut(|out| *out = text);
    ok
}

#[unsafe(no_mangle)]
pub extern "C" fn exit_code() -> i32 {
    EXIT_CODE.with_borrow(|c| *c)
}
