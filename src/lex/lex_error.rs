use std::fmt;

use crate::error::{At, Spanned};

#[derive(Debug)]
pub enum LexErrorKind {
    UnknownCharacter(char),
    MissingClosingQuote,
}

impl At for LexErrorKind {}

pub type LexError = Spanned<LexErrorKind>;

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::UnknownCharacter(c) => write!(f, "Unknown character ({c})"),
            LexErrorKind::MissingClosingQuote => write!(f, "Missing closing quote for string"),
        }
    }
}
