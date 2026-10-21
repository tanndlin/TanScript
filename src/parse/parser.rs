use std::collections::VecDeque;

use crate::{
    ast::{Block, Program},
    error::At,
    lex::{LexerAtomType, LexerToken, Tokens},
    parse::{
        ParseError, ParseErrorKind,
        parse_statement::{parse_import, parse_statement_or_expression},
    },
    types::{Span, Token},
};

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<LexerToken>,
    eof: Span,
    depth: u32,
}

pub const MAX_DEPTH: u32 = 50;

pub fn parse(tokens: Tokens) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    let mut imports = vec![];
    while let Some(Token::Atom(LexerAtomType::Identifier(keyword))) =
        parser.peek().map(|tok| &tok.token_type)
        && keyword == "use"
    {
        imports.push(parse_import(&mut parser)?);
    }

    let mut block = Block { children: vec![] };
    while let Some(s) = parse_statement_or_expression(&mut parser)? {
        block.children.push(s);
    }

    Ok(Program { imports, block })
}

pub fn expected(expected: &str, got: LexerToken) -> ParseError {
    ParseErrorKind::Expected {
        expected: expected.to_string(),
        got: got.token_type,
    }
    .at(got.span)
}

impl Parser {
    pub fn new(Tokens { tokens, eof }: Tokens) -> Parser {
        Parser {
            tokens,
            eof,
            depth: 0,
        }
    }

    pub fn descend(&mut self, span: Span) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(ParseErrorKind::TooDeep.at(span));
        }

        Ok(())
    }

    pub fn next_nested(&mut self) -> Result<LexerToken, ParseError> {
        let token = self.next_or_eof("token")?;
        self.descend(token.span)?;
        Ok(token)
    }

    pub fn nested<T>(
        &mut self,
        expected: &str,
        parse: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        let span = self.peek_or_eof(expected)?.span;
        let depth = self.depth;
        let result = self.descend(span).and_then(|()| parse(self));
        self.depth = depth;
        result
    }

    pub fn peek(&self) -> Option<&LexerToken> {
        self.tokens.front()
    }

    pub fn next_or_eof(&mut self, expected: &str) -> Result<LexerToken, ParseError> {
        self.next().ok_or_else(|| self.eof_error(expected))
    }

    pub fn peek_or_eof(&self, expected: &str) -> Result<&LexerToken, ParseError> {
        self.peek().ok_or_else(|| self.eof_error(expected))
    }

    pub fn eof_error(&self, expected: &str) -> ParseError {
        ParseErrorKind::UnexpectedEof {
            expected: expected.to_string(),
        }
        .at(self.eof)
    }

    pub fn expect(&mut self, expected: &str) -> Result<LexerToken, ParseError> {
        let token = self.next_or_eof(expected)?;

        // Extract what we "got" as a string
        let got = match &token.token_type {
            Token::Atom(cur) => match cur {
                LexerAtomType::Number(n) => n.to_string(),
                LexerAtomType::Char(c) => format!("'{c}'"),
                LexerAtomType::Identifier(s) | LexerAtomType::String(s) => s.clone(),
                LexerAtomType::Semicolon => ";".to_string(),
                LexerAtomType::Colon => ":".to_string(),
                LexerAtomType::Comma => ",".to_string(),
            },
            Token::Op(cur) => cur.to_string(),
        };

        if got == expected {
            Ok(token)
        } else {
            Err(ParseErrorKind::Expected {
                expected: expected.to_string(),
                got: token.token_type,
            }
            .at(token.span))
        }
    }
}

impl Iterator for Parser {
    type Item = LexerToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop_front()
    }
}
