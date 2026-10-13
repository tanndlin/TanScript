use std::fmt::{self, Display};

use crate::{ast::OperatorType, lex::LexerAtomType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Atom(LexerAtomType),
    Op(OperatorType),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Atom(lexer_atom_type) => write!(f, "{lexer_atom_type}"),
            Token::Op(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NumberType {
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    Coercible(i64),
}

impl Display for NumberType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberType::I32(n) => write!(f, "{n}"),
            NumberType::U32(n) => write!(f, "{n}"),
            NumberType::U64(n) => write!(f, "{n}"),
            NumberType::I64(n) | NumberType::Coercible(n) => write!(f, "{n}"),
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
