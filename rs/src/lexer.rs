use crate::types;

#[derive(Debug)]
pub struct LexerToken {
    pub token: types::Token,
    pub position: usize,
    pub line: usize,
}

pub fn tokenize(script: &str) -> Vec<LexerToken> {
    println!("Tokenize");

    let mut tokens: Vec<LexerToken> = vec![];
    let mut position = 0;
    let mut line_number = 1;

    while position < script.len() {
        if is_whitespace(script.chars().nth(position).unwrap()) {
            position += 1;
            continue;
        }

        let token_position = position;
        let (token, new_pos) = next_token(script, position);
        position = new_pos;

        let lexer_token = LexerToken {
            token: token,
            position: token_position,
            line: line_number,
        };

        tokens.push(lexer_token);
        if position < script.len() && script.chars().nth(position).unwrap() == '\n' {
            line_number += 1;
        }
    }

    tokens
}

fn next_token(script: &str, position: usize) -> (types::Token, usize) {
    let c = script.chars().nth(position).unwrap();
    if c.is_digit(10) {
        return lex_number(script, position);
    }

    if let Some(token) = match_operator(c) {
        return (token, position + 1);
    }

    (types::Token::Identifier(c.to_string()), position + 1)
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
        ';' => Some(types::Token::Semi),
        _ => None,
    }
}

fn lex_number(script: &str, position: usize) -> (types::Token, usize) {
    let mut number = String::new();
    let c = script.chars().nth(position).unwrap();
    number.push(c);

    let mut next_position = position + 1;
    while next_position < script.len() {
        let next_c = script.chars().nth(next_position).unwrap();
        if next_c.is_digit(10) {
            number.push(next_c);
            next_position += 1;
        } else {
            break;
        }
    }

    (
        types::Token::Number(number.parse::<i32>().unwrap()),
        next_position,
    )
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
        let (token, next_position) = lex_number("123", 0);
        assert_eq!(token, types::Token::Number(123));
        assert_eq!(next_position, 3);

        let (token, next_position) = lex_number("123abc", 0);
        assert_eq!(token, types::Token::Number(123));
        assert_eq!(next_position, 3);
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
