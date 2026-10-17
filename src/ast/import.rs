use std::fmt;

use crate::types::Span;

#[derive(Debug, Clone)]
pub struct Import {
    pub path: Vec<String>,
    pub names: Vec<ImportName>,
    pub span: Span,
}

impl Import {
    pub fn module_name(&self) -> String {
        self.path.join("::")
    }
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "use {}::{{{}}}",
            self.module_name(),
            self.names
                .iter()
                .map(|n| n.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone)]
pub struct ImportName {
    pub name: String,
    pub span: Span,
}
