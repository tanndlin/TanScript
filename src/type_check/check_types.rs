use crate::{
    ast::{AtomType, Expression, ExpressionKind, Program, UnaryOp},
    error::At,
    type_check::{Type, TypeCheckError, TypeCheckErrorKind, TypeScope, infer_type::unify},
    types::IntType,
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
                TypeCheckErrorKind::IdentifierNotFound(ident.clone()).at(expression.span)
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
                    TypeCheckErrorKind::TypeMismatch {
                        expected: lhs.clone(),
                        got: rhs.clone(),
                    }
                    .at(expression.span)
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
                    return Err(
                        TypeCheckErrorKind::NotCallableIdentifer(function_name.clone())
                            .at(expression.span),
                    );
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
                    return Err(TypeCheckErrorKind::TypeMismatch {
                        expected: Type::Pointer(Box::new(Type::Unknown)),
                        got: lhs_type,
                    }
                    .at(expression.span));
                }
            };

            // RHS must come down to a number
            if rhs_type != Type::Unknown && !rhs_type.is_number_type() {
                return Err(TypeCheckErrorKind::TypeMismatch {
                    expected: Type::Int(IntType::I64),
                    got: rhs_type,
                }
                .at(expression.span));
            }

            element_type
        }
        ExpressionKind::Cast(lhs, casted_type) => {
            let lhs_type = check_types_expression(lhs, type_scope)?;
            if lhs_type.castable(casted_type) {
                casted_type.clone()
            } else {
                return Err(TypeCheckErrorKind::Uncastable {
                    lhs_type,
                    casted_type: casted_type.clone(),
                }
                .at(expression.span));
            }
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
        (t, Type::Int(IntType::Coercible)) if t.is_number_type() => {
            narrow_expression(expression, expected.clone(), type_scope);
            Ok(())
        }
        _ => Err(TypeCheckErrorKind::TypeMismatch {
            expected: expected.clone(),
            got: got.clone(),
        }
        .at(expression.span)),
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
        lex::Lexer,
        parse::parse,
        type_check::{Type, TypeCheckError, TypeCheckErrorKind, check_types},
        types::IntType,
    };

    #[test]
    fn supports_coercing_literal_assignments() -> Result<(), TypeCheckError> {
        let mut program = parse(Lexer::new("let a: u32 = 1;").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_referenced_variable_in_assignment() {
        let mut program = parse(Lexer::new("let a: u32 = 1; let b: i32 = a;").unwrap()).unwrap();
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
        let mut program = parse(
            Lexer::new(
                "def add (a: u32, b: u32) {a + b} let a: i32 = 1; let b: u32 = 2; add(a,b);",
            )
            .unwrap(),
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
    fn type_error_for_coerced_variable_used_as_arg_then_assigned_to_different_type() {
        let mut program = parse(Lexer::new(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; add(a, b); let c: i32 = a;",
        ).unwrap())
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
        let mut program = parse(Lexer::new(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; let c: i32 = a; add(a, b);",
        ).unwrap())
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
            parse(Lexer::new("let a: bool = true; let b: bool = false; let c: bool = a;").unwrap())
                .unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_boolean_assigned_to_number() {
        let mut program = parse(Lexer::new("let a: u32 = true;").unwrap()).unwrap();
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
        let mut program = parse(Lexer::new("let a: bool = 1;").unwrap()).unwrap();
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
        let mut program = parse(Lexer::new("def f (a: u32): u32 {a} f(true);").unwrap()).unwrap();
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
        let mut program =
            parse(Lexer::new("let a: i32 = 1; let b: bool = a < 2;").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_using_function_params_in_body() -> Result<(), TypeCheckError> {
        let mut program =
            parse(Lexer::new("def f (arr: *i32, i: i32): i32 { let x = arr[i]; x }").unwrap())
                .unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_untyped_char_variable_as_char_arg() -> Result<(), TypeCheckError> {
        let mut program =
            parse(Lexer::new("def f(c: char) {} let c = 'c'; f(c);").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn untyped_char_declaration_is_char() {
        let mut program = parse(Lexer::new("let c = 'c';").unwrap()).unwrap();
        check_types(&mut program).unwrap();

        let StatementOrExpression::Statement(statement) = &program.block.children[0] else {
            panic!("expected a statement");
        };
        let Statement::Declaration(declaration) = statement.as_ref() else {
            panic!("expected a declaration");
        };
        assert_eq!(declaration.dtype, Some(Type::Char));
    }

    #[test]
    fn shadowed_declaration_keeps_its_own_type() {
        let mut program = parse(
            Lexer::new("let i: bool = true; for (let i = 0; i < 10; i++) { let j: i32 = i; }")
                .unwrap(),
        )
        .unwrap();
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
        assert_eq!(declaration.dtype, Some(Type::Int(IntType::I32)));
    }

    #[test]
    fn type_error_inside_loop_body() {
        let mut program =
            parse(Lexer::new("let a: u32 = 1; while true { let b: i32 = a; }").unwrap()).unwrap();
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
    fn cast_allows_assignment_of_different_type() {
        let mut program =
            parse(Lexer::new("let a: u32 = 1; let b: i32 = 2; a = b as u32;").unwrap()).unwrap();
        let result = check_types(&mut program);
        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[test]
    fn type_error_for_uncastable_types() {
        let mut program =
            parse(Lexer::new("let a: i64 = 1; let b: bool = a as bool;").unwrap()).unwrap();
        let result = check_types(&mut program);
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::Uncastable { .. },
                    ..
                })
            ),
            "expected Uncastable, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_mismatched_assignment() {
        let mut program =
            parse(Lexer::new("let a: u32 = 1; let b: i32 = 2; a = b;").unwrap()).unwrap();
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
    fn supports_indexing_with_unknown_type() -> Result<(), TypeCheckError> {
        let mut program =
            parse(Lexer::new("def f (s: *char) { let i = ftell(0); s[i] = 'a'; }").unwrap())
                .unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_number_stored_in_bool_pointer() {
        let mut program =
            parse(Lexer::new("def f (flags: *bool) { flags[0] = 300; }").unwrap()).unwrap();
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
    fn supports_return_matching_declared_return_type() -> Result<(), TypeCheckError> {
        let mut program = parse(Lexer::new("def f (a: u32): u32 { return a; }").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_implicit_return_matching_declared_return_type() -> Result<(), TypeCheckError> {
        let mut program =
            parse(Lexer::new("def f (a: i32, b: i32): i32 { a + b }").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_literal_return_coercing_to_declared_return_type() -> Result<(), TypeCheckError> {
        let mut program = parse(Lexer::new("def one (): u64 { 1 }").unwrap()).unwrap();
        check_types(&mut program)
    }

    #[test]
    fn supports_multiple_returns_matching_declared_return_type() -> Result<(), TypeCheckError> {
        let mut program =
            parse(Lexer::new("def f (a: u32, b: u32): u32 { return a; return b; }").unwrap())
                .unwrap();
        check_types(&mut program)
    }

    #[test]
    fn type_error_for_return_not_matching_declared_return_type() {
        let mut program = parse(Lexer::new("def f (a: i32): u32 { return a; }").unwrap()).unwrap();
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
    fn type_error_for_implicit_return_not_matching_declared_return_type() {
        let mut program = parse(Lexer::new("def f (a: u32): bool { a }").unwrap()).unwrap();
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
    fn type_error_for_number_literal_returned_from_bool_function() {
        let mut program = parse(Lexer::new("def f (): bool { 1 }").unwrap()).unwrap();
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
    fn type_error_for_nested_return_not_matching_declared_return_type() {
        let mut program =
            parse(Lexer::new("def f (a: i32): u32 { if true { return a; } return 1; }").unwrap())
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
    fn type_error_for_differing_return_types() {
        let mut program =
            parse(Lexer::new("def f (a: u32, b: i32) { return a; return b; }").unwrap()).unwrap();
        let result = check_types(&mut program);
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::DifferingReturnTypes { .. },
                    ..
                })
            ),
            "expected DifferingReturnTypes, got {result:?}"
        );
    }
}
