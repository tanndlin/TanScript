use std::fmt;

use crate::{
    ast::{Block, Expression, Statement},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    type_check::{TypeCheckError, TypeScope, check_types_expression},
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
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let id = register_handler.get_unique_id();
        let dst = register_handler.lease_register()?;
        let condition = self
            .condition
            .compile(compile_scope, dst, register_handler)?;

        let mut new_scope = CompileScope::new(Some(compile_scope));
        let block = self.block.compile(&mut new_scope, register_handler)?;
        let mut else_scope = CompileScope::new(Some(compile_scope));
        let else_block = match &self.else_block {
            None => None,
            Some(else_block) => Some(else_block.compile(&mut else_scope, register_handler)?),
        };

        register_handler.release_register(dst);
        Ok(format!(
            "; if {}\n\
            {condition}\n\
            test {dst}, {dst}\n\
            jz else{id}\n\
            ; then\n\
            {block}\n\
            jmp endif{id}\n\
            else{id}:\n\
            {}\n\
            endif{id}:",
            self.condition,
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

    fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&self.condition, type_scope)?;
        self.block.check_types(type_scope)?;
        if let Some(else_block) = &mut self.else_block {
            else_block.check_types(type_scope)?;
        }
        Ok(())
    }

    fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.block.resolve_types(type_scope);
        if let Some(else_block) = &mut self.else_block {
            else_block.resolve_types(type_scope);
        }
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
