use std::fmt::{self, Display};

use crate::{ast::OperatorType, lex::LexerAtomType};

#[derive(Clone, Debug, PartialEq, Eq, derive_more::Display)]
pub enum Token {
    Atom(LexerAtomType),
    Op(OperatorType),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntType {
    I32,
    I64,
    U32,
    U64,
    Coercible,
}

impl IntType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            _ => None,
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 | Self::Coercible => 8,
        }
    }

    pub fn is_signed(self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::Coercible)
    }
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
            Self::Coercible => write!(f, "CoercibleNumber"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Display)]
pub enum NumberType {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Coercible(i64),
}

impl NumberType {
    pub fn kind(self) -> IntType {
        match self {
            Self::I32(_) => IntType::I32,
            Self::I64(_) => IntType::I64,
            Self::U32(_) => IntType::U32,
            Self::U64(_) => IntType::U64,
            Self::Coercible(_) => IntType::Coercible,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}
