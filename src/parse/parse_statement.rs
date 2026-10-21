use std::str::FromStr;

use crate::{
    ast::{
        Arg, Assignment, AtomType, Block, BreakStatement, ContinueStatement, Declaration,
        EnumDefinition, Expression, ExpressionKind, ForLoop, FunctionDefinition, IfStatement,
        Import, ImportName, OperatorType, ReturnStatement, Statement, StatementOrExpression,
        StructDefinition, StructField, WhileLoop,
    },
    lex::LexerAtomType,
    parse::{
        ParseError, Parser, expected,
        parse_expression::{parse_condition, parse_expression, parse_expression_or_assignment},
    },
    type_check::Type,
    types::{Span, Token},
};

pub fn parse_statement_or_expression(
    tokens: &mut Parser,
) -> Result<Option<StatementOrExpression>, ParseError> {
    if tokens.peek().is_none() {
        Ok(None)
    } else {
        let result = parse_unterminated_statement_or_expression(tokens)?;
        // Do not expect semicolon for certain statements
        match &result {
            StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => (),
            _ => {
                tokens.expect(";")?;
            }
        }
        Ok(Some(result))
    }
}

/// Parses a statement or expression without consuming a trailing semicolon.
pub fn parse_unterminated_statement_or_expression(
    tokens: &mut Parser,
) -> Result<StatementOrExpression, ParseError> {
    Ok(match parse_statement(tokens)? {
        Some(statement) => StatementOrExpression::Statement(Box::new(statement)),
        None => parse_expression_or_assignment(tokens)?,
    })
}

fn parse_statement(tokens: &mut Parser) -> Result<Option<Statement>, ParseError> {
    let tok = tokens.peek_or_eof("statement")?;
    let keyword = match &tok.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => s.as_str(),
        Token::Op(_)
        | Token::Atom(
            LexerAtomType::Number(_)
            | LexerAtomType::String(_)
            | LexerAtomType::Char(_)
            | LexerAtomType::Colon
            | LexerAtomType::Comma
            | LexerAtomType::Semicolon,
        ) => {
            return Ok(None);
        }
    };

    Ok(Some(match keyword {
        "let" => parse_declaration(tokens)?.into(),
        "while" => parse_while_loop(tokens)?.into(),
        "for" => parse_for_loop(tokens)?.into(),
        "if" => parse_if_statement(tokens)?.into(),
        "def" => parse_function_definition(tokens)?.into(),
        "struct" => parse_struct_definition(tokens)?.into(),
        "enum" => parse_enum_definition(tokens)?.into(),
        "return" => parse_return(tokens)?.into(),
        "break" => {
            let span = tokens.next().unwrap().span;
            BreakStatement { span }.into()
        }
        "continue" => {
            let span = tokens.next().unwrap().span;
            ContinueStatement { span }.into()
        }
        _ => return Ok(None),
    }))
}

fn parse_while_loop(tokens: &mut Parser) -> Result<WhileLoop, ParseError> {
    tokens.expect("while")?;
    let condition = parse_condition(tokens)?;
    let block = parse_block(tokens)?;
    Ok(WhileLoop { condition, block })
}

fn parse_for_loop(tokens: &mut Parser) -> Result<ForLoop, ParseError> {
    tokens.expect("for")?;
    tokens.expect("(")?;
    let init = parse_unterminated_statement_or_expression(tokens)?;
    tokens.expect(";")?;
    let condition = parse_expression(tokens, 0)?;
    tokens.expect(";")?;
    let update = parse_unterminated_statement_or_expression(tokens)?;
    tokens.expect(")")?;
    let block = parse_block(tokens)?;
    Ok(ForLoop {
        init,
        condition,
        update,
        block,
    })
}

fn parse_if_statement(tokens: &mut Parser) -> Result<IfStatement, ParseError> {
    tokens.expect("if")?;
    let condition = parse_condition(tokens)?;
    let block = parse_block(tokens)?;
    let else_block = match tokens.peek() {
        None => None,
        Some(tok) => match &tok.token_type {
            Token::Atom(LexerAtomType::Identifier(s)) if s == "else" => {
                tokens.expect("else")?;
                Some(parse_else_block(tokens)?)
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

fn parse_else_block(tokens: &mut Parser) -> Result<Block, ParseError> {
    match tokens.peek().map(|tok| &tok.token_type) {
        Some(Token::Atom(LexerAtomType::Identifier(s))) if s == "if" => Ok(Block {
            children: vec![StatementOrExpression::Statement(Box::new(
                Statement::IfStatement(tokens.nested("if", parse_if_statement)?),
            ))],
        }),
        _ => parse_block(tokens),
    }
}

fn parse_block(tokens: &mut Parser) -> Result<Block, ParseError> {
    tokens.nested("{", parse_block_at_depth)
}

fn parse_block_at_depth(tokens: &mut Parser) -> Result<Block, ParseError> {
    tokens.expect("{")?;
    let mut children = vec![];
    // while let Some(statement) = parse_statement_or_expression(tokens)? {
    //     children.push(statement)
    // }

    loop {
        match tokens.peek_or_eof("}")?.token_type {
            Token::Op(OperatorType::CloseCurly) => break,
            _ => children
                .push(parse_statement_or_expression(tokens)?.ok_or_else(|| tokens.eof_error("}"))?),
        }
    }

    tokens.expect("}")?;

    Ok(Block { children })
}

fn parse_declaration(tokens: &mut Parser) -> Result<Declaration, ParseError> {
    let span = tokens.peek_or_eof("let")?.span;
    tokens.expect("let")?;

    let tok = tokens.next_or_eof("identifier")?;
    let ident_span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(ident)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };

    let dtype = parse_type_annotation(tokens)?;

    // Only a plain `=` is allowed; compound assigns need an existing value
    let tok = tokens.next_or_eof("=")?;
    if tok.token_type != Token::Op(OperatorType::Assign) {
        return Err(expected("=", tok));
    }

    Ok(Declaration {
        assign: Assignment {
            lhs: Expression::new(
                ExpressionKind::Atom(AtomType::Identifier(ident)),
                ident_span,
            ),
            expression: parse_expression(tokens, 0)?,
        },
        dtype,
        binding: None,
        _span: span,
    })
}

/// Parses an optional `: type` annotation
fn parse_type_annotation(tokens: &mut Parser) -> Result<Option<Type>, ParseError> {
    if let Some(Token::Atom(LexerAtomType::Colon)) = tokens.peek().map(|tok| &tok.token_type) {
        tokens.next();
        Ok(Some(parse_type(tokens)?))
    } else {
        Ok(None)
    }
}

pub fn parse_type(tokens: &mut Parser) -> Result<Type, ParseError> {
    parse_type_with_span(tokens).map(|(t, _)| t)
}

pub fn parse_type_with_span(tokens: &mut Parser) -> Result<(Type, Span), ParseError> {
    let tok = tokens.next_or_eof("type")?;
    if let Token::Op(OperatorType::Multiply) = tok.token_type {
        let (inner, span) = parse_type_with_span(tokens)?;
        return Ok((Type::Pointer(Box::new(inner)), span));
    }

    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(type_name)) = tok.token_type else {
        return Err(expected("type", tok));
    };

    if let Ok(t) = Type::from_str(&type_name) {
        return Ok((t, span));
    }

    let mut concrete_generics = vec![];

    // Check for generics
    if let Some(Token::Op(OperatorType::LessThan)) = tokens.peek().map(|tok| &tok.token_type) {
        tokens.expect("<")?;
        loop {
            concrete_generics.push(parse_type(tokens)?);
            let tok = tokens.next_or_eof(", or >")?;
            match tok.token_type {
                Token::Op(OperatorType::GreaterThan) => break,
                Token::Atom(LexerAtomType::Comma) => (),
                _ => return Err(expected(", or >", tok)),
            }
        }
    }

    Ok((
        Type::Named {
            name: type_name,
            concrete_generics,
        },
        span,
    ))
}

fn parse_function_definition(tokens: &mut Parser) -> Result<FunctionDefinition, ParseError> {
    let span = tokens.expect("def")?.span;

    let tok = tokens.next_or_eof("function name")?;
    let Token::Atom(LexerAtomType::Identifier(name)) = tok.token_type else {
        return Err(expected("function name", tok));
    };

    tokens.expect("(")?;

    let mut args = vec![];

    loop {
        let tok = tokens.next_or_eof("argument name or )")?;
        let span = tok.span;
        let name = match tok.token_type {
            Token::Atom(LexerAtomType::Identifier(name)) => name,
            Token::Op(OperatorType::CloseParen) => break,
            _ => return Err(expected("argument name or )", tok)),
        };

        tokens.expect(":")?;
        args.push(Arg {
            name,
            atype: parse_type(tokens)?,
            span,
        });

        let tok = tokens.next_or_eof(", or )")?;
        match tok.token_type {
            Token::Atom(LexerAtomType::Comma) => {}
            Token::Op(OperatorType::CloseParen) => break,
            _ => return Err(expected(", or )", tok)),
        }
    }

    let (return_type, return_type_span) =
        if let Some(Token::Atom(LexerAtomType::Colon)) = tokens.peek().map(|tok| &tok.token_type) {
            tokens.next();
            parse_type_with_span(tokens)?
        } else {
            (Type::Unit, span)
        };
    let body = parse_function_body(tokens)?;

    Ok(FunctionDefinition {
        name,
        args,
        body,
        return_type,
        return_type_span,
        span,
    })
}

fn parse_function_body(tokens: &mut Parser) -> Result<Block, ParseError> {
    tokens.expect("{")?;

    let mut children = vec![];

    loop {
        let token = tokens.peek_or_eof("}")?;

        // `}` means the function has no implicit return expression.
        if token.token_type == Token::Op(OperatorType::CloseCurly) {
            break;
        }

        let item = parse_statement(tokens)?;

        if let Some(statement) = item {
            children.push(StatementOrExpression::Statement(Box::new(statement)));

            // Statements still require their normal semicolon rules.
            match children.last().unwrap() {
                StatementOrExpression::Statement(statement) if !statement.requires_semicolon() => {}

                _ => {
                    tokens.expect(";")?;
                }
            }
        } else {
            match parse_expression_or_assignment(tokens)? {
                // Assignments are statements, never an implicit return.
                StatementOrExpression::Statement(assign) => {
                    children.push(StatementOrExpression::Statement(assign));
                    tokens.expect(";")?;
                }

                StatementOrExpression::Expression(expression) => match tokens.peek() {
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
                        tokens.expect(";")?;
                        children.push(StatementOrExpression::Expression(expression));
                    }
                },
            }
        }
    }

    tokens.expect("}")?;

    Ok(Block { children })
}

fn parse_return(tokens: &mut Parser) -> Result<ReturnStatement, ParseError> {
    let span = tokens.peek_or_eof("return")?.span;
    tokens.expect("return")?;
    let expr = match tokens.peek().map(|tok| &tok.token_type) {
        Some(Token::Atom(LexerAtomType::Semicolon) | Token::Op(OperatorType::CloseCurly)) => None,
        _ => Some(parse_expression(tokens, 0)?),
    };
    Ok(ReturnStatement { expr, span })
}

fn parse_struct_definition(tokens: &mut Parser) -> Result<StructDefinition, ParseError> {
    tokens.expect("struct")?;
    let tok = tokens.next_or_eof("identifier")?;
    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(struct_name)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };

    let mut generics = vec![];

    // Check for generic type
    if let Token::Op(OperatorType::LessThan) = tokens.peek_or_eof("< or {")?.token_type {
        //TODO: Parse multiple generics
        tokens.expect("<")?;
        let tok = tokens.next_or_eof("generic name")?;
        let Token::Atom(LexerAtomType::Identifier(generic)) = tok.token_type else {
            return Err(expected("generic name", tok));
        };
        generics.push(generic);
        tokens.expect(">")?;
    }

    tokens.expect("{")?;
    let mut fields = vec![];
    loop {
        let tok = tokens.peek_or_eof("field name or }")?;
        if !matches!(tok.token_type, Token::Atom(LexerAtomType::Identifier(_))) {
            break;
        }
        let Token::Atom(LexerAtomType::Identifier(name)) = tokens.next().unwrap().token_type else {
            unreachable!();
        };
        tokens.expect(":")?;
        let (ftype, span) = parse_type_with_span(tokens)?;
        fields.push(StructField { name, ftype, span });

        let tok = tokens.peek_or_eof(", or }")?;
        if tok.token_type == Token::Atom(LexerAtomType::Comma) {
            tokens.next();
        } else {
            break;
        }
    }
    tokens.expect("}")?;

    Ok(StructDefinition {
        name: struct_name,
        generics,
        fields,
        span,
    })
}

pub fn parse_import(tokens: &mut Parser) -> Result<Import, ParseError> {
    let span = tokens.expect("use")?.span;
    let mut path = vec![parse_import_name(tokens)?.name];
    tokens.expect("::")?;

    let names = loop {
        if tokens.peek_or_eof("{")?.token_type == Token::Op(OperatorType::OpenCurly) {
            break parse_import_list(tokens)?;
        }

        let name = parse_import_name(tokens)?;
        if tokens
            .peek()
            .is_some_and(|tok| tok.token_type == Token::Op(OperatorType::PathSep))
        {
            tokens.next();
            path.push(name.name);
        } else {
            break vec![name];
        }
    };

    tokens.expect(";")?;
    Ok(Import { path, names, span })
}

fn parse_import_name(tokens: &mut Parser) -> Result<ImportName, ParseError> {
    let tok = tokens.next_or_eof("identifier")?;
    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(name)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };
    Ok(ImportName { name, span })
}

fn parse_import_list(tokens: &mut Parser) -> Result<Vec<ImportName>, ParseError> {
    tokens.expect("{")?;
    let mut names = vec![];
    loop {
        if tokens.peek_or_eof("}")?.token_type == Token::Op(OperatorType::CloseCurly) {
            break;
        }

        names.push(parse_import_name(tokens)?);

        let tok = tokens.peek_or_eof(", or }")?;
        if tok.token_type == Token::Atom(LexerAtomType::Comma) {
            tokens.next();
        } else {
            break;
        }
    }
    tokens.expect("}")?;
    Ok(names)
}

fn parse_enum_definition(tokens: &mut Parser) -> Result<EnumDefinition, ParseError> {
    tokens.expect("enum")?;
    let tok = tokens.next_or_eof("identifier")?;
    let span = tok.span;
    let Token::Atom(LexerAtomType::Identifier(enum_name)) = tok.token_type else {
        return Err(expected("identifier", tok));
    };

    tokens.expect("{")?;
    let mut variants = vec![];
    loop {
        let tok = tokens.peek_or_eof("variant name or }")?;
        if !matches!(tok.token_type, Token::Atom(LexerAtomType::Identifier(_))) {
            break;
        }
        let Token::Atom(LexerAtomType::Identifier(name)) = tokens.next().unwrap().token_type else {
            unreachable!();
        };

        variants.push(name);

        let tok = tokens.peek_or_eof(", or }")?;
        if tok.token_type == Token::Atom(LexerAtomType::Comma) {
            tokens.next();
        } else {
            break;
        }
    }
    tokens.expect("}")?;

    Ok(EnumDefinition {
        name: enum_name,
        variants,
        span,
    })
}
