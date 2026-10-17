use std::{
    fs,
    path::{Path, PathBuf},
    process,
};

use crate::{
    ast::{Program, Statement, StatementOrExpression},
    error::{CompilerError, CompilerErrorKind},
    lex::Lexer,
    parse::parse,
    preprocess::{PreprocessErrorKind, resolve_imports},
    source_map::SourceMap,
};

const MATH: &str = "
def square(n: u32): u32 { n * n }
def cube(n: u32): u32 { square(n) * n }
def unrelated(): u32 { 1 }
printf(\"math top level\");
";

fn write_files(test: &str, files: &[(&str, &str)]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tanscript-preprocess-{}-{test}", process::id()));
    let _ = fs::remove_dir_all(&dir);
    for (name, text) in files {
        let path = dir.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    dir
}

fn resolve(test: &str, files: &[(&str, &str)]) -> (Result<Program, CompilerError>, SourceMap) {
    let dir = write_files(test, files);
    let path = dir.join(files[0].0);
    let mut sources = SourceMap::default();
    let file = sources.add(path.clone(), fs::read_to_string(&path).unwrap());
    let mut program = parse(Lexer::for_file(&sources.get(file).text, file).unwrap()).unwrap();
    let result = resolve_imports(&mut program, file, &mut sources).map(|()| program);
    let _ = fs::remove_dir_all(&dir);
    (result, sources)
}

fn definitions(program: &Program) -> Vec<String> {
    program
        .block
        .children
        .iter()
        .filter_map(|child| match child {
            StatementOrExpression::Statement(s) => match s.as_ref() {
                Statement::FunctionDefinition(d) => Some(d.name.clone()),
                Statement::StructDefinition(d) => Some(d.name.clone()),
                _ => None,
            },
            StatementOrExpression::Expression(_) => None,
        })
        .collect()
}

fn error_kind(result: Result<Program, CompilerError>) -> PreprocessErrorKind {
    match result {
        Err(CompilerError {
            kind: CompilerErrorKind::Preprocess(kind),
            ..
        }) => kind,
        Err(e) => panic!("expected preprocess error, got {e}"),
        Ok(program) => panic!("expected preprocess error, got {program}"),
    }
}

#[test]
fn imports_requested_function() {
    let (result, _) = resolve(
        "requested",
        &[
            ("main.tan", "use math::{square};\nsquare(2);"),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square"]);
}

#[test]
fn imports_single_name_without_braces() {
    let (result, _) = resolve(
        "single",
        &[
            ("main.tan", "use math::square;\nsquare(2);"),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square"]);
}

#[test]
fn imports_transitive_dependencies_in_module_order() {
    let (result, _) = resolve(
        "transitive",
        &[
            ("main.tan", "use math::{cube};\ncube(2);"),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square", "cube"]);
}

#[test]
fn drops_top_level_statements_of_module() {
    let (result, _) = resolve(
        "top_level",
        &[("main.tan", "use math::{cube};"), ("math.tan", MATH)],
    );
    assert!(!result.unwrap().to_string().contains("math top level"));
}

#[test]
fn imported_items_come_before_own_code() {
    let (result, _) = resolve(
        "order",
        &[
            (
                "main.tan",
                "use math::{square};\ndef own(): u32 { square(3) }",
            ),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square", "own"]);
}

#[test]
fn imports_struct_used_by_function() {
    let (result, _) = resolve(
        "structs",
        &[
            ("main.tan", "use shapes::{origin};"),
            (
                "shapes.tan",
                "struct Point { x: u32, y: u32 }\ndef origin(): u32 { let p = Point { x: 0, y: 0 }; p.x }",
            ),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["Point", "origin"]);
}

#[test]
fn resolves_nested_module_paths() {
    let (result, _) = resolve(
        "nested",
        &[
            ("main.tan", "use lib::math::{square};"),
            ("lib/math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square"]);
}

#[test]
fn modules_resolve_their_own_imports() {
    let (result, _) = resolve(
        "chained",
        &[
            ("main.tan", "use geometry::{volume};"),
            (
                "geometry.tan",
                "use math::{cube};\ndef volume(n: u32): u32 { cube(n) }",
            ),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square", "cube", "volume"]);
}

#[test]
fn shared_dependencies_are_included_once() {
    let (result, _) = resolve(
        "diamond",
        &[
            ("main.tan", "use a::{fa};\nuse b::{fb};"),
            ("a.tan", "use math::{square};\ndef fa(): u32 { square(1) }"),
            ("b.tan", "use math::{square};\ndef fb(): u32 { square(2) }"),
            ("math.tan", MATH),
        ],
    );
    assert_eq!(definitions(&result.unwrap()), ["square", "fa", "fb"]);
}

#[test]
fn error_for_unknown_item() {
    let (result, _) = resolve(
        "unknown",
        &[("main.tan", "use math::{nope};"), ("math.tan", MATH)],
    );
    assert!(matches!(
        error_kind(result),
        PreprocessErrorKind::UnknownItem { name, .. } if name == "nope"
    ));
}

#[test]
fn error_for_missing_module() {
    let (result, _) = resolve("missing", &[("main.tan", "use nowhere::{x};")]);
    assert!(matches!(
        error_kind(result),
        PreprocessErrorKind::CouldNotRead { .. }
    ));
}

#[test]
fn error_for_circular_import() {
    let (result, _) = resolve(
        "circular",
        &[
            ("main.tan", "use a::{fa};"),
            ("a.tan", "use b::{fb};\ndef fa(): u32 { 1 }"),
            ("b.tan", "use a::{fa};\ndef fb(): u32 { 2 }"),
        ],
    );
    assert!(matches!(
        error_kind(result),
        PreprocessErrorKind::CircularImport(_)
    ));
}

#[test]
fn error_for_redefining_imported_name() {
    let (result, _) = resolve(
        "redefine",
        &[
            (
                "main.tan",
                "use math::{square};\ndef square(n: u32): u32 { n }",
            ),
            ("math.tan", MATH),
        ],
    );
    assert!(matches!(
        error_kind(result),
        PreprocessErrorKind::DuplicateDefinition(name) if name == "square"
    ));
}

#[test]
fn error_for_conflicting_imports() {
    let (result, _) = resolve(
        "conflict",
        &[
            ("main.tan", "use a::{f};\nuse b::{f};"),
            ("a.tan", "def f(): u32 { 1 }"),
            ("b.tan", "def f(): u32 { 2 }"),
        ],
    );
    assert!(matches!(
        error_kind(result),
        PreprocessErrorKind::DuplicateDefinition(name) if name == "f"
    ));
}

#[test]
fn errors_in_modules_point_at_module_file() {
    let (result, sources) = resolve(
        "module_error",
        &[
            ("main.tan", "use bad::{f};"),
            ("bad.tan", "\ndef f(: u32 { 1 }"),
        ],
    );
    let error = result.unwrap_err();
    assert!(matches!(error.kind, CompilerErrorKind::Parse(_)));
    assert_eq!(
        sources.get(error.span.file).path.file_name(),
        Path::new("bad.tan").file_name()
    );
    assert_eq!(error.span.line, 1);
}
