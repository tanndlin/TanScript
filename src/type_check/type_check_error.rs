use std::{error::Error, fmt};

use crate::{lex::Span, type_check::Type};

#[derive(Debug)]
pub enum TypeCheckError {
    TypeMismatch {
        expected: Type,
        got: Type,
        span: Span,
    },
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::TypeMismatch {
                expected,
                got,
                span,
            } => write!(f, "Type mismatch at {span}: expected {expected}, got {got}"),
        }
    }
}

impl Error for TypeCheckError {}
