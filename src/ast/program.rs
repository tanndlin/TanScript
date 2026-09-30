use std::fmt;

use crate::ast::Block;

#[derive(Debug)]
pub struct Program {
    pub block: Block,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.block)
    }
}
