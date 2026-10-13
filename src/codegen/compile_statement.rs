use crate::{
    ast::{
        Assignment, AtomType, BreakStatement, ContinueStatement, Declaration, ExpressionKind,
        ForLoop, FunctionDefinition, IfStatement, ReturnStatement, Statement, WhileLoop,
    },
    codegen::{
        CodegenError, CodegenErrorKind,
        address::{Address, Register},
        compile::store_element,
        compile_scope::CompileScope,
        loop_context::LoopContext,
        register_handler::RegisterHandler,
    },
    error::{At, ResultAt},
    type_check::Type,
    types::Span,
};

impl Statement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, CodegenError> {
        match self {
            Statement::Assignment(s) => s.compile(compile_scope, register_handler),
            Statement::Declaration(s) => s.compile(compile_scope, register_handler),
            Statement::ForLoop(s) => s.compile(compile_scope, register_handler),
            Statement::FunctionDefinition(_) => Ok(String::new()),
            Statement::IfStatement(s) => s.compile(compile_scope, register_handler),
            Statement::Return(s) => s.compile(compile_scope, register_handler),
            Statement::WhileLoop(s) => s.compile(compile_scope, register_handler),
            Statement::Break(s) => s.compile(compile_scope),
            Statement::Continue(s) => s.compile(compile_scope),
        }
    }
}

impl Assignment {
    /// Compiles without the leading debug comment, for callers that print their own
    pub fn compile_uncommented(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, CodegenError> {
        match &self.lhs.kind {
            ExpressionKind::Atom(AtomType::Identifier(ident)) => {
                let address = compile_scope.get_variable(ident).at(self.lhs.span)?;
                self.expression
                    .compile(compile_scope, address, register_handler)
            }
            ExpressionKind::Index(lhs, index) => {
                let element = compile_scope.element_type(lhs).at(lhs.span)?;
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
            _ => Err(CodegenErrorKind::InvalidAssignTarget(self.lhs.to_string()).at(self.lhs.span)),
        }
    }

    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, CodegenError> {
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
    ) -> Result<String, CodegenError> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            return Err(
                CodegenErrorKind::InvalidAssignTarget(self.assign.lhs.to_string())
                    .at(self.assign.lhs.span),
            );
        };

        let dtype = self.dtype.clone().unwrap_or(Type::Unknown);
        let alloc = compile_scope
            .allocate_variable(ident.clone(), dtype)
            .at(self.assign.lhs.span)?;
        let assign = self
            .assign
            .compile_uncommented(compile_scope, register_handler)?;
        Ok(format!("; {self}\n{alloc}\n{assign}"))
    }
}

impl ForLoop {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, CodegenError> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("for_loop_{unique_id}");
        let end_label = format!("for_end_{unique_id}");
        let continue_label = format!("for_continue_{unique_id}");

        let mut new_scope = CompileScope::new_loop(
            compile_scope,
            LoopContext {
                break_label: end_label.clone(),
                continue_label: continue_label.clone(),
            },
        );
        let init = self.init.compile(&mut new_scope, register_handler)?;

        let condition_dst_reg = register_handler.lease_register().at(self.condition.span)?;
        let condition =
            self.condition
                .compile(&mut new_scope, condition_dst_reg, register_handler)?;
        register_handler.release_register(condition_dst_reg);

        let block = self.block.compile(&mut new_scope, register_handler)?;
        let init_size = new_scope.stack_depth - compile_scope.stack_depth;
        let dealloc = if init_size > 0 {
            format!("\nadd rsp, {init_size}")
        } else {
            String::new()
        };
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
            {continue_label}:\n\
            \t; update\n\
            {update}\n\
            jmp {start_label}\n\
            {end_label}:{dealloc}",
            self.init, self.condition, self.update
        ))
    }
}

impl FunctionDefinition {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, CodegenError> {
        let mut scope = CompileScope::new(Some(compile_scope));

        let mut arg_setup = vec![];
        let arg_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
        for (i, arg) in self.args.iter().enumerate() {
            // Win64: after `push rbp`, the shadow space for the first 4 args starts at rbp+16,
            // and any further args were placed directly after it by the caller
            let offset = 16 + i32::try_from(i).expect("argument count fits in i32") * 8;
            let new_address = Address::Stack(-offset);

            // Move register args to their home in the shadow space
            if let Some(reg) = arg_registers.get(i) {
                arg_setup.push(format!("mov {new_address}, {reg}"));
            }

            // Update its location in the scope
            scope
                .add_variable(arg.name.clone(), new_address, arg.atype.clone())
                .at(arg.span)?;
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
    ) -> Result<String, CodegenError> {
        let id = register_handler.get_unique_id();
        let dst = register_handler.lease_register().at(self.condition.span)?;
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
    ) -> Result<String, CodegenError> {
        let instructions = register_handler.request_with_scope(
            Register::RAX,
            Register::RAX.into(),
            |register_handler, rax| self.expr.compile(compile_scope, rax, register_handler),
        )?;
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
    ) -> Result<String, CodegenError> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let reg = register_handler.lease_register().at(self.condition.span)?;
        let condition = self
            .condition
            .compile(compile_scope, reg, register_handler)?;
        register_handler.release_register(reg);

        let mut loop_scope = CompileScope::new_loop(
            compile_scope,
            LoopContext {
                break_label: end_label.clone(),
                continue_label: start_label.clone(),
            },
        );
        let block = self.block.compile(&mut loop_scope, register_handler)?;

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

fn jump_out_of_loop(
    compile_scope: &CompileScope,
    keyword: &'static str,
    span: Span,
    label: impl Fn(&LoopContext) -> &str,
) -> Result<String, CodegenError> {
    let loop_scope = compile_scope
        .loop_scope()
        .ok_or_else(|| CodegenErrorKind::OutsideLoop(keyword).at(span))?;
    let loop_context = loop_scope
        .loop_context()
        .expect("loop scope has a loop context");

    let mut instructions = vec![format!("; {keyword}")];
    let unwind = compile_scope.stack_depth - loop_scope.stack_depth;
    if unwind > 0 {
        instructions.push(format!("add rsp, {unwind}"));
    }
    instructions.push(format!("jmp {}", label(loop_context)));
    Ok(instructions.join("\n"))
}

impl BreakStatement {
    pub fn compile(&self, compile_scope: &CompileScope) -> Result<String, CodegenError> {
        jump_out_of_loop(compile_scope, "break", self.span, |l| &l.break_label)
    }
}

impl ContinueStatement {
    pub fn compile(&self, compile_scope: &CompileScope) -> Result<String, CodegenError> {
        jump_out_of_loop(compile_scope, "continue", self.span, |l| &l.continue_label)
    }
}
