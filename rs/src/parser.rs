use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, Expression, IfStatement, OperatorType, Program,
        Statement, StatementOrExpression, WhileLoop,
    },
    lexer::Lexer,
    symbol_table::FunctionDefinition,
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
    if lexer.peek().is_none() {
        Ok(None)
    } else {
        let result = match parse_statement(lexer)? {
            Some(statement) => StatementOrExpression::Statement(statement),
            None => StatementOrExpression::Expression(parse_expression(lexer, 0)?),
        };
        // Do not expect semicolon for certain statements
        match &result {
            StatementOrExpression::Statement(
                Statement::WhileLoop(_)
                | Statement::IfStatement(_)
                | Statement::FunctionDefintion(_),
            ) => (),
            _ => lexer.expect(";")?,
        }
        Ok(Some(result))
    }
}

fn parse_statement(lexer: &mut Lexer) -> Result<Option<Statement>, String> {
    match &lexer.peek() {
        None => Err("Ran out of tokens".to_string()),

        Some(tok) => Ok(match &tok.token_type {
            Token::Op(_) => None,
            Token::Atom(atom) => match atom {
                LexerAtomType::Identifier(s) => {
                    if let Some(keyword) = match s.as_str() {
                        "let" => Some(Statement::Declaration(parse_declaration(lexer)?)),
                        "while" => Some(Statement::WhileLoop(parse_while_loop(lexer)?)),
                        "if" => Some(Statement::IfStatement(parse_if_statement(lexer)?)),
                        "def" => Some(Statement::FunctionDefintion(parse_function_definition(
                            lexer,
                        )?)),
                        "return" => Some(Statement::Return(parse_return(lexer)?)),
                        _ => None,
                    } {
                        Some(keyword)
                    } else if let Some(next) = lexer.peek_next() {
                        match &next.token_type {
                            Token::Op(OperatorType::Assign) => {
                                Some(Statement::Assign(parse_assignment(lexer)?))
                            }
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
        }),
    }
}

fn parse_while_loop(lexer: &mut Lexer) -> Result<WhileLoop, String> {
    lexer.expect("while")?;
    let condition = parse_expression(lexer, 0)?;
    let block = parse_block(lexer)?;
    Ok(WhileLoop { condition, block })
}

fn parse_if_statement(lexer: &mut Lexer) -> Result<IfStatement, String> {
    lexer.expect("if")?;
    let condition = parse_expression(lexer, 0)?;
    let block = parse_block(lexer)?;
    let else_block = match lexer.peek() {
        None => None,
        Some(tok) => match &tok.token_type {
            Token::Atom(LexerAtomType::Identifier(s)) if s == "else" => {
                lexer.expect("else")?;
                Some(parse_block(lexer)?)
            }
            _ => None,
        },
    };

    Ok(IfStatement {
        condition,
        block,
        else_block,
    })
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
            Token::Op(OperatorType::CloseCurly) => break,
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
    let Token::Atom(LexerAtomType::Identifier(identifier)) = lexer
        .next()
        .ok_or("Expected identifier after declaration")?
        .token_type
    else {
        panic!("Expected identifier");
    };

    lexer.expect("=")?;

    Ok(Assignment {
        identifier,
        expression: parse_expression(lexer, 0)?,
    })
}

fn parse_expression(lexer: &mut Lexer, min_bp: u8) -> Result<Expression, String> {
    let token = lexer.next().ok_or("Ran out of tokens")?;

    let mut lhs = match &token.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => parse_identifier_or_function_call(lexer, s)?,
        Token::Atom(it) => Expression::Atom(AtomType::from_lexer_atom(it)),
        Token::Op(OperatorType::OpenParen) => {
            let lhs = parse_expression(lexer, 0)?;
            lexer.expect(")")?;
            lhs
        }
        Token::Op(op) => {
            let ((), r_bp) = prefix_binding_power(op);
            let rhs = parse_expression(lexer, r_bp)?;
            Expression::Operation(op.clone(), vec![rhs])
        }
    };

    loop {
        if lexer.peek().is_none() {
            break;
        }

        let op_token = lexer.peek().unwrap();
        let op = match &op_token.token_type {
            Token::Op(OperatorType::CloseParen) | Token::Atom(_) => break,
            Token::Op(op) => op.clone(),
        };

        if let Some((l_bp, ())) = postfix_binding_power(&op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            lhs = Expression::Operation(op, vec![lhs]);
            continue;
        }

        if let Some((l_bp, r_bp)) = infix_binding_power(&op) {
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = parse_expression(lexer, r_bp)?;
            lhs = Expression::Operation(op.clone(), vec![lhs, rhs]);
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn prefix_binding_power(op: &OperatorType) -> ((), u8) {
    match op {
        OperatorType::Subtract | OperatorType::Not => ((), 9),
        _ => panic!("bad op: {op:?}"),
    }
}

fn postfix_binding_power(_: &OperatorType) -> Option<(u8, ())> {
    // let res = match op {
    //     '!' => (9, ()),
    //     _ => return None,
    // };
    // Some(res)
    None
}

fn infix_binding_power(op: &OperatorType) -> Option<(u8, u8)> {
    let res = match op {
        OperatorType::And | OperatorType::Or => (1, 2),
        OperatorType::Equal | OperatorType::NotEqual => (3, 4),
        OperatorType::LessThan
        | OperatorType::LessOrEqual
        | OperatorType::GreaterThan
        | OperatorType::GreaterOrEqual => (5, 6),
        OperatorType::Add | OperatorType::Subtract => (7, 8),
        OperatorType::Multiply | OperatorType::Divide | OperatorType::Modulo => (9, 10),
        _ => return None,
    };

    Some(res)
}

fn parse_identifier_or_function_call(lexer: &mut Lexer, s: &str) -> Result<Expression, String> {
    Ok(if let Some(next) = lexer.peek() {
        match next.token_type {
            Token::Op(OperatorType::OpenParen) => parse_function_call(lexer, s.to_string())?,
            _ => Expression::Atom(AtomType::Identifier(s.to_string())),
        }
    } else {
        Expression::Atom(AtomType::Identifier(s.to_string()))
    })
}

fn parse_function_call(lexer: &mut Lexer, s: String) -> Result<Expression, String> {
    lexer.expect("(")?;

    let mut args = vec![];
    loop {
        let next = lexer
            .peek()
            .expect("Ran out of tokens parsing function call");

        if next.token_type == Token::Op(OperatorType::CloseParen) {
            lexer.next().unwrap();
            break;
        }

        let arg = parse_expression(lexer, 0)?;
        args.push(arg);
        let next = lexer
            .next()
            .ok_or("Ran out of tokens while parsing function call")?;
        match next.token_type {
            Token::Op(OperatorType::CloseParen) => break,
            Token::Op(OperatorType::Comma) => (),
            _ => panic!("Invalid token {next}"),
        }
    }

    Ok(Expression::FunctionCall(s, args))
}

fn parse_function_definition(lexer: &mut Lexer) -> Result<FunctionDefinition, String> {
    lexer.expect("def")?;
    let name = match lexer.next() {
        None => Err("Ran out of tokens parsing function defintion"),
        Some(tok) => Ok(match tok.token_type {
            Token::Atom(LexerAtomType::Identifier(name)) => Ok(name),
            _ => Err("Expected function name".to_string()),
        }?),
    }?;

    lexer.expect("(")?;
    let mut args = vec![];
    loop {
        let next_arg = match lexer.peek() {
            None => Err("Ran out of tokens parsing function definition"),
            Some(tok) => Ok(match &tok.token_type {
                Token::Atom(LexerAtomType::Identifier(arg_name)) => Some(arg_name),
                _ => None,
            }),
        }?;

        match next_arg {
            None => {
                lexer.expect(")")?;
                break;
            }
            Some(arg_name) => {
                args.push(arg_name.clone());
                lexer.next(); // consume the identifier

                match lexer.next() {
                    None => return Err("Ran out of tokens parsing function definition".to_string()),
                    Some(tok) => match tok.token_type {
                        Token::Op(OperatorType::Comma) => (),
                        Token::Op(OperatorType::CloseParen) => break,
                        _ => return Err(format!("Unexpected token: {tok}")),
                    },
                }
            }
        }
    }

    let body = parse_block(lexer)?;
    Ok(FunctionDefinition { name, args, body })
}

fn parse_return(lexer: &mut Lexer) -> Result<Expression, String> {
    lexer.expect("return")?;
    parse_expression(lexer, 0)
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::Lexer,
        parser::{parse, parse_expression},
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
            let program = parse($input).unwrap();
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
    fn parse_declaration() {
        integration_test!("let a = 1;", "let a = 1");
        integration_test!("let a = 1 + 2;", "let a = (+ 1 2)");
    }

    #[test]
    fn expect_statement_semicolon() {
        let program = parse("let a = 1 let b = 2;").unwrap_err();
        assert!(program.starts_with("Expected ;"));
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
}
