use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{Expression, Statement},
    compile::{Address, Register},
    compile_scope::CompileScope,
    register_handler::RegisterHandler,
    symbol_table::SymbolTable,
};

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}

impl Statement for ReturnStatement {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let instructions = self.expr.compile(
            compile_scope,
            Address::Register(Register::RAX),
            register_handler,
        )?;
        Ok(format!("{instructions}\nmov rsp, rbp\npop rbp\nret"))
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.expr.discover(symbols);
    }
}

impl fmt::Display for ReturnStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "return {}", self.expr)
    }
}
