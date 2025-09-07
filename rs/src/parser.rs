use crate::{
    ast::{
        Assignment, AtomType, Declaration, Expression, OperatorType, Program, Statement,
        StatementOrExpression,
    },
    lexer::Lexer,
    types::{LexerAtomType, Token},
};

pub fn parse(input: &str) -> Program {
    let mut lexer = Lexer::new(input);

    let mut program = Program { children: vec![] };
    while let Some(s) = parse_statement_or_expression(&mut lexer) {
        program.children.push(s);
    }

    program
}

fn parse_statement_or_expression(lexer: &mut Lexer) -> Option<StatementOrExpression> {
    lexer.peek()?;

    Some(match parse_statement(lexer) {
        Some(statement) => StatementOrExpression::Statement(statement),
        None => StatementOrExpression::Expression(parse_expression(lexer, 0)),
    })
}

fn parse_statement(lexer: &mut Lexer) -> Option<Statement> {
    let statement = match &lexer.peek().expect("Ran out of tokens").token_type {
        Token::Op(_) | Token::Eof => None,
        Token::Atom(atom) => match atom {
            LexerAtomType::Identifier(s) => {
                if let Some(keyword) = match s.as_str() {
                    "let" => Some(Statement::Declaration(parse_declaration(lexer))),
                    _ => None,
                } {
                    Some(keyword)
                } else if let Some(next) = lexer.peek_next() {
                    match &next.token_type {
                        Token::Op('=') => Some(Statement::Assign(parse_assignment(lexer))),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            LexerAtomType::Number(_) => None,
            LexerAtomType::Semicolon => {
                lexer.next();
                None
            }
        },
    };

    if statement.is_some() {
        lexer.expect(";");
    }
    statement
}

fn parse_declaration(lexer: &mut Lexer) -> Declaration {
    lexer.expect("let");

    Declaration {
        assign: parse_assignment(lexer),
    }
}

fn parse_assignment(lexer: &mut Lexer) -> Assignment {
    let identifier = match lexer.next().token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => s,
        _ => panic!("Expected identifier"),
    };

    lexer.expect("=");

    Assignment {
        identifier,
        expression: parse_expression(lexer, 0),
    }
}

fn parse_expression(lexer: &mut Lexer, min_bp: u8) -> Expression {
    let token = lexer.next();

    let mut lhs = match token.token_type {
        Token::Atom(it) => Expression::Atom(AtomType::from_lexer_atom(it)),
        Token::Op('(') => {
            let lhs = parse_expression(lexer, 0);
            // lexer.expect(')');
            lexer.next();
            lhs
        }
        Token::Op(op) => {
            let ((), r_bp) = prefix_binding_power(op);
            let rhs = parse_expression(lexer, r_bp);
            Expression::Operation(OperatorType::from_char(op), vec![rhs])
        }
        _ => panic!("bad token: {:?}", token),
    };

    loop {
        if lexer.peek().is_none() {
            break;
        }

        let op_token = lexer.peek().unwrap();
        let op = match op_token.token_type {
            Token::Eof => break,
            Token::Op(op) => {
                if op == ')' {
                    return lhs;
                }
                op
            }
            _ => break,
        };

        if let Some((l_bp, ())) = postfix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            lhs = Expression::Operation(OperatorType::from_char(op), vec![lhs]);
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = parse_expression(lexer, r_bp);
            lhs = Expression::Operation(OperatorType::from_char(op), vec![lhs, rhs]);
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
fn test_parse_basic_math() {
    let mut lexer = Lexer::new("1");
    let s = parse_expression(&mut lexer, 0);
    assert_eq!(s.to_string(), "1");

    let mut lexer = Lexer::new("1 + 2 * 3");
    let s = parse_expression(&mut lexer, 0);
    assert_eq!(s.to_string(), "(+ 1 (* 2 3))");

    let mut lexer = Lexer::new("a + b * c * d + e");
    let s = parse_expression(&mut lexer, 0);
    assert_eq!(s.to_string(), "(+ (+ a (* (* b c) d)) e)");
}

#[test]
fn test_parse_negative_numbers() {
    let mut lexer = Lexer::new("-9");
    let s = parse_expression(&mut lexer, 0);
    assert_eq!(s.to_string(), "(- 9)");
}

#[test]
fn test_parse_parentheses() {
    let mut lexer = Lexer::new("(1 + 2) * 3");
    let s = parse_expression(&mut lexer, 0);
    assert_eq!(s.to_string(), "(* (+ 1 2) 3)");
}

#[test]
fn test_parse_assignment() {
    let s = parse("a = 1;");
    assert_eq!(s.to_string(), "a = 1");

    let s = parse("a = 1 + 2;");
    assert_eq!(s.to_string(), "a = (+ 1 2)");
}

#[test]
fn test_parse_declaration() {
    let s = parse("let a = 1;");
    assert_eq!(s.to_string(), "let a = 1");

    let s = parse("let a = 1 + 2;");
    assert_eq!(s.to_string(), "let a = (+ 1 2)");
}
