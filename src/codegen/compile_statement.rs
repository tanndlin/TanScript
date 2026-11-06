use crate::{
    ast::{
        AssignOp, Assignment, AtomType, BinaryOp, BreakStatement, ContinueStatement, Declaration,
        Expression, ExpressionKind, ForLoop, FunctionDefinition, IfStatement, MatchStatement,
        ReturnStatement, Statement, WhileLoop,
    },
    codegen::{
        CodegenError, CodegenErrorKind,
        address::{Address, Register},
        compile::{
            NON_VOLATILE_REGISTERS, as_operand, compile_jump_unless, load_element, mem_copy,
            store_element, with_place_address,
        },
        compile_scope::{CompileScope, SLOT_SIZE},
        function_context::FunctionContext,
        instruction::{Condition, Instruction},
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
    ) -> Result<Vec<Instruction>, CodegenError> {
        match self {
            Statement::Assignment(s) => s.compile(compile_scope, register_handler),
            Statement::Declaration(s) => s.compile(compile_scope, register_handler),
            Statement::ForLoop(s) => s.compile(compile_scope, register_handler),
            Statement::FunctionDefinition(_) => Ok(vec![]),
            Statement::StructDefinition(def) => {
                compile_scope.insert_struct(def.clone());
                Ok(vec![])
            }
            Statement::EnumDefinition(def) => {
                compile_scope.insert_enum(def.clone());
                Ok(vec![])
            }
            Statement::MatchStatement(s) => s.compile(compile_scope, register_handler),
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
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self.lhs.kind {
            ExpressionKind::Atom(AtomType::Identifier(ident)) => {
                let address = compile_scope.get_variable(ident).at(self.lhs.span)?;
                match self.op {
                    AssignOp::Assign => {
                        self.expression
                            .compile(compile_scope, address, register_handler)
                    }
                    AssignOp::Compound(op) => self.apply(op, self.lhs.clone()).compile(
                        compile_scope,
                        address,
                        register_handler,
                    ),
                }
            }
            _ if let AssignOp::Compound(op) = self.op => {
                self.compile_compound(op, compile_scope, register_handler)
            }
            _ => with_place_address(
                &self.lhs,
                compile_scope,
                register_handler,
                &mut |compile_scope, register_handler, address, t| {
                    if let Type::Struct(_) = t {
                        return self
                            .expression
                            .compile(compile_scope, address, register_handler);
                    }

                    register_handler.lease_with_scope(|register_handler, value_dst| {
                        let mut instructions =
                            self.expression
                                .compile(compile_scope, value_dst, register_handler)?;
                        instructions.push(store_element(&t, address, value_dst));
                        Ok(instructions)
                    })
                },
            ),
        }
    }

    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = vec![Instruction::comment(self.to_string())];
        instructions.extend(self.compile_uncommented(compile_scope, register_handler)?);
        Ok(instructions)
    }

    /// Evaluates the place once so `a[next()] += 1` only calls `next()` once
    fn compile_compound(
        &self,
        op: BinaryOp,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        const CURRENT: &str = "$compound";
        with_place_address(
            &self.lhs,
            compile_scope,
            register_handler,
            &mut |compile_scope, register_handler, address, t| {
                register_handler.lease_with_scope(|register_handler, value| {
                    let mut scope = CompileScope::new(Some(compile_scope));
                    scope
                        .add_variable(CURRENT.to_string(), value.into(), t.clone())
                        .at(self.lhs.span)?;

                    let mut current = Expression::new(
                        ExpressionKind::Atom(AtomType::Identifier(CURRENT.to_string())),
                        self.lhs.span,
                    );
                    current.etype.clone_from(&self.lhs.etype);
                    let expression = self.apply(op, current);

                    let mut instructions = vec![load_element(&t, value, address)];
                    instructions.extend(expression.compile(&mut scope, value, register_handler)?);
                    instructions.push(store_element(&t, address, value));
                    Ok(instructions)
                })
            },
        )
    }

    fn apply(&self, op: BinaryOp, current: Expression) -> Expression {
        Expression::new(
            ExpressionKind::Binary(op, Box::new(current), Box::new(self.expression.clone())),
            self.lhs.span,
        )
    }
}

impl Declaration {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
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
        let mut instructions = vec![Instruction::comment(self.to_string()), alloc];
        instructions.extend(
            self.assign
                .compile_uncommented(compile_scope, register_handler)?,
        );
        Ok(instructions)
    }
}

impl ForLoop {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
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

        let condition = compile_jump_unless(
            &self.condition,
            &end_label,
            &mut new_scope,
            register_handler,
        )?;

        let block = self.block.compile(&mut new_scope, register_handler)?;
        let init_size = new_scope.stack_depth - compile_scope.stack_depth;
        let update = self.update.compile(&mut new_scope, register_handler)?;

        let mut instructions = vec![Instruction::comment(format!(
            "for ({}; {}; {})",
            self.init, self.condition, self.update
        ))];
        instructions.extend(init);
        instructions.push(Instruction::label(&start_label));
        instructions.extend(condition);
        instructions.push(Instruction::comment("body"));
        instructions.extend(block);
        instructions.push(Instruction::label(continue_label));
        instructions.push(Instruction::comment("update"));
        instructions.extend(update);
        instructions.push(Instruction::jmp(start_label));
        instructions.push(Instruction::label(end_label));
        if init_size > 0 {
            instructions.push(Instruction::add(Register::RSP, init_size));
        }
        Ok(instructions)
    }
}

impl FunctionDefinition {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let epilogue_label = format!("{}_epilogue", self.name);
        let mut scope = CompileScope::new_function(
            compile_scope,
            FunctionContext {
                epilogue_label: epilogue_label.clone(),
            },
        );

        let save_size = SLOT_SIZE * u32::try_from(NON_VOLATILE_REGISTERS.len()).unwrap();
        scope.stack_depth = save_size;

        let arg_registers: [Register; 4] =
            [Register::RCX, Register::RDX, Register::R8, Register::R9];
        let outer_touched = std::mem::take(&mut register_handler.touched);
        let mut arg_homes = vec![];
        let mut struct_copies = vec![];
        for (i, arg) in self.args.iter().enumerate() {
            // Win64: after `push rbp`, the shadow space for the first 4 args starts at rbp+16,
            // and any further args were placed directly after it by the caller
            let offset = 16 + i32::try_from(i).expect("argument count fits in i32") * 8;
            let new_address = Address::Stack(-offset);

            // Move register args to their home in the shadow space
            if let Some(reg) = arg_registers.get(i) {
                arg_homes.push(Instruction::mov(new_address, *reg));
            }

            let Type::Struct(struct_def) = &arg.atype else {
                // Update its location in the scope
                scope
                    .add_variable(arg.name.clone(), new_address, arg.atype.clone())
                    .at(arg.span)?;
                continue;
            };

            struct_copies.push(
                scope
                    .allocate_variable(arg.name.clone(), arg.atype.clone())
                    .at(arg.span)?,
            );
            let local = scope.get_variable(&arg.name).at(arg.span)?;
            struct_copies.extend(register_handler.lease_with_scope(|register_handler, ptr| {
                let mut copy = vec![Instruction::mov(ptr, new_address)];
                copy.extend(mem_copy(
                    struct_def,
                    local,
                    Address::Deref {
                        base: ptr,
                        offset: 0,
                    },
                    register_handler,
                )?);
                Ok::<_, CodegenError>(copy)
            })?);
        }

        let body = self.body.compile(&mut scope, register_handler);
        let touched = std::mem::replace(&mut register_handler.touched, outer_touched);
        let body = body?; // ? after the replace to preserve incase we implement multiple errors

        let save_slots = NON_VOLATILE_REGISTERS
            .into_iter()
            .zip((1..).map(|i| Address::Stack(i * i32::try_from(SLOT_SIZE).unwrap())))
            .filter(|(reg, _)| touched.contains(reg))
            .collect::<Vec<_>>();

        let mut instructions = vec![
            Instruction::label(&self.name),
            Instruction::push(Register::RBP),
            Instruction::mov(Register::RBP, Register::RSP),
            Instruction::sub(Register::RSP, save_size),
        ];
        instructions.extend(
            save_slots
                .iter()
                .map(|(reg, slot)| Instruction::mov(*slot, *reg)),
        );
        instructions.extend(arg_homes);
        instructions.extend(struct_copies);
        instructions.extend(body);
        instructions.push(Instruction::label(epilogue_label));
        instructions.extend(
            save_slots
                .iter()
                .map(|(reg, slot)| Instruction::mov(*reg, *slot)),
        );
        instructions.push(Instruction::mov(Register::RSP, Register::RBP));
        instructions.push(Instruction::pop(Register::RBP));
        instructions.push(Instruction::Ret);

        Ok(instructions)
    }
}

impl IfStatement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let id = register_handler.get_unique_id();
        let else_label = format!("else{id}");
        let end_label = format!("endif{id}");

        let false_target = match self.else_block {
            Some(_) => &else_label,
            None => &end_label,
        };

        let mut instructions = vec![Instruction::comment(format!("if {}", self.condition))];
        instructions.extend(compile_jump_unless(
            &self.condition,
            false_target,
            compile_scope,
            register_handler,
        )?);

        instructions.push(Instruction::comment("then"));
        instructions.extend(self.block.compile(
            &mut CompileScope::new(Some(compile_scope)),
            register_handler,
        )?);

        if let Some(else_block) = &self.else_block {
            instructions.push(Instruction::jmp(end_label.clone()));
            instructions.push(Instruction::label(else_label));
            instructions.extend(else_block.compile(
                &mut CompileScope::new(Some(compile_scope)),
                register_handler,
            )?);
        }

        instructions.push(Instruction::label(end_label));
        Ok(instructions)
    }
}

impl MatchStatement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let end_label = format!("endmatch{}", register_handler.get_unique_id());
        let case_labels: Vec<String> = self
            .cases
            .iter()
            .map(|_| format!("match_case{}", register_handler.get_unique_id()))
            .collect();

        // TODO: This is a valid program getting rejected
        // Use lease register because it returns OutOfRegisters
        // RegisterHander::lease_with_scope would attemp push r {instructions} pop r
        // but {instructions} would jump past the pop
        let lhs_reg = register_handler
            .lease_register()
            .map_err(|e| e.at(self.span))?;
        let chain =
            self.compile_compare_chain(compile_scope, register_handler, lhs_reg, &case_labels);
        register_handler.release_register(lhs_reg);

        let mut instructions = vec![Instruction::comment(format!("match {}", self.lhs))];
        instructions.extend(chain?);

        if let Some(default) = &self.default {
            instructions.push(Instruction::comment("Case: _".to_string()));
            instructions.extend(default.compile(
                &mut CompileScope::new(Some(compile_scope)),
                register_handler,
            )?);
        }
        if !self.cases.is_empty() {
            instructions.push(Instruction::jmp(end_label.clone()));
        }

        for (i, (case, label)) in self.cases.iter().zip(&case_labels).enumerate() {
            instructions.push(Instruction::label(label.clone()));
            instructions.push(Instruction::comment(format!("Case: {}", case.rhs)));
            instructions.extend(case.block.compile(
                &mut CompileScope::new(Some(compile_scope)),
                register_handler,
            )?);
            if i + 1 < self.cases.len() {
                instructions.push(Instruction::jmp(end_label.clone()));
            }
        }

        instructions.push(Instruction::label(end_label));
        Ok(instructions)
    }

    // For match statement
    fn compile_compare_chain(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
        lhs_reg: Register,
        case_labels: &[String],
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = self.lhs.compile(compile_scope, lhs_reg, register_handler)?;

        for (case, label) in self.cases.iter().zip(case_labels) {
            match as_operand(&case.rhs, compile_scope) {
                Some(rhs) => instructions.push(Instruction::cmp(lhs_reg, rhs)),
                None => instructions.extend(register_handler.lease_with_scope(
                    |register_handler, rhs_reg| {
                        let mut instructions =
                            case.rhs.compile(compile_scope, rhs_reg, register_handler)?;
                        instructions.push(Instruction::cmp(lhs_reg, rhs_reg));
                        Ok(instructions)
                    },
                )?),
            }
            instructions.push(Instruction::jcc(Condition::Equal, label.clone()));
        }

        Ok(instructions)
    }
}

impl ReturnStatement {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = vec![Instruction::comment(self.to_string())];
        if let Some(expr) = &self.expr {
            instructions.extend(register_handler.request_with_scope(
                Register::RAX,
                Register::RAX.into(),
                |register_handler, rax| expr.compile(compile_scope, rax, register_handler),
            )?);
        }

        let epilogue_label = &compile_scope
            .function_context()
            .ok_or_else(|| CodegenErrorKind::OutsideFunction.at(self.span))?
            .epilogue_label;
        instructions.push(Instruction::jmp(epilogue_label));
        Ok(instructions)
    }
}

impl WhileLoop {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let condition =
            compile_jump_unless(&self.condition, &end_label, compile_scope, register_handler)?;

        let mut loop_scope = CompileScope::new_loop(
            compile_scope,
            LoopContext {
                break_label: end_label.clone(),
                continue_label: start_label.clone(),
            },
        );
        let block = self.block.compile(&mut loop_scope, register_handler)?;

        let mut instructions = vec![
            Instruction::comment(format!("while {}", self.condition)),
            Instruction::label(&start_label),
        ];
        instructions.extend(condition);
        instructions.push(Instruction::comment("body"));
        instructions.extend(block);
        instructions.push(Instruction::jmp(start_label));
        instructions.push(Instruction::label(end_label));
        Ok(instructions)
    }
}

fn jump_out_of_loop(
    compile_scope: &CompileScope,
    keyword: &'static str,
    span: Span,
    label: impl Fn(&LoopContext) -> &str,
) -> Result<Vec<Instruction>, CodegenError> {
    let loop_scope = compile_scope
        .loop_scope()
        .ok_or_else(|| CodegenErrorKind::OutsideLoop(keyword).at(span))?;
    let loop_context = loop_scope
        .loop_context()
        .expect("loop scope has a loop context");

    let mut instructions = vec![Instruction::comment(keyword)];
    let unwind = compile_scope.stack_depth - loop_scope.stack_depth;
    if unwind > 0 {
        instructions.push(Instruction::add(Register::RSP, unwind));
    }
    instructions.push(Instruction::jmp(label(loop_context)));
    Ok(instructions)
}

impl BreakStatement {
    pub fn compile(&self, compile_scope: &CompileScope) -> Result<Vec<Instruction>, CodegenError> {
        jump_out_of_loop(compile_scope, "break", self.span, |l| &l.break_label)
    }
}

impl ContinueStatement {
    pub fn compile(&self, compile_scope: &CompileScope) -> Result<Vec<Instruction>, CodegenError> {
        jump_out_of_loop(compile_scope, "continue", self.span, |l| &l.continue_label)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Statement, StatementOrExpression},
        codegen::{
            CodegenError, CodegenErrorKind, compile_scope::CompileScope,
            register_handler::RegisterHandler,
        },
        lex::lex,
        parse::parse,
        type_check::check_types,
    };

    #[test]
    fn match_errors_instead_of_borrowing_when_out_of_registers() {
        let mut program = parse(lex("match 1 { 1 => 2; _ => 3; };", 0).unwrap()).unwrap();
        check_types(&mut program).unwrap();
        let StatementOrExpression::Statement(statement) = &program.block.children[0] else {
            panic!("expected a statement");
        };
        let Statement::MatchStatement(m) = statement.as_ref() else {
            panic!("expected a match statement");
        };

        let mut register_handler = RegisterHandler::new();
        while register_handler.lease_register().is_ok() {}

        let result = m.compile(&mut CompileScope::new(None), &mut register_handler);
        assert!(
            matches!(
                result,
                Err(CodegenError {
                    kind: CodegenErrorKind::OutOfRegisters,
                    ..
                })
            ),
            "{result:?}"
        );
    }
}
