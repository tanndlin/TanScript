use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{Expression, Statement},
    compile_scope::CompileScope,
    register_handler::RegisterHandler,
    symbol_table::SymbolTable,
};

#[derive(Debug, Clone)]
pub struct Assignment {
    pub identifier: String,
    pub expression: Expression,
}

impl Statement for Assignment {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let address = compile_scope.borrow().get_variable(&self.identifier)?;
        self.expression
            .compile(compile_scope, address, register_handler)
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.expression.discover(symbols);
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.identifier, self.expression)
    }
}
