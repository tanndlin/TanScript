mod parse_error;
mod parse_expression;
mod parse_statement;
#[cfg(test)]
mod tests;

pub use parse_error::{ParseError, ParseErrorKind};

use crate::{
    ast::{Block, Program},
    error::At,
    lex::{Lexer, LexerAtomType, LexerToken},
    types::Token,
};
use parse_statement::{parse_import, parse_statement_or_expression};

pub fn parse(mut lexer: Lexer) -> Result<Program, ParseError> {
    let mut imports = vec![];
    while let Some(Token::Atom(LexerAtomType::Identifier(keyword))) =
        lexer.peek().map(|tok| &tok.token_type)
        && keyword == "use"
    {
        imports.push(parse_import(&mut lexer)?);
    }

    let mut block = Block { children: vec![] };
    while let Some(s) = parse_statement_or_expression(&mut lexer)? {
        block.children.push(s);
    }

    Ok(Program { imports, block })
}

fn expected(expected: &str, got: LexerToken) -> ParseError {
    ParseErrorKind::Expected {
        expected: expected.to_string(),
        got: got.token_type,
    }
    .at(got.span)
}
