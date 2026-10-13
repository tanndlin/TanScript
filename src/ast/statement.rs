use std::fmt;

use crate::ast::{
    Assignment, BreakStatement, ContinueStatement, Declaration, ForLoop, FunctionDefinition,
    IfStatement, ReturnStatement, WhileLoop,
};

#[derive(Debug, Clone)]
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

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Assignment(s) => write!(f, "{s}"),
            Statement::Declaration(s) => write!(f, "{s}"),
            Statement::ForLoop(s) => write!(f, "{s}"),
            Statement::FunctionDefinition(s) => write!(f, "{s}"),
            Statement::IfStatement(s) => write!(f, "{s}"),
            Statement::WhileLoop(s) => write!(f, "{s}"),
            Statement::Return(s) => write!(f, "{s}"),
            Statement::Break(s) => write!(f, "{s}"),
            Statement::Continue(s) => write!(f, "{s}"),
        }
    }
}
