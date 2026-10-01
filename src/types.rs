use std::fmt::{self};

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
