use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, Expression, OperatorType, Program, Statement,
        StatementOrExpression, WhileLoop,
    },
    lexer::Lexer,
    types::{LexerAtomType, Token},
};

pub fn parse(input: &str) -> Result<Program, String> {
    let mut lexer = Lexer::new(input)?;

    let mut block = Block { children: vec![] };
    while let Some(s) = parse_statement_or_expression(&mut lexer)? {
        block.children.push(s);
    }

    Ok(Program { block })
}

fn parse_statement_or_expression(
    lexer: &mut Lexer,
) -> Result<Option<StatementOrExpression>, String> {
    let ret = match lexer.peek() {
        None => {
            println!("Returing none");
            Ok(None)
        }
        Some(tok) => match tok.token_type {
            Token::Eof => {
                lexer.next();
                parse_statement_or_expression(lexer)
            }
            _ => Ok(Some(match parse_statement(lexer)? {
                Some(statement) => StatementOrExpression::Statement(statement),
                None => StatementOrExpression::Expression(parse_expression(lexer, 0)?),
            })),
        },
    };

    if let Ok(Some(result)) = &ret {
        match result {
            StatementOrExpression::Statement(Statement::WhileLoop(_)) => (),
            _ => lexer.expect(";")?,
        }
    }

    ret
}

fn parse_statement(lexer: &mut Lexer) -> Result<Option<Statement>, String> {
    let statement = match &lexer.peek().expect("Ran out of tokens").token_type {
        Token::Op(_) | Token::Eof => None,
        Token::Atom(atom) => match atom {
            LexerAtomType::Identifier(s) => {
                if let Some(keyword) = match s.as_str() {
                    "let" => Some(Statement::Declaration(parse_declaration(lexer)?)),
                    "while" => Some(Statement::WhileLoop(parse_while_loop(lexer)?)),
                    _ => None,
                } {
                    Some(keyword)
                } else if let Some(next) = lexer.peek_next() {
                    match &next.token_type {
                        Token::Op('=') => Some(Statement::Assign(parse_assignment(lexer)?)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            LexerAtomType::Semicolon => {
                panic!("Hanging semicolon got left over")
            }
            LexerAtomType::Number(_) | LexerAtomType::String(_) => None,
        },
    };

    Ok(statement)
}

fn parse_while_loop(lexer: &mut Lexer) -> Result<WhileLoop, String> {
    lexer.expect("while")?;
    let condition = parse_expression(lexer, 0)?;
    let block = parse_block(lexer)?;
    Ok(WhileLoop { condition, block })
}

fn parse_block(lexer: &mut Lexer) -> Result<Block, String> {
    lexer.expect("{")?;
    let mut children = vec![];
    // while let Some(statement) = parse_statement_or_expression(lexer)? {
    //     children.push(statement)
    // }

    loop {
        match lexer
            .peek()
            .expect("Ran out of tokens parsing while loop")
            .token_type
        {
            Token::Op('}') => break,
            _ => children.push(match parse_statement_or_expression(lexer)? {
                None => return Err("Ran out of tokens parsing while loop".to_string()),
                Some(s) => s,
            }),
        }
    }

    lexer.expect("}")?;

    Ok(Block { children })
}

fn parse_declaration(lexer: &mut Lexer) -> Result<Declaration, String> {
    lexer.expect("let")?;

    Ok(Declaration {
        assign: parse_assignment(lexer)?,
    })
}

fn parse_assignment(lexer: &mut Lexer) -> Result<Assignment, String> {
    let identifier = match lexer.next().token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => s,
        _ => panic!("Expected identifier"),
    };

    lexer.expect("=")?;

    Ok(Assignment {
        identifier,
        expression: parse_expression(lexer, 0)?,
    })
}

fn parse_expression(lexer: &mut Lexer, min_bp: u8) -> Result<Expression, String> {
    let token = lexer.next();

    let mut lhs = match token.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => parse_identifier_or_function_call(lexer, s)?,
        Token::Atom(it) => Expression::Atom(AtomType::from_lexer_atom(it)),
        Token::Op('(') => {
            let lhs = parse_expression(lexer, 0)?;
            lexer.expect(")")?;
            lexer.next();
            lhs
        }
        Token::Op(op) => {
            let ((), r_bp) = prefix_binding_power(op);
            let rhs = parse_expression(lexer, r_bp)?;
            Expression::Operation(OperatorType::from_char(op), vec![rhs])
        }
        _ => return Err(format!("bad token: {:?}", token)),
    };

    loop {
        if lexer.peek().is_none() {
            break;
        }

        let op_token = lexer.peek().unwrap();
        let op = match op_token.token_type {
            Token::Eof => break,
            Token::Op(')') => break,
            Token::Op(op) => op,
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
            let rhs = parse_expression(lexer, r_bp)?;
            lhs = Expression::Operation(OperatorType::from_char(op), vec![lhs, rhs]);
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn prefix_binding_power(op: char) -> ((), u8) {
    match op {
        '-' => ((), 5),
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
        '<' | '>' => (1, 2),
        '+' | '-' => (3, 4),
        '*' | '/' => (5, 6),
        '.' => (10, 9),
        _ => return None,
    };
    Some(res)
}

fn parse_identifier_or_function_call(lexer: &mut Lexer, s: String) -> Result<Expression, String> {
    Ok(if let Some(next) = lexer.peek() {
        match next.token_type {
            Token::Op('(') => parse_function_call(lexer, s)?,
            _ => Expression::Atom(AtomType::Identifier(s)),
        }
    } else {
        Expression::Atom(AtomType::Identifier(s))
    })
}

fn parse_function_call(lexer: &mut Lexer, s: String) -> Result<Expression, String> {
    lexer.expect("(")?;

    let mut args = vec![];
    loop {
        let next = lexer
            .peek()
            .expect("Ran out of tokens parsing function call");
        let arg = match next.token_type {
            Token::Eof => {
                return Err(
                    "Ran out of tokens parsing function call (Are you missing a ')'".to_string(),
                );
            }
            _ => parse_expression(lexer, 0)?,
        };

        args.push(arg);
        let next = lexer.next();
        match next.token_type {
            Token::Op(')') => {
                println!("Function call done");
                break;
            }
            Token::Op(',') => {
                println!("more args");
                continue;
            }
            _ => panic!("Invalid token {}", next),
        }
    }

    Ok(Expression::FunctionCall(s, args))
}

#[test]
fn test_parse_basic_math() {
    let mut lexer = Lexer::new("1").unwrap();
    let s = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!(s.to_string(), "1");

    let mut lexer = Lexer::new("1 + 2 * 3").unwrap();
    let s = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!(s.to_string(), "(+ 1 (* 2 3))");

    let mut lexer = Lexer::new("a + b * c * d + e").unwrap();
    let s = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!(s.to_string(), "(+ (+ a (* (* b c) d)) e)");
}

#[test]
fn test_parse_negative_numbers() {
    let mut lexer = Lexer::new("-9").unwrap();
    let s = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!(s.to_string(), "(- 9)");
}

#[test]
fn test_parse_parentheses() {
    let mut lexer = Lexer::new("(1 + 2) * 3").unwrap();
    let s = parse_expression(&mut lexer, 0).unwrap();
    assert_eq!(s.to_string(), "(* (+ 1 2) 3)");
}

#[test]
fn test_parse_assignment() {
    let s = parse("a = 1;").unwrap();
    assert_eq!(s.to_string(), "a = 1");

    let s = parse("a = 1 + 2;").unwrap();
    assert_eq!(s.to_string(), "a = (+ 1 2)");
}

#[test]
fn test_parse_declaration() {
    let s = parse("let a = 1;").unwrap();
    assert_eq!(s.to_string(), "let a = 1");

    let s = parse("let a = 1 + 2;").unwrap();
    assert_eq!(s.to_string(), "let a = (+ 1 2)");
}
