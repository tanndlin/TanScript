use std::{error::Error, fmt};

use crate::{lex::Span, type_check::Type};

pub enum TypeCheckError {
    TypeMismatch {
        expected: Type,
        got: Type,
        span: Span,
    },
    NotCallableIdentifer(String),
    IdentifierNotFound(String),
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::TypeMismatch {
                expected,
                got,
                span,
            } => write!(
                f,
                "Type mismatch at {span}: expected ({expected}), got ({got})"
            ),
            TypeCheckError::NotCallableIdentifer(ident) => {
                write!(f, "Identifier: ({ident}) is not callable")
            }
            TypeCheckError::IdentifierNotFound(ident) => {
                write!(f, "Identifer: ({ident}) not found")
            } // TODO: This error probably isn't completely helpful
        }
    }
}

impl fmt::Debug for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for TypeCheckError {}
