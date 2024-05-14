use crate::ast;
use crate::compiler::{compile_declare, compile_expression};
use crate::lexer::tokenize;
use crate::parser::parse;

macro_rules! test {
    ($name:ident, $script:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let tokens = tokenize($script);
            let root = parse(tokens);

            assert_eq!(root.evaluate(), $expected);
        }
    };
}

test!(integration_parse_addition, "int a = 20 + 22;", 42);
test!(integration_parse_subtraction, "int a = 64-22;", 42);
test!(integration_parse_multiplication, "int a = 6 * 7;", 42);
test!(integration_parse_division, "int a = 84 / 2;", 42);
test!(integration_parse_modulus, "int a = 85 % 43;", 42);
test!(
    integration_parse_parentheses1,
    "int a = ( 2 + 2 ) * 10;",
    40
);
test!(integration_parse_parentheses2, "int a = 10 * (2 + 2);", 40);
test!(
    integration_follows_pemdas_mult_after,
    "int a = 2 + 2 * 10;",
    22
);
test!(
    integration_follows_pemdas_mult_before,
    "int a = 10 * 2 + 2;",
    22
);
test!(
    integration_parse_multiple_ops,
    "int a = 10 + 10 + 5 + 2;",
    27
);

macro_rules! test_compile {
    ($name:ident, $compile_fn:ident, $script:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let tokens = tokenize($script);
            let root = parse(tokens);

            assert_eq!($compile_fn(&root.children[0]), $expected);
        }
    };
}

test_compile!(
    integration_compile_uninitialized,
    compile_declare,
    "int a;",
    "int a"
);

test_compile!(
    integration_compile_declare_float,
    compile_declare,
    "float a;",
    "float a"
);

test_compile!(
    integration_compile_addition,
    compile_declare,
    "int a = 20 + 22;",
    "int a = 20 + 22"
);

test_compile!(
    integration_compile_pointer,
    compile_declare,
    "int** a;",
    "int** a"
);
