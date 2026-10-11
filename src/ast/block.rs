use std::fmt;

use crate::{
    ast::StatementOrExpression,
    type_check::{TypeCheckError, TypeScope, check_types_expression},
};

#[derive(Debug, Clone)]
pub struct Block {
    pub children: Vec<StatementOrExpression>,
}
impl Block {
    pub fn check_types(&self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        self.children.iter().try_for_each(|ast| match ast {
            StatementOrExpression::Statement(statement) => statement.check_types(type_scope),
            StatementOrExpression::Expression(expression) => {
                check_types_expression(expression, type_scope)
            }
        })
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for child in &self.children {
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }

            write!(f, "{child}")?;
        }

        Ok(())
    }
}
