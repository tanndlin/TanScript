use std::fmt;

use crate::types::{NumberType, Span, Token};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerAtomType {
    Number(NumberType),
    Identifier(String),
    String(String),
    Char(char),
    Semicolon,
    Colon,
}

impl fmt::Display for LexerAtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerAtomType::Number(n) => write!(f, "{n}"),
            LexerAtomType::Identifier(s) => write!(f, "{s}"),
            LexerAtomType::String(s) => write!(f, "\"{}\"", s.escape_default()),
            LexerAtomType::Char(c) => write!(f, "'{}'", c.escape_default()),
            LexerAtomType::Semicolon => write!(f, ";"),
            LexerAtomType::Colon => write!(f, ":"),
        }
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
