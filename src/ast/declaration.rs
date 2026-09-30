use std::fmt;

use crate::{
    ast::{Assignment, Statement},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
}

impl Statement for Declaration {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        compile_scope.add_variable(self.assign.identifier.clone(), None)?;
        self.assign.compile(compile_scope, register_handler)
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.assign.discover(symbols);
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
