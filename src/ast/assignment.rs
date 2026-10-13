use std::fmt;

use crate::ast::Expression;

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Expression,
    pub expression: Expression,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.expression)
    }
}
