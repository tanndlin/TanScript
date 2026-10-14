use crate::ast::{Expression, Statement};

#[derive(Debug, Clone, derive_more::Display)]
pub enum StatementOrExpression {
    Statement(Box<Statement>),
    Expression(Expression),
}
