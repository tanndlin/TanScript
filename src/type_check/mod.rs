mod check_statement;
mod check_types;
mod infer_type;
mod type_check_error;
mod type_scope;

pub use check_types::{check_types, check_types_expression, expect_type, narrow_expression};
pub use infer_type::unify;
pub use type_check_error::{TypeCheckError, TypeCheckErrorKind};
pub use type_scope::TypeScope;

use core::fmt;
use std::str::FromStr;

use crate::{
    ast::{AtomType, EnumDefinition, StructDefinition},
    types::IntType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub args: Vec<Type>,
    pub return_type: Type,
}

#[derive(Debug, Clone, Eq)]
pub enum Type {
    Int(IntType),
    Char,
    Boolean,
    Function(Box<FunctionType>), // Types of args
    Struct(Box<StructDefinition>),
    Enum(Box<EnumDefinition>),
    Pointer(Box<Type>),
    Named(String),
    Unit,
    Unknown,
}

impl Type {
    pub fn is_number_type(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Int(int) => int.size(),
            Self::Struct(s) => s.size(),
            Self::Unit => 0,
            Self::Boolean | Self::Char | Self::Enum(_) => 1,
            Self::Function(_) | Self::Pointer(_) | Self::Unknown => 8,
            Self::Named(name) => unreachable!("unresolved type ({name})"),
        }
    }

    pub fn castable(&self, casted_type: &Self) -> bool {
        match (self, casted_type) {
            _ if self == casted_type => true,
            (Type::Unknown, _)
            | (_, Type::Unknown)
            | (Type::Int(_) | Type::Char | Type::Boolean, Type::Int(_) | Type::Char)
            | (Type::Enum(_), Type::Int(_))
            | (Type::Pointer(_), Type::Pointer(_)) => true,
            (Type::Pointer(_), Type::Int(int)) | (Type::Int(int), Type::Pointer(_)) => {
                int.size() == 8
            }
            _ => false,
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

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Char, Self::Char)
            | (Self::Boolean, Self::Boolean)
            | (Self::Unit, Self::Unit)
            | (Self::Unknown, Self::Unknown) => true,
            (Self::Function(a), Self::Function(b)) => a == b,
            (Self::Struct(a), Self::Struct(b)) => a == b,
            (Self::Enum(a), Self::Enum(b)) => a == b,
            (Self::Pointer(a), Self::Pointer(b)) => a == b,
            (Self::Named(a), Self::Named(b)) => a == b,
            (Self::Named(name), Self::Struct(def)) | (Self::Struct(def), Self::Named(name)) => {
                *name == def.name
            }
            (Self::Named(name), Self::Enum(def)) | (Self::Enum(def), Self::Named(name)) => {
                *name == def.name
            }
            _ => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int(int) => write!(f, "{int}"),
            Type::Char => write!(f, "char"),
            Type::Function(_) => write!(f, "fn"),
            Type::Unit => write!(f, "()"),
            Type::Unknown => write!(f, "Unknown"),
            Type::Pointer(t) => write!(f, "*{t}"),
            Type::Boolean => write!(f, "bool"),
            Type::Struct(def) => write!(f, "{}", def.name),
            Type::Enum(def) => write!(f, "{}", def.name),
            Type::Named(name) => write!(f, "{name}"),
        }
    }
}

impl From<&AtomType> for Type {
    fn from(atom_type: &AtomType) -> Self {
        match atom_type {
            AtomType::Number(n) => Type::Int(n.kind()),
            AtomType::Identifier(_) => unreachable!(),
            AtomType::String(_) => Type::Pointer(Box::new(Type::Char)),
            AtomType::Boolean(_) => Type::Boolean,
            AtomType::Char(_) => Type::Char,
        }
    }
}
