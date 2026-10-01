use std::fmt;

use crate::lex::LexerAtomType;

#[derive(Debug, Clone)]
pub enum AtomType {
    Number(i32),
    Identifier(String),
    String(String),
    Colon,
}

impl AtomType {
    pub fn from_lexer_atom(atom: &LexerAtomType) -> AtomType {
        match atom {
            LexerAtomType::Number(n) => AtomType::Number(*n),
            LexerAtomType::Identifier(s) => AtomType::Identifier(s.clone()),
            LexerAtomType::String(s) => AtomType::String(s.clone()),
            LexerAtomType::Semicolon => panic!("Unexpected semicolon"),
            LexerAtomType::Colon => AtomType::Colon,
        }
    }
}

impl fmt::Display for AtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomType::Number(n) => write!(f, "{n}"),
            AtomType::Identifier(c) => write!(f, "{c}"),
            AtomType::String(s) => write!(f, "{s}"),
            AtomType::Colon => write!(f, ":"),
        }
    }
}
