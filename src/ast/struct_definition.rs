use std::fmt;

use crate::{type_check::Type, types::Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

impl StructDefinition {
    pub fn get_byte_offset(&self, field_name: &str) -> usize {
        let (index, _) = self
            .fields
            .iter()
            .enumerate()
            .find(|(_, f)| f.name == field_name)
            .unwrap();

        self.fields[..index].iter().map(|f| f.ftype.size()).sum()
    }

    pub fn get_field_type(&self, field_name: &str) -> Option<&Type> {
        self.fields
            .iter()
            .find(|f| f.name == field_name)
            .map(|f| &f.ftype)
    }

    pub fn size(&self) -> usize {
        self.fields.iter().map(|f| f.ftype.size()).sum()
    }
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
    pub span: Span,
}

impl fmt::Display for StructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ftype)
    }
}
