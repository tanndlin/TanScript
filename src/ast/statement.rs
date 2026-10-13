use std::fmt;

use crate::{
    ast::{
        Assignment, Declaration, ForLoop, FunctionDefinition, IfStatement, ReturnStatement,
        WhileLoop,
    },
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    type_check::{TypeCheckError, TypeScope},
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
}

impl Statement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match self {
            Statement::Assignment(s) => s.compile(compile_scope, register_handler),
            Statement::Declaration(s) => s.compile(compile_scope, register_handler),
            Statement::ForLoop(s) => s.compile(compile_scope, register_handler),
            Statement::FunctionDefinition(_) => Ok(String::new()),
            Statement::IfStatement(s) => s.compile(compile_scope, register_handler),
            Statement::Return(s) => s.compile(compile_scope, register_handler),
            Statement::WhileLoop(s) => s.compile(compile_scope, register_handler),
        }
    }

    pub fn discover(&self, symbols: &mut SymbolTable) {
        match self {
            Statement::Assignment(s) => s.discover(symbols),
            Statement::Declaration(s) => s.discover(symbols),
            Statement::ForLoop(s) => s.discover(symbols),
            Statement::FunctionDefinition(s) => s.discover(symbols),
            Statement::IfStatement(s) => s.discover(symbols),
            Statement::Return(s) => s.discover(symbols),
            Statement::WhileLoop(s) => s.discover(symbols),
        }
    }

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

    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        match self {
            Statement::Assignment(s) => s.check_types(type_scope),
            Statement::Declaration(s) => s.check_types(type_scope),
            Statement::ForLoop(s) => s.check_types(type_scope),
            Statement::FunctionDefinition(s) => s.check_types(type_scope),
            Statement::IfStatement(s) => s.check_types(type_scope),
            Statement::Return(s) => s.check_types(type_scope),
            Statement::WhileLoop(s) => s.check_types(type_scope),
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        match self {
            Statement::Declaration(s) => s.resolve_types(type_scope),
            Statement::ForLoop(s) => s.resolve_types(type_scope),
            Statement::FunctionDefinition(s) => s.resolve_types(type_scope),
            Statement::IfStatement(s) => s.resolve_types(type_scope),
            Statement::WhileLoop(s) => s.resolve_types(type_scope),
            Statement::Assignment(_) | Statement::Return(_) => {}
        }
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
            Statement::Return(s) => write!(f, "{s}"),
            Statement::WhileLoop(s) => write!(f, "{s}"),
        }
    }
}
