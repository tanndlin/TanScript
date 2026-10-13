use std::fmt;

use crate::types::Span;

#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub span: Span,
}

impl fmt::Display for BreakStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "break")
    }
}
