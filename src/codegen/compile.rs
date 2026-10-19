use crate::{
    ast::{
        Assignment, AtomType, BinaryOp, Block, Expression, ExpressionKind, PostfixOp, Program,
        StatementOrExpression::{self},
        StructDefinition, UnaryOp,
    },
    codegen::{
        CodegenError, CodegenErrorKind,
        address::{Address, Register},
        compile_scope::{CompileScope, SLOT_SIZE},
        instruction::{Condition, Instruction, Operand, Size},
        register_handler::RegisterHandler,
        symbol_table::SymbolTable,
    },
    error::{At, ResultAt},
    type_check::Type,
    types::{NumberType, Span},
};

impl Program {
    pub fn compile(&self) -> Result<String, CodegenError> {
        let mut global_scope = CompileScope::new(None);
        let mut register_handler = RegisterHandler::new();

        let mut symbol_table = SymbolTable::new();
        self.block.discover(&mut symbol_table);
        symbol_table.externs.insert("exit".to_string());
        for def in symbol_table.functions.values() {
            global_scope.insert_function(def.name.clone(), def.clone());
        }
        for def in symbol_table.structs.values() {
            global_scope.insert_struct(def.clone());
        }

        let instructions = render(
            &self
                .block
                .compile(&mut global_scope, &mut register_handler)?,
        );

        let functions = symbol_table
            .functions
            .values()
            .map(|def| {
                def.compile(&mut global_scope, &mut register_handler)
                    .map(|instructions| render(&instructions))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        let globals = symbol_table
            .functions
            .keys()
            .map(|name| format!("global {name}"))
            .collect::<Vec<_>>()
            .join("\n");

        let extern_functions = symbol_table
            .externs
            .iter()
            .map(|e| format!("extern {e}"))
            .collect::<Vec<_>>()
            .join("\n");

        let data = register_handler
            .data
            .iter()
            .map(|d| format!("\t{d}"))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(format!(
            "BITS 64

global main
{globals}
{extern_functions}

SECTION .data
{data}

SECTION .text

{functions}

main:
\tpush rbp
\tmov rbp, rsp
{instructions}
\tmov rsp, rbp
\tpop rbp
\tsub rsp, 40
\txor rcx, rcx
\tcall exit"
        ))
    }
}

pub fn render(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl Block {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut new_scope = CompileScope::new(Some(compile_scope));

        let mut instructions = self
            .children
            .iter()
            .map(|child| child.compile(&mut new_scope, register_handler))
            .collect::<Result<Vec<_>, CodegenError>>()?
            .concat();

        let alloc_size = new_scope.stack_depth - compile_scope.stack_depth;
        // A trailing return, break or continue jumps away, so a dealloc after it is unreachable
        if alloc_size > 0 && !matches!(instructions.last(), Some(Instruction::Jmp { .. })) {
            instructions.push(Instruction::add(Register::RSP, alloc_size));
        }

        Ok(instructions)
    }
}

impl StatementOrExpression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self {
            StatementOrExpression::Statement(statement) => {
                statement.compile(compile_scope, register_handler)
            }
            StatementOrExpression::Expression(expression) => Ok([
                vec![Instruction::comment(expression.to_string())],
                expression.compile_discarded(compile_scope, register_handler)?,
            ]
            .concat()),
        }
    }
}

impl Expression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        dst: impl Into<Address>,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let dst = dst.into();

        match &self.kind {
            ExpressionKind::Atom(a) => match a {
                AtomType::Number(n) => compile_number(*n, dst, register_handler),
                AtomType::Char(c) => Ok(vec![compile_char(*c, dst)]),
                AtomType::Identifier(name) => {
                    compile_variable(compile_scope, register_handler, name, self.span, dst)
                }
                AtomType::String(s) => compile_string(s, dst, register_handler),
                AtomType::Boolean(b) => Ok(vec![compile_boolean(*b, dst)]),
            },
            ExpressionKind::Binary(op, left, right) => {
                compile_binary(*op, left, right, compile_scope, dst, register_handler)
            }
            ExpressionKind::Index(..)
            | ExpressionKind::StructAccess { .. }
            | ExpressionKind::Unary(UnaryOp::Deref, _) => with_place_address(
                self,
                compile_scope,
                register_handler,
                &mut |_, register_handler, address, t| {
                    move_into(dst, register_handler, |reg| load_element(&t, reg, address))
                },
            ),
            ExpressionKind::Unary(op, child) => {
                compile_unary(*op, child, compile_scope, dst, register_handler)
            }
            ExpressionKind::Postfix(op, lhs) => {
                compile_postfix(*op, lhs, compile_scope, dst, register_handler)
            }
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, Some(dst))
            }
            ExpressionKind::Cast(inner, cast_type) => {
                compile_cast(inner, cast_type, compile_scope, dst, register_handler)
            }
            ExpressionKind::StructInstantiation {
                struct_type_name,
                fields,
            } => compile_struct_instantiation(
                struct_type_name,
                fields,
                compile_scope,
                dst,
                register_handler,
            ),
        }
    }

    // For when the output is not directly read. mostly used as a side affect like a postfix
    pub fn compile_discarded(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self.kind {
            ExpressionKind::Postfix(op, lhs) => {
                compile_postfix_update(*op, lhs, compile_scope, register_handler)
            }
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, None)
            }
            ExpressionKind::Unary(_, child) | ExpressionKind::Cast(child, _) => {
                child.compile_discarded(compile_scope, register_handler)
            }
            ExpressionKind::Binary(BinaryOp::Divide | BinaryOp::Modulo, ..) => register_handler
                .lease_with_scope(|register_handler, dst| {
                    self.compile(compile_scope, dst, register_handler)
                }),
            ExpressionKind::Binary(_, left, right) | ExpressionKind::Index(left, right) => {
                let mut instructions = left.compile_discarded(compile_scope, register_handler)?;
                instructions.extend(right.compile_discarded(compile_scope, register_handler)?);
                Ok(instructions)
            }
            ExpressionKind::Atom(_)
            | ExpressionKind::StructInstantiation { .. }
            | ExpressionKind::StructAccess { .. } => Ok(vec![]),
        }
    }
}

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

fn compile_string(
    s: &str,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let handle = register_handler.add_data(s);
    move_into(dst, register_handler, |reg| {
        Instruction::mov(reg, Operand::Label(handle))
    })
}

fn move_into(
    dst: Address,
    register_handler: &mut RegisterHandler,
    instruction: impl FnOnce(Register) -> Instruction,
) -> Result<Vec<Instruction>, CodegenError> {
    match dst {
        Address::Register(reg) => Ok(vec![instruction(reg)]),
        _ => register_handler
            .lease_with_scope(|_, reg| Ok(vec![instruction(reg), Instruction::mov(dst, reg)])),
    }
}

fn compile_function_call(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    args: &[Expression],
    dst: Option<Address>,
) -> Result<Vec<Instruction>, CodegenError> {
    let target_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
    let (register_args, stack_args) = args.split_at(args.len().min(target_registers.len()));

    // The callee may clobber any volatile register, so save the ones in use. dst is left out, as
    // the caller wants it overwritten
    let saved_registers = VOLATILE_REGISTERS
        .into_iter()
        .filter(|reg| register_handler.is_used(*reg) && dst != Some(Address::Register(*reg)))
        .collect::<Vec<_>>();
    let depth = compile_scope.stack_depth;
    let (mut instructions, temporaries) =
        struct_temporaries(args, compile_scope, register_handler)?;
    let temporaries_size = compile_scope.stack_depth - depth;
    let (register_temporaries, stack_temporaries) = temporaries.split_at(register_args.len());

    instructions.extend(saved_registers.iter().map(|reg| Instruction::push(*reg)));

    // Registers already in use were saved above, so only the free ones need to be claimed
    let mut requested_registers: Vec<Register> = vec![];
    for ((arg, temporary), dst_reg) in register_args
        .iter()
        .zip(register_temporaries)
        .zip(target_registers)
    {
        instructions.extend(compile_arg(
            arg,
            *temporary,
            dst_reg.into(),
            compile_scope,
            register_handler,
        )?);
        //  Prevent register from being modified
        if register_handler.request_register(dst_reg).is_ok() {
            requested_registers.push(dst_reg);
        }
    }

    // Win64: 32 bytes of shadow space, followed by any args past the 4th. rsp must be 16 byte
    // aligned at the call, including the locals and saved registers
    let pushed_size = compile_scope.stack_depth + u32::try_from(saved_registers.len()).unwrap() * 8;
    let stack_size: u32 = (pushed_size + 32 + u32::try_from(stack_args.len()).unwrap() * 8)
        .next_multiple_of(16)
        - pushed_size;
    instructions.push(Instruction::sub(Register::RSP, stack_size));

    // Register args are already in place, so evaluate these directly into their slots
    for (i, (arg, temporary)) in stack_args.iter().zip(stack_temporaries).enumerate() {
        let offset = 32 + i32::try_from(i).unwrap() * 8;
        if arg.contains_call() {
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

    instructions.push(Instruction::call(name));
    instructions.push(Instruction::add(Register::RSP, stack_size));
    if let Some(dst) = dst
        && dst != Address::Register(Register::RAX)
    {
        instructions.push(Instruction::mov(dst, Register::RAX));
    }

    // Give back the registers
    for reg in requested_registers {
        register_handler.release_register(reg);
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

/// Structs are passed as a pointer and the callee copies them into its own frame. Places already
/// have an address to point at, but anything else (a struct literal) is built into stack space
/// here first. This happens before the saved registers are pushed so `stack_depth` covers it when
/// aligning the call, and the caller frees it once the call returns
fn struct_temporaries(
    args: &[Expression],
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<(Vec<Instruction>, Vec<Option<Address>>), CodegenError> {
    let mut instructions = vec![];
    let mut temporaries = vec![];
    for arg in args {
        let arg_type = compile_scope.expression_type(arg).at(arg.span)?;
        if !matches!(arg_type, Type::Struct(_)) || arg.is_place() {
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

    if !matches!(
        compile_scope.expression_type(arg).at(arg.span)?,
        Type::Struct(_)
    ) {
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

fn compile_struct_instantiation(
    struct_type_name: &str,
    fields: &[(String, Expression)],
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let struct_def = compile_scope
        .lookup_struct(struct_type_name)
        .unwrap()
        .clone();

    let Address::Stack(stack_offset_start) = dst else {
        panic!("Cannot put a struct into a register")
    };

    let mut instructions = vec![];
    for (field_name, expr) in fields {
        let field_type = struct_def.get_field_type(field_name).unwrap().clone();
        let address = struct_def.field_address(stack_offset_start, field_name);

        instructions.extend(register_handler.lease_with_scope(|register_handler, reg| {
            let mut field = expr.compile(compile_scope, reg, register_handler)?;
            field.push(store_element(&field_type, address, reg));
            Ok(field)
        })?);
    }

    Ok(instructions)
}

type WithAddress<'f> = dyn FnMut(
        &mut CompileScope,
        &mut RegisterHandler,
        Address,
        Type,
    ) -> Result<Vec<Instruction>, CodegenError>
    + 'f;

pub fn with_place_address(
    place: &Expression,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    f: &mut WithAddress,
) -> Result<Vec<Instruction>, CodegenError> {
    match &place.kind {
        ExpressionKind::Atom(AtomType::Identifier(name)) => {
            let (address, t) = compile_scope.lookup(name).at(place.span)?.clone();
            f(compile_scope, register_handler, address, t)
        }
        ExpressionKind::Index(lhs, index) => {
            let element = compile_scope.element_type(lhs).at(lhs.span)?;
            register_handler.lease_with_scope(|register_handler, offset| {
                let mut instructions = index.compile(compile_scope, offset, register_handler)?;
                instructions.extend(register_handler.lease_with_scope(
                    |register_handler, base| {
                        let mut instructions =
                            lhs.compile(compile_scope, base, register_handler)?;
                        let address = Address::Indexed {
                            base,
                            index: offset,
                            size: u32::try_from(element.size()).unwrap(),
                        };
                        instructions.extend(f(compile_scope, register_handler, address, element)?);
                        Ok(instructions)
                    },
                )?);
                Ok(instructions)
            })
        }
        ExpressionKind::Unary(UnaryOp::Deref, pointer) => {
            let element = compile_scope.element_type(pointer).at(pointer.span)?;
            register_handler.lease_with_scope(|register_handler, base| {
                let mut instructions = pointer.compile(compile_scope, base, register_handler)?;
                let address = Address::Deref { base, offset: 0 };
                instructions.extend(f(compile_scope, register_handler, address, element)?);
                Ok(instructions)
            })
        }
        ExpressionKind::StructAccess { base, field_name } => with_place_address(
            base,
            compile_scope,
            register_handler,
            &mut |compile_scope, register_handler, address, struct_type| {
                let Type::Struct(struct_def) = struct_type else {
                    panic!("({base}) is not a struct")
                };
                let field_type = struct_def.get_field_type(field_name).unwrap().clone();
                let field_offset = u32::try_from(struct_def.get_byte_offset(field_name)).unwrap();
                match address {
                    Address::Stack(struct_address) => f(
                        compile_scope,
                        register_handler,
                        struct_def.field_address(struct_address, field_name),
                        field_type,
                    ),
                    Address::Deref { base, offset } => f(
                        compile_scope,
                        register_handler,
                        Address::Deref {
                            base,
                            offset: offset + field_offset,
                        },
                        field_type,
                    ),
                    _ => register_handler.lease_with_scope(|register_handler, base| {
                        let mut instructions = vec![Instruction::lea(base, address)];
                        let address = Address::Deref {
                            base,
                            offset: field_offset,
                        };
                        instructions.extend(f(
                            compile_scope,
                            register_handler,
                            address,
                            field_type,
                        )?);
                        Ok(instructions)
                    }),
                }
            },
        ),
        _ => unreachable!("({place}) is not a place"),
    }
}

fn narrow_size(element: &Type) -> Option<(Size, bool)> {
    match element {
        Type::Int(int) if int.size() == 4 => Some((Size::Dword, int.is_signed())),
        Type::Int(int) if int.size() == 1 => Some((Size::Byte, int.is_signed())),
        Type::Boolean | Type::Char => Some((Size::Byte, false)),
        _ => None,
    }
}

fn extend(element: &Type, dst: Register, src: impl Into<Address>) -> Option<Instruction> {
    narrow_size(element).map(|(size, signed)| {
        if signed {
            Instruction::movsx(dst, src, size)
        } else {
            Instruction::movzx(dst, src, size)
        }
    })
}

/// Loading from memory
fn load_element(element: &Type, dst: Register, memory: Address) -> Instruction {
    extend(element, dst, memory).unwrap_or_else(|| Instruction::mov(dst, memory))
}

/// Storing to memory
pub fn store_element(element: &Type, memory: Address, src: Register) -> Instruction {
    narrow_size(element).map_or_else(
        || Instruction::mov(memory, src),
        |(size, _)| Instruction::store(memory, src, size),
    )
}

// This will have to take other types eventually
pub fn mem_copy(
    element: &StructDefinition,
    dst: Address,
    src: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    register_handler.lease_with_scope(|_, reg| {
        let mut instructions = vec![];
        for field in &element.fields {
            let offset = u32::try_from(element.get_byte_offset(&field.name)).unwrap();
            instructions.push(load_element(&field.ftype, reg, offset_address(src, offset)));
            instructions.push(store_element(
                &field.ftype,
                offset_address(dst, offset),
                reg,
            ));
        }

        Ok::<_, CodegenError>(instructions)
    })
}

fn offset_address(address: Address, offset: u32) -> Address {
    match address {
        Address::Stack(start) => Address::Stack(start - i32::try_from(offset).unwrap()),
        Address::Deref {
            base,
            offset: start,
        } => Address::Deref {
            base,
            offset: start + offset,
        },
        _ => panic!("Cannot offset ({address})"),
    }
}

fn compile_number(
    n: NumberType,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    if fits_imm32(n) {
        Ok(vec![Instruction::mov(dst, n)])
    } else {
        move_into(dst, register_handler, |reg| Instruction::mov(reg, n))
    }
}

fn compile_char(c: char, dst: Address) -> Instruction {
    Instruction::mov(dst, c)
}

fn compile_variable(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    span: Span,
    dst: Address,
) -> Result<Vec<Instruction>, CodegenError> {
    let address = compile_scope.get_variable(name).at(span)?;
    if dst == address {
        return Ok(vec![]);
    }

    if let Type::Struct(struct_def) = compile_scope.get_type(name).at(span)? {
        return mem_copy(struct_def, dst, address, register_handler);
    }

    match address {
        Address::Register(_) => Ok(vec![Instruction::mov(dst, address)]),
        _ => move_into(dst, register_handler, |reg| Instruction::mov(reg, address)),
    }
}

fn compile_binary(
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
        BinaryOp::Or => {
            return compile_simple_binary(
                Instruction::or,
                compile_scope,
                left,
                right,
                dst,
                register_handler,
            );
        }
        BinaryOp::And => {
            return compile_simple_binary(
                Instruction::and,
                compile_scope,
                left,
                right,
                dst,
                register_handler,
            );
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

fn compile_cast(
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

fn compile_unary(
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

fn compile_postfix(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    // Postfix evaluates to the value before the update
    let mut instructions = lhs.compile(compile_scope, dst, register_handler)?;
    instructions.extend(compile_postfix_update(
        op,
        lhs,
        compile_scope,
        register_handler,
    )?);

    Ok(instructions)
}

fn compile_postfix_update(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    Assignment {
        lhs: lhs.clone(),
        expression: Expression::new(
            ExpressionKind::Binary(
                op.arithmetic(),
                Box::new(lhs.clone()),
                Box::new(Expression::new(
                    ExpressionKind::Atom(AtomType::Number(NumberType::Coercible(1))),
                    lhs.span,
                )),
            ),
            lhs.span,
        ),
    }
    .compile_uncommented(compile_scope, register_handler)
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
        compile_scope.unified_type(left, right).at(left.span)?,
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

fn compile_boolean(b: bool, dst: Address) -> Instruction {
    let n = NumberType::I64(i64::from(b));
    Instruction::mov(dst, n)
}

fn as_operand(expr: &Expression, scope: &CompileScope) -> Option<Operand> {
    match &expr.kind {
        ExpressionKind::Atom(AtomType::Number(n)) if fits_imm32(*n) => Some(Operand::Immediate(*n)),
        ExpressionKind::Atom(AtomType::Char(c)) => Some(Operand::from(*c)),
        ExpressionKind::Atom(AtomType::Identifier(name)) => {
            scope.get_variable(name).ok().map(Operand::from)
        }
        _ => None,
    }
}

fn fits_imm32(n: NumberType) -> bool {
    match n {
        NumberType::U8(_) | NumberType::I32(_) => true,
        NumberType::U32(n) => i32::try_from(n).is_ok(),
        NumberType::U64(n) => i32::try_from(n).is_ok(),
        NumberType::I64(n) | NumberType::Coercible(n) => i32::try_from(n).is_ok(),
    }
}

#[cfg(test)]
mod test {
    use super::NON_VOLATILE_REGISTERS;
    use crate::{lex::Lexer, parse::parse, type_check::check_types};

    fn compile(input: &str) -> String {
        let mut ast = parse(Lexer::new(input).unwrap()).unwrap();
        check_types(&mut ast).unwrap();
        ast.compile().unwrap()
    }

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
        assert!(asm.contains(", [rbp + 16]"), "{asm}");
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
        assert!(asm.contains("mov [rbp + 16], rcx"), "{asm}");
        assert!(asm.contains("mov [rbp + 40], r9"), "{asm}");
        assert!(asm.contains("[rbp + 48]"), "{asm}");
    }

    #[test]
    fn compile_function_with_locals() {
        let asm = compile("def f(a: i32): i32 { let b = 1; a + b } f(1);");
        assert!(asm.contains("mov [rbp + 16], rcx"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 48], 1"), "{asm}");
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

    #[test]
    fn compile_boolean_literals() {
        let asm = compile("let a = true; let b = false;");
        assert!(asm.contains("mov QWORD [rbp - 8], 1"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
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
        assert!(asm.contains(", [rbp - 16]\n\tmov [rbp - 24], "), "{asm}");
        assert!(asm.contains("\tand r"), "{asm}");
        assert!(asm.contains(", [rbp - 16]\n\tmov [rbp - 32], "), "{asm}");
        assert!(asm.contains("\tor r"), "{asm}");
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
    fn compile_discarded_postfix_skips_old_value() {
        let asm = compile("let i = 0; i++;");
        assert!(!asm.contains(", [rbp - 8]"), "{asm}");
    }

    #[test]
    fn compile_discarded_call_keeps_result_in_rax() {
        let asm = compile("def five(): i32 { 5 } five();");
        assert!(!asm.contains(", rax"), "{asm}");
    }

    #[test]
    fn compile_discarded_division_still_divides() {
        let asm = compile("let a = 7; a / 0;");
        assert!(asm.lines().any(|l| l.trim().starts_with("idiv ")), "{asm}");
    }

    #[test]
    fn compile_boolean_is_not_a_variable_lookup() {
        compile("if (true) { printf(\"yes\"); }");
        compile("let a = !false;");
    }

    #[test]
    fn compile_char_literals() {
        let asm = compile("let a: char = 'a'; let b: char = 'Z'; let c: char = '0';");
        assert!(asm.contains("mov QWORD [rbp - 8], 97"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 90"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 48"), "{asm}");
        assert!(!asm.contains("db "), "{asm}");
    }

    #[test]
    fn compile_char_escapes() {
        let asm = compile(r"let a: char = '\n'; let b: char = '\0'; let c: char = '\'';");
        assert!(asm.contains("mov QWORD [rbp - 8], 10"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 39"), "{asm}");
    }

    #[test]
    fn compile_char_arithmetic() {
        compile("let c: char = '7'; let d = c - '0';");
        compile("let c: char = 'a'; c += 1;");
        compile("let c: char = 'a'; c++;");
    }

    #[test]
    fn compile_char_comparison() {
        compile("let c: char = 'a'; if (c == 'a') { printf(\"yes\"); }");
        compile("let c: char = 'a'; let lower = c >= 'a' && c <= 'z';");
    }

    #[test]
    fn compile_char_as_function_arg() {
        let asm = compile("putchar('h');");
        assert!(asm.contains("104"), "{asm}");
        compile("printf(\"%c%c\", 'h', 'i');");
    }

    #[test]
    fn compile_char_function_param_and_return() {
        compile("def next(c: char): char { c + 1 } next('a');");
    }

    #[test]
    fn compile_large_literal_to_stack_goes_through_register() {
        let asm = compile("let big: u32 = 4000000000;");
        assert!(!asm.contains("], 4000000000"), "{asm}");
        assert!(asm.contains(", 4000000000"), "{asm}");
    }

    #[test]
    fn compile_small_literal_to_stack_is_immediate() {
        let asm = compile("let small: u32 = 5;");
        assert!(asm.contains("], 5"), "{asm}");
    }

    #[test]
    fn compile_store_char_element_is_byte() {
        let asm = compile("let s: *char = malloc(4); s[0] = 'h';");
        assert!(asm.contains("BYTE ["), "{asm}");
    }

    #[test]
    fn compile_load_char_element_is_byte() {
        let asm = compile("let s: *char = malloc(4); let c: char = s[0];");
        assert!(asm.contains("movzx"), "{asm}");
        assert!(asm.contains("BYTE ["), "{asm}");
    }

    /// Lines that access memory through a register other than rbp/rsp
    fn derefs(asm: &str) -> Vec<&str> {
        asm.lines()
            .map(str::trim)
            .filter(|l| {
                l.split_once('[')
                    .is_some_and(|(_, rest)| !rest.starts_with("rbp") && !rest.starts_with("rsp"))
            })
            .collect()
    }

    fn leas(asm: &str) -> Vec<&str> {
        asm.lines()
            .map(str::trim)
            .filter(|l| l.starts_with("lea "))
            .collect()
    }

    #[test]
    fn compile_address_of_local_uses_lea() {
        let asm = compile("let x = 5; let p = &x;");
        assert!(leas(&asm).iter().any(|l| l.ends_with("[rbp - 8]")), "{asm}");
    }

    #[test]
    fn compile_address_of_struct_uses_lea() {
        let asm = compile("struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let p = &s;");
        assert!(!leas(&asm).is_empty(), "{asm}");
    }

    #[test]
    fn compile_address_of_field_uses_field_offset() {
        let asm = compile(
            "struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let pa = &s.a; let pb = &s.b;",
        );
        let leas = leas(&asm);
        assert_eq!(leas.len(), 2, "{asm}");
        assert_ne!(
            leas[0].split_once(',').unwrap().1,
            leas[1].split_once(',').unwrap().1,
            "{asm}"
        );
    }

    #[test]
    fn compile_address_of_arg_uses_lea() {
        let asm = compile("def f(x: i64): *i64 { return &x; }");
        assert!(leas(&asm).iter().any(|l| l.contains("[rbp + ")), "{asm}");
    }

    #[test]
    fn compile_deref_load() {
        let asm = compile("let x = 5; let p = &x; let y = *p;");
        assert!(derefs(&asm).iter().any(|l| l.starts_with("mov ")), "{asm}");
    }

    #[test]
    fn compile_deref_load_u8_is_byte() {
        let asm = compile("let x: u8 = 5; let p = &x; let y: u8 = *p;");
        assert!(
            derefs(&asm)
                .iter()
                .any(|l| l.starts_with("movzx") && l.contains("BYTE [")),
            "{asm}"
        );
    }

    #[test]
    fn compile_deref_load_u32_is_dword() {
        let asm = compile("let x: u32 = 5; let p = &x; let y: u32 = *p;");
        assert!(derefs(&asm).iter().any(|l| l.contains("DWORD [")), "{asm}");
    }

    #[test]
    fn compile_deref_store() {
        let asm = compile("let x = 5; let p = &x; *p = 7;");
        assert!(derefs(&asm).iter().any(|l| l.starts_with("mov [")), "{asm}");
    }

    #[test]
    fn compile_deref_store_u8_is_byte() {
        let asm = compile("let x: u8 = 5; let p = &x; *p = 7;");
        let derefs = derefs(&asm);
        assert!(derefs.iter().any(|l| l.contains("BYTE [")), "{asm}");
        assert!(!derefs.iter().any(|l| l.contains("QWORD [")), "{asm}");
    }

    #[test]
    fn compile_double_deref_loads_twice() {
        let asm = compile("let x = 5; let p = &x; let pp = &p; let y = **pp;");
        assert!(derefs(&asm).len() >= 2, "{asm}");
    }

    #[test]
    fn compile_field_through_pointer_uses_offset() {
        let asm = compile(
            "struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let p = &s; let v = (*p).b;",
        );
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 8]")), "{asm}");
    }

    #[test]
    fn compile_nested_field_through_pointer_adds_offsets() {
        let asm = compile(
            "struct P { a: i64, b: i64, } struct Q { x: i64, p: P, } let r: *Q = malloc(24); let v = (*r).p.b;",
        );
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 16]")), "{asm}");
    }

    #[test]
    fn compile_field_store_through_pointer_uses_field_size() {
        let asm = compile(
            "struct P { a: i64, b: u8, } let s = P { a: 1, b: 2 }; let p = &s; (*p).b = 3;",
        );
        assert!(
            derefs(&asm)
                .iter()
                .any(|l| l.contains("BYTE [") && l.contains(" + 8]")),
            "{asm}"
        );
    }

    #[test]
    fn compile_passing_address_as_arg() {
        let asm = compile("def f(p: *i64): i64 { return *p; } let x = 5; f(&x);");
        assert!(!leas(&asm).is_empty(), "{asm}");
        assert!(!derefs(&asm).is_empty(), "{asm}");
    }
}
