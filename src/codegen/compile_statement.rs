use crate::{
    ast::{
        Arg, AssignOp, Assignment, BinaryOp, BreakStatement, ContinueStatement, Declaration,
        Expression, ExpressionKind, ForLoop, FunctionDefinition, IfStatement, MatchStatement,
        ReturnStatement, Statement, Walk, WhileLoop,
    },
    codegen::{
        Codegen, CodegenError, CodegenErrorKind,
        address::{Address, Register},
        compile::as_operand,
        compile_call::{NON_VOLATILE_REGISTERS, passed_by_pointer},
        compile_memory::{load_element, mem_copy, store_element, with_place_address},
        compile_operators::{compile_binary, compile_jump_unless},
        compile_scope::SLOT_SIZE,
        function_context::FunctionContext,
        instruction::{Condition, Instruction, Operand},
        loop_context::LoopContext,
    },
    error::{At, ResultAt},
    type_check::Type,
    types::Span,
};

impl Statement {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        match self {
            Statement::Assignment(s) => s.compile(codegen),
            Statement::Declaration(s) => s.compile(codegen),
            Statement::ForLoop(s) => s.compile(codegen),
            Statement::ExternDeclaration(_)
            | Statement::FunctionDefinition(_)
            | Statement::StructDefinition(_)
            | Statement::EnumDefinition(_) => Ok(vec![]),
            Statement::MatchStatement(s) => s.compile(codegen),
            Statement::IfStatement(s) => s.compile(codegen),
            Statement::Return(s) => s.compile(codegen),
            Statement::WhileLoop(s) => s.compile(codegen),
            Statement::Break(s) => s.compile(codegen),
            Statement::Continue(s) => s.compile(codegen),
        }
    }
}

impl Assignment {
    /// Compiles without the leading debug comment, for callers that print their own
    pub fn compile_uncommented(
        &self,
        codegen: &mut Codegen,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self.lhs.kind {
            ExpressionKind::Variable { name, .. } => {
                let address = codegen.get_variable(name).at(self.lhs.span)?;
                match self.op {
                    AssignOp::Assign => self.expression.compile(codegen, address),
                    AssignOp::Compound(op) => compile_binary(
                        op,
                        &self.lhs,
                        &self.expression,
                        &self.lhs.etype,
                        codegen,
                        address,
                    ),
                }
            }
            _ if let AssignOp::Compound(op) = self.op => self.compile_compound(op, codegen),
            _ => with_place_address(&self.lhs, codegen, &mut |codegen, address, t| {
                if let Type::Struct(_) = t {
                    return self.expression.compile(codegen, address);
                }

                codegen.lease_with_scope(|codegen, value_dst| {
                    let mut instructions = self.expression.compile(codegen, value_dst)?;
                    instructions.push(store_element(t, address, value_dst));
                    Ok(instructions)
                })
            }),
        }
    }

    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = vec![Instruction::comment(self.to_string())];
        instructions.extend(self.compile_uncommented(codegen)?);
        Ok(instructions)
    }

    /// Evaluates the place once so `a[next()] += 1` only calls `next()` once
    fn compile_compound(
        &self,
        op: BinaryOp,
        codegen: &mut Codegen,
    ) -> Result<Vec<Instruction>, CodegenError> {
        const CURRENT: &str = "$compound";
        with_place_address(&self.lhs, codegen, &mut |codegen, address, t| {
            codegen.lease_with_scope(|codegen, value| {
                codegen.with_block(|codegen| {
                    codegen
                        .scope_mut()
                        .add_variable(CURRENT.to_string(), value.into())
                        .at(self.lhs.span)?;

                    let mut current = Expression::new(
                        ExpressionKind::Variable {
                            name: CURRENT.to_string(),
                            binding: None,
                        },
                        self.lhs.span,
                    );
                    current.etype.clone_from(&self.lhs.etype);

                    let mut instructions = vec![load_element(t, value, address)];
                    instructions.extend(compile_binary(
                        op,
                        &current,
                        &self.expression,
                        t,
                        codegen,
                        value.into(),
                    )?);
                    instructions.push(store_element(t, address, value));
                    Ok(instructions)
                })
            })
        })
    }
}

impl Declaration {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let ExpressionKind::Variable { name, .. } = &self.assign.lhs.kind else {
            return Err(
                CodegenErrorKind::InvalidAssignTarget(self.assign.lhs.to_string())
                    .at(self.assign.lhs.span),
            );
        };

        let dtype = Type::clone(&self.assign.lhs.etype);
        let alloc = codegen
            .scope_mut()
            .allocate_variable(name.clone(), &dtype)
            .at(self.assign.lhs.span)?;
        let mut instructions = vec![Instruction::comment(self.to_string()), alloc];
        instructions.extend(self.assign.compile_uncommented(codegen)?);
        Ok(instructions)
    }
}

impl ForLoop {
    const COUNTER: &str = "$counter";
    const END: &str = "$end";

    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let unique_id = codegen.get_unique_id();
        let start_label = format!("for_loop_{unique_id}");
        let end_label = format!("for_end_{unique_id}");
        let continue_label = format!("for_continue_{unique_id}");

        let outer_depth = codegen.stack_depth();
        let loop_context = LoopContext {
            break_label: end_label.clone(),
            continue_label: continue_label.clone(),
        };
        codegen.with_loop(loop_context, |codegen| {
            let init = self.compile_bounds(codegen)?;

            let condition = compile_jump_unless(
                &self.compare(if self.range.inclusive {
                    BinaryOp::LessOrEqual
                } else {
                    BinaryOp::LessThan
                }),
                &end_label,
                codegen,
            )?;

            let block = self.block.compile(codegen)?;
            let init_size = codegen.stack_depth() - outer_depth;
            let update = self.compile_update(&end_label, codegen)?;

            let mut instructions = vec![Instruction::comment(format!(
                "for {} in {}",
                self.ident, self.range
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
        })
    }

    // 1. Allocates the looping variable
    // 2. Compile range.start directly into it
    // 3. Reserve the $end bound slot
    // 4. Compile range.end there
    // 5. Alias looping var ident to address of $counter
    fn compile_bounds(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let dtype = self.counter_type();
        let end_span = self.range.end.span;

        let mut instructions = vec![
            codegen
                .scope_mut()
                .allocate_variable(Self::COUNTER.to_string(), &dtype)
                .at(self.span)?,
        ];
        let counter = codegen.get_variable(Self::COUNTER).at(self.span)?;
        instructions.extend(self.range.start.compile(codegen, counter)?);

        instructions.push(
            codegen
                .scope_mut()
                .allocate_variable(Self::END.to_string(), &self.range.end.etype)
                .at(end_span)?,
        );
        let end = codegen.get_variable(Self::END).at(end_span)?;
        instructions.extend(self.range.end.compile(codegen, end)?);

        codegen
            .scope_mut()
            .add_variable(self.ident.clone(), counter)
            .at(self.span)?;
        Ok(instructions)
    }

    fn compile_update(
        &self,
        end_label: &str,
        codegen: &mut Codegen,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = vec![];
        if self.range.inclusive {
            instructions.extend(compile_jump_unless(
                &self.compare(BinaryOp::NotEqual),
                end_label,
                codegen,
            )?);
        }

        let increment = Assignment {
            lhs: Self::hidden_variable(Self::COUNTER, self.counter_type(), self.span),
            op: AssignOp::Compound(BinaryOp::Add),
            expression: (1, self.range.span).into(),
        };
        instructions.extend(increment.compile_uncommented(codegen)?);
        Ok(instructions)
    }

    fn compare(&self, op: BinaryOp) -> Expression {
        let mut compare = Expression::new(
            ExpressionKind::Binary(
                op,
                Box::new(Self::hidden_variable(
                    Self::COUNTER,
                    self.counter_type(),
                    self.span,
                )),
                Box::new(Self::hidden_variable(
                    Self::END,
                    Type::clone(&self.range.end.etype),
                    self.range.end.span,
                )),
            ),
            self.range.span,
        );
        *compare.etype = Type::Boolean;
        compare
    }

    fn hidden_variable(name: &str, etype: Type, span: Span) -> Expression {
        let mut variable = Expression::new(
            ExpressionKind::Variable {
                name: name.to_string(),
                binding: None,
            },
            span,
        );
        *variable.etype = etype;
        variable
    }

    fn counter_type(&self) -> Type {
        self.ident_type.clone().unwrap_or(Type::Unknown)
    }
}

impl FunctionDefinition {
    pub fn is_leaf_function(&self) -> bool {
        !self.body.calls_function()
    }

    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let is_leaf_function = self.is_leaf_function();

        let arg_registers: [Register; 4] =
            [Register::RCX, Register::RDX, Register::R8, Register::R9];

        let returns_odd_struct = passed_by_pointer(&self.signature.return_type);

        // Put the hidden arg first
        // Its a pointer to the address of the returned struct
        let hidden_return = returns_odd_struct.then(|| {
            Arg::new(
                "return",
                self.signature.return_type.clone().pointer_to(),
                self.signature.span,
            )
        });

        let (compiled, function_context) =
            codegen.with_function(FunctionContext::from(self), |codegen| {
                let mut arg_homes = vec![];
                let mut struct_copies = vec![];

                for (i, arg) in hidden_return.iter().chain(&self.signature.args).enumerate() {
                    // Win64: after `push rbp`, the shadow space for the first 4 args starts at rbp+16,
                    // and any further args were placed directly after it by the caller
                    let offset = 16 + i32::try_from(i).expect("argument count fits in i32") * 8;
                    let new_address = Address::Stack(-offset);

                    // Move register args to their home in the shadow space
                    if let Some(reg) = arg_registers.get(i) {
                        arg_homes.push(Instruction::mov(new_address, *reg));
                    }

                    let struct_def = match &arg.atype {
                        Type::Struct(struct_def) if passed_by_pointer(&arg.atype) => struct_def,
                        _ => {
                            // Update its location in the scope
                            codegen
                                .scope_mut()
                                .add_variable(arg.name.clone(), new_address)
                                .at(arg.span)?;
                            continue;
                        }
                    };

                    struct_copies.push(
                        codegen
                            .scope_mut()
                            .allocate_variable(arg.name.clone(), &arg.atype)
                            .at(arg.span)?,
                    );
                    let local = codegen.get_variable(&arg.name).at(arg.span)?;
                    struct_copies.extend(codegen.lease_with_scope(|codegen, ptr| {
                        let mut copy = vec![Instruction::mov(ptr, new_address)];
                        copy.extend(mem_copy(
                            struct_def,
                            local,
                            Address::Deref {
                                base: ptr,
                                offset: 0,
                            },
                            codegen,
                        )?);
                        Ok::<_, CodegenError>(copy)
                    })?);
                }

                let body = self.body.compile(codegen)?;
                Ok::<_, CodegenError>((arg_homes, struct_copies, body))
            });
        let (arg_homes, struct_copies, body) = compiled?;
        let FunctionContext {
            epilogue_label,
            touched,
            ..
        } = function_context;

        let save_slots = NON_VOLATILE_REGISTERS
            .into_iter()
            .filter(|reg| touched.contains(reg))
            .zip((1..).map(|i| Address::Stack(i * i32::try_from(SLOT_SIZE).unwrap())))
            .collect::<Vec<_>>();
        let save_size = (SLOT_SIZE * u32::try_from(save_slots.len()).unwrap()).next_multiple_of(16);
        let shift = |instruction: Instruction| {
            instruction.map_addresses(|address| match address {
                Address::Stack(n) if n > 0 => Address::Stack(n + i32::try_from(save_size).unwrap()),
                address => address,
            })
        };
        let struct_copies = struct_copies.into_iter().map(shift).collect::<Vec<_>>();
        let mut body = body.into_iter().map(shift).collect::<Vec<_>>();
        if matches!(body.last(), Some(Instruction::Jmp { label }) if *label == epilogue_label) {
            body.pop();
        }

        let frame = save_slots
            .iter()
            .map(|(reg, slot)| Instruction::mov(*slot, *reg))
            .chain(arg_homes)
            .chain(struct_copies)
            .chain(body)
            .chain([Instruction::label(epilogue_label)])
            .chain(
                save_slots
                    .iter()
                    .map(|(reg, slot)| Instruction::mov(*reg, *slot)),
            )
            .collect::<Vec<_>>();

        Ok(self.wrap_frame(frame, is_leaf_function, save_size))
    }

    fn wrap_frame(
        &self,
        frame: Vec<Instruction>,
        is_leaf_function: bool,
        save_size: u32,
    ) -> Vec<Instruction> {
        let (prologue, frame, epilogue) = if is_leaf_function {
            let (frame, frame_size) = to_rsp_frame(frame);
            let reserved = (frame_size > 0).then_some(frame_size);
            (
                reserved
                    .map(|size| Instruction::sub(Register::RSP, size))
                    .into_iter()
                    .collect(),
                frame,
                reserved
                    .map(|size| Instruction::add(Register::RSP, size))
                    .into_iter()
                    .collect(),
            )
        } else {
            (
                [
                    Instruction::push(Register::RBP),
                    Instruction::mov(Register::RBP, Register::RSP),
                ]
                .into_iter()
                .chain((save_size > 0).then(|| Instruction::sub(Register::RSP, save_size)))
                .collect::<Vec<_>>(),
                frame,
                vec![
                    Instruction::mov(Register::RSP, Register::RBP),
                    Instruction::pop(Register::RBP),
                ],
            )
        };

        std::iter::once(Instruction::label(&self.signature.name))
            .chain(prologue)
            .chain(frame)
            .chain(epilogue)
            .chain([Instruction::Ret])
            .collect()
    }
}

// Retro actively look at compiled asm to check for rsp changed.
// Map old addresses from rbp to rsp since rbp might not be setup if a leaf function
// Strips the body's rsp adjustments so the frame is fixed
// and tracks push/pop so offsets stay correct between them
fn to_rsp_frame(instructions: Vec<Instruction>) -> (Vec<Instruction>, u32) {
    let mut instructions = instructions
        .into_iter()
        .filter(|instruction| {
            !matches!(
                instruction,
                Instruction::Sub {
                    dst: Address::Register(Register::RSP),
                    src: Operand::Immediate(_),
                } | Instruction::Add {
                    dst: Address::Register(Register::RSP),
                    src: Operand::Immediate(_),
                }
            )
        })
        .collect::<Vec<_>>();

    let mut frame_size = 0;
    for instruction in &mut instructions {
        instruction.for_each_address_mut(|address| {
            if let Address::Stack(n) = *address {
                frame_size = frame_size.max(n);
            }
        });
    }

    let mut pushed = 0;
    let instructions = instructions
        .into_iter()
        .map(|instruction| {
            if matches!(instruction, Instruction::Pop { .. }) {
                pushed -= 8;
            }
            let is_push = matches!(instruction, Instruction::Push { .. });
            let instruction = instruction.map_addresses(|address| match address {
                Address::Stack(n) => Address::Deref {
                    base: Register::RSP,
                    offset: u32::try_from(frame_size - n - if n < 0 { 8 } else { 0 } + pushed)
                        .unwrap(),
                },
                address => address,
            });
            if is_push {
                pushed += 8;
            }
            instruction
        })
        .collect();

    (instructions, u32::try_from(frame_size).unwrap())
}

impl IfStatement {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let id = codegen.get_unique_id();
        let else_label = format!("else{id}");
        let end_label = format!("endif{id}");

        let false_target = match self.else_block {
            Some(_) => &else_label,
            None => &end_label,
        };

        let mut instructions = vec![Instruction::comment(format!("if {}", self.condition))];
        instructions.extend(compile_jump_unless(&self.condition, false_target, codegen)?);

        instructions.push(Instruction::comment("then"));
        instructions.extend(self.block.compile(codegen)?);

        if let Some(else_block) = &self.else_block {
            instructions.push(Instruction::jmp(end_label.clone()));
            instructions.push(Instruction::label(else_label));
            instructions.extend(else_block.compile(codegen)?);
        }

        instructions.push(Instruction::label(end_label));
        Ok(instructions)
    }
}

impl MatchStatement {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let end_label = format!("endmatch{}", codegen.get_unique_id());
        let case_labels: Vec<String> = self
            .cases
            .iter()
            .map(|_| format!("match_case{}", codegen.get_unique_id()))
            .collect();

        // TODO: This is a valid program getting rejected
        // Use lease register because it returns OutOfRegisters
        // RegisterHander::lease_with_scope would attemp push r {instructions} pop r
        // but {instructions} would jump past the pop
        let lhs_reg = codegen.lease_register().map_err(|e| e.at(self.span))?;
        let chain = self.compile_compare_chain(codegen, lhs_reg, &case_labels);
        codegen.registers.release_register(lhs_reg);

        let mut instructions = vec![Instruction::comment(format!("match {}", self.lhs))];
        instructions.extend(chain?);

        if let Some(default) = &self.default {
            instructions.push(Instruction::comment("Case: _".to_string()));
            instructions.extend(default.compile(codegen)?);
        }
        if !self.cases.is_empty() {
            instructions.push(Instruction::jmp(end_label.clone()));
        }

        for (i, (case, label)) in self.cases.iter().zip(&case_labels).enumerate() {
            instructions.push(Instruction::label(label.clone()));
            instructions.push(Instruction::comment(format!("Case: {}", case.rhs)));
            instructions.extend(case.block.compile(codegen)?);
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
        codegen: &mut Codegen,
        lhs_reg: Register,
        case_labels: &[String],
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = self.lhs.compile(codegen, lhs_reg)?;

        for (case, label) in self.cases.iter().zip(case_labels) {
            match as_operand(&case.rhs, codegen) {
                Some(rhs) => instructions.push(Instruction::cmp(lhs_reg, rhs)),
                None => instructions.extend(codegen.lease_with_scope(|codegen, rhs_reg| {
                    let mut instructions = case.rhs.compile(codegen, rhs_reg)?;
                    instructions.push(Instruction::cmp(lhs_reg, rhs_reg));
                    Ok(instructions)
                })?),
            }
            instructions.push(Instruction::jcc(Condition::Equal, label.clone()));
        }

        Ok(instructions)
    }
}

impl ReturnStatement {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let mut instructions = vec![Instruction::comment(self.to_string())];
        let function = codegen
            .function_context()
            .ok_or_else(|| CodegenErrorKind::OutsideFunction.at(self.span))?;
        let epilogue_label = function.epilogue_label.clone();
        let returns_by_pointer = passed_by_pointer(&function.return_type);

        if let Some(expr) = &self.expr {
            // Special case for returning structs that dont align to register,
            // need to move it into the address pointed to by rcx

            if returns_by_pointer {
                let address_of_pointer = codegen.get_variable("return").at(self.span)?;
                instructions.extend(codegen.request_with_scope(
                    Register::RAX,
                    Register::RAX.into(),
                    |codegen, rax| {
                        let mut instructions = vec![Instruction::mov(rax, address_of_pointer)];
                        instructions.extend(expr.compile(
                            codegen,
                            Address::Deref {
                                base: rax,
                                offset: 0,
                            },
                        )?);
                        Ok(instructions)
                    },
                )?);
            } else {
                instructions.extend(codegen.request_with_scope(
                    Register::RAX,
                    Register::RAX.into(),
                    |codegen, rax| expr.compile(codegen, rax),
                )?);
            }
        }

        instructions.push(Instruction::jmp(epilogue_label));
        Ok(instructions)
    }
}

impl WhileLoop {
    pub fn compile(&self, codegen: &mut Codegen) -> Result<Vec<Instruction>, CodegenError> {
        let unique_id = codegen.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let condition = compile_jump_unless(&self.condition, &end_label, codegen)?;

        let loop_context = LoopContext {
            break_label: end_label.clone(),
            continue_label: start_label.clone(),
        };
        let block = codegen.with_loop(loop_context, |codegen| self.block.compile(codegen))?;

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
    codegen: &Codegen,
    keyword: &'static str,
    span: Span,
    label: impl Fn(&LoopContext) -> &str,
) -> Result<Vec<Instruction>, CodegenError> {
    let loop_scope = codegen
        .loop_scope()
        .ok_or_else(|| CodegenErrorKind::OutsideLoop(keyword).at(span))?;
    let loop_context = loop_scope
        .loop_context()
        .expect("loop scope has a loop context");

    let mut instructions = vec![Instruction::comment(keyword)];
    let unwind = codegen.stack_depth() - loop_scope.stack_depth;
    if unwind > 0 {
        instructions.push(Instruction::add(Register::RSP, unwind));
    }
    instructions.push(Instruction::jmp(label(loop_context)));
    Ok(instructions)
}

impl BreakStatement {
    pub fn compile(&self, codegen: &Codegen) -> Result<Vec<Instruction>, CodegenError> {
        jump_out_of_loop(codegen, "break", self.span, |l| &l.break_label)
    }
}

impl ContinueStatement {
    pub fn compile(&self, codegen: &Codegen) -> Result<Vec<Instruction>, CodegenError> {
        jump_out_of_loop(codegen, "continue", self.span, |l| &l.continue_label)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Statement, StatementOrExpression},
        codegen::{Codegen, CodegenError, CodegenErrorKind},
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

        let mut codegen = Codegen::default();
        while codegen.lease_register().is_ok() {}

        let result = m.compile(&mut codegen);
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
