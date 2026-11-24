use crate::{lex::LexerAtomType, types::NumberType};

#[derive(Debug, Clone, derive_more::Display)]
pub enum AtomType {
    Number(NumberType),
    #[display("'{}'", _0.escape_default())]
    Char(char),
    Boolean(bool),
    #[display("null")]
    Null,
    #[display("\"{}\"", _0.escape_default())]
    String(String),
}

impl TryFrom<LexerAtomType> for AtomType {
    // Returns the value itself when Err so that the ownership is not gone to the void
    type Error = LexerAtomType;

    fn try_from(atom: LexerAtomType) -> Result<Self, Self::Error> {
        Ok(match atom {
            LexerAtomType::Number(n) => AtomType::Number(n),
            LexerAtomType::String(s) => AtomType::String(s),
            LexerAtomType::Char(c) => AtomType::Char(c),
            other @ (LexerAtomType::Identifier(_)
            | LexerAtomType::Semicolon
            | LexerAtomType::Comma
            | LexerAtomType::Colon) => {
                return Err(other);
            }
        })
    }
}
