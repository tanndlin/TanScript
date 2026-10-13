mod parse_error;
mod parse_expression;
mod parse_statement;
#[cfg(test)]
mod tests;

pub use parse_error::{ParseError, ParseErrorKind};

use crate::{
    ast::{Block, Program},
    error::At,
    lex::{Lexer, LexerToken},
};
use parse_statement::parse_statement_or_expression;

pub fn parse(mut lexer: Lexer) -> Result<Program, ParseError> {
    let mut block = Block { children: vec![] };
    while let Some(s) = parse_statement_or_expression(&mut lexer)? {
        block.children.push(s);
    }

    Ok(Program { block })
}

fn expected(expected: &str, got: LexerToken) -> ParseError {
    ParseErrorKind::Expected {
        expected: expected.to_string(),
        got: got.token_type,
    }
    .at(got.span)
}
