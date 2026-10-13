use std::fmt;

use crate::ast::{Block, Expression};

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub block: Block,
    pub else_block: Option<Block>,
}

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(else_block) = &self.else_block {
            write!(
                f,
                "if {} {{\n{}\n}} else {{\n{}\n}}",
                self.condition, self.block, else_block
            )
        } else {
            write!(f, "if {} {{\n{}\n}}", self.condition, self.block)
        }
    }
}
