mod check_types;
mod infer_type;
mod type_check_error;
mod type_scope;

pub use check_types::{check_types, check_types_expression, expect_type, narrow_expression};
pub use type_check_error::TypeCheckError;
pub use type_scope::TypeScope;

use core::fmt;
use std::str::FromStr;

use crate::{ast::AtomType, types::NumberType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub args: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I32,
    I64,
    U32,
    U64,
    CoercibleNumber,
    Boolean,
    Function(Box<FunctionType>), // Types of args
    Pointer(Box<Type>),
    Unknown,
}

impl Type {
    pub fn is_number_type(&self) -> bool {
        matches!(
            self,
            Self::I32 | Self::I64 | Self::U32 | Self::U64 | Self::CoercibleNumber
        )
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Boolean => 1,
            Self::I32 | Self::U32 => 4,
            _ => 8,
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(t) = match s {
            "i32" => Some(Type::I32),
            "i64" => Some(Type::I64),
            "u32" => Some(Type::U32),
            "u64" => Some(Type::U64),
            "bool" => Some(Type::Boolean),
            _ => None,
        } {
            return Ok(t);
        }

        if let Some(inner) = s.strip_prefix('*') {
            Ok(Type::Pointer(Box::new(Type::from_str(inner)?)))
        } else {
            Err(format!("Unknown type: {s}"))
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::CoercibleNumber => write!(f, "CoercibleNumber"),
            Type::Function(_) => write!(f, "fn"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Pointer(t) => write!(f, "*{t}"),
            Type::Boolean => write!(f, "bool"),
        }
    }
}

impl From<&AtomType> for Type {
    fn from(atom_type: &AtomType) -> Self {
        match atom_type {
            AtomType::Number(NumberType::I32(_)) => Type::I32,
            AtomType::Number(NumberType::U32(_)) => Type::U32,
            AtomType::Number(NumberType::I64(_)) => Type::I64,
            AtomType::Number(NumberType::U64(_)) => Type::U64,
            AtomType::Number(NumberType::Coercible(_)) => Type::CoercibleNumber,
            AtomType::Identifier(_) => unreachable!(),
            AtomType::String(_) => todo!(), // Type::String,
            AtomType::Colon => panic!("Colon as rhs for expression in type check"),
            AtomType::Boolean(_) => Type::Boolean,
        }
    }
}
