#[derive(Debug)]
pub enum Token {
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
}
impl Token {
    pub fn char_to_token(cur_char: char) -> Token {
        match cur_char {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            _ => panic!("Unexpected token ({})", cur_char),
        }
    }
}

#[derive(Debug)]
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
