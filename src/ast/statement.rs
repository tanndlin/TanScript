use crate::ast::{
    Assignment, BreakStatement, ContinueStatement, Declaration, ForLoop, FunctionDefinition,
    IfStatement, ReturnStatement, WhileLoop,
};

#[derive(Debug, Clone, derive_more::Display)]
pub enum Statement {
    Assignment(Assignment),
    Declaration(Declaration),
    ForLoop(ForLoop),
    FunctionDefinition(FunctionDefinition),
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
                | Statement::FunctionDefinition(_)
                | Statement::IfStatement(_)
                | Statement::WhileLoop(_)
        )
    }
}
