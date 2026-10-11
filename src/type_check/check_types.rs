use crate::{
    ast::{Expression, ExpressionKind, Program},
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
        ExpressionKind::Binary(binary_op, expression, expression1) => todo!(),
        ExpressionKind::Unary(unary_op, expression) => todo!(),
        ExpressionKind::Postfix(postfix_op, expression) => todo!(),
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
                        if infered_type == expected
                            || (expected.is_number_type() && infered_type == Type::CoercibleNumber)
                        {
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
        ExpressionKind::Index(expression, expression1) => todo!(),
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
}
