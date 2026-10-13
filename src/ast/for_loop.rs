use std::fmt;

use crate::ast::{Block, Expression, StatementOrExpression};

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub init: StatementOrExpression,
    pub condition: Expression,
    pub update: StatementOrExpression,
    pub block: Block,
}

impl fmt::Display for ForLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "for ({};{};{}) {{\n{}\n}}",
            self.init, self.condition, self.update, self.block
        )
    }
}
