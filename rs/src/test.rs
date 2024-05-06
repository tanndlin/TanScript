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

test!(parse_addition, "20 + 22", 42);
test!(parse_subtraction, "64-22", 42);
test!(parse_multiplication, "6 * 7", 42);
test!(parse_division, "84 / 2", 42);
test!(parse_modulus, "85 % 43", 42);
test!(follows_pemdas_mult_after, "2 + 2 * 10", 22);
test!(follows_pemdas_mult_before, "10 * 2 + 2", 22);
test!(parse_multiple_ops, "10 + 10 + 5 + 2", 27);
