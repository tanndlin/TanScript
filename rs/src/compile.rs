use core::panic;
use std::fmt;

use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, Expression, OperatorType, Program, Statement,
        StatementOrExpression,
    },
    compile_scope::CompileScope,
};

pub struct RegisterHandler {
    registers: [Register; 12],
    used: [bool; 12],
}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {
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

    pub fn lease_register(&mut self) -> Register {
        for i in 0..12 {
            if !self.used[i] {
                self.used[i] = true;
                return self.registers[i].clone();
            }
        }

        panic!("No registers avaible")
    }

    pub fn release_register(&mut self, register: Register) {
        for i in 0..12 {
            if register == self.registers[i] {
                self.used[i] = false;
            }
        }
    }
}

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
    pub fn compile(&self) -> String {
        let mut global_scope = CompileScope::new();
        let mut register_handler = RegisterHandler::new();
        let instructions = self.block.compile(&mut global_scope, &mut register_handler);

        format!(
            "BITS 64

global main
extern printf
extern ExitProcess

SECTION .data

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
            instructions
        )
        .to_string()
    }
}

impl Block {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> String {
        let mut instructions = self
            .children
            .iter()
            .map(|s| s.compile(compile_scope, register_handler))
            .collect::<Vec<String>>()
            .join("\n");

        if compile_scope.num_variables > 0 {
            let alloc_size = compile_scope.num_variables * 8;
            let alloc = format!("sub rsp, {}", alloc_size);
            let dealloc = format!("add rsp, {}", alloc_size);

            instructions = format!("{}\n{}\n{}", alloc, instructions, dealloc);
        }

        instructions
            .split("\n")
            .map(|s| format!("\t{}", s))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl StatementOrExpression {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> String {
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
    ) -> String {
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
    ) -> String {
        compile_scope.add_variable(self.assign.identifier.clone());
        self.assign.compile(compile_scope, register_handler)
    }
}

impl Assignment {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> String {
        let address = compile_scope.get_variable(&self.identifier).clone();
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
    ) -> String {
        match self {
            Expression::Atom(a) => match a {
                AtomType::Number(n) => compile_number(n, dst),
                AtomType::Identifier(name) => compile_variable(compile_scope, name, dst),
            },
            Expression::Operation(v, children) => {
                compile_operator(v, children, compile_scope, dst, register_handler)
            }
        }
    }
}

fn compile_number(n: &i32, dst: &Address) -> String {
    format!("mov QWORD {}, {}", dst, n)
}

fn compile_variable(compile_scope: &CompileScope, name: &str, dst: &Address) -> String {
    let address = compile_scope.get_variable(name);
    format!("mov r8, {}\nmov {}, r8", address, dst)
}

fn compile_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &mut CompileScope,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> String {
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
) -> String {
    let right_reg = register_handler.lease_register();

    let left = children
        .first()
        .unwrap_or_else(|| panic!("Missing first child for infix operator: {}", op))
        .compile(compile_scope, dst, register_handler);
    let right = children
        .get(1)
        .unwrap_or_else(|| panic!("Missing first child for infix operator: {}", op))
        .compile(
            compile_scope,
            &Address::Register(right_reg.clone()),
            register_handler,
        );

    let perform = match op {
        OperatorType::Add => format!("add {}, {}", dst, right_reg),
        OperatorType::Subtract => format!("sub {}, {}", dst, right_reg),
        OperatorType::Multiply => todo!(),
        OperatorType::Divide => todo!(),
    };

    register_handler.release_register(right_reg);

    format!("{}\n{}\n{}", left, right, perform)
}
