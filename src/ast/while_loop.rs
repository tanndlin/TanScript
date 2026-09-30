use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{Block, Expression, Statement},
    compile::Address,
    compile_scope::CompileScope,
    register_handler::RegisterHandler,
    symbol_table::SymbolTable,
};

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub block: Block,
}

impl Statement for WhileLoop {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let reg = register_handler.lease_register()?;
        let condition =
            self.condition
                .compile(compile_scope, Address::Register(reg), register_handler)?;
        register_handler.release_register(reg);

        let block = self.block.compile(compile_scope, register_handler)?;

        Ok(format!(
            "{start_label}:\n\
            {condition}\n\
            cmp {reg}, 0\n\
            je {end_label}\n\
            {block}\n\
            jmp {start_label}\n\
            {end_label}:"
        ))
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.condition.discover(symbols);
        self.block.discover(symbols);
    }

    fn requires_semicolon(&self) -> bool {
        false
    }
}

impl fmt::Display for WhileLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "while {} {{\n{}\n}}", self.condition, self.block)
    }
}
