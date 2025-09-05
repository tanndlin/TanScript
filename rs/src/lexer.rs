use crate::types::{LexerToken, Token};

pub struct Lexer {
    pub index: usize,
    pub chars: Vec<char>,
    pub line_number: u32,
}

impl Lexer {
    pub fn new(chars: &[char]) -> Lexer {
        Lexer {
            index: 0,
            line_number: 0,
            chars: chars.to_vec(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<LexerToken> {
        let mut tokens = vec![];

        while let Some(token) = self.next_token() {
            tokens.push(token);
        }

        tokens
    }

    pub fn next_token(&mut self) -> Option<LexerToken> {
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.line_number += 1;
            }
            if c.is_whitespace() {
                self.index += 1;
            } else {
                break;
            }
        }

        match self.peek() {
            Some(cur_char) => {
                let token = if cur_char.is_ascii_digit() {
                    Token::Number(self.get_number())
                } else {
                    let token = Token::char_to_token(cur_char);
                    self.index += 1;
                    token
                };

                Some(LexerToken::new(token, self.line_number))
            }
            None => None,
        }
    }

    pub fn peek(&self) -> Option<char> {
        if self.index < self.chars.len() {
            Some(self.chars[self.index])
        } else {
            None
        }
    }

    fn get_number(&mut self) -> i32 {
        let mut cur_num: i32 = 0;

        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }

            if let Some(c_as_num) = c.to_digit(10) {
                cur_num = cur_num * 10 + c_as_num as i32;
            } else {
                panic!("Something happended converting {} to a number", c);
            }

            self.index += 1;
        }

        cur_num
    }
}
