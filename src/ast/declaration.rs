use std::fmt;

use crate::{
    ast::{Assignment, AtomType, Expression, Statement},
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
        let Expression::Atom(AtomType::Identifier(ident)) = &self.assign.lhs else {
            return Err("Declaration LHS can only be a identifier".to_string());
        };

        compile_scope.add_variable(ident.clone(), None)?;
        let assign = self
            .assign
            .compile_uncommented(compile_scope, register_handler)?;
        Ok(format!("; {self}\n{assign}"))
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
