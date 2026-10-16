use std::fmt;

use crate::{
    error::{At, Spanned},
    type_check::Type,
};

#[derive(Debug)]
pub enum TypeCheckErrorKind {
    TypeMismatch { expected: Type, got: Type },
    NotCallableIdentifer(String),
    IdentifierNotFound(String),
    Uncastable { lhs_type: Type, casted_type: Type },
    IncorrectNumberOfArgs { expected: usize, given: usize },
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
        }
    }
}
