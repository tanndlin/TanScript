use std::fmt::{self};

use crate::ast::OperatorType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerAtomType {
    Number(i32),
    Identifier(String),
    String(String),
    Semicolon,
}

impl fmt::Display for LexerAtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LexerAtomType::*;
        match self {
            Number(n) => write!(f, "{}", n),
            Identifier(s) => write!(f, "{}", s),
            String(s) => write!(f, "\"{}\"", s),
            Semicolon => write!(f, ";"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Atom(LexerAtomType),
    Op(OperatorType),
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Eof => Ok(()),
            Token::Atom(lexer_atom_type) => write!(f, "{}", lexer_atom_type),
            Token::Op(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LexerToken {
    pub token_type: Token,
    pub line_number: u32,
}

impl LexerToken {
    pub fn new(token_type: Token, line_number: u32) -> LexerToken {
        LexerToken {
            token_type,
            line_number,
        }
    }
}

impl fmt::Display for LexerToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} on line {}", self.token_type, self.line_number)
    }
}
