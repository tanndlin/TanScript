use crate::{
    ast::{BinaryOp, Expression, ExpressionKind, PostfixOp, UnaryOp},
    codegen::{
        CodegenError, CodegenErrorKind,
        address::{Address, Register},
        compile::{as_operand, move_into},
        compile_memory::{extend, load_element, narrow_size, store_element, with_place_address},
        compile_scope::CompileScope,
        instruction::{Condition, Instruction, Operand},
        register_handler::RegisterHandler,
    },
    error::{At, ResultAt},
    type_check::Type,
    types::NumberType,
};

pub fn compile_binary(
    op: BinaryOp,
    left: &Expression,
    right: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let cond = match op {
        BinaryOp::Multiply => {
            return compile_multiply(compile_scope, left, right, dst, register_handler);
        }
        BinaryOp::Divide => {
            return compile_divide(
                compile_scope,
                left,
                right,
                dst,
                register_handler,
                Register::RAX,
            );
        }
        BinaryOp::Modulo => {
            return compile_divide(
                compile_scope,
                left,
                right,
                dst,
                register_handler,
                Register::RDX,
            );
        }
        BinaryOp::Add => {
            return compile_simple_binary(
                Instruction::add,
                compile_scope,
                left,
                right,
                dst,
                register_handler,
            );
        }
        BinaryOp::Subtract => {
            return compile_simple_binary(
                Instruction::sub,
                compile_scope,
                left,
                right,
                dst,
                register_handler,
            );
        }
        BinaryOp::Or | BinaryOp::And => {
            unreachable!("logical operators are compiled by compile_logical_binary")
        }
        BinaryOp::LessThan => Condition::Less,
        BinaryOp::LessOrEqual => Condition::LessEqual,
        BinaryOp::GreaterThan => Condition::Greater,
        BinaryOp::GreaterOrEqual => Condition::GreaterEqual,
        BinaryOp::Equal => Condition::Equal,
        BinaryOp::NotEqual => Condition::NotEqual,
    };

    let (mut instructions, cond) =
        compile_compare(cond, compile_scope, left, right, dst, register_handler)?;
    instructions.push(Instruction::mov(dst, NumberType::I64(0)));
    instructions.push(Instruction::set(cond, dst));
    Ok(instructions)
}

/// Compiles `left` into `dst`, `right` into a scratch register, unless right is a leaf or dst is a register
fn compile_simple_binary(
    instruction: impl FnOnce(Address, Operand) -> Instruction,
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let left_is_dst =
        matches!(as_operand(left, compile_scope), Some(Operand::Address(a)) if a == dst);
    let right_is_imm = matches!(
        as_operand(right, compile_scope),
        Some(Operand::Immediate(_))
    );
    if dst.is_memory() && !left_is_dst && !right_is_imm {
        return register_handler.lease_with_scope(|register_handler, reg| {
            let mut instructions = compile_simple_binary(
                instruction,
                compile_scope,
                left,
                right,
                reg.into(),
                register_handler,
            )?;
            instructions.push(Instruction::mov(dst, reg));
            Ok(instructions)
        });
    }

    let mut instructions = left.compile(compile_scope, dst, register_handler)?;
    if let Some(rhs_op) = as_operand(right, compile_scope)
        && !(dst.is_memory() && rhs_op.is_memory())
    {
        instructions.push(instruction(dst, rhs_op));
        Ok(instructions)
    } else {
        instructions.extend(
            register_handler.lease_with_scope(|register_handler, right_reg| {
                let mut right = right.compile(compile_scope, right_reg, register_handler)?;
                right.push(instruction(dst, right_reg.into()));
                Ok(right)
            })?,
        );

        Ok(instructions)
    }
}

pub fn compile_logical_binary(
    condition: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let id = register_handler.get_unique_id();
    let false_label = format!("logic_false_{id}");
    let end_label = format!("logic_end_{id}");
    let demoted_dst = match dst {
        Address::Register(reg) if register_handler.mark_destination(reg) => Some(reg),
        _ => None,
    };

    let mut instructions = compile_jump(
        condition,
        &false_label,
        false,
        compile_scope,
        register_handler,
    )?;

    if let Some(reg) = demoted_dst {
        register_handler.mark_hot(reg);
    }
    instructions.push(Instruction::mov(dst, NumberType::I64(1)));
    instructions.push(Instruction::jmp(end_label.clone()));
    instructions.push(Instruction::label(false_label));
    instructions.push(Instruction::mov(dst, NumberType::I64(0)));
    instructions.push(Instruction::label(end_label));
    Ok(instructions)
}

fn compile_compare(
    cond: Condition,
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<(Vec<Instruction>, Condition), CodegenError> {
    let (left, right, cond) = match (
        as_operand(left, compile_scope),
        as_operand(right, compile_scope),
    ) {
        (Some(Operand::Immediate(_)), Some(Operand::Immediate(_))) => (left, right, cond),
        (Some(Operand::Immediate(_)), _) => (right, left, cond.swapped()),
        _ => (left, right, cond),
    };

    let mut instructions = vec![];
    let lhs = if let Some(Operand::Address(address)) = as_operand(left, compile_scope) {
        address
    } else {
        instructions.extend(left.compile(compile_scope, dst, register_handler)?);
        dst
    };

    match as_operand(right, compile_scope) {
        Some(rhs) if !(lhs.is_memory() && rhs.is_memory()) => {
            instructions.push(Instruction::cmp(lhs, rhs));
        }
        _ => instructions.extend(register_handler.lease_with_scope(
            |register_handler, right_reg| {
                let mut right = right.compile(compile_scope, right_reg, register_handler)?;
                right.push(Instruction::cmp(lhs, right_reg));
                Ok(right)
            },
        )?),
    }

    Ok((instructions, cond))
}

fn comparison(op: BinaryOp) -> Option<Condition> {
    match op {
        BinaryOp::LessThan => Some(Condition::Less),
        BinaryOp::LessOrEqual => Some(Condition::LessEqual),
        BinaryOp::GreaterThan => Some(Condition::Greater),
        BinaryOp::GreaterOrEqual => Some(Condition::GreaterEqual),
        BinaryOp::Equal => Some(Condition::Equal),
        BinaryOp::NotEqual => Some(Condition::NotEqual),
        _ => None,
    }
}

pub fn compile_jump_unless(
    condition: &Expression,
    label: &str,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    compile_jump(condition, label, false, compile_scope, register_handler)
}

fn compile_jump(
    condition: &Expression,
    label: &str,
    jump_when: bool,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let ExpressionKind::Binary(op, left, right) = &condition.kind else {
        return compile_jump_nonzero(condition, label, jump_when, compile_scope, register_handler);
    };

    if let Some(cond) = comparison(*op) {
        let mut cond = cond;
        let mut instructions = register_handler.lease_with_scope(|register_handler, dst| {
            let (instructions, compared) = compile_compare(
                cond.clone(),
                compile_scope,
                left,
                right,
                dst.into(),
                register_handler,
            )?;
            cond = compared;
            Ok(instructions)
        })?;
        let cond = if jump_when { cond } else { cond.negated() };
        instructions.push(Instruction::jcc(cond, label));
        return Ok(instructions);
    }

    match op {
        BinaryOp::And | BinaryOp::Or => {
            if matches!(op, BinaryOp::Or) == jump_when {
                let mut instructions =
                    compile_jump(left, label, jump_when, compile_scope, register_handler)?;
                instructions.extend(compile_jump(
                    right,
                    label,
                    jump_when,
                    compile_scope,
                    register_handler,
                )?);
                Ok(instructions)
            } else {
                let skip = format!("cond_skip_{}", register_handler.get_unique_id());
                let mut instructions =
                    compile_jump(left, &skip, !jump_when, compile_scope, register_handler)?;
                instructions.extend(compile_jump(
                    right,
                    label,
                    jump_when,
                    compile_scope,
                    register_handler,
                )?);
                instructions.push(Instruction::label(skip));
                Ok(instructions)
            }
        }
        _ => compile_jump_nonzero(condition, label, jump_when, compile_scope, register_handler),
    }
}

fn compile_jump_nonzero(
    condition: &Expression,
    label: &str,
    jump_when: bool,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let mut instructions = register_handler.lease_with_scope(|register_handler, dst| {
        let mut instructions = condition.compile(compile_scope, dst, register_handler)?;
        instructions.push(Instruction::test(dst, dst));
        Ok(instructions)
    })?;
    let cond = if jump_when {
        Condition::NotEqual
    } else {
        Condition::Equal
    };
    instructions.push(Instruction::jcc(cond, label));
    Ok(instructions)
}

pub fn compile_cast(
    inner: &Expression,
    cast_type: &Type,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let mut instructions = inner.compile(compile_scope, dst, register_handler)?;
    let truncate = |reg: Register| extend(cast_type, reg, reg);

    match dst {
        Address::Register(reg) => instructions.extend(truncate(reg)),
        _ => {
            if truncate(Register::RAX).is_some() {
                instructions.extend(register_handler.lease_with_scope(|_, reg| {
                    Ok::<_, CodegenError>(vec![
                        Instruction::mov(reg, dst),
                        truncate(reg).unwrap(),
                        Instruction::mov(dst, reg),
                    ])
                })?);
            }
        }
    }

    Ok(instructions)
}

pub fn compile_unary(
    op: UnaryOp,
    child: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let instruction = match op {
        UnaryOp::Negate => Instruction::neg(dst),
        UnaryOp::Not => Instruction::xor(dst, NumberType::I64(1)),
        UnaryOp::AddressOf => {
            return with_place_address(
                child,
                compile_scope,
                register_handler,
                &mut |_, register_handler, address, _| {
                    move_into(dst, register_handler, |reg| Instruction::lea(reg, address))
                },
            );
        }
        UnaryOp::Deref => unreachable!("deref is compiled as a place"),
    };

    let mut instructions = child.compile(compile_scope, dst, register_handler)?;
    instructions.push(instruction);
    Ok(instructions)
}

pub fn compile_postfix(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    compile_postfix_in_place(op, lhs, compile_scope, Some(dst), register_handler)
}

pub fn compile_postfix_update(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    compile_postfix_in_place(op, lhs, compile_scope, None, register_handler)
}

/// Evaluates the place once so `a[next()]++` only calls `next()` once
fn compile_postfix_in_place(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    dst: Option<Address>,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    with_place_address(
        lhs,
        compile_scope,
        register_handler,
        &mut |_, register_handler, address, t| {
            let update = |target: Address| match op {
                PostfixOp::Increment => Instruction::add(target, 1),
                PostfixOp::Decrement => Instruction::sub(target, 1),
            };
            if dst.is_none() && narrow_size(&t).is_none() {
                return Ok(vec![update(address)]);
            }
            register_handler.lease_with_scope(|_, value| {
                let mut instructions = vec![load_element(&t, value, address)];
                if let Some(dst) = dst {
                    instructions.push(Instruction::mov(dst, value));
                }
                instructions.push(update(value.into()));
                instructions.push(store_element(&t, address, value));
                Ok(instructions)
            })
        },
    )
}

fn compile_multiply(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    match dst {
        Address::Register(dst_reg) => {
            let mut instructions = left.compile(compile_scope, dst, register_handler)?;
            instructions.extend(register_handler.lease_with_scope(
                |register_handler, right_reg| {
                    let mut right = right.compile(compile_scope, right_reg, register_handler)?;
                    right.push(Instruction::imul(dst_reg, right_reg));
                    Ok(right)
                },
            )?);
            Ok(instructions)
        }
        _ => register_handler.lease_with_scope(|register_handler, left_reg| {
            let mut instructions = left.compile(compile_scope, left_reg, register_handler)?;

            instructions.extend(register_handler.lease_with_scope(
                |register_handler, right_reg| {
                    let mut right = right.compile(compile_scope, right_reg, register_handler)?;
                    right.push(Instruction::imul(left_reg, right_reg));
                    right.push(Instruction::mov(dst, left_reg));
                    Ok(right)
                },
            )?);

            Ok(instructions)
        }),
    }
}

/// `div` leaves the quotient in rax and the remainder in rdx, so `result` picks between / and %
fn compile_divide(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
    result: Register,
) -> Result<Vec<Instruction>, CodegenError> {
    let unsigned = matches!(
        compile_scope.unified_type(left, right).at(left.span)?.as_ref(),
        Type::Int(int) if !int.is_signed()
    );

    register_handler.lease_with_scope(|register_handler, right_reg| {
        // Only happens when every other register is leased, and div overwrites both
        if matches!(right_reg, Register::RAX | Register::RDX) {
            return Err(CodegenErrorKind::OutOfRegisters.at(left.span));
        }

        let mut instructions = right.compile(compile_scope, right_reg, register_handler)?;

        let divide =
            register_handler.request_with_scope(Register::RAX, dst, |register_handler, rax| {
                let mut instructions = left.compile(compile_scope, rax, register_handler)?;

                instructions.extend(register_handler.request_with_scope(
                    Register::RDX,
                    dst,
                    |_, _| {
                        Ok(if unsigned {
                            vec![
                                Instruction::xor(Register::RDX, Register::RDX),
                                Instruction::div(right_reg),
                                Instruction::mov(dst, result),
                            ]
                        } else {
                            vec![
                                Instruction::Cqo,
                                Instruction::idiv(right_reg),
                                Instruction::mov(dst, result),
                            ]
                        })
                    },
                )?);

                Ok(instructions)
            })?;

        // right was compiled first so it has to run first, or its own rax use clobbers left
        instructions.extend(divide);
        Ok(instructions)
    })
}

#[cfg(test)]
mod test {
    use crate::codegen::compile::test::compile;

    #[test]
    fn compile_assignment_reads_target_before_overwriting_it() {
        for input in [
            "let a: i64 = 25; a = 0 - a;",
            "let a: i64 = 25; a = 1 + a;",
            "let a: i64 = 25; let z: i64 = 3; a = z - a;",
            "let a: i64 = 25; a = 1 + (a - 5);",
        ] {
            let asm = compile(input);
            let assignment = asm
                .lines()
                .skip_while(|l| !l.starts_with("; a = "))
                .collect::<Vec<_>>();
            let first_write = assignment
                .iter()
                .position(|l| l.contains("[rbp - 8],"))
                .unwrap();
            let last_read = assignment
                .iter()
                .rposition(|l| l.contains(", [rbp - 8]"))
                .unwrap();
            assert!(last_read < first_write, "{input}\n{asm}");
        }
    }

    #[test]
    fn compile_negate() {
        let asm = compile("let a = 5; let b = -a;");
        assert!(asm.contains("neg QWORD"));
    }

    #[test]
    fn compile_divide_uses_separate_lines() {
        for input in ["let a = 7 / 2;", "let a = 7 % 2;"] {
            let asm = compile(input);
            assert!(asm.lines().any(|l| l.trim() == "cqo"), "{asm}");
            assert!(asm.lines().any(|l| l.trim().starts_with("idiv ")), "{asm}");
        }
    }

    #[test]
    fn compile_unsigned_divide_uses_div() {
        for input in [
            "let a: u64 = 7; let b = a / 2;",
            "let a: u32 = 7; let b = a % 2;",
        ] {
            let asm = compile(input);
            assert!(asm.lines().any(|l| l.trim().starts_with("div ")), "{asm}");
            assert!(!asm.lines().any(|l| l.trim() == "cqo"), "{asm}");
        }
    }

    #[test]
    fn compile_divide_by_unsigned_call_uses_div() {
        let asm = compile("def five(): u64 { return 5; } let a = five() / 2;");
        assert!(asm.lines().any(|l| l.trim().starts_with("div ")), "{asm}");
    }

    #[test]
    fn compile_divide_and_modulo_pick_result_register() {
        assert!(compile("let a = 7 / 2;").contains("mov [rbp - 8], rax"));
        assert!(compile("let a = 7 % 2;").contains("mov [rbp - 8], rdx"));
    }

    #[test]
    fn compile_nested_division() {
        compile("let a = 100; let b = (a / 5) / 2;");
        compile("printf(\"%d %d %d\", 1, 2, 7 / 2);");
        compile("printf(\"%d %d %d\", 1, 2, 17 % 5);");
    }

    #[test]
    fn compile_division_runs_right_side_first() {
        // The right side's division uses rax, so it must finish before left is loaded into rax
        let asm = compile("let a = 100; let b = a / (20 / 2);");
        let load_right = asm.find("rax, 20").unwrap();
        let load_left = asm.find("rax, [rbp - 8]").unwrap();
        assert!(load_right < load_left, "{asm}");
    }

    #[test]
    fn compile_comparisons_set_their_condition() {
        for (op, set) in [
            ("<", "setl"),
            ("<=", "setle"),
            (">", "setg"),
            (">=", "setge"),
            ("==", "sete"),
            ("!=", "setne"),
        ] {
            let asm = compile(&format!("let a = 1; let b = a {op} 2;"));
            assert!(asm.contains("cmp QWORD [rbp - 8], 2"), "{asm}");
            assert!(asm.contains(&format!("	{set} [rbp - 16]")), "{asm}");
        }
    }

    #[test]
    fn compile_logical_ops_on_values() {
        let asm = compile("let a = true; let b = false; let c = a && b; let d = a || b;");
        assert!(!asm.contains("	and "), "{asm}");
        assert!(!asm.contains("	or "), "{asm}");
        assert!(asm.contains("logic_false_"), "{asm}");
        assert!(asm.contains("logic_end_"), "{asm}");
    }

    #[test]
    fn compile_and_value_skips_right_side() {
        let asm = compile("def f(): bool { true } let a = false; let b = a && f();");
        let jump = asm.find("je logic_false_").unwrap();
        let call = asm.find("call f").unwrap();
        assert!(jump < call, "{asm}");
    }

    #[test]
    fn compile_and_condition_jumps_to_end_on_either_failure() {
        let asm = compile("let a = 1; if (a < 2 && a > 0) { printf(\"x\"); }");
        assert!(asm.contains("jge endif"), "{asm}");
        assert!(asm.contains("jle endif"), "{asm}");
        assert!(!asm.contains("cond_skip"), "{asm}");
    }

    #[test]
    fn compile_or_condition_skips_second_check() {
        let asm = compile("let a = 1; if (a < 2 || a > 5) { printf(\"x\"); }");
        let skip = asm.find("jl cond_skip_").unwrap();
        let second = asm.find("jle endif").unwrap();
        let label = asm
            .find(
                "
cond_skip_",
            )
            .unwrap();
        assert!(skip < second && second < label, "{asm}");
    }

    #[test]
    fn compile_if_on_bool_tests_value() {
        let asm = compile("let a = true; if (a) { printf(\"x\"); }");
        assert!(asm.lines().any(|l| l.trim().starts_with("test ")), "{asm}");
        assert!(asm.contains("je endif"), "{asm}");
    }

    #[test]
    fn compile_discarded_division_still_divides() {
        let asm = compile("let a = 7; a / 0;");
        assert!(asm.lines().any(|l| l.trim().starts_with("idiv ")), "{asm}");
    }
}
