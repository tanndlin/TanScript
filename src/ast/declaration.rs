use std::fmt;

use crate::{ast::Assignment, type_check::Type, types::Span};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
    pub dtype: Option<Type>,
    pub binding: Option<usize>,
    pub _span: Span,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
