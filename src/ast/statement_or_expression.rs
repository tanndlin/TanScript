use std::fmt;

use crate::{
    ast::{Expression, Statement},
    type_check::{TypeCheckError, TypeScope, check_types_expression},
};

#[derive(Debug, Clone)]
pub enum StatementOrExpression {
    Statement(Box<dyn Statement>),
    Expression(Expression),
}

impl StatementOrExpression {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        match self {
            StatementOrExpression::Statement(statement) => statement.check_types(type_scope),
            StatementOrExpression::Expression(expression) => {
                check_types_expression(expression, type_scope).map(|_| ())
            }
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let StatementOrExpression::Statement(statement) = self {
            statement.resolve_types(type_scope);
        }
    }
}

impl fmt::Display for StatementOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementOrExpression::Expression(e) => write!(f, "{e}"),
            StatementOrExpression::Statement(s) => write!(f, "{s}"),
        }
    }
}
