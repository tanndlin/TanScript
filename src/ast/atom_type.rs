use crate::{lex::LexerAtomType, types::NumberType};

#[derive(Debug, Clone, derive_more::Display)]
pub enum AtomType {
    Number(NumberType),
    #[display("'{}'", _0.escape_default())]
    Char(char),
    Identifier(String),
    Boolean(bool),
    #[display("\"{}\"", _0.escape_default())]
    String(String),
    #[display(":")]
    Colon,
}

impl AtomType {
    pub fn from_lexer_atom(atom: &LexerAtomType) -> Option<AtomType> {
        Some(match atom {
            LexerAtomType::Number(n) => AtomType::Number(*n),
            LexerAtomType::Identifier(s) => AtomType::Identifier(s.clone()),
            LexerAtomType::String(s) => AtomType::String(s.clone()),
            LexerAtomType::Char(c) => AtomType::Char(*c),
            LexerAtomType::Semicolon | LexerAtomType::Comma => return None,
            LexerAtomType::Colon => AtomType::Colon,
        })
    }
}
