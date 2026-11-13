use std::fmt;

use crate::ast::Signature;

#[derive(Debug, Clone)]
pub struct ExternDeclaration {
    pub signature: Signature,
}

impl fmt::Display for ExternDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "extern def {}", self.signature)
    }
}
