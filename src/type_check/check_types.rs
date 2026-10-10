use crate::{
    ast::{Expression, Program, StatementOrExpression},
    type_check::TypeCheckError,
};

pub fn check_types(program: &Program) -> Result<(), TypeCheckError> {
    program.block.children.iter().try_for_each(|ast| match ast {
        StatementOrExpression::Statement(statement) => statement.check_types(),
        StatementOrExpression::Expression(expression) => check_types_expression(expression),
    })
}

fn check_types_expression(_expression: &Expression) -> Result<(), TypeCheckError> {
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
}
