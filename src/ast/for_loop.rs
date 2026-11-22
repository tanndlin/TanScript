use std::fmt;

use crate::{
    ast::{Block, Range},
    type_check::Type,
    types::Span,
};

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub ident: String,
    pub span: Span,
    pub dtype: Option<Type>,
    pub binding: Option<usize>,
    pub range: Range,
    pub block: Block,
}

impl fmt::Display for ForLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "for {} in {} {{\n{}\n}}",
            self.ident, self.range, self.block
        )
    }
}
