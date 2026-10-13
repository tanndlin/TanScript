use std::fmt;

use crate::ast::{Block, Expression};

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub block: Block,
}

impl fmt::Display for WhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while {} {{\n{}\n}}", self.condition, self.block)
    }
}
