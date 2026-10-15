use std::fmt;

use crate::error::{At, Spanned};

#[derive(Debug)]
pub enum LexErrorKind {
    UnknownCharacter(char),
    MissingClosingQuote,
    MissingClosingCharQuote,
    EmptyChar,
    UnknownEscape(char),
}

impl At for LexErrorKind {}

pub type LexError = Spanned<LexErrorKind>;

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexErrorKind::UnknownCharacter(c) => write!(f, "Unknown character ({c})"),
            LexErrorKind::MissingClosingQuote => write!(f, "Missing closing quote for string"),
            LexErrorKind::MissingClosingCharQuote => write!(f, "Missing closing quote for char"),
            LexErrorKind::EmptyChar => write!(f, "Empty char literal"),
            LexErrorKind::UnknownEscape(c) => write!(f, "Unknown escape sequence (\\{c})"),
        }
    }
}
