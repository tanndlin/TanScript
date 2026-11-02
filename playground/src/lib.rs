//! Minimal WebAssembly interface to the `TanScript` compiler.
//!
//! No wasm-bindgen: the host writes UTF-8 source into a buffer from [`alloc`],
//! calls [`compile`], then reads the result back with [`output_ptr`] and
//! [`output_len`]. `compile` returns 1 on success (output is assembly) and 0 on
//! a compile error (output is the rendered diagnostic).

use std::{cell::RefCell, path::PathBuf};

use tanscript::{compile as compile_source, diagnostic::Style, source_map::SourceMap};

thread_local! {
    static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
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
