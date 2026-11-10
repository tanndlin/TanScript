use crate::ast::{Expression, Statement};

#[derive(Debug, Clone, derive_more::Display)]
pub enum StatementOrExpression {
    Statement(Box<Statement>),
    Expression(Expression),
}

impl From<Statement> for StatementOrExpression {
    fn from(stmt: Statement) -> Self {
        StatementOrExpression::Statement(Box::new(stmt))
    }
}

impl From<Expression> for StatementOrExpression {
    fn from(expr: Expression) -> Self {
        StatementOrExpression::Expression(expr)
    }
}
