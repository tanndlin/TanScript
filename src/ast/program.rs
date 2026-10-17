use std::fmt;

use crate::ast::{Block, Import};

#[derive(Debug)]
pub struct Program {
    pub imports: Vec<Import>,
    pub block: Block,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for import in &self.imports {
            writeln!(f, "{import}")?;
        }

        write!(f, "{}", self.block)
    }
}
