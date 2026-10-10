use crate::{
    ast::{Expression, Program, StatementOrExpression},
    type_check::{TypeCheckError, TypeScope},
};

pub fn check_types(program: &Program) -> Result<(), TypeCheckError> {
    let mut type_scope = TypeScope::default();

    // TODO: Somewhere in here functions will need there  own type scope
    program.block.children.iter().try_for_each(|ast| match ast {
        StatementOrExpression::Statement(statement) => statement.check_types(&mut type_scope),
        StatementOrExpression::Expression(expression) => {
            check_types_expression(expression, &mut type_scope)
        }
    })
}

fn check_types_expression(
    _expression: &Expression,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    Ok(())
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
        assert!(matches!(
            check_types(&program),
            Err(TypeCheckError::TypeMismatch { .. })
        ));
    }
}
