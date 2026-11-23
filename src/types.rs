use std::fmt::{self, Display};

use crate::{ast::OperatorType, lex::LexerAtomType};

#[derive(Clone, Debug, PartialEq, Eq, derive_more::Display, derive_more::From)]
pub enum Token {
    Atom(LexerAtomType),
    Op(OperatorType),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IntType {
    U8,
    I16,
    U16,
    I32,
    I64,
    U32,
    U64,
    Coercible,
    CoercibleNegative,
}

impl IntType {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "u8" => Some(Self::U8),
            "i16" => Some(Self::I16),
            "u16" => Some(Self::U16),
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "u32" => Some(Self::U32),
            "u64" => Some(Self::U64),
            _ => None,
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 | Self::Coercible | Self::CoercibleNegative => 8,
        }
    }

    pub fn is_signed(self) -> bool {
        matches!(
            self,
            Self::I16 | Self::I32 | Self::I64 | Self::Coercible | Self::CoercibleNegative
        )
    }

    pub fn widens_to(self, to: Self) -> bool {
        !to.is_coercible() && self.size() < to.size() && (!self.is_signed() || to.is_signed())
    }

    pub fn is_coercible(self) -> bool {
        matches!(self, Self::Coercible | Self::CoercibleNegative)
    }

    pub fn coerces_to(self, to: Self) -> bool {
        match self {
            Self::Coercible => true,
            Self::CoercibleNegative => to.is_signed() && to != Self::Coercible,
            _ => false,
        }
    }

    pub fn negated(self) -> Self {
        match self {
            Self::Coercible => Self::CoercibleNegative,
            t => t,
        }
    }
}

impl Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::U8 => write!(f, "u8"),
            Self::I16 => write!(f, "i16"),
            Self::U16 => write!(f, "u16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
            Self::Coercible => write!(f, "CoercibleNumber"),
            Self::CoercibleNegative => write!(f, "CoercibleNegativeNumber"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Display)]
pub enum NumberType {
    U8(u8),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Coercible(u64),
}

impl NumberType {
    pub fn kind(self) -> IntType {
        match self {
            Self::U8(_) => IntType::U8,
            Self::I32(_) => IntType::I32,
            Self::I64(_) => IntType::I64,
            Self::U32(_) => IntType::U32,
            Self::U64(_) => IntType::U64,
            Self::Coercible(_) => IntType::Coercible,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Position {
    pub line: u32,
    pub col: u32,
}

impl Position {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub file: u32,
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(file: u32, start: Position) -> Self {
        Self {
            file,
            start,
            end: start,
        }
    }

    pub fn to(self, other: Span) -> Self {
        Self {
            end: other.end,
            ..self
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)
    }
}
