mod check_statement;
mod check_types;
mod infer_type;
mod type_check_error;
mod type_scope;

pub use check_types::{check_types, check_types_expression, expect_type, narrow_expression};
pub use type_check_error::{TypeCheckError, TypeCheckErrorKind};
pub use type_scope::TypeScope;

use core::fmt;
use std::str::FromStr;

use crate::{ast::AtomType, types::IntType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub args: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int(IntType),
    Char,
    Boolean,
    Function(Box<FunctionType>), // Types of args
    Pointer(Box<Type>),
    Unknown,
}

impl Type {
    pub fn is_number_type(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Int(int) => int.size(),
            Self::Boolean | Self::Char => 1,
            _ => 8,
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(int) = IntType::from_name(s) {
            return Ok(Type::Int(int));
        }

        match s {
            "bool" => return Ok(Type::Boolean),
            "char" => return Ok(Type::Char),
            _ => {}
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
            Type::Int(int) => write!(f, "{int}"),
            Type::Char => write!(f, "char"),
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
            AtomType::Number(n) => Type::Int(n.kind()),
            AtomType::Identifier(_) => unreachable!(),
            AtomType::String(_) => todo!(), // Type::String,
            AtomType::Colon => panic!("Colon as rhs for expression in type check"),
            AtomType::Boolean(_) => Type::Boolean,
            AtomType::Char(_) => Type::Char,
        }
    }
}
