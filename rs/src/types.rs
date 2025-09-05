#[derive(Copy, Clone, Debug)]
pub enum Token {
    Atom(char),
    Op(char),
    Eof,
}

#[derive(Copy, Clone, Debug)]
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
