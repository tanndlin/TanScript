use crate::{
    ast::{
        AssignOp, Assignment, AtomType, BinaryOp, Expression, ExpressionKind, OperatorType,
        PostfixOp, Statement, StatementOrExpression, UnaryOp,
    },
    error::At,
    lex::{Lexer, LexerAtomType},
    parse::{ParseError, ParseErrorKind, expected},
    types::{Span, Token},
};

/// Parses an expression, then treats it as the lhs of an assignment if an assign op follows.
pub fn parse_expression_or_assignment(
    lexer: &mut Lexer,
) -> Result<StatementOrExpression, ParseError> {
    let expression = parse_expression(lexer, 0)?;

    let is_assign = matches!(
        lexer.peek().map(|tok| &tok.token_type),
        Some(Token::Op(op)) if op.as_assign().is_some()
    );

    Ok(if is_assign {
        StatementOrExpression::Statement(Box::new(Statement::Assignment(
            parse_assignment_with_lhs(lexer, expression)?,
        )))
    } else {
        StatementOrExpression::Expression(expression)
    })
}

fn parse_assignment_with_lhs(lexer: &mut Lexer, lhs: Expression) -> Result<Assignment, ParseError> {
    match lhs.kind {
        ExpressionKind::Atom(AtomType::Identifier(_)) | ExpressionKind::Index(_, _) => {}
        _ => return Err(ParseErrorKind::InvalidAssignTarget(lhs.to_string()).at(lhs.span)),
    }

    let tok = lexer.next_or_eof("assignment operator")?;
    let Some(op) = (match &tok.token_type {
        Token::Op(op) => op.as_assign(),
        Token::Atom(_) => None,
    }) else {
        return Err(expected("assignment operator", tok));
    };

    let rhs = parse_expression(lexer, 0)?;
    let expression = match op {
        AssignOp::Compound(op) => Expression::new(
            ExpressionKind::Binary(op, Box::new(lhs.clone()), Box::new(rhs)),
            lhs.span,
        ),
        AssignOp::Assign => rhs,
    };

    Ok(Assignment { lhs, expression })
}

pub fn parse_expression(lexer: &mut Lexer, min_bp: u8) -> Result<Expression, ParseError> {
    let token = lexer.next_or_eof("expression")?;

    let mut lhs = match &token.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => {
            parse_identifier_or_function_call(lexer, s, token.span)?
        }
        Token::Atom(it) => Expression::new(
            ExpressionKind::Atom(AtomType::from_lexer_atom(it)),
            token.span,
        ),
        Token::Op(OperatorType::OpenParen) => {
            let mut lhs = parse_expression(lexer, 0)?;
            lexer.expect(")")?;
            lhs.span = token.span;
            lhs
        }
        Token::Op(op) => {
            let Some(op) = UnaryOp::from_token(op) else {
                return Err(expected("expression", token));
            };
            let rhs = parse_expression(lexer, UnaryOp::BINDING_POWER)?;
            Expression::new(ExpressionKind::Unary(op, Box::new(rhs)), token.span)
        }
    };

    while let Some(op_token) = lexer.peek() {
        let Token::Op(op) = &op_token.token_type else {
            break;
        };

        if let OperatorType::OpenBracket = op {
            const INDEX_BINDING_POWER: u8 = 13;
            if INDEX_BINDING_POWER < min_bp {
                break;
            }

            lexer.next();
            let index = parse_expression(lexer, 0)?;
            lexer.expect("]")?;
            let span = lhs.span;
            lhs = Expression::new(ExpressionKind::Index(Box::new(lhs), Box::new(index)), span);
            continue;
        }

        if let Some(op) = PostfixOp::from_token(op) {
            if PostfixOp::BINDING_POWER < min_bp {
                break;
            }
            lexer.next();
            let operand = match lhs.kind {
                ExpressionKind::Atom(AtomType::Identifier(_)) | ExpressionKind::Index(_, _) => lhs,
                _ => {
                    return Err(ParseErrorKind::InvalidAssignTarget(lhs.to_string()).at(lhs.span));
                }
            };

            let span = operand.span;
            lhs = Expression::new(ExpressionKind::Postfix(op, Box::new(operand)), span);
            continue;
        }

        if let Some(op) = BinaryOp::from_token(op) {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            lexer.next();
            let rhs = parse_expression(lexer, r_bp)?;
            let span = lhs.span;
            lhs = Expression::new(
                ExpressionKind::Binary(op, Box::new(lhs), Box::new(rhs)),
                span,
            );
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn parse_identifier_or_function_call(
    lexer: &mut Lexer,
    s: &str,
    span: Span,
) -> Result<Expression, ParseError> {
    if matches!(s, "true" | "false") {
        return Ok(Expression::new(
            ExpressionKind::Atom(AtomType::Boolean(s == "true")),
            span,
        ));
    }

    Ok(match lexer.peek().map(|t| &t.token_type) {
        Some(Token::Op(OperatorType::OpenParen)) => {
            parse_function_call(lexer, s.to_string(), span)?
        }
        _ => Expression::new(
            ExpressionKind::Atom(AtomType::Identifier(s.to_string())),
            span,
        ),
    })
}

fn parse_function_call(lexer: &mut Lexer, s: String, span: Span) -> Result<Expression, ParseError> {
    lexer.expect("(")?;

    let mut args = vec![];
    loop {
        let next = lexer.peek_or_eof(")")?;

        if next.token_type == Token::Op(OperatorType::CloseParen) {
            lexer.next().unwrap();
            break;
        }

        let arg = parse_expression(lexer, 0)?;
        args.push(arg);
        let next = lexer.next_or_eof(", or )")?;
        match next.token_type {
            Token::Op(OperatorType::CloseParen) => break,
            Token::Op(OperatorType::Comma) => (),
            _ => return Err(expected(", or )", next)),
        }
    }

    Ok(Expression::new(ExpressionKind::FunctionCall(s, args), span))
}
