use std::fmt;

use crate::{
    ast::{Block, Expression},
    types::Span,
};

#[derive(Debug, Clone)]
pub struct Case {
    pub rhs: Expression,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct MatchStatement {
    pub lhs: Expression,
    pub cases: Vec<Case>,
    pub default: Option<Block>,
    pub span: Span,
}

impl fmt::Display for MatchStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "match {} {{", self.lhs)?;
        for case in &self.cases {
            writeln!(
                f,
                "{} => {{
{}
}}",
                case.rhs, case.block
            )?;
        }
        if let Some(default) = &self.default {
            writeln!(
                f,
                "_ => {{
{default}
}}"
            )?;
        }
        write!(f, "}}")
    }
}
