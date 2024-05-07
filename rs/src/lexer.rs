use crate::types;

#[derive(Debug, Clone)]
pub struct LexerToken {
    pub token: types::Token,
    pub position: usize,
    pub line: usize,
}

pub fn tokenize(script: &str) -> Vec<LexerToken> {
    let mut tokens: Vec<LexerToken> = vec![];
    let mut position = 0;
    let mut line_number = 1;

    while position < script.len() {
        if is_whitespace(script.chars().nth(position).unwrap()) {
            position += 1;
            continue;
        }

        let lexer_token = LexerToken {
            position: position,
            token: next_token(script, &mut position),
            line: line_number,
        };

        tokens.push(lexer_token);
        if position < script.len() && script.chars().nth(position).unwrap() == '\n' {
            line_number += 1;
        }
    }

    tokens
}

fn next_token(script: &str, position: &mut usize) -> types::Token {
    let c = script.chars().nth(*position).unwrap();
    if c.is_digit(10) {
        return lex_number(script, position);
    }

    if let Some(token) = match_operator(c) {
        *position += 1;
        return token;
    }

    if !c.is_alphanumeric() {
        panic!("Unexpected character: {}", c);
    }

    let possibly_ident = lex_identifier(script, position);
    if let Some(keyword) = types::Keywords::from_string(&possibly_ident) {
        return keyword.to_token();
    }

    types::Token::Identifier(possibly_ident.to_string())
}

fn lex_identifier(script: &str, position: &mut usize) -> String {
    let mut ident = String::new();
    while *position < script.len() {
        let c = script.chars().nth(*position).unwrap();
        if c.is_alphanumeric() {
            ident.push(c);
            *position += 1;
        } else {
            break;
        }
    }

    ident
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\t'
}

fn match_operator(c: char) -> Option<types::Token> {
    match c {
        '+' => Some(types::Token::Add),
        '-' => Some(types::Token::Subtract),
        '*' => Some(types::Token::Multiply),
        '/' => Some(types::Token::Divide),
        '%' => Some(types::Token::Mod),
        ';' => Some(types::Token::Semi),
        '=' => Some(types::Token::Assign),
        _ => None,
    }
}

fn lex_number(script: &str, position: &mut usize) -> types::Token {
    let mut number = String::new();

    while *position < script.len() {
        let next_c = script.chars().nth(*position).unwrap();
        if next_c.is_digit(10) {
            number.push(next_c);
            *position += 1;
        } else {
            break;
        }
    }

    types::Token::Number(number.parse::<i32>().unwrap())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_whitespace() {
        assert_eq!(is_whitespace(' '), true);
        assert_eq!(is_whitespace('\n'), true);
        assert_eq!(is_whitespace('\t'), true);
        assert_eq!(is_whitespace('a'), false);
    }

    #[test]
    fn test_match_operator() {
        assert_eq!(match_operator('+'), Some(types::Token::Add));
        assert_eq!(match_operator('-'), Some(types::Token::Subtract));
        assert_eq!(match_operator('*'), Some(types::Token::Multiply));
        assert_eq!(match_operator('/'), Some(types::Token::Divide));
        assert_eq!(match_operator('a'), None);
    }

    #[test]
    fn test_lex_number() {
        let token = lex_number("123", &mut 0);
        assert_eq!(token, types::Token::Number(123));

        let token = lex_number("123abc", &mut 0);
        assert_eq!(token, types::Token::Number(123));
    }

    #[test]
    fn lex_simple_expression() {
        let tokens = tokenize("1 + 2");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, types::Token::Number(1));
        assert_eq!(tokens[1].token, types::Token::Add);
        assert_eq!(tokens[2].token, types::Token::Number(2));
    }
}
