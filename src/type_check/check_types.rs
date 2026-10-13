use crate::{
    ast::{AtomType, Expression, ExpressionKind, Program, UnaryOp},
    type_check::{Type, TypeCheckError, TypeCheckErrorKind, TypeScope, infer_type::unify},
};

pub fn check_types(program: &mut Program) -> Result<(), TypeCheckError> {
    let mut type_scope = TypeScope::default();

    program.block.check_types(&mut type_scope)?;
    program.block.resolve_types(&type_scope);
    Ok(())
}

pub fn check_types_expression(
    expression: &Expression,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    Ok(match &expression.kind {
        ExpressionKind::Atom(AtomType::Identifier(ident)) => type_scope
            .lookup(ident)
            .ok_or_else(|| {
                TypeCheckError::new(
                    TypeCheckErrorKind::IdentifierNotFound(ident.clone()),
                    expression.span,
                )
            })?
            .clone(),
        ExpressionKind::Atom(AtomType::String(_)) => Type::Unknown,
        ExpressionKind::Atom(atom_type) => Type::from(atom_type),
        ExpressionKind::Binary(op, lhs, rhs) => {
            let lhs = check_types_expression(lhs, type_scope)?;
            let rhs = check_types_expression(rhs, type_scope)?;

            let unified = unify(&lhs, &rhs)
                .or_else(|| unify(&rhs, &lhs))
                .ok_or_else(|| {
                    TypeCheckError::new(
                        TypeCheckErrorKind::TypeMismatch {
                            expected: lhs.clone(),
                            got: rhs.clone(),
                        },
                        expression.span,
                    )
                })?;

            if op.returns_boolean() {
                Type::Boolean
            } else {
                unified
            }
        }
        ExpressionKind::Unary(UnaryOp::Not, inner) => {
            check_types_expression(inner, type_scope)?;
            Type::Boolean
        }
        ExpressionKind::Unary(_, inner) | ExpressionKind::Postfix(_, inner) => {
            check_types_expression(inner, type_scope)?
        }
        ExpressionKind::FunctionCall(function_name, given_args) => {
            let given_types = given_args
                .iter()
                .map(|a| check_types_expression(a, type_scope))
                .collect::<Result<Vec<_>, _>>()?;

            match type_scope.lookup(function_name) {
                Some(Type::Function(function_type)) => {
                    let function_type = function_type.clone();
                    given_args
                        .iter()
                        .zip(given_types)
                        .zip(&function_type.args)
                        .try_for_each(|((given, got), expected)| {
                            expect_type(given, expected, &got, type_scope)
                        })?;
                    function_type.return_type
                }
                Some(_) => {
                    return Err(TypeCheckError::new(
                        TypeCheckErrorKind::NotCallableIdentifer(function_name.clone()),
                        expression.span,
                    ));
                }
                // Treat the function as an unknown extern
                None => Type::Unknown,
            }
        }
        ExpressionKind::Index(lhs, rhs) => {
            let lhs_type = check_types_expression(lhs, type_scope)?;
            let rhs_type = check_types_expression(rhs, type_scope)?;

            // LHS must be a pointer
            let element_type = match lhs_type {
                Type::Pointer(t) => *t,
                Type::Unknown => Type::Unknown,
                _ => {
                    return Err(TypeCheckError::new(
                        TypeCheckErrorKind::TypeMismatch {
                            expected: Type::Pointer(Box::new(Type::Unknown)),
                            got: lhs_type,
                        },
                        expression.span,
                    ));
                }
            };

            // RHS must come down to a number
            if !rhs_type.is_number_type() {
                return Err(TypeCheckError::new(
                    TypeCheckErrorKind::TypeMismatch {
                        expected: Type::I64,
                        got: rhs_type,
                    },
                    expression.span,
                ));
            }

            element_type
        }
    })
}

pub fn expect_type(
    expression: &Expression,
    expected: &Type,
    got: &Type,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    match (expected, got) {
        _ if expected == got => Ok(()),
        (Type::Unknown, _) | (_, Type::Unknown) => Ok(()),
        (t, Type::CoercibleNumber) if t.is_number_type() => {
            narrow_expression(expression, expected.clone(), type_scope);
            Ok(())
        }
        _ => Err(TypeCheckError::new(
            TypeCheckErrorKind::TypeMismatch {
                expected: expected.clone(),
                got: got.clone(),
            },
            expression.span,
        )),
    }
}

pub fn narrow_expression(expression: &Expression, t: Type, type_scope: &mut TypeScope) {
    match &expression.kind {
        ExpressionKind::Atom(AtomType::Identifier(ident)) => type_scope.narrow(ident, t),
        ExpressionKind::Binary(_, lhs, rhs) => {
            narrow_expression(lhs, t.clone(), type_scope);
            narrow_expression(rhs, t, type_scope);
        }
        ExpressionKind::Unary(_, inner) | ExpressionKind::Postfix(_, inner) => {
            narrow_expression(inner, t, type_scope);
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Statement, StatementOrExpression},
        parser::parse,
        type_check::{Type, TypeCheckError, TypeCheckErrorKind, check_types},
    };

    #[test]
    fn supports_coercing_literal_assignments() -> Result<(), TypeCheckError> {
        let mut program = parse("let a: u32 = 1;").unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_referenced_variable_in_assignment() {
        let mut program = parse("let a: u32 = 1; let b: i32 = a;").unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_incorrect_function_arg() {
        let mut program =
            parse("def add (a: u32, b: u32) {a + b} let a: i32 = 1; let b: u32 = 2; add(a,b);")
                .unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_coerced_variable_used_as_arg_then_assigned_to_different_type() {
        let mut program = parse(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; add(a, b); let c: i32 = a;",
        )
        .unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_coerced_variable_assigned_then_used_as_arg_of_different_type() {
        let mut program = parse(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; let c: i32 = a; add(a, b);",
        )
        .unwrap();
        let result = check_types(&mut program);
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
    fn supports_boolean_declaration() -> Result<(), TypeCheckError> {
        let mut program =
            parse("let a: bool = true; let b: bool = false; let c: bool = a;").unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_boolean_assigned_to_number() {
        let mut program = parse("let a: u32 = true;").unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_number_assigned_to_boolean() {
        let mut program = parse("let a: bool = 1;").unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_boolean_function_arg() {
        let mut program = parse("def f (a: u32): u32 {a} f(true);").unwrap();
        let result = check_types(&mut program);
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
    fn supports_comparison_assigned_to_boolean() -> Result<(), TypeCheckError> {
        let mut program = parse("let a: i32 = 1; let b: bool = a < 2;").unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_using_function_params_in_body() -> Result<(), TypeCheckError> {
        let mut program = parse("def f (arr: *i32, i: i32): i32 { let x = arr[i]; x }").unwrap();
        check_types(&mut program)
    }

    #[test]
    fn shadowed_declaration_keeps_its_own_type() {
        let mut program =
            parse("let i: bool = true; for (let i = 0; i < 10; i++) { let j: i32 = i; }").unwrap();
        check_types(&mut program).unwrap();

        let StatementOrExpression::Statement(statement) = &program.block.children[1] else {
            panic!("expected a statement");
        };
        let Statement::ForLoop(for_loop) = statement.as_ref() else {
            panic!("expected a for loop");
        };
        let StatementOrExpression::Statement(init) = &for_loop.init else {
            panic!("expected a statement");
        };
        let Statement::Declaration(declaration) = init.as_ref() else {
            panic!("expected a declaration");
        };
        assert_eq!(declaration.dtype, Some(Type::I32));
    }

    #[test]
    fn type_error_inside_loop_body() {
        let mut program = parse("let a: u32 = 1; while true { let b: i32 = a; }").unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_mismatched_assignment() {
        let mut program = parse("let a: u32 = 1; let b: i32 = 2; a = b;").unwrap();
        let result = check_types(&mut program);
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
    fn type_error_for_number_stored_in_bool_pointer() {
        let mut program = parse("def f (flags: *bool) { flags[0] = 300; }").unwrap();
        let result = check_types(&mut program);
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
