use crate::types::{LexerAtomType, LexerToken, Token};

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<LexerToken>,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Lexer, String> {
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

            if *cur == '"' {
                tokens.push(LexerToken::new(
                    Token::Atom(LexerAtomType::String(get_string(&mut chars))),
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
                '+' | '-' | '*' | '/' | '=' | '(' | ')' | ',' | '}' | '{' | '<' => {
                    tokens.push(LexerToken::new(
                        Token::Op(chars.pop().unwrap()),
                        line_number,
                    ));
                }
                _ => {
                    return Err(format!(
                        "Lexer: Unknown character: {} on line: {}",
                        cur, line_number
                    ));
                }
            }
        }

        Ok(Lexer { tokens })
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

    pub fn expect(&mut self, expected: &str) -> Result<(), String> {
        let token = self.next();

        // Extract what we "got" as a string
        let got = match &token.token_type {
            Token::Atom(cur) => match cur {
                LexerAtomType::Number(n) => n.to_string(),
                LexerAtomType::Identifier(s) => s.clone(),
                LexerAtomType::Semicolon => ";".to_string(),
                LexerAtomType::String(s) => s.clone(),
            },
            Token::Op(cur) => cur.to_string(),
            Token::Eof => "End-Of-File".to_string(),
        };

        if got != expected {
            Err(format!(
                "Error: Expected {}, got {:?} on line: {}",
                expected, got, token.line_number
            ))
        } else {
            Ok(())
        }
    }
}

fn get_number(input: &mut Vec<char>) -> i32 {
    let mut chars = vec![];

    while let Some(c) = input.last() {
        if !c.is_ascii_digit() {
            break;
        }

        chars.push(input.pop().unwrap());
    }

    chars
        .into_iter()
        .rev()
        .collect::<String>()
        .parse::<i32>()
        .unwrap()
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

fn get_string(input: &mut Vec<char>) -> String {
    // Remove the leading quote
    input.pop();
    let mut chars = vec![];

    while let Some(c) = input.last() {
        if *c == '"' {
            break;
        }

        chars.push(*c);
        input.pop();
    }

    // Remove the closing quote
    input.pop().expect("No closing quote found for string");

    chars.reverse();
    chars.into_iter().collect()
}
