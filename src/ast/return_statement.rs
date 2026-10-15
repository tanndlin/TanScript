use std::fmt;

use crate::{ast::Expression, types::Span};

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Option<Expression>,
    pub span: Span,
}

impl fmt::Display for ReturnStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.expr {
            Some(expr) => write!(f, "return {expr}"),
            None => write!(f, "return"),
        }
    }
}
