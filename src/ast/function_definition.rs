use std::fmt;

use crate::{ast::Block, type_check::Type, types::Span};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Block,
    pub return_type: Type,
    pub span: Span,
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let return_type = match &self.return_type {
            Type::Unit => String::new(),
            t => format!(": {t}"),
        };
        write!(
            f,
            "def {}({}){} {{{}}}",
            self.name,
            self.args
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            return_type,
            self.body
        )
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub atype: Type,
    pub span: Span,
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.atype)
    }
}
