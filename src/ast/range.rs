use crate::{ast::Expression, types::Span};

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
