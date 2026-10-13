use std::fmt;

use crate::ast::Expression;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}

impl fmt::Display for ReturnStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expr)
    }
}
