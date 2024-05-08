use crate::types;
use crate::types::Token;

#[derive(Debug, Clone)]
pub struct LexerToken {
    pub token: Token,
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
        if position < script.len() && is_newline(script.chars().nth(position).unwrap()) {
            line_number += 1;
        }
    }

    tokens
}

fn next_token(script: &str, position: &mut usize) -> Token {
    let c = script.chars().nth(*position).unwrap();
    if c.is_digit(10) {
        return lex_number(script, position);
    }

    if let Some(token) = match_operator(&script, position) {
        return token;
    }

    if !c.is_alphanumeric() {
        panic!("Unexpected character: {}", c);
    }

    let possibly_ident = lex_identifier(script, position);
    if let Some(keyword) = types::Keywords::from_string(&possibly_ident) {
        return keyword.to_token();
    }

    Token::Identifier(possibly_ident.to_string())
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
    c == ' ' || c == '\n' || c == '\t' || c == '\r'
}

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn match_operator(script: &str, position: &mut usize) -> Option<Token> {
    let c = script.chars().nth(*position).unwrap();

    // Try double char operators first
    if let Some(token) = match_double_char_op(script, position) {
        return Some(token);
    }

    let ret = match c {
        '+' => Some(Token::Operator(types::Operator::Add)),
        '-' => Some(Token::Operator(types::Operator::Subtract)),
        '*' => Some(Token::Operator(types::Operator::Multiply)),
        '/' => Some(Token::Operator(types::Operator::Divide)),
        '%' => Some(Token::Operator(types::Operator::Mod)),
        '<' => Some(Token::LessThan),
        '>' => Some(Token::GreaterThan),
        '!' => Some(Token::Not),
        ';' => Some(Token::Semi),
        '=' => Some(Token::Assign),
        '(' => Some(Token::LParen),
        ')' => Some(Token::RParen),
        '{' => Some(Token::LCurly),
        '}' => Some(Token::RCurly),
        ',' => Some(Token::Comma),
        _ => None,
    };

    if ret.is_some() {
        *position += 1;
    }

    ret
}

fn match_double_char_op(script: &str, position: &mut usize) -> Option<Token> {
    if (*position + 1) >= script.len() {
        return None;
    }

    let c = script.chars().nth(*position).unwrap();
    let next_c = script.chars().nth(*position + 1).unwrap();
    let matched = match (c, next_c) {
        ('=', '=') => Some(Token::Eq),
        ('!', '=') => Some(Token::NotEq),
        ('<', '=') => Some(Token::Leq),
        ('>', '=') => Some(Token::Geq),
        ('&', '&') => Some(Token::And),
        ('|', '|') => Some(Token::Or),
        ('+', '=') => Some(Token::ShortAssign(types::Operator::Add)),
        ('-', '=') => Some(Token::ShortAssign(types::Operator::Subtract)),
        ('*', '=') => Some(Token::ShortAssign(types::Operator::Multiply)),
        ('/', '=') => Some(Token::ShortAssign(types::Operator::Divide)),
        ('%', '=') => Some(Token::ShortAssign(types::Operator::Mod)),
        ('+', '+') => Some(Token::Increment),
        ('-', '-') => Some(Token::Decrement),
        _ => None,
    };

    if matched.is_some() {
        *position += 2;
    }

    matched
}

fn lex_number(script: &str, position: &mut usize) -> Token {
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

    Token::Number(number.parse::<i32>().unwrap())
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
        assert_eq!(
            match_operator("+", &mut 0),
            Some(Token::Operator(types::Operator::Add))
        );
        assert_eq!(
            match_operator("-", &mut 0),
            Some(Token::Operator(types::Operator::Subtract))
        );
        assert_eq!(
            match_operator("*", &mut 0),
            Some(Token::Operator(types::Operator::Multiply))
        );
        assert_eq!(
            match_operator("/", &mut 0),
            Some(Token::Operator(types::Operator::Divide))
        );
        assert_eq!(
            match_operator("%", &mut 0),
            Some(Token::Operator(types::Operator::Mod))
        );
        assert_eq!(match_operator("a", &mut 0), None);
    }

    #[test]
    fn test_lex_number() {
        let token = lex_number("123", &mut 0);
        assert_eq!(token, Token::Number(123));

        let token = lex_number("123abc", &mut 0);
        assert_eq!(token, Token::Number(123));
    }

    #[test]
    fn lex_simple_expression() {
        let tokens = tokenize("1 + 2");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token, Token::Number(1));
        assert_eq!(tokens[1].token, Token::Operator(types::Operator::Add));
        assert_eq!(tokens[2].token, Token::Number(2));
    }
}
