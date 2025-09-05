use crate::{ast::S, lexer::Lexer, types::Token};

pub fn expr(input: &str) -> S {
    let mut lexer = Lexer::new(input);
    expr_bp(&mut lexer, 0)
}

fn expr_bp(lexer: &mut Lexer, min_bp: u8) -> S {
    let token = lexer.next();
    let mut lhs = match token.token_type {
        Token::Atom(it) => S::Atom(it),
        Token::Op('(') => {
            let lhs = expr_bp(lexer, 0);
            lexer.expect(')');
            lexer.next();
            lhs
        }
        Token::Op(op) => {
            let ((), r_bp) = prefix_binding_power(op);
            let rhs = expr_bp(lexer, r_bp);
            S::Cons(op, vec![rhs])
        }
        _ => panic!("bad token: {:?}", token),
    };

    loop {
        let op_token = lexer.peek();
        let op = match op_token.token_type {
            Token::Eof => break,
            Token::Op(op) => {
                if op == ')' {
                    dbg!("Returing lhs", &lhs);
                    return lhs;
                }
                op
            }
            _ => panic!("bad token as rhs: {:?}", op_token),
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            lhs = S::Cons(op, vec![lhs]);
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = expr_bp(lexer, r_bp);
            lhs = S::Cons(op, vec![lhs, rhs]);
            continue;
        }

        break;
    }

    lhs
}

fn prefix_binding_power(op: char) -> ((), u8) {
    match op {
        '+' | '-' => ((), 5),
        _ => panic!("bad op: {:?}", op),
    }
}

fn postfix_binding_power(op: char) -> Option<(u8, ())> {
    let res = match op {
        '!' => (7, ()),
        _ => return None,
    };
    Some(res)
}

fn infix_binding_power(op: char) -> Option<(u8, u8)> {
    let res = match op {
        '+' | '-' => (1, 2),
        '*' | '/' => (3, 4),
        '.' => (10, 9),
        _ => return None,
    };
    Some(res)
}

#[test]
fn parser_tests() {
    let s = expr("1");
    assert_eq!(s.to_string(), "1");
    let s = expr("1 + 2 * 3");
    assert_eq!(s.to_string(), "(+ 1 (* 2 3))");
    let s = expr("a + b * c * d + e");
    assert_eq!(s.to_string(), "(+ (+ a (* (* b c) d)) e)");
}

#[test]
fn parse_negative_numbers() {
    let s = expr("-9");
    assert_eq!(s.to_string(), "(- 9)");
}

#[test]
fn parse_postfix() {
    let s = expr("-9!");
    assert_eq!(s.to_string(), "(- (! 9))");
}

#[test]
fn parse_parentheses() {
    let s = expr("(1 + 2) * 3");
    assert_eq!(s.to_string(), "(* (+ 1 2) 3)");
}
