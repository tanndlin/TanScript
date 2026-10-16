use std::str::FromStr;

use crate::{
    ast::{
        Arg, Assignment, AtomType, Block, BreakStatement, ContinueStatement, Declaration,
        Expression, ExpressionKind, ForLoop, FunctionDefinition, IfStatement, OperatorType,
        ReturnStatement, Statement, StatementOrExpression, StructDefinition, StructField,
        WhileLoop,
    },
    error::At,
    lex::{Lexer, LexerAtomType},
    parse::{
        ParseError, ParseErrorKind, expected,
        parse_expression::{parse_expression, parse_expression_or_assignment},
    },
    type_check::Type,
    types::Token,
};

pub fn parse_statement_or_expression(
    lexer: &mut Lexer,
) -> Result<Option<StatementOrExpression>, ParseError> {
    if lexer.peek().is_none() {
        Ok(None)
    } else {
        let result = parse_unterminated_statement_or_expression(lexer)?;
        // Do not expect semicolon for certain statements
        match &result {
            StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => (),
            _ => {
                lexer.expect(";")?;
            }
        }
        Ok(Some(result))
    }
}

/// Parses a statement or expression without consuming a trailing semicolon.
pub fn parse_unterminated_statement_or_expression(
    lexer: &mut Lexer,
) -> Result<StatementOrExpression, ParseError> {
    Ok(match parse_statement(lexer)? {
        Some(statement) => StatementOrExpression::Statement(Box::new(statement)),
        None => parse_expression_or_assignment(lexer)?,
    })
}

fn parse_statement(lexer: &mut Lexer) -> Result<Option<Statement>, ParseError> {
    let tok = lexer.peek_or_eof("statement")?;
    let keyword = match &tok.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => s.as_str(),
        Token::Atom(LexerAtomType::Semicolon) => panic!("Hanging semicolon got left over"),
        Token::Op(_)
        | Token::Atom(
            LexerAtomType::Number(_)
            | LexerAtomType::String(_)
            | LexerAtomType::Char(_)
            | LexerAtomType::Colon
            | LexerAtomType::Comma,
        ) => {
            return Ok(None);
        }
    };

    Ok(Some(match keyword {
        "let" => parse_declaration(lexer)?.into(),
        "while" => parse_while_loop(lexer)?.into(),
        "for" => parse_for_loop(lexer)?.into(),
        "if" => parse_if_statement(lexer)?.into(),
        "def" => parse_function_definition(lexer)?.into(),
        "struct" => parse_struct_definition(lexer)?.into(),
        "return" => parse_return(lexer)?.into(),
        "break" => {
            let span = lexer.next().unwrap().span;
            BreakStatement { span }.into()
        }
        "continue" => {
            let span = lexer.next().unwrap().span;
            ContinueStatement { span }.into()
        }
        _ => return Ok(None),
    }))
}

fn parse_while_loop(lexer: &mut Lexer) -> Result<WhileLoop, ParseError> {
    lexer.expect("while")?;
    let condition = parse_expression(lexer, 0)?;
    let block = parse_block(lexer)?;
    Ok(WhileLoop { condition, block })
}

fn parse_for_loop(lexer: &mut Lexer) -> Result<ForLoop, ParseError> {
    lexer.expect("for")?;
    lexer.expect("(")?;
    let init = parse_unterminated_statement_or_expression(lexer)?;
    lexer.expect(";")?;
    let condition = parse_expression(lexer, 0)?;
    lexer.expect(";")?;
    let update = parse_unterminated_statement_or_expression(lexer)?;
    lexer.expect(")")?;
    let block = parse_block(lexer)?;
    Ok(ForLoop {
        init,
        condition,
        update,
        block,
    })
}

fn parse_if_statement(lexer: &mut Lexer) -> Result<IfStatement, ParseError> {
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

fn parse_block(lexer: &mut Lexer) -> Result<Block, ParseError> {
    lexer.expect("{")?;
    let mut children = vec![];
    // while let Some(statement) = parse_statement_or_expression(lexer)? {
    //     children.push(statement)
    // }

    loop {
        match lexer.peek_or_eof("}")?.token_type {
            Token::Op(OperatorType::CloseCurly) => break,
            _ => children.push(parse_statement_or_expression(lexer)?.ok_or_else(|| {
                ParseErrorKind::UnexpectedEof {
                    expected: "}".to_string(),
                }
                .at(lexer.eof)
            })?),
        }
    }

    lexer.expect("}")?;

    Ok(Block { children })
}

fn parse_declaration(lexer: &mut Lexer) -> Result<Declaration, ParseError> {
    let span = lexer.peek_or_eof("let")?.span;
    lexer.expect("let")?;

    let tok = lexer.next_or_eof("identifier")?;
    let ident_span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(ident)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };

    let dtype = parse_type_annotation(lexer)?;

    // Only a plain `=` is allowed; compound assigns need an existing value
    let tok = lexer.next_or_eof("=")?;
    if tok.token_type != Token::Op(OperatorType::Assign) {
        return Err(expected("=", tok));
    }

    Ok(Declaration {
        assign: Assignment {
            lhs: Expression::new(
                ExpressionKind::Atom(AtomType::Identifier(ident)),
                ident_span,
            ),
            expression: parse_expression(lexer, 0)?,
        },
        dtype,
        binding: None,
        _span: span,
    })
}

/// Parses an optional `: type` annotation
fn parse_type_annotation(lexer: &mut Lexer) -> Result<Option<Type>, ParseError> {
    if let Some(Token::Atom(LexerAtomType::Colon)) = lexer.peek().map(|tok| &tok.token_type) {
        lexer.next();
        Ok(Some(parse_type(lexer)?))
    } else {
        Ok(None)
    }
}

pub fn parse_type(lexer: &mut Lexer) -> Result<Type, ParseError> {
    let tok = lexer.next_or_eof("type")?;
    if let Token::Op(OperatorType::Multiply) = tok.token_type {
        return Ok(Type::Pointer(Box::new(parse_type(lexer)?)));
    }

    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(type_name)) = tok.token_type else {
        return Err(expected("type", tok));
    };
    Type::from_str(&type_name).map_err(|_| ParseErrorKind::UnknownType(type_name).at(span))
}

fn parse_function_definition(lexer: &mut Lexer) -> Result<FunctionDefinition, ParseError> {
    let span = lexer.expect("def")?.span;

    let tok = lexer.next_or_eof("function name")?;
    let Token::Atom(LexerAtomType::Identifier(name)) = tok.token_type else {
        return Err(expected("function name", tok));
    };

    lexer.expect("(")?;

    let mut args = vec![];

    loop {
        let tok = lexer.next_or_eof("argument name or )")?;
        let span = tok.span;
        let name = match tok.token_type {
            Token::Atom(LexerAtomType::Identifier(name)) => name,
            Token::Op(OperatorType::CloseParen) => break,
            _ => return Err(expected("argument name or )", tok)),
        };

        lexer.expect(":")?;
        args.push(Arg {
            name,
            atype: parse_type(lexer)?,
            span,
        });

        let tok = lexer.next_or_eof(", or )")?;
        match tok.token_type {
            Token::Atom(LexerAtomType::Comma) => {}
            Token::Op(OperatorType::CloseParen) => break,
            _ => return Err(expected(", or )", tok)),
        }
    }

    let return_type = parse_type_annotation(lexer)?.unwrap_or(Type::Unit);
    let body = parse_function_body(lexer)?;

    Ok(FunctionDefinition {
        name,
        args,
        body,
        return_type,
        span,
    })
}

fn parse_function_body(lexer: &mut Lexer) -> Result<Block, ParseError> {
    lexer.expect("{")?;

    let mut children = vec![];

    loop {
        let token = lexer.peek_or_eof("}")?;

        // `}` means the function has no implicit return expression.
        if token.token_type == Token::Op(OperatorType::CloseCurly) {
            break;
        }

        let item = parse_statement(lexer)?;

        if let Some(statement) = item {
            children.push(StatementOrExpression::Statement(Box::new(statement)));

            // Statements still require their normal semicolon rules.
            match children.last().unwrap() {
                StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => {}

                _ => {
                    lexer.expect(";")?;
                }
            }
        } else {
            match parse_expression_or_assignment(lexer)? {
                // Assignments are statements, never an implicit return.
                StatementOrExpression::Statement(assign) => {
                    children.push(StatementOrExpression::Statement(assign));
                    lexer.expect(";")?;
                }

                StatementOrExpression::Expression(expression) => match lexer.peek() {
                    // Final expression: implicit return.
                    Some(tok) if tok.token_type == Token::Op(OperatorType::CloseCurly) => {
                        children.push(StatementOrExpression::Statement(Box::new(
                            Statement::Return(ReturnStatement {
                                span: expression.span,
                                expr: Some(expression),
                            }),
                        )));
                        break;
                    }

                    // Non-final expression must have a semicolon.
                    _ => {
                        lexer.expect(";")?;
                        children.push(StatementOrExpression::Expression(expression));
                    }
                },
            }
        }
    }

    lexer.expect("}")?;

    Ok(Block { children })
}

fn parse_return(lexer: &mut Lexer) -> Result<ReturnStatement, ParseError> {
    let span = lexer.peek_or_eof("return")?.span;
    lexer.expect("return")?;
    let expr = match lexer.peek().map(|tok| &tok.token_type) {
        Some(Token::Atom(LexerAtomType::Semicolon) | Token::Op(OperatorType::CloseCurly)) => None,
        _ => Some(parse_expression(lexer, 0)?),
    };
    Ok(ReturnStatement { expr, span })
}

fn parse_struct_definition(lexer: &mut Lexer) -> Result<StructDefinition, ParseError> {
    lexer.expect("struct")?;
    let tok = lexer.next_or_eof("identifier")?;
    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(struct_name)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };

    lexer.expect("{")?;
    let mut fields = vec![];
    loop {
        let tok = lexer.peek_or_eof("field name or }")?;
        if !matches!(tok.token_type, Token::Atom(LexerAtomType::Identifier(_))) {
            break;
        }
        let Token::Atom(LexerAtomType::Identifier(name)) = lexer.next().unwrap().token_type else {
            unreachable!();
        };
        lexer.expect(":")?;
        let ftype = parse_type(lexer)?;
        fields.push(StructField { name, ftype });

        let tok = lexer.peek_or_eof(", or }")?;
        if tok.token_type == Token::Atom(LexerAtomType::Comma) {
            lexer.next();
        } else {
            break;
        }
    }
    lexer.expect("}")?;

    Ok(StructDefinition {
        name: struct_name,
        fields,
        span,
    })
}
