# TanScript

A statically typed, C-like language that compiles to x86-64 NASM assembly for Windows, written in Rust with Test Driven Development.

```
def numDivisors(n: u32): u32 {
    let total = 0;

    for (let i = 1; i * i <= n; i++) {
        if n % i != 0 {
            continue;
        }

        if i * i == n {
            total++;
        } else {
            total += 2;
        }
    }

    total
}

printf("%d\n", numDivisors(28));
```

## Features

- **Types**: `u8`, `i32`, `i64`, `u32`, `u64`, `char`, `bool`, pointers (`*T`), and structs. Variable types are inferred from their first assignment; function arguments require annotations.
- **Type Checking**: Mismatched types, wrong argument counts, and invalid casts are reported with the source location and compile stage.
- **Casting**: Explicit casts with `as` (`big as u8`, `address as *char`). Integers implicitly widen when no information is lost.
- **Variables**: `let` declarations, with optional annotations (`let c: char = 'a';`).
- **Control Flow**: `if`/`else`, `while`, `for`, `break`, and `continue`.
- **Functions**: `def name(arg: type): return_type { ... }`. The return type defaults to unit, and the last expression in the body is returned implicitly. `return` is also supported.
- **Structs**: `struct` definitions, `Name { field: value }` instantiation, and field access and assignment with `.`.
- **Pointers and Indexing**: Index any pointer with `[]`, e.g. memory from `malloc`.
- **Operators**: `+ - * / %`, `< <= > >= == !=`, `&& || !`, `++ --`, and `+= -= *= /= %=`.
- **Literals**: Numbers, `true`/`false`, chars (`'a'`, `'\n'`), and strings (`"hello\n"`).
- **Imports**: `use module::{name, other};` at the top of a file pulls functions and structs from `module.tan` (nested paths like `lib::math` map to `lib/math.tan`), resolved relative to the importing file. Anything the imported items depend on in that module comes along too, and the module's top-level statements are left out.
- **C Interop**: Calls to undefined functions are treated as externs, so C runtime functions like `printf`, `putchar`, `malloc`, and `fopen` work directly.

## Usage

Requires [Rust](https://rustup.rs), [NASM](https://nasm.us), and a MinGW `gcc` on your `PATH`.

```
cargo run -- script.tan     # writes script.asm
just asm                    # assembles, links, and runs script.asm
```

Inspect intermediate stages with `--emit`:

```
cargo run -- script.tan --emit tokens
cargo run -- script.tan --emit ast
```

## Testing

```
cargo test
```

Golden tests in `tests/golden` compile each `.tan` file, run the resulting executable, and compare its output to the matching `.out` file. Many of them are [Project Euler](https://projecteuler.net) solutions. Executables only run on Windows; on other platforms only compilation is checked.
