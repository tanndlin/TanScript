use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{Block, Expression, Statement},
    compile::Address,
    compile_scope::CompileScope,
    register_handler::RegisterHandler,
    symbol_table::SymbolTable,
};

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub block: Block,
    pub else_block: Option<Block>,
}

impl Statement for IfStatement {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let id = register_handler.get_unique_id();
        let dst = register_handler.lease_register()?;
        let condition =
            self.condition
                .compile(compile_scope, Address::Register(dst), register_handler)?;

        let new_scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));
        let block = self.block.compile(&new_scope, register_handler)?;
        let else_scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));
        let else_block = match &self.else_block {
            None => None,
            Some(else_block) => Some(else_block.compile(&else_scope, register_handler)?),
        };

        register_handler.release_register(dst);
        Ok(format!(
            "{condition}\ntest {dst}, {dst}\njz else{id}\n{block}\njmp endif{id}\nelse{id}:\n{}\nendif{id}:",
            else_block.unwrap_or(String::new())
        ))
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.condition.discover(symbols);
        self.block.discover(symbols);
        if let Some(else_block) = &self.else_block {
            else_block.discover(symbols);
        }
    }

    fn requires_semicolon(&self) -> bool {
        false
    }
}

impl fmt::Display for IfStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(else_block) = &self.else_block {
            write!(
                f,
                "if {} {{\n{}\n}} else {{\n{}\n}}",
                self.condition, self.block, else_block
            )
        } else {
            write!(f, "if {} {{\n{}\n}}", self.condition, self.block)
        }
    }
}
