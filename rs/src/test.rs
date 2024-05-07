use crate::ast;
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

test!(integration_parse_addition, "let a = 20 + 22;", 42);
test!(integration_parse_subtraction, "let a = 64-22;", 42);
test!(integration_parse_multiplication, "let a = 6 * 7;", 42);
test!(integration_parse_division, "let a = 84 / 2;", 42);
test!(integration_parse_modulus, "let a = 85 % 43;", 42);
test!(
    integration_follows_pemdas_mult_after,
    "let a = 2 + 2 * 10;",
    22
);
test!(
    integration_follows_pemdas_mult_before,
    "let a = 10 * 2 + 2;",
    22
);
test!(
    integration_parse_multiple_ops,
    "let a = 10 + 10 + 5 + 2;",
    27
);
