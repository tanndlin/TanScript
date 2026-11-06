use std::fmt;

use crate::ast::{AssignOp, Expression};

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Expression,
    pub op: AssignOp,
    pub expression: Expression,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            AssignOp::Assign => write!(f, "{} = {}", self.lhs, self.expression),
            AssignOp::Compound(op) => write!(f, "{} {op}= {}", self.lhs, self.expression),
        }
    }
}
