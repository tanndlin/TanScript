use crate::{
    ast::{
        Assignment, AtomType, Declaration, ExpressionKind, ForLoop, FunctionDefinition,
        IfStatement, ReturnStatement, Statement, WhileLoop,
    },
    codegen::{
        address::{Address, Register},
        compile::store_element,
        compile_scope::CompileScope,
        register_handler::RegisterHandler,
    },
    type_check::Type,
};

impl Statement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match self {
            Statement::Assignment(s) => s.compile(compile_scope, register_handler),
            Statement::Declaration(s) => s.compile(compile_scope, register_handler),
            Statement::ForLoop(s) => s.compile(compile_scope, register_handler),
            Statement::FunctionDefinition(_) => Ok(String::new()),
            Statement::IfStatement(s) => s.compile(compile_scope, register_handler),
            Statement::Return(s) => s.compile(compile_scope, register_handler),
            Statement::WhileLoop(s) => s.compile(compile_scope, register_handler),
        }
    }
}

impl Assignment {
    /// Compiles without the leading debug comment, for callers that print their own
    pub fn compile_uncommented(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match &self.lhs.kind {
            ExpressionKind::Atom(AtomType::Identifier(ident)) => {
                let address = compile_scope.get_variable(ident)?;
                self.expression
                    .compile(compile_scope, address, register_handler)
            }
            ExpressionKind::Index(lhs, index) => {
                let element = compile_scope.element_type(lhs)?;
                register_handler.lease_with_scope(|register_handler, offset_ptr| {
                    let data_size = element.size();
                    let index_asm = index.compile(compile_scope, offset_ptr, register_handler)?;
                    register_handler.lease_with_scope(|register_handler, ptr| {
                        let lhs = lhs.compile(compile_scope, ptr, register_handler)?;

                        register_handler.lease_with_scope(|register_handler, value_dst| {
                            let rhs = self.expression.compile(
                                compile_scope,
                                value_dst,
                                register_handler,
                            )?;
                            Ok([
                                index_asm,
                                format!("imul {offset_ptr}, {offset_ptr}, {data_size}"),
                                lhs,
                                rhs,
                                store_element(
                                    &element,
                                    &format!("[{ptr}+{offset_ptr}]"),
                                    value_dst,
                                ),
                            ]
                            .join("\n"))
                        })
                    })
                })
            }
            _ => Err("Invalid LHS for assign".to_string()),
        }
    }

    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let asm = self.compile_uncommented(compile_scope, register_handler)?;
        Ok(format!(
            "; {self}
{asm}"
        ))
    }
}

impl Declaration {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            return Err("Declaration LHS can only be a identifier".to_string());
        };

        let dtype = self.dtype.clone().unwrap_or(Type::Unknown);
        compile_scope.add_variable(ident.clone(), None, dtype)?;
        let assign = self
            .assign
            .compile_uncommented(compile_scope, register_handler)?;
        Ok(format!("; {self}\n{assign}"))
    }
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
}

impl FunctionDefinition {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let mut scope = CompileScope::new(Some(compile_scope));

        let mut arg_setup = vec![];
        let arg_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
        for (i, arg) in self.args.iter().enumerate() {
            // Win64: after `push rbp`, the shadow space for the first 4 args starts at rbp+16,
            // and any further args were placed directly after it by the caller
            let offset = 16 + i32::try_from(i).map_err(|_| "Index out of range")? * 8;
            let new_address = Address::Stack(-offset);

            // Move register args to their home in the shadow space
            if let Some(reg) = arg_registers.get(i) {
                arg_setup.push(format!("mov {new_address}, {reg}"));
            }

            // Update its location in the scope
            scope.add_variable(arg.name.clone(), Some(new_address), arg.atype.clone())?;
            // This is not a variable that needs to be deallocated later
            scope.num_variables -= 1;
        }
        let arg_setup = arg_setup.join("\n\t");
        let instructions = self.body.compile(&mut scope, register_handler)?;
        let ret = if instructions.ends_with("ret") {
            ""
        } else {
            "pop rbp\n\tret"
        };

        let preamble = format!("{}:\n\tpush rbp\n\tmov rbp, rsp", self.name);
        Ok(format!(
            "{preamble}\n\t{arg_setup}\n{instructions}\n\t{ret}",
        ))
    }
}

impl IfStatement {
    pub fn compile(
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
}

impl ReturnStatement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let instructions = self
            .expr
            .compile(compile_scope, Register::RAX, register_handler)?;
        Ok(format!(
            "; {self}\n{instructions}\nmov rsp, rbp\npop rbp\nret"
        ))
    }
}

impl WhileLoop {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let reg = register_handler.lease_register()?;
        let condition = self
            .condition
            .compile(compile_scope, reg, register_handler)?;
        register_handler.release_register(reg);

        let block = self.block.compile(compile_scope, register_handler)?;

        Ok(format!(
            "; while {}\n\
            {start_label}:\n\
            {condition}\n\
            cmp {reg}, 0\n\
            je {end_label}\n\
            ; body\n\
            {block}\n\
            jmp {start_label}\n\
            {end_label}:",
            self.condition
        ))
    }
}
