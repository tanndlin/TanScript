use crate::{
    ast::OperatorType,
    types::{LexerAtomType, LexerToken, Token},
};

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<LexerToken>,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Lexer, String> {
        let mut line_number = 0u32;
        let mut chars = input.chars().rev().collect::<Vec<char>>();

        let mut tokens = vec![];
        while let Some(cur) = chars.last() {
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
                '"' => {
                    tokens.push(LexerToken::new(
                        Token::Atom(LexerAtomType::String(get_string(&mut chars)?)),
                        line_number,
                    ));
                }
                ';' => {
                    chars.pop();
                    tokens.push(LexerToken::new(
                        Token::Atom(LexerAtomType::Semicolon),
                        line_number,
                    ));
                }
                '+' | '-' | '*' | '/' | '=' | '(' | ')' | ',' | '}' | '{' | '<' | '>' | '!' => {
                    let op = get_operator(&mut chars);
                    tokens.push(LexerToken::new(Token::Op(op), line_number));
                }
                '\n' => {
                    line_number += 1;
                    chars.pop();
                }
                _ => {
                    return Err(format!(
                        "Lexer: Unknown character: {} on line: {}",
                        cur, line_number
                    ));
                }
            }
        }

        tokens.reverse();
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

    string_vec.into_iter().collect::<String>()
}

fn get_string(input: &mut Vec<char>) -> Result<String, String> {
    // Remove the leading quote
    let popped = input.pop();
    if popped != Some('"') {
        if let Some(c) = popped {
            Err(format!("Expected a starting quote. Got: {}", c).to_string())
        } else {
            Err("Expected a starting quote".to_string())
        }?;
    };

    let mut chars = vec![];

    while let Some(c) = input.last() {
        if *c == '"' {
            break;
        }

        chars.push(*c);
        input.pop();
    }

    // Remove the closing quote
    input
        .pop()
        .ok_or_else(|| "No closing quote found for string".to_string())?;

    Ok(chars.into_iter().collect())
}

fn get_operator(chars: &mut Vec<char>) -> OperatorType {
    let cur = chars
        .pop()
        .expect("Expected an operator, ran out of tokens");
    if let Some(next) = chars.last()
        && let Some(compound_op) = match format!("{}{}", cur, next).as_str() {
            "<=" => Some(OperatorType::LessOrEqual),
            ">=" => Some(OperatorType::GreaterOrEqual),
            "==" => Some(OperatorType::Equal),
            "!=" => Some(OperatorType::NotEqual),
            _ => None,
        }
    {
        chars.pop();
        compound_op
    } else {
        OperatorType::from_char(cur)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::OperatorType,
        lexer::Lexer,
        types::{LexerAtomType, Token},
    };

    macro_rules! token_eq {
        ($lexer:expr, $expected:expr) => {{
            assert!($lexer.tokens.len() > 0);
            let token = $lexer.tokens.pop().unwrap();
            assert_eq!(token.token_type, $expected);
        }};
    }

    macro_rules! lex_token {
        ($input:expr, $expected:expr) => {
            let lexer = Lexer::new($input).unwrap();
            let token = lexer.peek().unwrap();
            assert_eq!(token.token_type, $expected)
        };
    }

    #[test]
    fn lex_string() {
        let mut lexer = Lexer::new("\"This is a string 123\"").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::String("This is a string 123".to_string()))
        );
    }

    #[test]
    fn test_get_identifier() {
        let mut lexer = Lexer::new("variableName123").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("variableName123".to_string()))
        );
    }

    #[test]
    fn lex_number() {
        let mut lexer = Lexer::new("123").unwrap();
        token_eq!(lexer, Token::Atom(LexerAtomType::Number(123)));
    }

    #[test]
    fn lex_indentifier() {
        let mut lexer = Lexer::new("abc123").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("abc123".to_string()))
        )
    }

    #[test]
    fn lex_basic_math() {
        let mut lexer = Lexer::new("abc123+def456").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("abc123".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Add));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("def456".to_string()))
        );
    }

    #[test]
    fn lex_declaration() {
        let mut lexer = Lexer::new("let a = 1;").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("let".to_string()))
        );
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Assign));
        token_eq!(lexer, Token::Atom(LexerAtomType::Number(1)));
    }

    #[test]
    fn lex_operators() {
        use OperatorType::*;

        lex_token!("+", Token::Op(Add));
        lex_token!("-", Token::Op(Subtract));
        lex_token!("*", Token::Op(Multiply));
        lex_token!("/", Token::Op(Divide));
        lex_token!("<", Token::Op(LessThan));
        lex_token!("<=", Token::Op(LessOrEqual));
        lex_token!(">", Token::Op(GreaterThan));
        lex_token!(">=", Token::Op(GreaterOrEqual));
        lex_token!("==", Token::Op(Equal));
        lex_token!("!=", Token::Op(NotEqual));
        lex_token!("!", Token::Op(Not));

        let mut lexer = Lexer::new("< = > = ! = = <= >= != ==").unwrap();
        token_eq!(lexer, Token::Op(LessThan));
        token_eq!(lexer, Token::Op(Assign));
        token_eq!(lexer, Token::Op(GreaterThan));
        token_eq!(lexer, Token::Op(Assign));
        token_eq!(lexer, Token::Op(Not));
        token_eq!(lexer, Token::Op(Assign));
        token_eq!(lexer, Token::Op(Assign));
        token_eq!(lexer, Token::Op(LessOrEqual));
        token_eq!(lexer, Token::Op(GreaterOrEqual));
        token_eq!(lexer, Token::Op(NotEqual));
        token_eq!(lexer, Token::Op(Equal));
    }

    #[test]
    fn lex_add() {
        lex_token!("+", Token::Op(OperatorType::Add));
    }

    #[test]
    fn lex_subtract() {
        lex_token!("-", Token::Op(OperatorType::Subtract));
    }

    #[test]
    fn lex_multiply() {
        lex_token!("*", Token::Op(OperatorType::Multiply));
    }

    #[test]
    fn lex_divide() {
        lex_token!("/", Token::Op(OperatorType::Divide));
    }

    #[test]
    fn lex_less_than() {
        lex_token!("<", Token::Op(OperatorType::LessThan));
    }

    #[test]
    fn lex_less_or_equal() {
        lex_token!("<=", Token::Op(OperatorType::LessOrEqual));
    }

    #[test]
    fn lex_greater_than() {
        lex_token!(">", Token::Op(OperatorType::GreaterThan));
    }

    #[test]
    fn lex_greater_or_equal() {
        lex_token!(">=", Token::Op(OperatorType::GreaterOrEqual));
    }

    #[test]
    fn lex_not() {
        lex_token!("!", Token::Op(OperatorType::Not));
    }

    #[test]
    fn lex_equal() {
        lex_token!("==", Token::Op(OperatorType::Equal));
    }

    #[test]
    fn lex_not_equal() {
        lex_token!("!=", Token::Op(OperatorType::NotEqual));
    }

    #[test]
    fn lex_semicolon() {
        lex_token!(";", Token::Atom(LexerAtomType::Semicolon));
    }

    #[test]
    fn lex_number_in_whitespace() {
        lex_token!("   \n\t  42  ", Token::Atom(LexerAtomType::Number(42)));
    }
}
