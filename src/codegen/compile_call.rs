use crate::{
    ast::{Expression, Walk},
    codegen::{
        CodegenError,
        address::{Address, Register},
        compile::move_into,
        compile_memory::with_place_address,
        compile_scope::{CompileScope, SLOT_SIZE},
        instruction::Instruction,
        register_handler::RegisterHandler,
    },
    error::ResultAt,
    type_check::Type,
};

/// Registers the Win64 calling convention allows a callee to clobber
const VOLATILE_REGISTERS: [Register; 7] = [
    Register::RAX,
    Register::RCX,
    Register::RDX,
    Register::R8,
    Register::R9,
    Register::R10,
    Register::R11,
];

pub const NON_VOLATILE_REGISTERS: [Register; 5] = [
    Register::RBX,
    Register::R12,
    Register::R13,
    Register::R14,
    Register::R15,
];

pub fn passed_by_pointer(t: &Type) -> bool {
    matches!(t, Type::Struct(struct_def) if !matches!(struct_def.size(), 1 | 2 | 4 | 8))
}

pub fn compile_function_call(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    args: &[Expression],
    dst: Option<Address>,
) -> Result<Vec<Instruction>, CodegenError> {
    let return_type = compile_scope
        .lookup_function(name)
        .map(|function| function.return_type.clone());

    // The callee may clobber any volatile register, so save the ones in use. dst is left out, as
    // the caller wants it overwritten
    let saved_registers = VOLATILE_REGISTERS
        .into_iter()
        .filter(|reg| register_handler.is_used(*reg) && dst != Some(Address::Register(*reg)))
        .collect::<Vec<_>>();
    let depth = compile_scope.stack_depth;
    let mut instructions = vec![];

    let demoted_dst = match dst {
        Some(Address::Register(reg)) if register_handler.mark_destination(reg) => Some(reg),
        _ => None,
    };

    let hidden_return = match (&return_type, dst) {
        (Some(t), Some(dst)) if passed_by_pointer(t) && !matches!(dst, Address::Register(_)) => {
            Some(dst)
        }
        // Make extra room for the returned struct to go into stack
        (Some(t), _) if passed_by_pointer(t) => {
            let size = u32::try_from(t.size()).unwrap().next_multiple_of(SLOT_SIZE);
            compile_scope.stack_depth += size;
            instructions.push(Instruction::sub(Register::RSP, size));
            Some(Address::Stack(compile_scope.stack_depth.cast_signed()))
        }
        _ => None,
    };

    let all_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
    let target_registers = &all_registers[usize::from(hidden_return.is_some())..];
    let (register_args, stack_args) = args.split_at(args.len().min(target_registers.len()));

    let (temporary_instructions, temporaries) =
        struct_temporaries(args, return_type.is_none(), compile_scope, register_handler)?;
    instructions.extend(temporary_instructions);
    let temporaries_size = compile_scope.stack_depth - depth;
    let (register_temporaries, stack_temporaries) = temporaries.split_at(register_args.len());

    instructions.extend(saved_registers.iter().map(|reg| Instruction::push(*reg)));

    // Registers already in use were saved above, so only the free ones need to be claimed
    let mut requested_registers: Vec<Register> = vec![];
    if let Some(address) = hidden_return {
        instructions.push(Instruction::lea(Register::RCX, address));
        if register_handler.request_register(Register::RCX).is_ok() {
            requested_registers.push(Register::RCX);
        }
    }

    for ((arg, temporary), dst_reg) in register_args
        .iter()
        .zip(register_temporaries)
        .zip(target_registers.iter().copied())
    {
        //  Prevent register from being modified, including by calls inside the arg
        if register_handler.request_register(dst_reg).is_ok() {
            requested_registers.push(dst_reg);
        }
        instructions.extend(compile_arg(
            arg,
            *temporary,
            dst_reg.into(),
            compile_scope,
            register_handler,
        )?);
    }

    // Win64: 32 bytes of shadow space, followed by any args past the 4th. rsp must be 16 byte
    // aligned at the call, including the locals and saved registers
    let pushed_size = compile_scope.stack_depth + u32::try_from(saved_registers.len()).unwrap() * 8;
    let stack_size: u32 = (pushed_size + 32 + u32::try_from(stack_args.len()).unwrap() * 8)
        .next_multiple_of(16)
        - pushed_size;
    instructions.push(Instruction::sub(Register::RSP, stack_size));

    instructions.extend(compile_stack_args(
        stack_args,
        stack_temporaries,
        compile_scope,
        register_handler,
    )?);

    instructions.push(Instruction::call(name));
    instructions.push(Instruction::add(Register::RSP, stack_size));
    if hidden_return.is_none()
        && let Some(dst) = dst
        && dst != Address::Register(Register::RAX)
    {
        instructions.push(Instruction::mov(dst, Register::RAX));
    }

    // Give back the registers
    for reg in requested_registers {
        register_handler.release_register(reg);
    }
    if let Some(reg) = demoted_dst {
        register_handler.mark_hot(reg);
    }

    instructions.extend(
        saved_registers
            .iter()
            .rev()
            .map(|reg| Instruction::pop(*reg)),
    );

    if temporaries_size > 0 {
        instructions.push(Instruction::add(Register::RSP, temporaries_size));
        compile_scope.stack_depth -= temporaries_size;
    }

    Ok(instructions)
}

fn compile_stack_args(
    stack_args: &[Expression],
    stack_temporaries: &[Option<Address>],
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let mut instructions = vec![];
    for (i, (arg, temporary)) in stack_args.iter().zip(stack_temporaries).enumerate() {
        let offset = 32 + i32::try_from(i).unwrap() * 8;
        if arg.calls_function() {
            instructions.extend(register_handler.lease_with_scope(|register_handler, tmp| {
                let mut arg =
                    compile_arg(arg, *temporary, tmp.into(), compile_scope, register_handler)?;
                arg.push(Instruction::mov(Address::StackArg(offset), tmp));
                Ok(arg)
            })?);
        } else {
            instructions.extend(compile_arg(
                arg,
                *temporary,
                Address::StackArg(offset),
                compile_scope,
                register_handler,
            )?);
        }
    }
    Ok(instructions)
}

/// Structs of 1, 2, 4 or 8 bytes are passed by value, any other struct is passed as a pointer.
/// `TanScript` callees copy those into their own frame, so places can point at the original, but an
/// extern may modify it, so it gets a copy. Anything else (a struct literal) is built into stack
/// space here first. This happens before the saved registers are pushed so `stack_depth` covers it
/// when aligning the call, and the caller frees it once the call returns
fn struct_temporaries(
    args: &[Expression],
    is_extern: bool,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<(Vec<Instruction>, Vec<Option<Address>>), CodegenError> {
    let mut instructions = vec![];
    let mut temporaries = vec![];
    for arg in args {
        let arg_type = compile_scope.expression_type(arg).at(arg.span)?;
        if !passed_by_pointer(&arg_type) || (arg.is_place() && !is_extern) {
            temporaries.push(None);
            continue;
        }

        let size = u32::try_from(arg_type.size())
            .unwrap()
            .next_multiple_of(SLOT_SIZE);
        compile_scope.stack_depth += size;
        let temporary = Address::Stack(i32::try_from(compile_scope.stack_depth).unwrap());
        instructions.push(Instruction::sub(Register::RSP, size));
        instructions.extend(arg.compile(compile_scope, temporary, register_handler)?);
        temporaries.push(Some(temporary));
    }

    Ok((instructions, temporaries))
}

fn compile_arg(
    arg: &Expression,
    temporary: Option<Address>,
    dst: Address,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    if let Some(temporary) = temporary {
        return move_into(dst, register_handler, |reg| {
            Instruction::lea(reg, temporary)
        });
    }

    if !passed_by_pointer(compile_scope.expression_type(arg).at(arg.span)?.as_ref()) {
        return arg.compile(compile_scope, dst, register_handler);
    }

    with_place_address(
        arg,
        compile_scope,
        register_handler,
        &mut |_, register_handler, address, _| {
            move_into(dst, register_handler, |reg| Instruction::lea(reg, address))
        },
    )
}

#[cfg(test)]
mod test {
    use super::NON_VOLATILE_REGISTERS;
    use crate::codegen::compile::test::compile;

    #[test]
    fn compile_call_into_busy_rax_keeps_result() {
        let asm = compile("def five(): i32 { 5 } let a = five() / 2;");
        assert!(!asm.contains("pop rax"), "{asm}");
    }

    #[test]
    fn compile_return_saves_rax_around_call() {
        let asm = compile("def f(n: i32): i32 { 35 + f(n) } f(1);");
        assert!(asm.contains("push rax"), "{asm}");
        assert!(asm.contains("pop rax"), "{asm}");
    }

    #[test]
    fn compile_nested_call_skips_saving_unwritten_dst() {
        let asm = compile(
            "def add2(a: i32, b: i32): i32 { a + b } def add3(a: i32, b: i32, c: i32): i32 { return add2(c, add2(a, b)); } add3(1, 2, 3);",
        );
        assert!(!asm.contains("push rax"), "{asm}");
    }

    #[test]
    fn compile_call_with_stack_args() {
        let asm = compile("printf(\"%d %d %d %d %d\", 1, 2, 3, 4, 5);");
        // 32 shadow + 2 stack args, rounded up to 16
        assert!(asm.contains("sub rsp, 48"), "{asm}");
        assert!(asm.contains("[rsp + 32], "), "{asm}");
        assert!(asm.contains("[rsp + 40], "), "{asm}");
        assert!(asm.contains("add rsp, 48"), "{asm}");
    }

    #[test]
    fn compile_call_stack_arg_written_after_nested_call_restores_registers() {
        let asm = compile("def f(x: i32): i32 { x } printf(\"%d %d %d %d %d\", 1, 2, 3, f(4), 5);");
        let store = asm.find("[rsp + 32], ").expect(&asm);
        let restore = asm.rfind("pop rcx").expect(&asm);
        assert!(restore < store, "{asm}");
    }

    #[test]
    fn compile_struct_arg_passes_pointer_and_callee_copies() {
        let asm = compile(
            "struct P { a: i64, b: i64, } def f(p: P): i64 { p.a } let p = P { a: 1, b: 2 }; f(p);",
        );
        assert!(asm.contains("lea rcx, [rbp - 16]"), "{asm}");
        assert!(asm.contains(", [rsp + 24]"), "{asm}");
        assert!(asm.contains("+ 8]"), "{asm}");
    }

    #[test]
    fn compile_struct_literal_arg_uses_temporary() {
        let asm =
            compile("struct P { a: i64, b: i64, } def f(p: P): i64 { p.a } f(P { a: 1, b: 2 });");
        let call = asm.find("call f").expect(&asm);
        assert!(asm[..call].contains("sub rsp, 16"), "{asm}");
        assert!(asm[call..].contains("add rsp, 16"), "{asm}");
        assert!(asm.contains("lea rcx, [rbp - "), "{asm}");
    }

    #[test]
    fn compile_function_reads_args_from_caller_frame() {
        let asm = compile(
            "def f(a: i32, b: i32, c: i32, d: i32, e: i32): i32 { a + e } f(1, 2, 3, 4, 5);",
        );
        assert!(asm.contains("mov [rsp + 8], rcx"), "{asm}");
        assert!(asm.contains("mov [rsp + 32], r9"), "{asm}");
        assert!(asm.contains("[rsp + 40]"), "{asm}");
    }

    #[test]
    fn compile_function_with_locals() {
        let asm = compile("def f(a: i32): i32 { let b = 1; a + b } f(1);");
        assert!(asm.contains("mov [rsp + 16], rcx"), "{asm}");
        assert!(asm.contains("mov QWORD [rsp], 1"), "{asm}");
        assert_eq!(
            asm.lines().filter(|l| l.trim() == "ret").count(),
            1,
            "{asm}"
        );
    }

    #[test]
    fn compile_function_saves_only_touched_non_volatile_registers() {
        let asm = compile("def f(a: i32): i32 { a } f(1);");
        for reg in NON_VOLATILE_REGISTERS {
            assert!(!asm.contains(&format!("], {reg}")), "{asm}");
        }

        let asm = compile("def f(a: i32): i32 { a * 2 + a * 3 } f(1);");
        let saved = NON_VOLATILE_REGISTERS
            .iter()
            .filter(|reg| asm.contains(&format!("], {reg}")))
            .collect::<Vec<_>>();
        for reg in &saved {
            assert!(asm.contains(&format!("\tmov {reg}, [rbp - ")), "{asm}");
        }
    }

    #[test]
    fn compile_nested_call_saves_loaded_args() {
        // r8 already holds 2 when f is called, and f is free to clobber it
        let asm =
            compile("def f(a: i32, b: i32): i32 { a + b } printf(\"%d %d %d\", 1, 2, f(1, 1));");
        assert!(asm.contains("push r8"), "{asm}");
        assert!(asm.contains("pop r8"), "{asm}");
    }
}
