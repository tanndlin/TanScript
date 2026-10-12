use std::{any::Any, fmt};

use crate::{
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    type_check::{TypeCheckError, TypeScope},
};

pub trait Statement: fmt::Display + fmt::Debug + StatementClone + AsAny {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String>;

    fn discover(&self, symbols: &mut SymbolTable);

    // Block statements (while, if, def) are not terminated by a semicolon
    fn requires_semicolon(&self) -> bool {
        true
    }

    fn check_types(&mut self, _type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        Ok(())
    }

    fn resolve_types(&mut self, _type_scope: &TypeScope) {}
}

impl dyn Statement {
    pub fn downcast_ref<T: Statement + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref()
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub trait StatementClone {
    fn clone_box(&self) -> Box<dyn Statement>;
}

impl<T: 'static + Statement + Clone> StatementClone for T {
    fn clone_box(&self) -> Box<dyn Statement> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Statement> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
