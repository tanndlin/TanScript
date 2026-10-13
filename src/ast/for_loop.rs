use std::fmt;

use crate::{
    ast::{Block, Expression, StatementOrExpression},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    type_check::{TypeCheckError, TypeScope, check_types_expression},
};

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub init: StatementOrExpression,
    pub condition: Expression,
    pub update: StatementOrExpression,
    pub block: Block,
}

impl ForLoop {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("for_loop_{unique_id}");
        let end_label = format!("for_end_{unique_id}");

        let mut new_scope = CompileScope::new(Some(compile_scope));
        let init = self.init.compile(&mut new_scope, register_handler)?;

        let condition_dst_reg = register_handler.lease_register()?;
        let condition =
            self.condition
                .compile(&mut new_scope, condition_dst_reg, register_handler)?;
        register_handler.release_register(condition_dst_reg);

        let block = self.block.compile(&mut new_scope, register_handler)?;
        // Indent the update to match the block it runs with
        let update = self
            .update
            .compile(&mut new_scope, register_handler)?
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

    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.condition.discover(symbols);
        self.block.discover(symbols);
    }

    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self.init.check_types(type_scope).and_then(|()| {
            check_types_expression(&self.condition, type_scope)?;
            self.update.check_types(type_scope)?;
            self.block.check_types(type_scope)
        });
        type_scope.pop();
        result
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.init.resolve_types(type_scope);
        self.block.resolve_types(type_scope);
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
