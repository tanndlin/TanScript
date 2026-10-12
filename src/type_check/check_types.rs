use crate::{
    ast::{AtomType, Expression, ExpressionKind, Program},
    type_check::{Type, TypeCheckError, TypeScope, infer_type_from_expression},
};

pub fn check_types(program: &Program) -> Result<(), TypeCheckError> {
    let mut type_scope = TypeScope::default();

    program.block.check_types(&mut type_scope)
}

pub fn check_types_expression(
    expression: &Expression,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    match &expression.kind {
        ExpressionKind::Atom(_) => Ok(()), // Don't need to check types on inert atom
        ExpressionKind::Binary(_, _, _) => todo!(),
        ExpressionKind::Unary(_, _) => todo!(),
        ExpressionKind::Postfix(_, _) => todo!(),
        ExpressionKind::FunctionCall(function_name, given_args) => {
            given_args
                .iter()
                .map(|a| check_types_expression(a, type_scope))
                .collect::<Result<Vec<_>, _>>()?;

            match type_scope.lookup(function_name) {
                Ok(Type::Function(function_type)) => given_args
                    .iter()
                    .zip(function_type.args.clone())
                    .try_for_each(|(given, expected)| {
                        let infered_type = infer_type_from_expression(given, type_scope)?;
                        if infered_type == expected {
                            Ok(())
                        } else if expected.is_number_type() && infered_type == Type::CoercibleNumber
                        {
                            narrow_expression(given, expected, type_scope);
                            Ok(())
                        } else {
                            Err(TypeCheckError::TypeMismatch {
                                expected,
                                got: infered_type,
                                span: expression.span,
                            })
                        }
                    }),
                Ok(_) => Err(TypeCheckError::NotCallableIdentifer(function_name.clone())),
                Err(_) => {
                    // Treat the function as an unknown extern
                    Ok(())
                }
            }
        }
        ExpressionKind::Index(lhs, rhs) => {
            check_types_expression(lhs, type_scope)?;
            check_types_expression(rhs, type_scope)?;

            // LHS must be a pointer
            let lhs_type = infer_type_from_expression(lhs, type_scope)?;
            if !matches!(lhs_type, Type::Pointer(_) | Type::Unknown) {
                return Err(TypeCheckError::TypeMismatch {
                    expected: Type::Pointer(Box::new(Type::Unknown)),
                    got: lhs_type,
                    span: expression.span,
                });
            }

            // RHS must come down to a number
            let rhs_type = infer_type_from_expression(rhs, type_scope)?;
            if !rhs_type.is_number_type() {
                return Err(TypeCheckError::TypeMismatch {
                    expected: Type::I64,
                    got: rhs_type,
                    span: expression.span,
                });
            }

            Ok(())
        }
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
        parser::parse,
        type_check::{TypeCheckError, check_types},
    };

    #[test]
    fn supports_coercing_literal_assignments() -> Result<(), TypeCheckError> {
        let program = parse("let a: u32 = 1;").unwrap();
        check_types(&program)
    }

    #[test]
    fn type_error_for_referenced_variable_in_assignment() {
        let program = parse("let a: u32 = 1; let b: i32 = a;").unwrap();
        let result = check_types(&program);
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_incorrect_function_arg() {
        let program =
            parse("def add (a: u32, b: u32) {a + b} let a: i32 = 1; let b: u32 = 2; add(a,b);")
                .unwrap();
        let result = check_types(&program);
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_coerced_variable_used_as_arg_then_assigned_to_different_type() {
        let program = parse(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; add(a, b); let c: i32 = a;",
        )
        .unwrap();
        let result = check_types(&program);
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_coerced_variable_assigned_then_used_as_arg_of_different_type() {
        let program = parse(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; let c: i32 = a; add(a, b);",
        )
        .unwrap();
        let result = check_types(&program);
        assert!(
            matches!(result, Err(TypeCheckError::TypeMismatch { .. })),
            "expected TypeMismatch, got {result:?}"
        );
    }
}
