use std::fmt;

use crate::{ast::Expression, types::Span};

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Expression,
    pub end: Expression,
    pub inclusive: bool,
    pub span: Span,
}

impl Range {
    pub fn new(start: Expression, end: Expression, inclusive: bool, span: Span) -> Self {
        Self {
            start,
            end,
            inclusive,
            span,
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = if self.inclusive { "..=" } else { ".." };
        write!(f, "{}{op}{}", self.start, self.end)
    }
}
