use crate::{
    ast::{
        AssignOp, Assignment, AtomType, BinaryOp, Expression, ExpressionKind, OperatorType,
        PostfixOp, Statement, StatementOrExpression, UnaryOp,
    },
    error::At,
    lex::{Lexer, LexerAtomType},
    parse::{ParseError, ParseErrorKind, expected, parse_statement::parse_type},
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
    if !lhs.is_place() {
        return Err(ParseErrorKind::InvalidAssignTarget(lhs.to_string()).at(lhs.span));
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
    parse_expression_with(lexer, min_bp, true)
}

pub fn parse_condition(lexer: &mut Lexer) -> Result<Expression, ParseError> {
    parse_expression_with(lexer, 0, false)
}

fn parse_expression_with(
    lexer: &mut Lexer,
    min_bp: u8,
    allow_struct: bool,
) -> Result<Expression, ParseError> {
    lexer.nested("expression", |lexer| {
        parse_expression_at_depth(lexer, min_bp, allow_struct)
    })
}

fn parse_expression_at_depth(
    lexer: &mut Lexer,
    min_bp: u8,
    allow_struct: bool,
) -> Result<Expression, ParseError> {
    let token = lexer.next_or_eof("expression")?;

    let mut lhs = match &token.token_type {
        Token::Atom(LexerAtomType::Identifier(s)) => {
            parse_identifier_or_function_call_or_struct(lexer, s, token.span, allow_struct)?
        }
        Token::Atom(it) => {
            let Some(atom) = AtomType::from_lexer_atom(it) else {
                return Err(expected("expression", token));
            };
            Expression::new(ExpressionKind::Atom(atom), token.span)
        }
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
            let rhs = parse_expression_with(lexer, UnaryOp::BINDING_POWER, allow_struct)?;
            if op == UnaryOp::AddressOf && !rhs.is_place() {
                return Err(ParseErrorKind::NotAddressable(rhs.to_string()).at(rhs.span));
            }
            Expression::new(ExpressionKind::Unary(op, Box::new(rhs)), token.span)
        }
    };

    while let Some(op_token) = lexer.peek() {
        let Token::Op(op) = &op_token.token_type else {
            break;
        };

        if let OperatorType::OpenBracket | OperatorType::Dot = op {
            const ACCESS_BINDING_POWER: u8 = 13;
            if ACCESS_BINDING_POWER < min_bp {
                break;
            }

            let is_index = matches!(op, OperatorType::OpenBracket);
            lexer.next_nested()?;
            let span = lhs.span;
            let kind = if is_index {
                let index = parse_expression(lexer, 0)?;
                lexer.expect("]")?;
                ExpressionKind::Index(Box::new(lhs), Box::new(index))
            } else {
                if !lhs.is_place() {
                    return Err(ParseErrorKind::NotAddressable(lhs.to_string()).at(span));
                }
                let token = lexer.next_or_eof("field name")?;
                let Token::Atom(LexerAtomType::Identifier(field_name)) = &token.token_type else {
                    return Err(expected("field name", token));
                };
                ExpressionKind::StructAccess {
                    base: Box::new(lhs),
                    field_name: field_name.clone(),
                }
            };
            lhs = Expression::new(kind, span);
            continue;
        }

        if let OperatorType::Cast = op {
            const CAST_BINDING_POWER: u8 = 10;
            if CAST_BINDING_POWER < min_bp {
                break;
            }

            lexer.next_nested()?;
            let cast_type = parse_type(lexer)?;
            let span = lhs.span;
            lhs = Expression::new(ExpressionKind::Cast(Box::new(lhs), cast_type), span);
            continue;
        }

        if let Some(op) = PostfixOp::from_token(op) {
            if PostfixOp::BINDING_POWER < min_bp {
                break;
            }
            lexer.next_nested()?;
            if !lhs.is_place() {
                return Err(ParseErrorKind::InvalidAssignTarget(lhs.to_string()).at(lhs.span));
            }

            let span = lhs.span;
            lhs = Expression::new(ExpressionKind::Postfix(op, Box::new(lhs)), span);
            continue;
        }

        if let Some(op) = BinaryOp::from_token(op) {
            let (l_bp, r_bp) = op.binding_power();
            if l_bp < min_bp {
                break;
            }
            lexer.next_nested()?;
            let rhs = parse_expression_with(lexer, r_bp, allow_struct)?;
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

fn parse_identifier_or_function_call_or_struct(
    lexer: &mut Lexer,
    s: &str,
    span: Span,
    allow_struct: bool,
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
        Some(Token::Op(OperatorType::OpenCurly)) if allow_struct => {
            parse_struct_instantiation(lexer, s.to_string(), span)?
        }
        Some(Token::Op(OperatorType::PathSep)) => parse_enum_variant(lexer, s.to_string(), span)?,
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
            Token::Atom(LexerAtomType::Comma) => (),
            _ => return Err(expected(", or )", next)),
        }
    }

    Ok(Expression::new(ExpressionKind::FunctionCall(s, args), span))
}

fn parse_enum_variant(
    lexer: &mut Lexer,
    enum_name: String,
    span: Span,
) -> Result<Expression, ParseError> {
    lexer.expect("::")?;
    let token = lexer.next_or_eof("variant name")?;
    let Token::Atom(LexerAtomType::Identifier(variant_name)) = token.token_type else {
        return Err(expected("variant name", token));
    };

    Ok(Expression::new(
        ExpressionKind::EnumVariant {
            enum_name,
            variant_name,
        },
        span,
    ))
}

fn parse_struct_instantiation(
    lexer: &mut Lexer,
    struct_type_name: String,
    span: Span,
) -> Result<Expression, ParseError> {
    lexer.expect("{")?;
    let mut fields = vec![];
    loop {
        let next = lexer.peek_or_eof("}")?;
        if next.token_type == Token::Op(OperatorType::CloseCurly) {
            lexer.next().unwrap();
            break;
        }

        let token = lexer.next_or_eof("field name")?;
        let Token::Atom(LexerAtomType::Identifier(field_name)) = &token.token_type else {
            return Err(expected("field name", token));
        };
        let field_name = field_name.clone();

        lexer.expect(":")?;
        let value = parse_expression(lexer, 0)?;
        fields.push((field_name, value));

        let next = lexer.next_or_eof(", or }")?;
        match next.token_type {
            Token::Op(OperatorType::CloseCurly) => break,
            Token::Atom(LexerAtomType::Comma) => (),
            _ => return Err(expected(", or }", next)),
        }
    }

    Ok(Expression::new(
        ExpressionKind::StructInstantiation {
            struct_type_name,
            fields,
        },
        span,
    ))
}
