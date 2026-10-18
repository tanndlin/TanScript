use std::fmt;

use crate::{
    error::{At, Spanned},
    lex::MAX_DEPTH,
    types::Token,
};

#[derive(Debug)]
pub enum ParseErrorKind {
    Expected { expected: String, got: Token },
    UnexpectedEof { expected: String },
    InvalidAssignTarget(String),
    NotAddressable(String),
    TooDeep,
}

impl At for ParseErrorKind {}

pub type ParseError = Spanned<ParseErrorKind>;

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::Expected { expected, got } => {
                write!(f, "Expected ({expected}), but got ({got})")
            }
            ParseErrorKind::UnexpectedEof { expected } => {
                write!(f, "Expected ({expected}), but ran out of tokens")
            }
            ParseErrorKind::InvalidAssignTarget(target) => {
                write!(f, "Cannot assign to ({target})")
            }
            ParseErrorKind::NotAddressable(expr) => {
                write!(f, "({expr}) is not addressable")
            }
            ParseErrorKind::TooDeep => {
                write!(f, "Nesting is deeper than the max of ({MAX_DEPTH})")
            }
        }
    }
}
