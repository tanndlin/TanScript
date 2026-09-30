use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    compile_scope::CompileScope, register_handler::RegisterHandler, symbol_table::SymbolTable,
};

pub trait Statement: fmt::Display + fmt::Debug + StatementClone {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String>;

    fn discover(&self, symbols: &mut SymbolTable);

    // Block statements (while, if, def) are not terminated by a semicolon
    fn requires_semicolon(&self) -> bool {
        true
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
