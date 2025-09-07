use crate::types::{LexerAtomType, LexerToken, Token};

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<LexerToken>,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut line_number = 0u32;
        let mut chars = input.chars().collect::<Vec<char>>();

        let mut tokens = vec![];
        while let Some(cur) = chars.last() {
            if cur == &'\n' {
                line_number += 1;
                chars.pop();
                continue;
            }

            if cur.is_whitespace() {
                chars.pop();
                continue;
            }

            if cur.is_ascii_digit() {
                tokens.push(LexerToken::new(
                    Token::Atom(LexerAtomType::Number(get_number(&mut chars))),
                    line_number,
                ));
                continue;
            }

            if cur.is_alphabetic() {
                tokens.push(LexerToken::new(
                    Token::Atom(LexerAtomType::Identifier(get_identifier(&mut chars))),
                    line_number,
                ));
                continue;
            }

            match cur {
                ';' => {
                    chars.pop();
                    tokens.push(LexerToken::new(
                        Token::Atom(LexerAtomType::Semicolon),
                        line_number,
                    ));
                    continue;
                }
                '+' | '-' | '*' | '/' | '=' | '(' | ')' => {
                    tokens.push(LexerToken::new(
                        Token::Op(chars.pop().unwrap()),
                        line_number,
                    ));
                }
                _ => panic!("Unknown character: {} on line: {}", cur, line_number),
            }
        }

        Lexer { tokens }
    }

    pub fn next(&mut self) -> LexerToken {
        self.tokens.pop().unwrap_or(LexerToken::new(Token::Eof, 0))
    }

    pub fn peek(&self) -> Option<&LexerToken> {
        self.tokens.last()
    }

    pub fn peek_next(&self) -> Option<&LexerToken> {
        if self.tokens.len() >= 2 {
            Some(&self.tokens[self.tokens.len() - 2])
        } else {
            None
        }
    }

    pub fn expect(&mut self, expected: &str) {
        let token = self.next();

        let error: Option<String> = match token.token_type {
            Token::Atom(cur) => match cur {
                LexerAtomType::Number(n) => {
                    if n.to_string() != expected {
                        Some(n.to_string())
                    } else {
                        None
                    }
                }
                LexerAtomType::Identifier(s) => {
                    if s != expected {
                        Some(s.clone())
                    } else {
                        None
                    }
                }
                LexerAtomType::Semicolon => {
                    if ";" != expected {
                        Some(";".to_string())
                    } else {
                        None
                    }
                }
            },
            Token::Op(cur) => {
                if cur.to_string() != expected {
                    Some(cur.to_string())
                } else {
                    None
                }
            }
            Token::Eof => Some("End-Of-File".to_string()),
        };

        if let Some(got) = error {
            panic!(
                "Error: Expected {}, got {:?} on line: {}",
                expected, got, token.line_number
            );
        }
    }
}

fn get_number(input: &mut Vec<char>) -> i32 {
    let mut n = 0i32;

    while let Some(c) = input.last() {
        if !c.is_ascii_digit() {
            break;
        }

        n = n * 10 + c.to_digit(10).unwrap() as i32;
        input.pop();
    }

    n
}

fn get_identifier(input: &mut Vec<char>) -> String {
    let mut string_vec = vec![];

    while let Some(c) = &input.last() {
        if !c.is_alphanumeric() {
            break;
        }

        string_vec.push(input.pop().unwrap());
    }

    string_vec.into_iter().rev().collect::<String>()
}
