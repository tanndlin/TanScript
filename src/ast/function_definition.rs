use std::fmt;

use crate::ast::{Block, Signature};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub signature: Signature,
    pub body: Block,
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "def {} {{{}}}", self.signature, self.body)
    }
}
