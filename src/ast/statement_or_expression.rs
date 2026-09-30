use std::fmt;

use crate::ast::{Expression, Statement};

#[derive(Debug, Clone)]
pub enum StatementOrExpression {
    Statement(Box<dyn Statement>),
    Expression(Expression),
}

impl fmt::Display for StatementOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementOrExpression::Expression(e) => write!(f, "{e}"),
            StatementOrExpression::Statement(s) => write!(f, "{s}"),
        }
    }
}
