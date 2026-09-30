use std::fmt;

use crate::{
    ast::{Block, Expression, Statement, StatementOrExpression},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
};

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub init: StatementOrExpression,
    pub condition: Expression,
    pub update: StatementOrExpression,
    pub block: Block,
}

impl Statement for ForLoop {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("for_loop_{unique_id}");
        let end_label = format!("for_end_{unique_id}");

        let init = self.init.compile(compile_scope, register_handler)?;

        let condition_dst_reg = register_handler.lease_register()?;
        let condition =
            self.condition
                .compile(compile_scope, condition_dst_reg, register_handler)?;
        register_handler.release_register(condition_dst_reg);

        let mut new_scope = CompileScope::new(Some(compile_scope));
        let block = self.block.compile(&mut new_scope, register_handler)?;
        // Indent the update to match the block it runs with
        let update = self
            .update
            .compile(compile_scope, register_handler)?
            .lines()
            .map(|line| format!("\t{line}"))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(format!(
            "; for ({}; {}; {})\n\
            {init}\n\
            {start_label}:\n\
            {condition}\n\
            cmp {condition_dst_reg}, 0\n\
            je {end_label}\n\
            ; body\n\
            {block}\n\
            \t; update\n\
            {update}\n\
            jmp {start_label}\n\
            {end_label}:",
            self.init, self.condition, self.update
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

impl fmt::Display for ForLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "for ({};{};{}) {{\n{}\n}}",
            self.init, self.condition, self.update, self.block
        )
    }
}
