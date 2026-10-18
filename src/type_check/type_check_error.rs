use std::fmt;

use crate::{
    ast::StructDefinition,
    error::{At, Spanned},
    type_check::Type,
};

#[derive(Debug)]
pub enum TypeCheckErrorKind {
    TypeMismatch {
        expected: Type,
        got: Type,
    },
    NotCallableIdentifer(String),
    IdentifierNotFound(String),
    Uncastable {
        lhs_type: Type,
        casted_type: Type,
    },
    IncorrectNumberOfArgs {
        expected: usize,
        given: usize,
    },
    NotAStruct(String),
    NoSuchField {
        struct_type: StructDefinition,
        field_name: String,
    },
    DuplicateIdentifiers(String),
    StructAsArgument,
    UnknownType(String),
    NotDerefable(Type),
}

impl At for TypeCheckErrorKind {}

pub type TypeCheckError = Spanned<TypeCheckErrorKind>;

impl fmt::Display for TypeCheckErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected ({expected}), got ({got})")
            }
            Self::NotCallableIdentifer(ident) => {
                write!(f, "Identifier: ({ident}) is not callable")
            }
            Self::IdentifierNotFound(ident) => {
                write!(f, "Identifer: ({ident}) not found")
            }
            Self::Uncastable {
                lhs_type,
                casted_type,
            } => {
                write!(f, "Cannot cast type: ({lhs_type}) as ({casted_type})")
            }
            Self::IncorrectNumberOfArgs { expected, given } => {
                write!(
                    f,
                    "Incorrect number of args passed to function. Expected: ({expected}), got: ({given})"
                )
            }
            Self::NotAStruct(given_type) => write!(f, "Type: ({given_type}) is not a struct"),
            Self::NoSuchField {
                struct_type,
                field_name,
            } => write!(
                f,
                "Struct: ({struct_type}) does not contain a field called ({field_name})"
            ),
            Self::DuplicateIdentifiers(ident) => write!(f, "Identifier ({ident}) already exists"),
            Self::StructAsArgument => write!(f, "Structs cannot be passed as function arguments"),
            Self::UnknownType(name) => write!(f, "Unknown type ({name})"),
            Self::NotDerefable(t) => write!(f, "Type: ({t}) cannot be dereferenced"),
        }
    }
}
