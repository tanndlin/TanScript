#[derive(Clone, Debug)]
pub enum LexerAtomType {
    Number(i32),
    Identifier(String),
    Semicolon,
}

#[derive(Clone, Debug)]
pub enum Token {
    Atom(LexerAtomType),
    Op(char),
    Eof,
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
