use std::{error::Error, fmt};

use crate::{lex::Span, type_check::Type};

pub struct TypeCheckError {
    pub kind: TypeCheckErrorKind,
    pub span: Span,
}

impl TypeCheckError {
    pub fn new(kind: TypeCheckErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub enum TypeCheckErrorKind {
    TypeMismatch { expected: Type, got: Type },
    NotCallableIdentifer(String),
    IdentifierNotFound(String),
}

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
        }
    }
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

impl fmt::Debug for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl Error for TypeCheckError {}
