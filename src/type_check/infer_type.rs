use crate::{
    ast::{AtomType, BinaryOp, Expression, ExpressionKind},
    type_check::{Type, TypeCheckError, TypeScope},
};

pub fn infer_type_from_expression(
    expression: &Expression,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    Ok(match &expression.kind {
        ExpressionKind::Atom(atom_type) => {
            if let AtomType::Identifier(ident) = atom_type {
                type_scope.lookup(ident).unwrap().clone() // TODO: This shouldn't be falible because the parser checks for this i think maybe
            } else {
                Type::from(atom_type)
            }
        }
        ExpressionKind::Binary(binary_op, lhs, rhs) => {
            // TODO: There are probably cases where 2 non-exact types are ok in binary op
            let lhs = infer_type_from_expression(lhs, type_scope)?;
            let rhs = infer_type_from_expression(rhs, type_scope)?;

            match (&lhs, &rhs) {
                (Type::I32, Type::I32) => Type::I32,
                (Type::I32, Type::I64) | (Type::I64, Type::I32) => Type::I64,
                (Type::U32, Type::U32) => Type::U32,
                (Type::U32, Type::U64) | (Type::U64, Type::U32) => Type::U64,
                (Type::I32, Type::U32 | Type::U64) | (Type::U32, Type::I32 | Type::I64) => {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: lhs,
                        got: rhs,
                        span: expression.span,
                    });
                }
                (Type::I32 | Type::U32 | Type::I64 | Type::U64, Type::CoercibleNumber) => lhs,
                (Type::CoercibleNumber, Type::I32 | Type::U32 | Type::I64 | Type::U64) => rhs,
                (Type::CoercibleNumber, Type::CoercibleNumber) => Type::CoercibleNumber,
                // TODO: This will need to get updated when we support floats
                (_, Type::CoercibleNumber) => {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: Type::CoercibleNumber,
                        got: lhs,
                        span: expression.span,
                    });
                }
                (Type::CoercibleNumber, _) => {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: Type::CoercibleNumber,
                        got: rhs,
                        span: expression.span,
                    });
                }
                (Type::Function(f), _) if f.return_type == rhs => rhs,
                (_, Type::Function(f)) if f.return_type == lhs => lhs,
                (Type::Unknown, _) => rhs,
                (_, Type::Unknown) => lhs,
                (l, r) if l == r => lhs,
                _ => {
                    return Err(TypeCheckError::TypeMismatch {
                        expected: lhs,
                        got: rhs,
                        span: expression.span,
                    });
                }
            }
        }
        ExpressionKind::Unary(unary_op, expression) => todo!(),
        ExpressionKind::Postfix(postfix_op, expression) => todo!(),
        ExpressionKind::FunctionCall(_, expressions) => todo!(),
        ExpressionKind::Index(expression, expression1) => todo!(),
    })
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Expression, StatementOrExpression},
        parser::parse,
        type_check::{Type, TypeScope, infer_type_from_expression},
    };

    #[test]
    fn infers_type_from_literal() {
        let program = parse("1;").unwrap();
        let Some(StatementOrExpression::Expression(expression)) = program.block.children.first()
        else {
            panic!("expected an expression");
        };
        assert_eq!(
            infer_type_from_expression(expression, &mut TypeScope::default()).unwrap(),
            Type::CoercibleNumber
        );
    }

    #[test]
    fn infers_type_from_variable() {
        let program = parse("let a: u32 = 1; a;").unwrap();
        let mut type_scope = TypeScope::default();

        let StatementOrExpression::Statement(declaration) = &program.block.children[0] else {
            panic!("expected a statement");
        };
        declaration.check_types(&mut type_scope).unwrap();

        let StatementOrExpression::Expression(expression) = &program.block.children[1] else {
            panic!("expected an expression");
        };
        assert_eq!(
            infer_type_from_expression(expression, &mut type_scope).unwrap(),
            Type::U32
        );
    }

    #[test]
    fn infers_type_from_binary_op_literals() {
        let program = parse("1 + 2;").unwrap();
        let mut type_scope = TypeScope::default();

        let StatementOrExpression::Expression(expr) = &program.block.children[0] else {
            panic!("expected an expression");
        };

        assert_eq!(
            infer_type_from_expression(expr, &mut type_scope).unwrap(),
            Type::CoercibleNumber
        );
    }
}
