use crate::{type_check::Type, types::IntType};

pub(super) fn unify(a: &Type, b: &Type) -> Option<Type> {
    match (a, b) {
        _ if a == b => Some(a.clone()),
        (Type::Int(IntType::I32), Type::Int(IntType::I64)) => Some(Type::Int(IntType::I64)),
        (Type::Int(IntType::U32), Type::Int(IntType::U64)) => Some(Type::Int(IntType::U64)),
        (Type::Int(_), Type::Int(IntType::Coercible) | Type::Boolean) => Some(a.clone()),
        (Type::Char, Type::Int(IntType::Coercible)) => Some(Type::Char),
        (Type::Function(f), _) if f.return_type == *b => Some(b.clone()),
        (Type::Unknown, _) => Some(b.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::StatementOrExpression,
        lex::Lexer,
        parse::parse,
        type_check::{Type, TypeCheckError, TypeCheckErrorKind, TypeScope, check_types_expression},
        types::IntType,
    };

    fn infer_last(code: &str) -> Result<Type, TypeCheckError> {
        let mut program = parse(Lexer::new(code).unwrap()).unwrap();
        let mut type_scope = TypeScope::default();
        let (last, rest) = program.block.children.split_last_mut().unwrap();
        for child in rest {
            if let StatementOrExpression::Statement(statement) = child {
                statement.check_types(&mut type_scope).unwrap();
            }
        }
        let StatementOrExpression::Expression(expression) = last else {
            panic!("expected an expression");
        };
        check_types_expression(expression, &mut type_scope)
    }

    #[test]
    fn infers_type_from_literal() {
        let program = parse(Lexer::new("1;").unwrap()).unwrap();
        let Some(StatementOrExpression::Expression(expression)) = program.block.children.first()
        else {
            panic!("expected an expression");
        };
        assert_eq!(
            check_types_expression(expression, &mut TypeScope::default()).unwrap(),
            Type::Int(IntType::Coercible)
        );
    }

    #[test]
    fn infers_type_from_variable() {
        let mut program = parse(Lexer::new("let a: u32 = 1; a;").unwrap()).unwrap();
        let mut type_scope = TypeScope::default();

        let StatementOrExpression::Statement(declaration) = &mut program.block.children[0] else {
            panic!("expected a statement");
        };
        declaration.check_types(&mut type_scope).unwrap();

        let StatementOrExpression::Expression(expression) = &program.block.children[1] else {
            panic!("expected an expression");
        };
        assert_eq!(
            check_types_expression(expression, &mut type_scope).unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_binary_op_literals() {
        let program = parse(Lexer::new("1 + 2;").unwrap()).unwrap();
        let mut type_scope = TypeScope::default();

        let StatementOrExpression::Expression(expr) = &program.block.children[0] else {
            panic!("expected an expression");
        };

        assert_eq!(
            check_types_expression(expr, &mut type_scope).unwrap(),
            Type::Int(IntType::Coercible)
        );
    }

    #[test]
    fn infers_type_from_untyped_variable() {
        assert_eq!(
            infer_last("let a = 1; a;").unwrap(),
            Type::Int(IntType::Coercible)
        );
    }

    #[test]
    fn infers_type_from_typed_variable_plus_literal() {
        assert_eq!(
            infer_last("let a: i32 = 1; a + 2;").unwrap(),
            Type::Int(IntType::I32)
        );
    }

    #[test]
    fn infers_type_from_literal_plus_typed_variable() {
        assert_eq!(
            infer_last("let a: u64 = 1; 2 + a;").unwrap(),
            Type::Int(IntType::U64)
        );
    }

    #[test]
    fn infers_type_from_same_typed_variables() {
        assert_eq!(
            infer_last("let a: i64 = 1; let b: i64 = 2; a + b;").unwrap(),
            Type::Int(IntType::I64)
        );
    }

    #[test]
    fn infers_type_from_nested_binary_ops() {
        assert_eq!(
            infer_last("let a: u32 = 1; let b: u32 = 2; a + b * 3;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn type_error_for_mismatched_variables() {
        let result = infer_last("let a: u32 = 1; let b: i32 = 2; a + b;");
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_mismatched_nested_binary_ops() {
        let result = infer_last("let a: u32 = 1; let b: i64 = 2; (a + 1) * (b + 2);");
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn infers_type_from_function_call() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} add(1, 2);").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_function_call_plus_variable() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} let c: u32 = 3; add(1, 2) + c;")
                .unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_unary_op() {
        assert_eq!(
            infer_last("let a: i32 = 1; -a;").unwrap(),
            Type::Int(IntType::I32)
        );
    }

    #[test]
    fn infers_type_from_postfix_op() {
        assert_eq!(
            infer_last("let a: i32 = 1; a++;").unwrap(),
            Type::Int(IntType::I32)
        );
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

    // Might want to enable this again once I implement `true as u32`
    // #[test]
    // fn type_error_for_boolean_plus_number() {
    //     let result = infer_last("let a: i32 = 1; true + a;");
    //     assert!(
    //         matches!(
    //             result,
    //             Err(TypeCheckError {
    //                 kind: TypeCheckErrorKind::TypeMismatch { .. },
    //                 ..
    //             })
    //         ),
    //         "expected TypeMismatch, got {result:?}"
    //     );
    // }

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
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }
}
