use crate::types::{LexerToken, Token};

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<LexerToken>,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut line_number = 0u32;

        let mut tokens = input
            .chars()
            // .filter(|it| !it.is_whitespace())
            .filter_map(|c| match c {
                '\n' => {
                    line_number += 1;
                    None
                }
                ' ' | '\t' | '\r' => None,
                '0'..='9' | 'a'..='z' | 'A'..='Z' => {
                    Some(LexerToken::new(Token::Atom(c), line_number))
                }
                _ => Some(LexerToken::new(Token::Op(c), line_number)),
            })
            .collect::<Vec<LexerToken>>();

        tokens.reverse();
        Lexer { tokens }
    }

    pub fn next(&mut self) -> LexerToken {
        self.tokens.pop().unwrap_or(LexerToken::new(Token::Eof, 0))
    }

    pub fn peek(&self) -> LexerToken {
        self.tokens
            .last()
            .copied()
            .unwrap_or(LexerToken::new(Token::Eof, 0))
    }

    pub fn expect(&self, c: char) {
        match self.peek().token_type {
            Token::Atom(cur) => {
                if cur != c {
                    panic!("Error: Expected {}, got {:?}", c, self.peek());
                }
            }
            Token::Op(cur) => {
                if cur != c {
                    panic!("Error: Expected {}, got {:?}", c, self.peek());
                }
            }
            _ => panic!("Error: Expected {}, got {:?}", c, self.peek()),
        }
    }
}
