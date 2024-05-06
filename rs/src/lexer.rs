use crate::types;

#[derive(Debug)]
pub struct LexerToken {
    pub token: types::Token,
    pub position: usize,
    pub value: String,
}

pub fn tokenize(script: &str) -> Vec<LexerToken> {
    println!("Tokenize");

    let tokens: Vec<LexerToken> = vec![];
    let mut position = 0;

    while position < script.len() {
        let c = script.chars().nth(position).unwrap();
        if is_whitespace(c) {
            position += 1;
            continue;
        }

        if c.is_digit(10) {
            let mut number = String::new();
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

            let token = LexerToken {
                token: types::Token::Number(number.parse::<i32>().unwrap()),
                position: position,
                value: number,
            };

            println!("{:?}", token);

            position = next_position;
            continue;
        }

        if let Some(token) = char_to_token(c) {
            let token = LexerToken {
                token: token,
                position: position,
                value: c.to_string(),
            };

            println!("{:?}", token);

            position += 1;
            continue;
        }

        position += 1;
    }

    tokens
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\n' || c == '\t'
}

fn char_to_token(c: char) -> Option<types::Token> {
    match c {
        '+' => Some(types::Token::Add),
        '-' => Some(types::Token::Subtract),
        '*' => Some(types::Token::Multiply),
        '/' => Some(types::Token::Divide),
        _ => None,
    }
}
