use crate::{
    ast::{AtomType, Expression, ExpressionKind, UnaryOp},
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
        ExpressionKind::Binary(op, lhs, rhs) => {
            // TODO: There are probably cases where 2 non-exact types are ok in binary op
            let lhs = infer_type_from_expression(lhs, type_scope)?;
            let rhs = infer_type_from_expression(rhs, type_scope)?;

            let unified = unify(&lhs, &rhs)
                .or_else(|| unify(&rhs, &lhs))
                .ok_or_else(|| TypeCheckError::TypeMismatch {
                    expected: lhs.clone(),
                    got: rhs.clone(),
                    span: expression.span,
                })?;

            if op.returns_boolean() {
                Type::Boolean
            } else {
                unified
            }
        }
        ExpressionKind::Unary(UnaryOp::Not, expression) => {
            infer_type_from_expression(expression, type_scope)?;
            Type::Boolean
        }
        ExpressionKind::Unary(_, expression) => infer_type_from_expression(expression, type_scope)?,
        ExpressionKind::Postfix(_, expression) => {
            infer_type_from_expression(expression, type_scope)?
        }
        ExpressionKind::FunctionCall(name, _) => match type_scope.lookup(name) {
            Ok(Type::Function(f)) => f.return_type.clone(),
            Ok(t) => t.clone(),
            Err(_) => Type::Unknown,
        },
        ExpressionKind::Index(lhs, _) => match infer_type_from_expression(lhs, type_scope)? {
            Type::Pointer(t) => *t,
            _ => Type::Unknown,
        },
    })
}

fn unify(a: &Type, b: &Type) -> Option<Type> {
    match (a, b) {
        _ if a == b => Some(a.clone()),
        (Type::I32, Type::I64) => Some(Type::I64),
        (Type::U32, Type::U64) => Some(Type::U64),
        (Type::I32 | Type::U32 | Type::I64 | Type::U64, Type::CoercibleNumber) => Some(a.clone()),
        (Type::Function(f), _) if f.return_type == *b => Some(b.clone()),
        (Type::Unknown, _) => Some(b.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::StatementOrExpression,
        parser::parse,
        type_check::{Type, TypeCheckError, TypeScope, infer_type_from_expression},
    };

    fn infer_last(code: &str) -> Result<Type, TypeCheckError> {
        let program = parse(code).unwrap();
        let mut type_scope = TypeScope::default();
        let (last, rest) = program.block.children.split_last().unwrap();
        for child in rest {
            if let StatementOrExpression::Statement(statement) = child {
                statement.check_types(&mut type_scope).unwrap();
            }
        }
        let StatementOrExpression::Expression(expression) = last else {
            panic!("expected an expression");
        };
        infer_type_from_expression(expression, &mut type_scope)
    }

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

    #[test]
    fn infers_type_from_untyped_variable() {
        assert_eq!(infer_last("let a = 1; a;").unwrap(), Type::CoercibleNumber);
    }

    #[test]
    fn infers_type_from_typed_variable_plus_literal() {
        assert_eq!(infer_last("let a: i32 = 1; a + 2;").unwrap(), Type::I32);
    }

    #[test]
    fn infers_type_from_literal_plus_typed_variable() {
        assert_eq!(infer_last("let a: u64 = 1; 2 + a;").unwrap(), Type::U64);
    }

    #[test]
    fn infers_type_from_same_typed_variables() {
        assert_eq!(
            infer_last("let a: i64 = 1; let b: i64 = 2; a + b;").unwrap(),
            Type::I64
        );
    }

    #[test]
    fn infers_type_from_nested_binary_ops() {
        assert_eq!(
            infer_last("let a: u32 = 1; let b: u32 = 2; a + b * 3;").unwrap(),
            Type::U32
        );
    }

    #[test]
    fn type_error_for_mismatched_variables() {
        let result = infer_last("let a: u32 = 1; let b: i32 = 2; a + b;");
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_mismatched_nested_binary_ops() {
        let result = infer_last("let a: u32 = 1; let b: i64 = 2; (a + 1) * (b + 2);");
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn infers_type_from_function_call() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} add(1, 2);").unwrap(),
            Type::U32
        );
    }

    #[test]
    fn infers_type_from_function_call_plus_variable() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} let c: u32 = 3; add(1, 2) + c;")
                .unwrap(),
            Type::U32
        );
    }

    #[test]
    fn infers_type_from_unary_op() {
        assert_eq!(infer_last("let a: i32 = 1; -a;").unwrap(), Type::I32);
    }

    #[test]
    fn infers_type_from_postfix_op() {
        assert_eq!(infer_last("let a: i32 = 1; a++;").unwrap(), Type::I32);
    }

    #[test]
    fn infers_type_from_boolean_literal() {
        assert_eq!(infer_last("true;").unwrap(), Type::Boolean);
        assert_eq!(infer_last("false;").unwrap(), Type::Boolean);
    }

    #[test]
    fn infers_type_from_boolean_variable() {
        assert_eq!(infer_last("let a = true; a;").unwrap(), Type::Boolean);
    }

    #[test]
    fn infers_type_from_boolean_binary_op() {
        assert_eq!(infer_last("true && false;").unwrap(), Type::Boolean);
    }

    #[test]
    fn type_error_for_boolean_plus_number() {
        let result = infer_last("let a: i32 = 1; true + a;");
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn infers_boolean_from_comparison() {
        for op in ["<", "<=", ">", ">=", "==", "!="] {
            assert_eq!(
                infer_last(&format!("let a: i32 = 1; let b: i32 = 2; a {op} b;")).unwrap(),
                Type::Boolean,
                "{op}"
            );
        }
    }

    #[test]
    fn infers_boolean_from_not() {
        assert_eq!(infer_last("let a: i32 = 1; !a;").unwrap(), Type::Boolean);
    }

    #[test]
    fn type_error_for_comparing_mismatched_types() {
        let result = infer_last("let a: u32 = 1; let b: i32 = 2; a < b;");
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }
}
