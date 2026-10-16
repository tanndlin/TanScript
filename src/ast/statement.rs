use crate::ast::{
    Assignment, BreakStatement, ContinueStatement, Declaration, ForLoop, FunctionDefinition,
    IfStatement, ReturnStatement, StructDefinition, WhileLoop,
};
use derive_more::{Display, From};

#[derive(Debug, Clone, From, Display)]
pub enum Statement {
    Assignment(Assignment),
    Declaration(Declaration),
    ForLoop(ForLoop),
    FunctionDefinition(FunctionDefinition),
    StructDefinition(StructDefinition),
    IfStatement(IfStatement),
    Return(ReturnStatement),
    WhileLoop(WhileLoop),
    Break(BreakStatement),
    Continue(ContinueStatement),
}

impl Statement {
    // Block statements (while, if, def) are not terminated by a semicolon
    pub fn requires_semicolon(&self) -> bool {
        !matches!(
            self,
            Statement::ForLoop(_)
                | Statement::IfStatement(_)
                | Statement::WhileLoop(_)
                | Statement::FunctionDefinition(_)
                | Statement::StructDefinition(_)
        )
    }
}
