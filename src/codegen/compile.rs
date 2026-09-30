use crate::{
    ast::{
        Assignment, AtomType, BinaryOp, Block, Expression, PostfixOp, Program, Statement,
        StatementOrExpression::{self},
        UnaryOp,
    },
    codegen::{
        address::{Address, Register},
        compile_scope::CompileScope,
        register_handler::RegisterHandler,
        symbol_table::SymbolTable,
    },
};

impl Program {
    pub fn compile(&self) -> Result<String, String> {
        let mut global_scope = CompileScope::new(None);
        let mut register_handler = RegisterHandler::new();

        let mut symbol_table = SymbolTable::new();
        self.block.discover(&mut symbol_table);
        let instructions = self
            .block
            .compile(&mut global_scope, &mut register_handler)?;

        let functions = symbol_table
            .functions
            .values()
            .map(|def| def.compile(&mut global_scope, &mut register_handler))
            .collect::<Result<Vec<_>, _>>()?
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
extern ExitProcess
{extern_functions}

SECTION .data
{data}

SECTION .text

{functions}

main:
\tsub rsp, 32
\tpush rbp
\tmov rbp, rsp
{instructions}
\tadd rsp, 32
\tpop rbp
\txor rcx, rcx
\tcall ExitProcess"
        )
        .to_string())
    }
}

impl Block {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let mut new_scope = CompileScope::new(Some(compile_scope));

        let mut instructions = self
            .children
            .iter()
            .map(|child| child.compile(&mut new_scope, register_handler))
            .collect::<Result<Vec<String>, String>>()?
            .join("\n");

        if new_scope.num_variables > 0 {
            let alloc_size = new_scope.num_variables * 8;
            // Align stack
            let alloc_size = if alloc_size % 16 != 0 {
                alloc_size + (16 - alloc_size % 16)
            } else {
                alloc_size
            };

            let alloc = format!("sub rsp, {alloc_size}");
            let dealloc = format!("add rsp, {alloc_size}");

            instructions = format!("{alloc}\n{instructions}\n{dealloc}");
        }

        Ok(instructions
            .split('\n')
            .map(|s| format!("\t{s}"))
            .collect::<Vec<String>>()
            .join("\n"))
    }
}

impl StatementOrExpression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match &self {
            StatementOrExpression::Statement(statement) => {
                statement.compile(compile_scope, register_handler)
            }
            StatementOrExpression::Expression(expression) => expression.compile(
                compile_scope,
                Address::Register(Register::R15),
                register_handler,
            ),
        }
    }
}

impl Expression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        dst: impl Into<Address>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let dst = dst.into();

        match self {
            Expression::Atom(a) => match a {
                AtomType::Number(n) => Ok(compile_number(*n, dst)),
                AtomType::Identifier(name) => {
                    compile_variable(compile_scope, register_handler, name, dst)
                }
                AtomType::String(s) => Ok(compile_string(s, dst, register_handler)),
            },
            Expression::Binary(op, left, right) => {
                compile_binary(*op, left, right, compile_scope, dst, register_handler)
            }
            Expression::Unary(op, child) => {
                compile_unary(*op, child, compile_scope, dst, register_handler)
            }
            Expression::Postfix(op, lhs) => {
                compile_postfix(*op, lhs, compile_scope, dst, register_handler)
            }
            Expression::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, dst)
            }
            Expression::Index(lhs, index) => {
                compile_index(compile_scope, register_handler, dst, lhs, index)
            }
        }
    }
}

fn compile_string(s: &str, dst: Address, register_handler: &mut RegisterHandler) -> String {
    let handle = register_handler.add_data(s);
    match dst {
        Address::Register(_) => format!("mov QWORD {dst}, {handle}"),
        Address::Stack(_) => register_handler
            .lease_with_scope(|_, reg| Ok(format!("mov {reg}, {handle}\nmov {dst}, {reg}")))
            .unwrap(),
    }
}

fn compile_function_call(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    args: &[Expression],
    dst: Address,
) -> Result<String, String> {
    if args.len() > 4 {
        todo!("More than 4 args not supported")
    }

    let mut instructions = vec![];

    let target_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
    let mut moved_registers = vec![];
    let zipped = args.iter().zip(target_registers);
    for (arg, dst_reg) in zipped {
        instructions.push(arg.compile(
            compile_scope,
            Address::Register(dst_reg),
            register_handler,
        )?);
        //  Prevent register from being modified
        if register_handler.request_register(dst_reg).is_err() {
            // if the destination is leased, move temporarily
            instructions.insert(instructions.len() - 1, format!("push {dst_reg}"));
            moved_registers.push(dst_reg);
        }
    }

    instructions.push(
        register_handler.request_with_scope(Register::RAX, dst, |_, rax| {
            Ok([
                "sub rsp, 32".to_string(),
                format!("call {name}"),
                "add rsp, 32".to_string(),
                format!("mov {dst}, {rax}"),
            ]
            .join("\n"))
        })?,
    );

    // Give back the registers
    target_registers
        .iter()
        .take(args.len())
        .filter(|reg| !moved_registers.contains(reg))
        .for_each(|r| register_handler.release_register(*r));

    let undo = moved_registers
        .iter()
        .rev()
        .map(|reg| format!("pop {reg}"))
        .collect::<Vec<_>>();

    instructions.extend(undo);

    Ok(instructions.join("\n"))
}

fn compile_index(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    dst: Address,
    lhs: &Expression,
    index: &Expression,
) -> Result<String, String> {
    register_handler.lease_with_scope(|register_handler, ptr| {
        let data_size = 8; // TODO: This will have to change at some point. Assuming 8 bytes
        let index = index.compile(compile_scope, Address::Register(ptr), register_handler)?;
        register_handler.lease_with_scope(|register_handler, lhs_dst| {
            let lhs = lhs.compile(compile_scope, Address::Register(lhs_dst), register_handler)?;

            let load = match dst {
                Address::Register(_) => format!("mov {dst}, [{lhs_dst}+{ptr}]"),
                Address::Stack(_) => {
                    format!("mov {lhs_dst}, [{lhs_dst}+{ptr}]\nmov {dst}, {lhs_dst}")
                }
            };

            Ok(format!(
                "{index}\n\
                imul {ptr}, {ptr}, {data_size}\n\
                {lhs}
                {load}\n"
            ))
        })
    })
}

fn compile_number(n: i32, dst: Address) -> String {
    format!("mov QWORD {dst}, {n}")
}

fn compile_variable(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    dst: Address,
) -> Result<String, String> {
    let address = compile_scope.get_variable(name)?;
    match dst {
        //  You can always move to a register
        Address::Register(_) => Ok(format!("mov {dst}, {address}")),
        Address::Stack(_) => match address {
            Address::Register(_) => Ok(format!("mov {dst}, {address}")),

            // Need an intermediate register
            Address::Stack(_) => register_handler
                .lease_with_scope(|_, reg| Ok(format!("mov {reg}, {address}\nmov {dst}, {reg}"))),
        },
    }
}

fn compile_binary(
    op: BinaryOp,
    left: &Expression,
    right: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let set_condition = match op {
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
            return compile_simple_binary("add", compile_scope, left, right, dst, register_handler);
        }
        BinaryOp::Subtract => {
            return compile_simple_binary("sub", compile_scope, left, right, dst, register_handler);
        }
        BinaryOp::Or => {
            return compile_simple_binary("or", compile_scope, left, right, dst, register_handler);
        }
        BinaryOp::And => {
            return compile_simple_binary("and", compile_scope, left, right, dst, register_handler);
        }
        BinaryOp::LessThan => "setl",
        BinaryOp::LessOrEqual => "setle",
        BinaryOp::GreaterThan => "setg",
        BinaryOp::GreaterOrEqual => "setge",
        BinaryOp::Equal => "sete",
        BinaryOp::NotEqual => "setne",
    };

    let compare = compile_simple_binary("cmp", compile_scope, left, right, dst, register_handler)?;
    Ok(format!(
        "{compare}\nmov QWORD {dst}, 0\n{set_condition} {}",
        dst.lower_8_bits()
    ))
}

/// Compiles `left` into `dst`, `right` into a scratch register, then runs `{instruction} dst, scratch`
fn compile_simple_binary(
    instruction: &str,
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let left = left.compile(compile_scope, dst, register_handler)?;
    let right = register_handler.lease_with_scope(|register_handler, right_reg| {
        let right = right.compile(
            compile_scope,
            Address::Register(right_reg),
            register_handler,
        )?;
        Ok(format!("{right}\n{instruction} {dst}, {right_reg}"))
    })?;

    Ok(format!("{left}\n{right}"))
}

fn compile_unary(
    op: UnaryOp,
    child: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let child_asm = child.compile(compile_scope, dst, register_handler)?;

    let asm = match op {
        UnaryOp::Negate => format!("neg QWORD {dst}"),
        UnaryOp::Not => format!("xor QWORD {dst}, 1"),
    };

    Ok(format!("{child_asm}\n{asm}"))
}

fn compile_postfix(
    op: PostfixOp,
    lhs: &Expression,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    // Postfix evaluates to the value before the update
    let old_value = lhs.compile(compile_scope, dst, register_handler)?;
    let update = Assignment {
        lhs: lhs.clone(),
        expression: Expression::Binary(
            op.arithmetic(),
            Box::new(lhs.clone()),
            Box::new(Expression::Atom(AtomType::Number(1))),
        ),
    }
    .compile(compile_scope, register_handler)?;

    Ok(format!("{old_value}\n{update}"))
}

fn compile_multiply(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    register_handler.lease_with_scope(|register_handler, left_reg| {
        let left = left.compile(compile_scope, Address::Register(left_reg), register_handler)?;

        register_handler.lease_with_scope(|register_handler, right_reg| {
            let right = right.compile(
                compile_scope,
                Address::Register(right_reg),
                register_handler,
            )?;

            Ok([
                left,
                right,
                format!("imul {left_reg}, {right_reg}"),
                format!("mov {dst}, {left_reg}"),
            ]
            .join("\n"))
        })
    })
}

/// `idiv` leaves the quotient in rax and the remainder in rdx, so `result` picks between / and %
fn compile_divide(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: Address,
    register_handler: &mut RegisterHandler,
    result: Register,
) -> Result<String, String> {
    register_handler.lease_with_scope(|register_handler, right_reg| {
        // Only happens when every other register is leased, and idiv overwrites both
        if matches!(right_reg, Register::RAX | Register::RDX) {
            return Err("Ran out of registers compiling division".to_string());
        }

        let right = right.compile(
            compile_scope,
            Address::Register(right_reg),
            register_handler,
        )?;

        let divide =
            register_handler.request_with_scope(Register::RAX, dst, |register_handler, rax| {
                let left = left.compile(compile_scope, Address::Register(rax), register_handler)?;

                let divide = register_handler.request_with_scope(Register::RDX, dst, |_, _| {
                    Ok([
                        "cqo".to_string(),
                        format!("idiv {right_reg}"),
                        format!("mov {dst}, {result}"),
                    ]
                    .join("\n"))
                })?;

                Ok(format!("{left}\n{divide}"))
            })?;

        // right was compiled first so it has to run first, or its own rax use clobbers left
        Ok(format!("{right}\n{divide}"))
    })
}

#[cfg(test)]
mod test {
    use crate::parser::parse;

    fn compile(input: &str) -> String {
        parse(input).unwrap().compile().unwrap()
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
        let asm = compile("def five() { 5 } let a = five() / 2;");
        assert!(!asm.contains("pop rax"), "{asm}");
    }
}
