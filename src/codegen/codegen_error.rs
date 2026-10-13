use std::fmt;

use crate::error::{At, Spanned};

#[derive(Debug)]
pub enum CodegenErrorKind {
    UndeclaredVariable(String),
    AlreadyDeclared(String),
    InvalidAssignTarget(String),
    OutOfRegisters,
    OutsideLoop(&'static str),
}

impl At for CodegenErrorKind {}

pub type CodegenError = Spanned<CodegenErrorKind>;

impl fmt::Display for CodegenErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenErrorKind::UndeclaredVariable(name) => {
                write!(f, "Variable ({name}) not found")
            }
            CodegenErrorKind::AlreadyDeclared(name) => {
                write!(f, "Variable ({name}) already declared")
            }
            CodegenErrorKind::InvalidAssignTarget(target) => {
                write!(f, "Cannot assign to ({target})")
            }
            CodegenErrorKind::OutOfRegisters => write!(f, "Ran out of registers"),
            CodegenErrorKind::OutsideLoop(keyword) => write!(f, "({keyword}) used outside of a loop"),
        }
    }
}
