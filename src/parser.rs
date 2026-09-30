use crate::{
    ast::{
        AssignOp, Assignment, AtomType, BinaryOp, Block, Declaration, Expression,
        FunctionDefinition, IfStatement, OperatorType, PostfixOp, Program, ReturnStatement,
        Statement, StatementOrExpression, UnaryOp, WhileLoop,
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
    if lexer.peek().is_none() {
        Ok(None)
    } else {
        let result = match parse_statement(lexer)? {
            Some(statement) => StatementOrExpression::Statement(statement),
            None => StatementOrExpression::Expression(parse_expression(lexer, 0)?),
        };
        // Do not expect semicolon for certain statements
        match &result {
            StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => (),
            _ => lexer.expect(";")?,
        }
        Ok(Some(result))
    }
}

fn parse_statement(lexer: &mut Lexer) -> Result<Option<Box<dyn Statement>>, String> {
    match &lexer.peek() {
        None => Err("Ran out of tokens".to_string()),

        Some(tok) => Ok(match &tok.token_type {
            Token::Op(_) => None,
            Token::Atom(atom) => match atom {
                LexerAtomType::Identifier(s) => {
                    let keyword: Option<Box<dyn Statement>> = match s.as_str() {
                        "let" => Some(Box::new(parse_declaration(lexer)?)),
                        "while" => Some(Box::new(parse_while_loop(lexer)?)),
                        "if" => Some(Box::new(parse_if_statement(lexer)?)),
                        "def" => Some(Box::new(parse_function_definition(lexer)?)),
                        "return" => Some(Box::new(parse_return(lexer)?)),
                        _ => None,
                    };

                    if keyword.is_some() {
                        keyword
                    } else if let Some(next) = lexer.peek_next() {
                        match &next.token_type {
                            Token::Op(op) if op.as_assign().is_some() => {
                                Some(Box::new(parse_assignment(lexer)?))
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

    if let Some(tok) = lexer.peek_next()
        && tok.token_type != Token::Op(OperatorType::Assign)
    {
        return Err(format!("Expected = in declaration, got {tok}"));
    }

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

    let tok = lexer
        .next()
        .ok_or("Expected =, += or -= but ran out of tokens")?;
    let op = match &tok.token_type {
        Token::Op(op) => op.as_assign(),
        Token::Atom(_) => None,
    }
    .ok_or_else(|| format!("Expected =, += or -=, got {tok}"))?;

    let rhs = parse_expression(lexer, 0)?;
    let expression = match op {
        AssignOp::Compound(op) => Expression::Binary(
            op,
            Box::new(Expression::Atom(AtomType::Identifier(identifier.clone()))),
            Box::new(rhs),
        ),
        AssignOp::Assign => rhs,
    };

    Ok(Assignment {
        identifier,
        expression,
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
            let op = UnaryOp::from_token(op)
                .ok_or_else(|| format!("Unexpected operator {token} at start of expression"))?;
            let rhs = parse_expression(lexer, UnaryOp::BINDING_POWER)?;
            Expression::Unary(op, Box::new(rhs))
        }
    };

    while let Some(op_token) = lexer.peek() {
        let Token::Op(op) = &op_token.token_type else {
            break;
        };

        if let Some(op) = PostfixOp::from_token(op) {
            if PostfixOp::BINDING_POWER < min_bp {
                break;
            }
            let op_token = lexer.next().ok_or("Ran out of tokens")?;
            let Expression::Atom(AtomType::Identifier(identifier)) = lhs else {
                return Err(format!(
                    "{op} can only be applied to a variable, got {lhs} at {op_token}"
                ));
            };
            lhs = Expression::Postfix(op, identifier);
            continue;
        }

        if let Some(op) = BinaryOp::from_token(op) {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = parse_expression(lexer, r_bp)?;
            lhs = Expression::Binary(op, Box::new(lhs), Box::new(rhs));
            continue;
        }

        break;
    }

    Ok(lhs)
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
            _ => return Err(format!("Invalid token {next}")),
        }
    }

    Ok(Expression::FunctionCall(s, args))
}

fn parse_function_definition(lexer: &mut Lexer) -> Result<FunctionDefinition, String> {
    lexer.expect("def")?;

    let name = match lexer.next() {
        None => return Err("Ran out of tokens parsing function definition".to_string()),
        Some(tok) => match tok.token_type {
            Token::Atom(LexerAtomType::Identifier(name)) => name,
            _ => return Err("Expected function name".to_string()),
        },
    };

    lexer.expect("(")?;

    let mut args = vec![];

    loop {
        let next_arg = lexer
            .peek()
            .ok_or("Ran out of tokens parsing function definition")?;

        match &next_arg.token_type {
            Token::Atom(LexerAtomType::Identifier(arg_name)) => {
                args.push(arg_name.clone());
                lexer.next();

                match lexer.next() {
                    Some(tok) => match tok.token_type {
                        Token::Op(OperatorType::Comma) => {}
                        Token::Op(OperatorType::CloseParen) => break,
                        _ => return Err(format!("Unexpected token: {tok}")),
                    },
                    None => return Err("Ran out of tokens parsing function definition".to_string()),
                }
            }

            Token::Op(OperatorType::CloseParen) => {
                lexer.next();
                break;
            }

            _ => return Err(format!("Unexpected token: {next_arg}")),
        }
    }

    let body = parse_function_body(lexer)?;

    Ok(FunctionDefinition { name, args, body })
}

fn parse_function_body(lexer: &mut Lexer) -> Result<Block, String> {
    lexer.expect("{")?;

    let mut children = vec![];

    loop {
        let token = lexer
            .peek()
            .ok_or("Ran out of tokens parsing function body")?;

        // `}` means the function has no implicit return expression.
        if token.token_type == Token::Op(OperatorType::CloseCurly) {
            break;
        }

        let item = parse_statement(lexer)?;

        if let Some(statement) = item {
            children.push(StatementOrExpression::Statement(statement));

            // Statements still require their normal semicolon rules.
            match children.last().unwrap() {
                StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => {}

                _ => lexer.expect(";")?,
            }
        } else {
            // We have an expression.
            let expression = parse_expression(lexer, 0)?;

            match lexer.peek() {
                // Final expression: implicit return.
                Some(tok) if tok.token_type == Token::Op(OperatorType::CloseCurly) => {
                    children.push(StatementOrExpression::Statement(Box::new(
                        ReturnStatement { expr: expression },
                    )));
                    break;
                }

                // Non-final expression must have a semicolon.
                _ => {
                    lexer.expect(";")?;
                    children.push(StatementOrExpression::Expression(expression));
                }
            }
        }
    }

    lexer.expect("}")?;

    Ok(Block { children })
}

fn parse_return(lexer: &mut Lexer) -> Result<ReturnStatement, String> {
    lexer.expect("return")?;
    Ok(ReturnStatement {
        expr: parse_expression(lexer, 0)?,
    })
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
    fn parse_add_assign() {
        integration_test!("a += 1;", "a = (+ a 1)");
    }

    #[test]
    fn parse_sub_assign() {
        integration_test!("a -= 1;", "a = (- a 1)");
    }

    #[test]
    fn parse_compound_assign_groups_right_side() {
        // The whole right side is the operand, not just the first term
        integration_test!("a -= 1 + 2;", "a = (- a (+ 1 2))");
        integration_test!("a += b * 2;", "a = (+ a (* b 2))");
        integration_test!("a -= b - c;", "a = (- a (- b c))");
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
        let err = parse("let a += 1;").unwrap_err();
        assert!(err.starts_with("Expected = in declaration"));
        let err = parse("let a -= 1;").unwrap_err();
        assert!(err.starts_with("Expected = in declaration"));
    }

    #[test]
    fn expect_compound_assign_semicolon() {
        let program = parse("a += 1 b -= 2;").unwrap_err();
        assert!(program.starts_with("Expected ;"));
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
        let program = parse("a++ b--;").unwrap_err();
        assert!(program.starts_with("Expected ;"));
    }

    #[test]
    fn reject_increment_on_non_identifier() {
        assert!(parse("5++;").is_err());
        assert!(parse("(a + b)--;").is_err());
        assert!(parse("foo()++;").is_err());
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
            "def add(a, b) { return a + b; }",
            "def add(a, b) {return (+ a b)}"
        );
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
        integration_test!("def add(a, b) { a + b }", "def add(a, b) {return (+ a b)}");
    }
}
