use std::fmt;

use crate::types::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<String>,
    pub span: Span,
}

impl EnumDefinition {
    pub fn variant_index(&self, variant_name: &str) -> Option<usize> {
        self.variants.iter().position(|v| v == variant_name)
    }
}

impl fmt::Display for EnumDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enum {} {{{}}}", self.name, self.variants.join(", "))
    }
}
