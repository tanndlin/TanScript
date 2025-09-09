use std::fmt;

use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, Expression, OperatorType, Program, Statement,
        StatementOrExpression,
    },
    compile_scope::CompileScope,
};

pub struct RegisterHandler {
    data: Vec<String>,
    registers: [Register; 12],
    used: [bool; 12],
}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {
            data: vec![],
            registers: [
                Register::RAX,
                Register::RBX,
                Register::RCX,
                Register::RDX,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R11,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ],
            used: [false; 12],
        }
    }

    pub fn lease_register(&mut self) -> Result<Register, String> {
        for i in 0..12 {
            if !self.used[i] {
                self.used[i] = true;
                return Ok(self.registers[i].clone());
            }
        }

        Err("No registers avaible".to_string())
    }

    pub fn release_register(&mut self, register: Register) {
        for i in 0..12 {
            if register == self.registers[i] {
                self.used[i] = false;
            }
        }
    }

    fn add_data(&mut self, s: &str) -> String {
        let name = format!("data_{}", self.data.len());
        let format = format_data(&name, s);
        self.data.push(format);

        name
    }

    fn request_register(&mut self, requested: &Register) -> Result<Register, String> {
        for i in 0..12 {
            if self.registers[i] == *requested {
                return match self.used[i] {
                    true => Err(format!("Register {} already in use", requested)),
                    false => Ok(requested.clone()),
                };
            }
        }

        Err(format!("Register {} is not leasable", requested))
    }
}

fn format_data(name: &String, s: &str) -> String {
    let mut pieces: Vec<&str> = s.split("\\n").collect();
    if s.ends_with("\\n") {
        pieces.pop();
    }

    let mut newlines = pieces.join("\", 10, \"");
    if s.ends_with("\\n") {
        newlines.push_str("\", 10");
    } else {
        newlines.push('"');
    }

    format!("{} db \"{}, 0", name, newlines)
}

#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Clone)]
pub enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::RAX => write!(f, "rax"),
            Register::RBX => write!(f, "rbx"),
            Register::RCX => write!(f, "rcx"),
            Register::RDX => write!(f, "rdx"),
            Register::R8 => write!(f, "r8"),
            Register::R9 => write!(f, "r9"),
            Register::R10 => write!(f, "r10"),
            Register::R11 => write!(f, "r11"),
            Register::R12 => write!(f, "r12"),
            Register::R13 => write!(f, "r13"),
            Register::R14 => write!(f, "r14"),
            Register::R15 => write!(f, "r15"),
        }
    }
}

#[derive(Clone)]
pub enum Address {
    Register(Register),
    Stack(i32), // Offset in the stack
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Register(reg) => write!(f, "{}", reg),
            Address::Stack(off) => write!(f, "[rbp - {}]", off),
        }
    }
}

impl Program {
    pub fn compile(&self) -> Result<String, String> {
        let mut global_scope = CompileScope::new();
        let mut register_handler = RegisterHandler::new();
        let instructions = self
            .block
            .compile(&mut global_scope, &mut register_handler)?;

        let data = register_handler
            .data
            .iter()
            .map(|d| format!("\t{}", d))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(format!(
            "BITS 64

global main
extern printf
extern ExitProcess

SECTION .data
{}
SECTION .text

main:
\tsub rsp, 40
\tpush rbp
\tmov rbp, rsp
{}
\tadd rsp, 40
\tpop rbp
\txor rcx, rcx
\tcall ExitProcess",
            data, instructions
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
        let mut instructions = self
            .children
            .iter()
            .map(|child| child.compile(compile_scope, register_handler))
            .collect::<Result<Vec<String>, String>>()?
            .join("\n");

        if compile_scope.num_variables > 0 {
            let alloc_size = compile_scope.num_variables * 8;
            let alloc = format!("sub rsp, {}", alloc_size);
            let dealloc = format!("add rsp, {}", alloc_size);

            instructions = format!("{}\n{}\n{}", alloc, instructions, dealloc);
        }

        Ok(instructions
            .split("\n")
            .map(|s| format!("\t{}", s))
            .collect::<Vec<String>>()
            .join("\n"))
    }
}

impl StatementOrExpression {
    fn compile(
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
                &Address::Register(Register::R15),
                register_handler,
            ),
        }
    }
}

impl Statement {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match &self {
            Statement::Declaration(declaration) => {
                declaration.compile(compile_scope, register_handler)
            }
            Statement::Assign(assignment) => assignment.compile(compile_scope, register_handler),
        }
    }
}

impl Declaration {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        compile_scope.add_variable(self.assign.identifier.clone())?;
        self.assign.compile(compile_scope, register_handler)
    }
}

impl Assignment {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let address = compile_scope.get_variable(&self.identifier)?.clone();
        self.expression
            .compile(compile_scope, &address, register_handler)
    }
}

impl Expression {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        dst: &Address,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match self {
            Expression::Atom(a) => match a {
                AtomType::Number(n) => Ok(compile_number(n, dst)),
                AtomType::Identifier(name) => compile_variable(compile_scope, name, dst),
                AtomType::String(s) => Ok(compile_string(s, dst, register_handler)),
                AtomType::FunctionCall(name, args) => {
                    compile_function_call(compile_scope, register_handler, name, args)
                }
            },
            Expression::Operation(v, children) => {
                compile_operator(v, children, compile_scope, dst, register_handler)
            }
        }
    }
}

fn compile_string(s: &str, dst: &Address, register_handler: &mut RegisterHandler) -> String {
    let handle = register_handler.add_data(s);
    format!("mov {}, {}", dst, handle)
}

fn compile_function_call(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    args: &[Expression],
) -> Result<String, String> {
    let function_def = compile_scope.get_function(name)?;
    if let Some(num_args) = function_def.num_args
        && num_args as usize != args.len()
    {
        return Err(format!(
            "Function '{}' expects {} arguments, got {}",
            name,
            num_args,
            args.len()
        ));
    }

    if args.len() > 4 {
        todo!("More than 4 args not supported")
    }

    let mut instructions = vec![];

    let target_registers = vec![Register::RCX, Register::RDX, Register::R8, Register::R9];
    let zipped = args.iter().zip(target_registers);
    for (arg, reg) in zipped {
        instructions.push(arg.compile(compile_scope, &Address::Register(reg), register_handler)?)
    }

    instructions.push("sub rsp, 32".to_string());
    instructions.push(format!("call {}", name));
    instructions.push("add rsp, 32".to_string());

    Ok(instructions.join("\n"))
}

fn compile_number(n: &i32, dst: &Address) -> String {
    format!("mov QWORD {}, {}", dst, n)
}

fn compile_variable(
    compile_scope: &CompileScope,
    name: &str,
    dst: &Address,
) -> Result<String, String> {
    let address = compile_scope.get_variable(name)?;
    Ok(format!("mov r8, {}\nmov {}, r8", address, dst))
}

fn compile_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &mut CompileScope,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    match op {
        OperatorType::Add
        | OperatorType::Subtract
        | OperatorType::Multiply
        | OperatorType::Divide => {
            compile_infix_operator(op, children, compile_scope, dst, register_handler)
        }
    }
}

fn compile_infix_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &mut CompileScope,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let left = children
        .first()
        .ok_or("Infix operator missing left operand")?;
    let right = children
        .get(1)
        .ok_or("Infix operator missing right operand")?;
    match op {
        OperatorType::Divide => compile_divide(compile_scope, left, right, dst, register_handler),
        OperatorType::Multiply => {
            compile_multiply(compile_scope, left, right, dst, register_handler)
        }
        _ => {
            let left = left.compile(compile_scope, dst, register_handler)?;
            let right_reg = register_handler.lease_register()?;
            let right = right.compile(
                compile_scope,
                &Address::Register(right_reg.clone()),
                register_handler,
            )?;

            let perform = match op {
                OperatorType::Add => format!("add {}, {}", dst, right_reg),
                OperatorType::Subtract => format!("sub {}, {}", dst, right_reg),
                OperatorType::Multiply => {
                    return Err("This shouldn't be possible. Use compile_multiply".to_string());
                }
                OperatorType::Divide => {
                    return Err("This shouldn't be possible. Use compile_divide".to_string());
                }
            };

            register_handler.release_register(right_reg);

            Ok(format!("{}\n{}\n{}", left, right, perform))
        }
    }
}

fn compile_multiply(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let left_reg = register_handler.lease_register()?;
    let right_reg = register_handler.lease_register()?;
    let left = left.compile(
        compile_scope,
        &Address::Register(left_reg.clone()),
        register_handler,
    )?;
    let right = right.compile(
        compile_scope,
        &Address::Register(right_reg.clone()),
        register_handler,
    )?;

    let instructions = [
        left,
        right,
        format!("imul {}, {}", left_reg, right_reg),
        format!("mov {}, {}", dst, left_reg),
    ];

    register_handler.release_register(left_reg);
    register_handler.release_register(right_reg);

    Ok(instructions.join("\n"))
}

fn compile_divide(
    compile_scope: &mut CompileScope,
    left: &Expression,
    right: &Expression,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let rax = register_handler.request_register(&Register::RAX)?;
    let rdx = register_handler.request_register(&Register::RDX)?;
    let right_reg = register_handler.lease_register()?;
    let left = left.compile(
        compile_scope,
        &Address::Register(rax.clone()),
        register_handler,
    )?;
    let right = right.compile(
        compile_scope,
        &Address::Register(right_reg.clone()),
        register_handler,
    )?;

    let instructions = [
        left,
        right,
        format!("xor {}, {}", rdx, rdx),
        format!("div {}", right_reg),
        format!("mov {}, {}", dst, rax),
    ];

    register_handler.release_register(rax);

    Ok(instructions.join("\n"))
}
