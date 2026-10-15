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
    DifferingReturnTypes { types: Vec<Type> },
}

impl At for TypeCheckErrorKind {}

pub type TypeCheckError = Spanned<TypeCheckErrorKind>;

impl fmt::Display for TypeCheckErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckErrorKind::TypeMismatch { expected, got } => {
                write!(f, "Type mismatch: expected ({expected}), got ({got})")
            }
            TypeCheckErrorKind::NotCallableIdentifer(ident) => {
                write!(f, "Identifier: ({ident}) is not callable")
            }
            TypeCheckErrorKind::IdentifierNotFound(ident) => {
                write!(f, "Identifer: ({ident}) not found")
            }
            TypeCheckErrorKind::DifferingReturnTypes { types } => {
                let types: Vec<String> = types.iter().map(ToString::to_string).collect();
                write!(f, "Got multiple return types: ({})", types.join(", "))
            }
        }
    }
}
