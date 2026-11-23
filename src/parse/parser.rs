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
    last: Span,
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

pub fn expected(expected: &'static str, got: LexerToken) -> ParseError {
    expected_at(expected, got.token_type, got.span)
}

pub fn expected_at(expected: &'static str, got: impl Into<Token>, span: Span) -> ParseError {
    ParseErrorKind::Expected {
        expected,
        got: got.into(),
    }
    .at(span)
}

impl Parser {
    pub fn new(Tokens { tokens, eof }: Tokens) -> Parser {
        Parser {
            tokens,
            eof,
            last: eof,
            depth: 0,
        }
    }

    pub fn last_span(&self) -> Span {
        self.last
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

    pub fn peek_nth(&self, n: usize) -> Option<&LexerToken> {
        self.tokens.get(n)
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

    pub fn expect(&mut self, expected_str: &'static str) -> Result<LexerToken, ParseError> {
        let token = self.next_or_eof(expected_str)?;

        let got = match &token.token_type {
            Token::Atom(LexerAtomType::Identifier(s)) => s.as_str(),
            Token::Atom(LexerAtomType::Semicolon) => ";",
            Token::Atom(LexerAtomType::Colon) => ":",
            Token::Atom(LexerAtomType::Comma) => ",",
            Token::Atom(
                LexerAtomType::Number(_) | LexerAtomType::Char(_) | LexerAtomType::String(_),
            ) => return Err(expected(expected_str, token)),
            Token::Op(op) => op.symbol(),
        };

        if got == expected_str {
            Ok(token)
        } else {
            Err(expected(expected_str, token))
        }
    }
}

impl Iterator for Parser {
    type Item = LexerToken;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.pop_front()?;
        self.last = token.span;
        Some(token)
    }
}
