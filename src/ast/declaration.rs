use std::fmt;

use crate::{ast::Expression, type_check::Type, types::Span};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub ident: String,
    pub span: Span,
    pub annotation: Option<Type>,
    pub ident_type: Option<Type>,
    pub binding: Option<usize>,
    pub value: Expression,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {} = {}", self.ident, self.value)
    }
}
