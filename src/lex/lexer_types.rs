use std::fmt;

use crate::types::{NumberType, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerAtomType {
    Number(NumberType),
    Identifier(String),
    String(String),
    Semicolon,
    Colon,
}

impl fmt::Display for LexerAtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerAtomType::Number(n) => write!(f, "{n}"),
            LexerAtomType::Identifier(s) => write!(f, "{s}"),
            LexerAtomType::String(s) => write!(f, "\"{s}\""),
            LexerAtomType::Semicolon => write!(f, ";"),
            LexerAtomType::Colon => write!(f, ":"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Span {
    pub line: u32,
    pub col: u32,
}

impl Span {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line + 1, self.col + 1)
    }
}

#[derive(Debug)]
pub struct LexerToken {
    pub token_type: Token,
    pub span: Span,
}

impl LexerToken {
    pub fn new(token_type: Token, span: Span) -> LexerToken {
        LexerToken { token_type, span }
    }
}

impl fmt::Display for LexerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}", self.token_type, self.span)
    }
}
