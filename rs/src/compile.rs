use core::panic;
use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, Expression, IfStatement, OperatorType, Program,
        Statement, StatementOrExpression, WhileLoop,
    },
    compile_scope::CompileScope,
    register_handler::RegisterHandler,
    symbol_table::{FunctionDefinition, SymbolTable},
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub enum Address {
    Register(Register),
    Stack(i32), // Offset in the stack
}

impl Address {
    pub fn lower_8_bits(&self) -> String {
        match self {
            Address::Stack(off) => format!("[rbp - {off}]"),
            Address::Register(reg) => match reg {
                Register::RAX => "al".to_string(),
                Register::RBX => "bl".to_string(),
                Register::RCX => "cl".to_string(),
                Register::RDX => "dl".to_string(),
                Register::R8 => "r8b".to_string(),
                Register::R9 => "r9b".to_string(),
                Register::R10 => "r10b".to_string(),
                Register::R11 => "r11b".to_string(),
                Register::R12 => "r12b".to_string(),
                Register::R13 => "r13b".to_string(),
                Register::R14 => "r14b".to_string(),
                Register::R15 => "r15b".to_string(),
            },
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Register(reg) => write!(f, "{reg}"),
            Address::Stack(off) => write!(f, "[rbp - {off}]"),
        }
    }
}

impl Program {
    pub fn compile(&self) -> Result<String, String> {
        let global_scope = Rc::new(RefCell::new(CompileScope::new(None)));
        let mut register_handler = RegisterHandler::new();

        let mut symbol_table = SymbolTable::new();
        self.block.discover(&mut symbol_table);
        let instructions = self.block.compile(&global_scope, &mut register_handler)?;

        let functions = symbol_table
            .functions
            .values()
            .map(|def| def.compile(&global_scope, &mut register_handler))
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
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let new_scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));

        let mut instructions = self
            .children
            .iter()
            .map(|child| match child {
                StatementOrExpression::Statement(Statement::FunctionDefintion(_)) => {
                    Ok(String::new())
                }
                _ => child.compile(&new_scope, register_handler),
            })
            .collect::<Result<Vec<String>, String>>()?
            .join("\n");

        if new_scope.borrow().num_variables > 0 {
            let alloc_size = new_scope.borrow().num_variables * 8;
            // Align stack
            let alloc_size = if alloc_size % 16 != 0 {
                alloc_size + alloc_size % 16
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
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
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
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match &self {
            Statement::Declaration(declaration) => {
                declaration.compile(compile_scope, register_handler)
            }
            Statement::Assign(assignment) => assignment.compile(compile_scope, register_handler),
            Statement::WhileLoop(while_loop) => while_loop.compile(compile_scope, register_handler),
            Statement::IfStatement(if_statement) => {
                if_statement.compile(compile_scope, register_handler)
            }
            Statement::FunctionDefintion(_) => Ok(String::new()),
            Statement::Return(expr) => {
                // TODO: This will wipe whatever is in RAX
                let instructions = expr.compile(
                    compile_scope,
                    &Address::Register(Register::RAX),
                    register_handler,
                )?;
                Ok(format!("{instructions}\nmov rsp, rbp\npop rbp\nret"))
            }
        }
    }
}

impl Declaration {
    pub fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        compile_scope
            .borrow_mut()
            .add_variable(self.assign.identifier.clone(), None)?;
        self.assign.compile(compile_scope, register_handler)
    }
}

impl Assignment {
    pub fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let address = compile_scope
            .borrow()
            .get_variable(&self.identifier)?
            .clone();
        self.expression
            .compile(compile_scope, &address, register_handler)
    }
}

impl WhileLoop {
    pub fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut crate::compile::RegisterHandler,
    ) -> Result<String, String> {
        let unique_id = register_handler.get_unique_id();
        let start_label = format!("while_start_{unique_id}");
        let end_label = format!("while_end_{unique_id}");

        let reg = register_handler.lease_register()?;
        let condition = self.condition.compile(
            compile_scope,
            &Address::Register(reg.clone()),
            register_handler,
        )?;
        register_handler.release_register(reg.clone());

        let block = self.block.compile(compile_scope, register_handler)?;

        Ok(format!(
            "{start_label}:\n\
            {condition}\n\
            cmp {reg}, 0\n\
            je {end_label}\n\
            {block}\n\
            jmp {start_label}\n\
            {end_label}:"
        ))
    }
}

impl IfStatement {
    pub fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let id = register_handler.get_unique_id();
        let dst = register_handler.lease_register()?;
        let condition = self.condition.compile(
            compile_scope,
            &Address::Register(dst.clone()),
            register_handler,
        )?;

        let new_scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));
        let block = self.block.compile(&new_scope, register_handler)?;
        let new_scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));
        let else_block = match &self.else_block {
            None => None,
            Some(else_block) => Some(else_block.compile(&new_scope, register_handler)?),
        };

        Ok(format!(
            "{condition}\ntest {dst}, {dst}\njz else{id}\n{block}\njmp endif{id}\nelse{id}:\n{}endif{id}:",
            else_block.unwrap_or(String::new())
        ))
    }
}

impl FunctionDefinition {
    pub fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let scope = Rc::new(RefCell::new(CompileScope::new(Some(compile_scope))));

        let mut arg_setup = vec![];
        // TODO: Support more than 4 args with the stack
        let arg_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
        for (i, (arg, reg)) in self.args.iter().zip(arg_registers).enumerate() {
            if i >= 4 {
                break;
            }

            // Move the arg to the shadow space
            let new_address =
                Address::Stack((i32::try_from(i).map_err(|_| "Index out of range")? + 1) * 8);
            arg_setup.push(format!("mov {new_address}, {reg}"));
            let mut scope_borrow = scope.borrow_mut();

            // Update its location in the scope
            scope_borrow.add_variable(arg.clone(), Some(new_address))?;
            // This is not a variable that needs to be deallocated later
            scope_borrow.num_variables -= 1;
        }
        let arg_setup = arg_setup.join("\n\t");
        let instructions = self.body.compile(&scope, register_handler)?;
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

impl Expression {
    fn compile(
        &self,
        compile_scope: &Rc<RefCell<CompileScope>>,
        dst: &Address,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match self {
            Expression::Atom(a) => match a {
                AtomType::Number(n) => Ok(compile_number(*n, dst)),
                AtomType::Identifier(name) => {
                    compile_variable(compile_scope, register_handler, name, dst)
                }
                AtomType::String(s) => Ok(compile_string(s, dst, register_handler)),
            },
            Expression::Operation(v, children) => {
                compile_operator(v, children, compile_scope, dst, register_handler)
            }
            Expression::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, dst)
            }
        }
    }
}

fn compile_string(s: &str, dst: &Address, register_handler: &mut RegisterHandler) -> String {
    let handle = register_handler.add_data(s);
    format!("mov {dst}, {handle}")
}

fn compile_function_call(
    compile_scope: &Rc<RefCell<CompileScope>>,
    register_handler: &mut RegisterHandler,
    name: &str,
    args: &[Expression],
    dst: &Address,
) -> Result<String, String> {
    if args.len() > 4 {
        todo!("More than 4 args not supported")
    }

    let mut instructions = vec![];

    let target_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
    let mut moved_registers = vec![];
    let zipped = args.iter().zip(target_registers.clone());
    for (arg, dst_reg) in zipped {
        instructions.push(arg.compile(
            compile_scope,
            &Address::Register(dst_reg.clone()),
            register_handler,
        )?);
        //  Prevent register from being modified
        if register_handler.request_register(&dst_reg).is_err() {
            // if the destination is leased, move temporarily
            instructions.insert(instructions.len() - 1, format!("push {dst_reg}"));
            moved_registers.push(dst_reg);
        }
    }

    instructions.push(register_handler.request_with_scope(&Register::RAX, |rax| {
        Ok([
            "sub rsp, 32".to_string(),
            format!("call {name}"),
            "add rsp, 32".to_string(),
            format!("mov {dst}, {rax}"),
        ]
        .join("\n"))
    })?);

    // Give back the registers
    target_registers
        .iter()
        .take(args.len())
        .filter(|reg| !moved_registers.contains(reg))
        .for_each(|r| register_handler.release_register(r.clone()));

    let undo = moved_registers
        .iter()
        .rev()
        .map(|reg| format!("pop {reg}"))
        .collect::<Vec<_>>();

    instructions.extend(undo);

    Ok(instructions.join("\n"))
}

fn compile_number(n: i32, dst: &Address) -> String {
    format!("mov QWORD {dst}, {n}")
}

fn compile_variable(
    compile_scope: &Rc<RefCell<CompileScope>>,
    register_handler: &mut RegisterHandler,
    name: &str,
    dst: &Address,
) -> Result<String, String> {
    let binding = compile_scope.borrow();
    let address = binding.get_variable(name)?;
    match dst {
        //  You can always move to a register
        Address::Register(_) => Ok(format!("mov {dst}, {address}")),
        Address::Stack(_) => match address {
            Address::Register(_) => Ok(format!("mov {dst}, {address}")),

            // Need an intermediate register
            Address::Stack(_) => register_handler
                .lease_with_scope(|reg| Ok(format!("mov {reg}, {address}\nmov {dst}, {reg}"))),
        },
    }
}

fn compile_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &Rc<RefCell<CompileScope>>,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    match op {
        OperatorType::Add
        | OperatorType::Subtract
        | OperatorType::Multiply
        | OperatorType::Divide
        | OperatorType::Modulo
        | OperatorType::LessThan
        | OperatorType::LessOrEqual
        | OperatorType::GreaterThan
        | OperatorType::GreaterOrEqual
        | OperatorType::Equal
        | OperatorType::NotEqual
        | OperatorType::Or
        | OperatorType::And => {
            compile_infix_operator(op, children, compile_scope, dst, register_handler)
        }
        OperatorType::Not => {
            compile_prefix_operator(op, children, compile_scope, dst, register_handler)
        }
        OperatorType::Assign
        | OperatorType::OpenCurly
        | OperatorType::CloseCurly
        | OperatorType::OpenParen
        | OperatorType::CloseParen
        | OperatorType::Comma => {
            panic!("Unexpected operator{}", op)
        }
    }
}

fn compile_infix_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &Rc<RefCell<CompileScope>>,
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
        OperatorType::Modulo => compile_modulo(compile_scope, left, right, dst, register_handler),
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
                OperatorType::Add => format!("add {dst}, {right_reg}"),
                OperatorType::Subtract => format!("sub {dst}, {right_reg}"),
                OperatorType::Multiply => {
                    return Err("This shouldn't be possible. Use compile_multiply".to_string());
                }
                OperatorType::Divide => {
                    return Err("This shouldn't be possible. Use compile_divide".to_string());
                }
                OperatorType::Modulo => {
                    return Err("This shouldn't be possible. Use compile_modulo".to_string());
                }
                OperatorType::LessThan => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsetl {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::LessOrEqual => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsetle {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::GreaterThan => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsetg {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::GreaterOrEqual => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsetge {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::Equal => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsete {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::NotEqual => {
                    format!(
                        "cmp {dst}, {right_reg}\nmov {dst}, 0\nsetne {}",
                        dst.lower_8_bits()
                    )
                }
                OperatorType::Or => {
                    format!("or {dst}, {right_reg}")
                }
                OperatorType::And => {
                    format!("and {dst}, {right_reg}")
                }
                OperatorType::Assign
                | OperatorType::OpenCurly
                | OperatorType::CloseCurly
                | OperatorType::OpenParen
                | OperatorType::CloseParen
                | OperatorType::Not
                | OperatorType::Comma => panic!("Somehow called compile operator on {}", op),
            };

            register_handler.release_register(right_reg);

            Ok(format!("{left}\n{right}\n{perform}"))
        }
    }
}

fn compile_prefix_operator(
    op: &OperatorType,
    children: &[Expression],
    compile_scope: &Rc<RefCell<CompileScope>>,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let child = children.first().ok_or("Infix operator missing child")?;
    let child_asm = child.compile(compile_scope, dst, register_handler)?;

    let asm = match op {
        OperatorType::Not => {
            format!("xor {dst}, 1\n")
        }
        _ => panic!("Unexpected operator in compile_prefix_operator: {}", op),
    };

    Ok(format!("{child_asm}\n{asm}"))
}

fn compile_multiply(
    compile_scope: &Rc<RefCell<CompileScope>>,
    left: &Expression,
    right: &Expression,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let left_reg = register_handler.lease_register()?;
    let left = left.compile(
        compile_scope,
        &Address::Register(left_reg.clone()),
        register_handler,
    )?;

    let right_reg = register_handler.lease_register()?;
    let right = right.compile(
        compile_scope,
        &Address::Register(right_reg.clone()),
        register_handler,
    )?;

    let instructions = [
        left,
        right,
        format!("imul {left_reg}, {right_reg}"),
        format!("mov {dst}, {left_reg}"),
    ];

    register_handler.release_register(left_reg);
    register_handler.release_register(right_reg);

    Ok(instructions.join("\n"))
}

fn compile_divide(
    compile_scope: &Rc<RefCell<CompileScope>>,
    left: &Expression,
    right: &Expression,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let right_reg = register_handler.lease_register()?;
    let right = right.compile(
        compile_scope,
        &Address::Register(right_reg.clone()),
        register_handler,
    )?;

    let rax = register_handler.request_register(&Register::RAX)?;
    let left = left.compile(
        compile_scope,
        &Address::Register(rax.clone()),
        register_handler,
    )?;

    let instructions = [left, right];
    let div = register_handler.request_with_scope(&Register::RDX, |rdx| {
        Ok(format!(
            "xor {rdx}, {rdx}\ndiv {right_reg}\nmov {dst}, {rax}"
        ))
    })?;

    register_handler.release_register(right_reg);
    register_handler.release_register(rax);

    Ok(instructions.join("\n") + &div)
}

fn compile_modulo(
    compile_scope: &Rc<RefCell<CompileScope>>,
    left: &Expression,
    right: &Expression,
    dst: &Address,
    register_handler: &mut RegisterHandler,
) -> Result<String, String> {
    let right_reg = register_handler.lease_register()?;
    let right = right.compile(
        compile_scope,
        &Address::Register(right_reg.clone()),
        register_handler,
    )?;

    let rax = register_handler.request_register(&Register::RAX)?;
    let left = left.compile(
        compile_scope,
        &Address::Register(rax.clone()),
        register_handler,
    )?;

    let rdx = register_handler.request_register(&Register::RDX)?;
    let instructions = [
        left,
        right,
        format!("xor {rdx}, {rdx}"),
        format!("div {right_reg}"),
        format!("mov {dst}, {rdx}"),
    ];

    register_handler.release_register(right_reg);
    register_handler.release_register(rax);
    register_handler.release_register(rdx);

    Ok(instructions.join("\n"))
}
