use std::fmt;

use crate::{type_check::Type, types::Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

impl fmt::Display for StructDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "struct {} {{{}}}",
            self.name,
            self.fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub name: String,
    pub ftype: Type,
}

impl fmt::Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ftype)
    }
}
