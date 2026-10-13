use std::fmt;

use crate::types::Span;

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub span: Span,
}

impl fmt::Display for ContinueStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "continue")
    }
}
