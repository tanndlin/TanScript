use crate::{
    ast::{AtomType, ExpressionKind, Statement, StatementOrExpression},
    lex::Lexer,
    parse::{ParseErrorKind, parse, parse_expression::parse_expression},
    type_check::Type,
};

macro_rules! test_parse_expression {
    ($input:expr, $expected:expr) => {
        let mut lexer = Lexer::new($input).unwrap();
        let s = parse_expression(&mut lexer, 0).unwrap();
        assert_eq!(s.to_string(), $expected);
    };
}

macro_rules! integration_test {
    ($input:expr, $expected:expr) => {
        let program = parse(Lexer::new($input).unwrap()).unwrap();
        assert_eq!(program.to_string(), $expected);
    };
}

#[test]
fn parse_basic_math() {
    test_parse_expression!("1", "1");
    test_parse_expression!("1 + 2 * 3", "(+ 1 (* 2 3))");
    test_parse_expression!("a + b * c * d + e", "(+ (+ a (* (* b c) d)) e)");
}

#[test]
fn parse_cast() {
    test_parse_expression!("a as i32", "(a as i32)");
    test_parse_expression!("a + b as i32 * c", "(+ a (* (b as i32) c))");
    test_parse_expression!("p as *char", "(p as *char)");
    test_parse_expression!("a as i64 as char", "((a as i64) as char)");
}

#[test]
fn parse_negative_numbers() {
    test_parse_expression!("-9", "(- 9)");
}

#[test]
fn parse_parentheses() {
    test_parse_expression!("(1 + 2) * 3", "(* (+ 1 2) 3)");
}

#[test]
fn parse_assignment() {
    integration_test!("a = 1;", "a = 1");
    integration_test!("a = 1 + 2;", "a = (+ 1 2)");
}

#[test]
fn parse_add_assign() {
    integration_test!("a += 1;", "a = (+ a 1)");
}

#[test]
fn parse_sub_assign() {
    integration_test!("a -= 1;", "a = (- a 1)");
}

#[test]
fn parse_mul_assign() {
    integration_test!("a *= 2;", "a = (* a 2)");
}

#[test]
fn parse_div_assign() {
    integration_test!("a /= 2;", "a = (/ a 2)");
}

#[test]
fn parse_mod_assign() {
    integration_test!("a %= 2;", "a = (% a 2)");
}

#[test]
fn parse_compound_assign_groups_right_side() {
    // The whole right side is the operand, not just the first term
    integration_test!("a -= 1 + 2;", "a = (- a (+ 1 2))");
    integration_test!("a += b * 2;", "a = (+ a (* b 2))");
    integration_test!("a -= b - c;", "a = (- a (- b c))");
    integration_test!("a *= 1 + 2;", "a = (* a (+ 1 2))");
    integration_test!("a /= b - c;", "a = (/ a (- b c))");
    integration_test!("a %= b + 1;", "a = (% a (+ b 1))");
}

#[test]
fn parse_compound_assign_function_call() {
    integration_test!("a += add(1, 2);", "a = (+ a add(1, 2))");
}

#[test]
fn parse_compound_assign_in_while_loop() {
    integration_test!(
        "while (a < 10) { a += 1; }",
        "while (< a 10) {\na = (+ a 1)\n}"
    );
}

#[test]
fn parse_compound_assign_in_if_else() {
    integration_test!(
        "if (a < 10) { a += 1; } else { a -= 1; }",
        "if (< a 10) {\na = (+ a 1)\n} else {\na = (- a 1)\n}"
    );
}

#[test]
fn reject_compound_assign_in_declaration() {
    let err = parse(Lexer::new("let a += 1;").unwrap()).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "="));
    let err = parse(Lexer::new("let a -= 1;").unwrap()).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "="));
}

#[test]
fn expect_compound_assign_semicolon() {
    let program = parse(Lexer::new("a += 1 b -= 2;").unwrap()).unwrap_err();
    assert!(
        matches!(program.kind, ParseErrorKind::Expected { ref expected, .. } if expected == ";")
    );
}

#[test]
fn parse_declaration() {
    integration_test!("let a = 1;", "let a = 1");
    integration_test!("let a = 1 + 2;", "let a = (+ 1 2)");
}

#[test]
fn expect_statement_semicolon() {
    let program = parse(Lexer::new("let a = 1 let b = 2;").unwrap()).unwrap_err();
    assert!(
        matches!(program.kind, ParseErrorKind::Expected { ref expected, .. } if expected == ";")
    );
}

#[test]
fn parse_modulo() {
    test_parse_expression!("5 % 2", "(% 5 2)");
}

#[test]
fn parse_less_than() {
    test_parse_expression!("1 < 2", "(< 1 2)");
}

#[test]
fn parse_less_than_or_equal() {
    test_parse_expression!("1 <= 2", "(<= 1 2)");
}

#[test]
fn parse_greater_than() {
    test_parse_expression!("1 > 2", "(> 1 2)");
}

#[test]
fn parse_greater_than_or_equal() {
    test_parse_expression!("1 >= 2", "(>= 1 2)");
}

#[test]
fn parse_equals() {
    test_parse_expression!("1 == 2", "(== 1 2)");
}

#[test]
fn parse_not_equal() {
    test_parse_expression!("1 != 2", "(!= 1 2)");
}

#[test]
fn parse_logical_or() {
    test_parse_expression!("1 || 0", "(|| 1 0)");
}

#[test]
fn parse_logical_and() {
    test_parse_expression!("1 && 0", "(&& 1 0)");
}

#[test]
fn parse_increment() {
    test_parse_expression!("a++", "(++ a)");
}

#[test]
fn parse_decrement() {
    test_parse_expression!("a--", "(-- a)");
}

#[test]
fn parse_increment_decrement_binds_tighter_than_infix() {
    test_parse_expression!("a++ + 1", "(+ (++ a) 1)");
    test_parse_expression!("1 + a++", "(+ 1 (++ a))");
    test_parse_expression!("a-- * 2", "(* (-- a) 2)");
    test_parse_expression!("a++ < b--", "(< (++ a) (-- b))");
}

#[test]
fn parse_increment_binds_tighter_than_prefix() {
    test_parse_expression!("-a++", "(- (++ a))");
    test_parse_expression!("!a--", "(! (-- a))");
}

#[test]
fn parse_increment_followed_by_add() {
    test_parse_expression!("a+++b", "(+ (++ a) b)");
}

#[test]
fn parse_increment_decrement_statement() {
    integration_test!("a++;", "(++ a)");
    integration_test!("a--;", "(-- a)");
}

#[test]
fn parse_increment_in_declaration() {
    integration_test!("let b = a++;", "let b = (++ a)");
}

#[test]
fn parse_increment_in_assignment() {
    integration_test!("b = a--;", "b = (-- a)");
}

#[test]
fn parse_increment_as_function_arg() {
    test_parse_expression!("foo(a++)", "foo((++ a))");
    test_parse_expression!("foo(a++, b--)", "foo((++ a), (-- b))");
}

#[test]
fn parse_increment_in_while_loop() {
    integration_test!("while (i < 10) { i++; }", "while (< i 10) {\n(++ i)\n}");
}

#[test]
fn expect_increment_statement_semicolon() {
    let program = parse(Lexer::new("a++ b--;").unwrap()).unwrap_err();
    assert!(
        matches!(program.kind, ParseErrorKind::Expected { ref expected, .. } if expected == ";")
    );
}

#[test]
fn reject_stray_comma() {
    let err = parse(
        Lexer::new(
            "if (true) {,
}",
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(
        matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "expression")
    );
}

#[test]
fn reject_colon_as_expression() {
    let err = parse(Lexer::new("let a = :;").unwrap()).unwrap_err();
    assert!(
        matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "expression")
    );
}

#[test]
fn reject_stray_semicolon() {
    let err = parse(Lexer::new("let a = 1;;").unwrap()).unwrap_err();
    assert!(
        matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "expression")
    );
}

#[test]
fn reject_increment_on_non_identifier() {
    assert!(parse(Lexer::new("5++;").unwrap()).is_err());
    assert!(parse(Lexer::new("(a + b)--;").unwrap()).is_err());
    assert!(parse(Lexer::new("foo()++;").unwrap()).is_err());
}

#[test]
fn parse_boolean_negate() {
    test_parse_expression!("!1", "(! 1)");
    test_parse_expression!("!(1 < 2)", "(! (< 1 2))");
}

#[test]
fn parse_while_loop() {
    integration_test!(
        "while (a < 10) { a = a + 1; }",
        "while (< a 10) {\na = (+ a 1)\n}"
    );
}

#[test]
fn parse_if_statement() {
    integration_test!(
        "if (a < 10) { a = a + 1; }",
        "if (< a 10) {\na = (+ a 1)\n}"
    );
}

#[test]
fn parse_if_else_statement() {
    integration_test!(
        "if (a < 10) { a = a + 1; } else { a = a - 1; }",
        "if (< a 10) {\na = (+ a 1)\n} else {\na = (- a 1)\n}"
    );
}

#[test]
fn parse_function_definition() {
    integration_test!(
        "def add(a: i32, b: i32) { return a + b; }",
        "def add(a: i32, b: i32) {return (+ a b)}"
    );
}

#[test]
fn parse_function_definition_with_return_type() {
    integration_test!(
        "def add(a: i32, b: i32): i32 { return a + b; }",
        "def add(a: i32, b: i32): i32 {return (+ a b)}"
    );
}

#[test]
fn parse_function_definition_no_args_with_return_type() {
    integration_test!("def one(): u64 { 1 }", "def one(): u64 {return 1}");
}

#[test]
fn parse_function_call() {
    test_parse_expression!("add(1, 2)", "add(1, 2)");
    test_parse_expression!("add(a, b + c)", "add(a, (+ b c))");
}

#[test]
fn parse_function_call_no_args() {
    test_parse_expression!("foo()", "foo()");
}

#[test]
fn parse_nested_function_calls() {
    test_parse_expression!("add(mul(2, 3), 4)", "add(mul(2, 3), 4)");
}

#[test]
fn parse_function_implicit_return() {
    integration_test!(
        "def add(a: i32, b: i32) { a + b }",
        "def add(a: i32, b: i32) {return (+ a b)}"
    );
}

#[test]
fn parse_function_empty_return() {
    integration_test!("def f() { return; }", "def f() {return}");
}

#[test]
fn parse_index() {
    test_parse_expression!("a[0]", "(a[0])");
    test_parse_expression!("a [ 0 ]", "(a[0])");
}

#[test]
fn parse_index_with_expression() {
    test_parse_expression!("a[i]", "(a[i])");
    test_parse_expression!("a[i + 1]", "(a[(+ i 1)])");
    test_parse_expression!("a[(i + 1) * 2]", "(a[(* (+ i 1) 2)])");
}

#[test]
fn parse_chained_index() {
    test_parse_expression!("a[0][1]", "((a[0])[1])");
    test_parse_expression!("a[0][1][2]", "(((a[0])[1])[2])");
}

#[test]
fn parse_nested_index() {
    test_parse_expression!("a[b[0]]", "(a[(b[0])])");
    test_parse_expression!("a[b[c[0]]]", "(a[(b[(c[0])])])");
}

#[test]
fn parse_index_binds_tighter_than_infix() {
    test_parse_expression!("a[0] + 1", "(+ (a[0]) 1)");
    test_parse_expression!("1 + a[0]", "(+ 1 (a[0]))");
    test_parse_expression!("a[0] + b[1]", "(+ (a[0]) (b[1]))");
    test_parse_expression!("1 + a[0] * 2", "(+ 1 (* (a[0]) 2))");
    test_parse_expression!("a[0] < b[1] && c[2]", "(&& (< (a[0]) (b[1])) (c[2]))");
}

#[test]
fn parse_index_binds_tighter_than_prefix() {
    test_parse_expression!("-a[0]", "(- (a[0]))");
    test_parse_expression!("!a[0]", "(! (a[0]))");
}

#[test]
fn parse_index_on_parenthesized_expression() {
    test_parse_expression!("(a)[0]", "(a[0])");
    test_parse_expression!("(a + b)[0]", "((+ a b)[0])");
}

#[test]
fn parse_index_on_function_call() {
    test_parse_expression!("foo()[0]", "(foo()[0])");
    test_parse_expression!("foo(1, 2)[i]", "(foo(1, 2)[i])");
}

#[test]
fn parse_index_as_function_arg() {
    test_parse_expression!("foo(a[0])", "foo((a[0]))");
    test_parse_expression!("foo(a[0], b[i + 1])", "foo((a[0]), (b[(+ i 1)]))");
}

#[test]
fn parse_increment_on_index() {
    test_parse_expression!("a[0]++", "(++ (a[0]))");
    test_parse_expression!("a[i]--", "(-- (a[i]))");
    integration_test!("a[0]++;", "(++ (a[0]))");
}

#[test]
fn parse_increment_inside_index() {
    test_parse_expression!("a[i++]", "(a[(++ i)])");
}

#[test]
fn parse_index_in_declaration() {
    integration_test!("let x = a[0];", "let x = (a[0])");
    integration_test!(
        "let x = a[i] + a[i + 1];",
        "let x = (+ (a[i]) (a[(+ i 1)]))"
    );
}

#[test]
fn parse_assign_to_index() {
    integration_test!("a[0] = 5;", "(a[0]) = 5");
    integration_test!("a[i] = a[i] + 1;", "(a[i]) = (+ (a[i]) 1)");
    integration_test!("a[0][1] = 5;", "((a[0])[1]) = 5");
}

#[test]
fn parse_compound_assign_to_index() {
    integration_test!("a[0] += 1;", "(a[0]) = (+ (a[0]) 1)");
    integration_test!("a[i] -= 2;", "(a[i]) = (- (a[i]) 2)");
    integration_test!("a[i] *= 3;", "(a[i]) = (* (a[i]) 3)");
}

#[test]
fn parse_index_in_while_loop() {
    integration_test!(
        "while (a[i] < 10) { a[i] = a[i] + 1; }",
        "while (< (a[i]) 10) {\n(a[i]) = (+ (a[i]) 1)\n}"
    );
}

#[test]
fn reject_unclosed_index() {
    assert!(parse(Lexer::new("a[0;").unwrap()).is_err());
    assert!(parse(Lexer::new("let x = a[0;").unwrap()).is_err());
    assert!(parse(Lexer::new("a[b[0];").unwrap()).is_err());
}

#[test]
fn reject_empty_index() {
    assert!(parse(Lexer::new("a[];").unwrap()).is_err());
}

#[test]
fn reject_unmatched_close_bracket() {
    assert!(parse(Lexer::new("a];").unwrap()).is_err());
    assert!(parse(Lexer::new("a[0]];").unwrap()).is_err());
}

#[test]
fn expression_spans() {
    let mut lexer = Lexer::new(
        "1 +
  foo(-a, (b)[0])",
    )
    .unwrap();
    let expr = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!((expr.span.line, expr.span.col), (0, 0));

    let ExpressionKind::Binary(_, _, rhs) = &expr.kind else {
        panic!("expected binary, got {expr}");
    };
    assert_eq!((rhs.span.line, rhs.span.col), (1, 2));

    let ExpressionKind::FunctionCall(_, args) = &rhs.kind else {
        panic!("expected function call, got {rhs}");
    };
    let spans: Vec<_> = args.iter().map(|a| (a.span.line, a.span.col)).collect();
    assert_eq!(spans, vec![(1, 6), (1, 10)]);
}

#[test]
fn parse_boolean_literals() {
    for input in ["true", "false"] {
        let mut lexer = Lexer::new(input).unwrap();
        let expression = parse_expression(&mut lexer, 0).unwrap();
        assert!(
            matches!(
                expression.kind,
                ExpressionKind::Atom(AtomType::Boolean(b)) if b == (input == "true")
            ),
            "{input} parsed as {expression:?}"
        );
    }
}

#[test]
fn parse_boolean_followed_by_token() {
    let program = parse(Lexer::new("true; false;").unwrap()).unwrap();
    let values: Vec<_> = program
        .block
        .children
        .iter()
        .map(|child| match child {
            StatementOrExpression::Expression(e) => &e.kind,
            StatementOrExpression::Statement(s) => panic!("expected an expression, got {s}"),
        })
        .collect();
    assert!(
        matches!(
            values.as_slice(),
            [
                ExpressionKind::Atom(AtomType::Boolean(true)),
                ExpressionKind::Atom(AtomType::Boolean(false))
            ]
        ),
        "{values:?}"
    );
}

#[test]
fn parse_boolean_expressions() {
    test_parse_expression!("!true", "(! true)");
    test_parse_expression!("true && false", "(&& true false)");
    test_parse_expression!("(true || false) && a", "(&& (|| true false) a)");
}

#[test]
fn parse_boolean_as_function_arg() {
    integration_test!("f(true, false);", "f(true, false)");
}

#[test]
fn parse_boolean_in_if_condition() {
    integration_test!(
        "if (true) { a = 1; }",
        "if true {
a = 1
}"
    );
}

#[test]
fn parse_char_literals() {
    for (input, expected) in [
        ("'a'", 'a'),
        ("'Z'", 'Z'),
        ("'0'", '0'),
        ("' '", ' '),
        ("'é'", 'é'),
        (r"'\n'", '\n'),
        (r"'\0'", '\0'),
        (r"'\\'", '\\'),
        (r"'\''", '\''),
    ] {
        let mut lexer = Lexer::new(input).unwrap();
        let expression = parse_expression(&mut lexer, 0).unwrap();
        assert!(
            matches!(
                expression.kind,
                ExpressionKind::Atom(AtomType::Char(c)) if c == expected
            ),
            "{input} parsed as {expression:?}"
        );
    }
}

#[test]
fn parse_char_expressions() {
    test_parse_expression!("'a' + 1", "(+ 'a' 1)");
    test_parse_expression!("c - '0'", "(- c '0')");
    test_parse_expression!("c == 'z' || c == 'Z'", "(|| (== c 'z') (== c 'Z'))");
    test_parse_expression!("-'a'", "(- 'a')");
}

#[test]
fn parse_char_declaration() {
    integration_test!("let c: char = 'a';", "let c = 'a'");

    let program = parse(Lexer::new("let c: char = 'a';").unwrap()).unwrap();
    let StatementOrExpression::Statement(statement) = &program.block.children[0] else {
        panic!("expected a statement, got {program}");
    };
    let Statement::Declaration(declaration) = statement.as_ref() else {
        panic!("expected a declaration, got {statement}");
    };
    assert_eq!(declaration.dtype, Some(Type::Char));
}

#[test]
fn parse_char_assignment() {
    integration_test!(
        "let c: char = 'a'; c = 'x';",
        "let c = 'a'
c = 'x'"
    );
    integration_test!(
        "let c: char = 'a'; c += 'a';",
        "let c = 'a'
c = (+ c 'a')"
    );
    integration_test!(
        "let s: *char = str; s[0] = 'x';",
        "let s = str
(s[0]) = 'x'"
    );
}

#[test]
fn parse_char_as_function_arg() {
    integration_test!("putchar('h');", "putchar('h')");
    integration_test!(r"f('a', '\n');", r"f('a', '\n')");
}

#[test]
fn parse_char_as_index() {
    test_parse_expression!("a['a']", "(a['a'])");
}

#[test]
fn parse_char_statement() {
    integration_test!("'a';", "'a'");
}

#[test]
fn parse_char_in_if_condition() {
    integration_test!(
        "if (c == ';') { a = 1; }",
        "if (== c ';') {
a = 1
}"
    );
}

#[test]
fn expect_char_statement_semicolon() {
    let err = parse(Lexer::new("'a' 'b';").unwrap()).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == ";"));
}

#[test]
fn parse_struct_definition() {
    integration_test!(
        "struct P { x: u32, y: *char }",
        "struct P {x: u32, y: *char}"
    );
}

#[test]
fn parse_struct_instantiation() {
    test_parse_expression!("P { x: 1, y: 2 }", "P { x: 1, y: 2 }");
    test_parse_expression!("P { x: a + 1 }", "P { x: (+ a 1) }");
    test_parse_expression!("P {}", "P {  }");
}

#[test]
fn parse_struct_instantiation_with_struct_field() {
    test_parse_expression!("A { b: B { x: 1 } }", "A { b: B { x: 1 } }");
}

#[test]
fn parse_struct_access() {
    test_parse_expression!("p.x", "p.x");
    test_parse_expression!("p.x + p.y", "(+ p.x p.y)");
    test_parse_expression!("foo(p.x)", "foo(p.x)");
}

#[test]
fn parse_struct_access_assignment() {
    integration_test!("p.x = 3;", "p.x = 3");
    integration_test!("p.x += 1;", "p.x = (+ p.x 1)");
}

#[test]
fn parse_if_condition_ending_in_identifier_is_not_struct() {
    integration_test!("if a == b { c++; }", "if (== a b) {\n(++ c)\n}");
}

#[test]
fn parse_while_condition_ending_in_identifier_is_not_struct() {
    integration_test!("while a { a--; }", "while a {\n(-- a)\n}");
}

#[test]
fn parse_struct_in_parenthesized_condition() {
    integration_test!("if (P { x: 1 }) == p { }", "if (== P { x: 1 } p) {\n\n}");
}

#[test]
fn expect_colon_in_struct_instantiation() {
    let err = parse(Lexer::new("let p = P { x 1 };").unwrap()).unwrap_err();
    assert!(matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == ":"));
}

#[test]
fn expect_field_name_in_struct_access() {
    let err = parse(Lexer::new("p.1;").unwrap()).unwrap_err();
    assert!(
        matches!(err.kind, ParseErrorKind::Expected { ref expected, .. } if expected == "field name")
    );
}

#[test]
fn parse_imports() {
    integration_test!(
        "use math::{square, cube};",
        "use math::{square, cube}
"
    );
    integration_test!(
        "use math::square;",
        "use math::{square}
"
    );
    integration_test!(
        "use lib::math::{square,};",
        "use lib::math::{square}
"
    );
    integration_test!(
        "use a::{x};
use b::c::y;
x();",
        "use a::{x}
use b::c::{y}
x()"
    );
}

#[test]
fn parse_import_errors() {
    assert!(parse(Lexer::new("use math;").unwrap()).is_err());
    assert!(parse(Lexer::new("use math::{square}").unwrap()).is_err());
    assert!(parse(Lexer::new("use math::{1};").unwrap()).is_err());
}

#[test]
fn parse_address_of() {
    test_parse_expression!("&x", "(& x)");
    test_parse_expression!("&p.name", "(& p.name)");
    test_parse_expression!("f(&x, &y)", "f((& x), (& y))");
}

#[test]
fn parse_address_of_index() {
    test_parse_expression!("&a[1]", "(& (a[1]))");
}

#[test]
fn parse_deref() {
    test_parse_expression!("*p", "(* p)");
    test_parse_expression!("**pp", "(* (* pp))");
    test_parse_expression!("*&x", "(* (& x))");
}

#[test]
fn parse_deref_binds_tighter_than_infix() {
    test_parse_expression!("*p + 1", "(+ (* p) 1)");
    test_parse_expression!("a * *p", "(* a (* p))");
    test_parse_expression!("*a * *b", "(* (* a) (* b))");
    test_parse_expression!("*p == *q", "(== (* p) (* q))");
}

#[test]
fn parse_index_binds_tighter_than_deref() {
    test_parse_expression!("*p[0]", "(* (p[0]))");
    test_parse_expression!("(*p)[0]", "((* p)[0])");
}

#[test]
fn parse_deref_mixed_with_prefix_ops() {
    test_parse_expression!("-*p", "(- (* p))");
    test_parse_expression!("!*p", "(! (* p))");
}

#[test]
fn parse_field_access_through_deref() {
    test_parse_expression!("(*p).name", "(* p).name");
    test_parse_expression!("(*p).age + 1", "(+ (* p).age 1)");
    test_parse_expression!("&(*p).age", "(& (* p).age)");
}

#[test]
fn parse_chained_field_access() {
    test_parse_expression!("q.p.a", "q.p.a");
    test_parse_expression!("(*r).p.a", "(* r).p.a");
    test_parse_expression!("a[0].x", "(a[0]).x");
    test_parse_expression!("*p.x", "(* p.x)");
}

#[test]
fn reject_field_access_on_non_place() {
    assert!(parse(Lexer::new("f().a;").unwrap()).is_err());
    assert!(parse(Lexer::new("(a + b).a;").unwrap()).is_err());
}

#[test]
fn parse_deref_as_function_arg() {
    test_parse_expression!("printf(\"%d\", *p)", "printf(\"%d\", (* p))");
}

#[test]
fn parse_assign_through_deref() {
    integration_test!("*p = 1;", "(* p) = 1");
    integration_test!("**pp = 1;", "(* (* pp)) = 1");
    integration_test!("*p = *q;", "(* p) = (* q)");
}

#[test]
fn parse_compound_assign_through_deref() {
    integration_test!("*p += 1;", "(* p) = (+ (* p) 1)");
}

#[test]
fn parse_assign_to_field_through_deref() {
    integration_test!("(*p).age = 2;", "(* p).age = 2");
}

#[test]
fn parse_increment_through_deref() {
    integration_test!("(*p)++;", "(++ (* p))");
}

#[test]
fn parse_pointer_declarations() {
    integration_test!("let p = &x;", "let p = (& x)");
    integration_test!("let p: *i64 = &x;", "let p = (& x)");
    integration_test!("let y = *p;", "let y = (* p)");
}

#[test]
fn reject_address_of_non_place() {
    assert!(parse(Lexer::new("&1;").unwrap()).is_err());
    assert!(parse(Lexer::new("&(a + b);").unwrap()).is_err());
    assert!(parse(Lexer::new("&f();").unwrap()).is_err());
}
