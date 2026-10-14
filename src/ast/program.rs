use crate::ast::Block;

#[derive(Debug, derive_more::Display)]
pub struct Program {
    pub block: Block,
}
