use std::fmt;

use derive_more::From;

use crate::{
    codegen::address::{Address, Register},
    types::NumberType,
};

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub enum Operand {
    Address(Address),
    Immediate(NumberType),
    Label(String),
}

impl Operand {
    pub fn is_memory(&self) -> bool {
        match self {
            Operand::Address(address) => address.is_memory(),
            Operand::Immediate(_) | Operand::Label(_) => false,
        }
    }
}

impl From<Register> for Operand {
    fn from(reg: Register) -> Self {
        Self::Address(reg.into())
    }
}

impl From<i32> for Operand {
    fn from(n: i32) -> Self {
        NumberType::I32(n).into()
    }
}

impl From<u32> for Operand {
    fn from(n: u32) -> Self {
        NumberType::U32(n).into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    Byte,
    Dword,
    Qword,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Mov {
        dst: Address,
        src: Operand,
    },
    Add {
        dst: Address,
        src: Operand,
    },
    Sub {
        dst: Address,
        src: Operand,
    },
    Or {
        dst: Address,
        src: Operand,
    },
    And {
        dst: Address,
        src: Operand,
    },
    Xor {
        dst: Address,
        src: Operand,
    },
    Neg {
        dst: Address,
    },
    Imul {
        dst: Register,
        src: Operand,
    },
    Idiv {
        src: Register,
    },
    Cqo,
    Store {
        dst: Address,
        src: Register,
        size: Size,
    },
    Test {
        lhs: Address,
        rhs: Operand,
    },
    Movsx {
        dst: Register,
        src: Address,
        size: Size,
    },
    Movzx {
        dst: Register,
        src: Address,
        size: Size,
    },
    Cmp {
        lhs: Address,
        rhs: Operand,
    },
    Set {
        cond: Condition,
        dst: Address,
    },
    Jmp {
        label: String,
    },
    Jcc {
        cond: Condition,
        label: String,
    },
    Push {
        src: Operand,
    },
    Pop {
        dst: Address,
    },
    Call {
        target: String,
    },
    Label {
        name: String,
    },
    Ret,
    Comment(String),
}

impl Instruction {
    pub fn mov(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::Mov {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn add(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::Add {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn sub(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::Sub {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn or(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::Or {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn and(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::And {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn xor(dst: impl Into<Address>, src: impl Into<Operand>) -> Self {
        Self::Xor {
            dst: dst.into(),
            src: src.into(),
        }
    }

    pub fn neg(dst: impl Into<Address>) -> Self {
        Self::Neg { dst: dst.into() }
    }

    pub fn idiv(src: Register) -> Self {
        Self::Idiv { src }
    }

    pub fn store(dst: impl Into<Address>, src: Register, size: Size) -> Self {
        Self::Store {
            dst: dst.into(),
            src,
            size,
        }
    }

    pub fn test(lhs: impl Into<Address>, rhs: impl Into<Operand>) -> Self {
        Self::Test {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
    }

    pub fn imul(dst: Register, src: impl Into<Operand>) -> Self {
        Self::Imul {
            dst,
            src: src.into(),
        }
    }

    pub fn movsx(dst: Register, src: impl Into<Address>, size: Size) -> Self {
        Self::Movsx {
            dst,
            src: src.into(),
            size,
        }
    }

    pub fn movzx(dst: Register, src: impl Into<Address>, size: Size) -> Self {
        Self::Movzx {
            dst,
            src: src.into(),
            size,
        }
    }

    pub fn cmp(lhs: impl Into<Address>, rhs: impl Into<Operand>) -> Self {
        Self::Cmp {
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
    }

    pub fn set(cond: Condition, dst: impl Into<Address>) -> Self {
        Self::Set {
            cond,
            dst: dst.into(),
        }
    }

    pub fn jmp(label: impl Into<String>) -> Self {
        Self::Jmp {
            label: label.into(),
        }
    }

    pub fn jcc(cond: Condition, label: impl Into<String>) -> Self {
        Self::Jcc {
            cond,
            label: label.into(),
        }
    }

    pub fn push(src: impl Into<Operand>) -> Self {
        Self::Push { src: src.into() }
    }

    pub fn pop(dst: impl Into<Address>) -> Self {
        Self::Pop { dst: dst.into() }
    }

    pub fn call(target: impl Into<String>) -> Self {
        Self::Call {
            target: target.into(),
        }
    }

    pub fn label(name: impl Into<String>) -> Self {
        Self::Label { name: name.into() }
    }

    pub fn comment(text: impl Into<String>) -> Self {
        Self::Comment(text.into())
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Size::Byte => write!(f, "BYTE"),
            Size::Dword => write!(f, "DWORD"),
            Size::Qword => write!(f, "QWORD"),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Address(addr) => write!(f, "{addr}"),
            Operand::Immediate(n) => write!(f, "{n}"),
            Operand::Label(l) => write!(f, "{l}"),
        }
    }
}

impl Condition {
    pub fn suffix(&self) -> &'static str {
        match self {
            Condition::Equal => "e",
            Condition::NotEqual => "ne",
            Condition::Less => "l",
            Condition::LessEqual => "le",
            Condition::Greater => "g",
            Condition::GreaterEqual => "ge",
        }
    }

    pub fn negated(&self) -> Self {
        match self {
            Condition::Equal => Condition::NotEqual,
            Condition::NotEqual => Condition::Equal,
            Condition::Less => Condition::GreaterEqual,
            Condition::LessEqual => Condition::Greater,
            Condition::Greater => Condition::LessEqual,
            Condition::GreaterEqual => Condition::Less,
        }
    }

    pub fn swapped(&self) -> Self {
        match self {
            Condition::Equal => Condition::Equal,
            Condition::NotEqual => Condition::NotEqual,
            Condition::Less => Condition::Greater,
            Condition::LessEqual => Condition::GreaterEqual,
            Condition::Greater => Condition::Less,
            Condition::GreaterEqual => Condition::LessEqual,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Mov { dst, src } => write_binary(f, "mov", *dst, src),
            Instruction::Add { dst, src } => write_binary(f, "add", *dst, src),
            Instruction::Sub { dst, src } => write_binary(f, "sub", *dst, src),
            Instruction::Or { dst, src } => write_binary(f, "or", *dst, src),
            Instruction::And { dst, src } => write_binary(f, "and", *dst, src),
            Instruction::Xor { dst, src } => write_binary(f, "xor", *dst, src),
            Instruction::Cmp { lhs, rhs } => write_binary(f, "cmp", *lhs, rhs),
            Instruction::Test { lhs, rhs } => write_binary(f, "test", *lhs, rhs),
            Instruction::Neg { dst } => write!(f, "\tneg QWORD {dst}"),
            Instruction::Imul { dst, src } => write!(f, "\timul {dst}, {src}"),
            Instruction::Idiv { src } => write!(f, "\tidiv {src}"),
            Instruction::Cqo => write!(f, "\tcqo"),
            Instruction::Movsx { dst, src, size } => match size {
                Size::Byte => write!(f, "\tmovsx {dst}, {size} {src}"),
                Size::Dword => write!(f, "\tmovsxd {dst}, {size} {src}"),
                Size::Qword => write!(f, "\tmov {dst}, {src}"),
            },
            Instruction::Movzx { dst, src, size } => match size {
                Size::Byte => write!(f, "\tmovzx {dst}, {size} {src}"),
                Size::Dword => write!(f, "\tmov {}, {size} {src}", dst.lower_32_bits()),
                Size::Qword => write!(f, "\tmov {dst}, {src}"),
            },
            Instruction::Store { dst, src, size } => match size {
                Size::Byte => write!(
                    f,
                    "\tmov {size} {dst}, {}",
                    Address::from(*src).lower_8_bits()
                ),
                Size::Dword => write!(f, "\tmov {size} {dst}, {}", src.lower_32_bits()),
                Size::Qword => write!(f, "\tmov {dst}, {src}"),
            },
            Instruction::Set { cond, dst } => {
                write!(f, "\tset{} {}", cond.suffix(), dst.lower_8_bits())
            }
            Instruction::Jmp { label } => write!(f, "\tjmp {label}"),
            Instruction::Jcc { cond, label } => write!(f, "\tj{} {label}", cond.suffix()),
            Instruction::Push { src } => write!(f, "\tpush {src}"),
            Instruction::Pop { dst } => write!(f, "\tpop {dst}"),
            Instruction::Call { target } => write!(f, "\tcall {target}"),
            Instruction::Label { name } => write!(f, "{name}:"),
            Instruction::Ret => write!(f, "\tret"),
            Instruction::Comment(c) => write!(f, "; {c}"),
        }
    }
}

fn write_binary(f: &mut fmt::Formatter<'_>, op: &str, dst: Address, src: &Operand) -> fmt::Result {
    match (dst, src) {
        (Address::Register(_), _) => write!(f, "\t{op} {dst}, {src}"),
        (_, Operand::Immediate(n)) => write!(f, "\t{op} {} {dst}, {src}", size_specifier(*n)),
        _ => write!(f, "\t{op} {dst}, {src}"),
    }
}

fn size_specifier(n: NumberType) -> Size {
    match n.kind().size() {
        4 => Size::Dword,
        _ => Size::Qword,
    }
}
