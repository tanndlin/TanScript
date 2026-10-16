use crate::{
    ast::{OPERATORS, OperatorType},
    error::At,
    lex::{LexError, LexErrorKind, LexerAtomType, LexerToken},
    parse::{ParseError, ParseErrorKind},
    types::{NumberType, Span, Token},
};

#[derive(Debug)]
pub struct Lexer {
    pub tokens: Vec<LexerToken>,
    pub eof: Span,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Lexer, LexError> {
        let mut line_number = 0u32;
        let mut chars = input.chars().rev().collect::<Vec<char>>();
        let mut line_start = chars.len();

        let mut tokens = vec![];
        while let Some(&cur) = chars.last() {
            if cur == '\n' {
                line_number += 1;
                chars.pop();
                line_start = chars.len();
                continue;
            }

            if cur.is_whitespace() {
                chars.pop();
                continue;
            }

            let col = u32::try_from(line_start - chars.len()).unwrap_or(u32::MAX);
            let span = Span::new(line_number, col);

            if cur.is_ascii_digit() {
                tokens.push(LexerToken::new(
                    Token::Atom(LexerAtomType::Number(get_number(&mut chars))),
                    span,
                ));
                continue;
            }

            if cur.is_alphabetic() || cur == '_' {
                let identifier = get_identifier(&mut chars);
                let token = if identifier == OperatorType::Cast.symbol() {
                    Token::Op(OperatorType::Cast)
                } else {
                    Token::Atom(LexerAtomType::Identifier(identifier))
                };
                tokens.push(LexerToken::new(token, span));
                continue;
            }

            if let Some(op) = get_operator(&mut chars) {
                tokens.push(LexerToken::new(Token::Op(op), span));
                continue;
            }

            match cur {
                '"' => {
                    tokens.push(LexerToken::new(
                        Token::Atom(LexerAtomType::String(get_string(&mut chars, span)?)),
                        span,
                    ));
                }
                '\'' => {
                    tokens.push(LexerToken::new(
                        Token::Atom(LexerAtomType::Char(get_char(&mut chars, span)?)),
                        span,
                    ));
                }
                ';' => {
                    chars.pop();
                    tokens.push(LexerToken::new(Token::Atom(LexerAtomType::Semicolon), span));
                }
                ':' => {
                    chars.pop();
                    tokens.push(LexerToken::new(Token::Atom(LexerAtomType::Colon), span));
                }
                ',' => {
                    chars.pop();
                    tokens.push(LexerToken::new(Token::Atom(LexerAtomType::Comma), span));
                }
                _ => {
                    return Err(LexErrorKind::UnknownCharacter(cur).at(span));
                }
            }
        }

        tokens.reverse();
        let eof = Span::new(
            line_number,
            u32::try_from(line_start - chars.len()).unwrap_or(u32::MAX),
        );
        Ok(Lexer { tokens, eof })
    }

    pub fn next(&mut self) -> Option<LexerToken> {
        self.tokens.pop()
    }

    pub fn peek(&self) -> Option<&LexerToken> {
        self.tokens.last()
    }

    pub fn next_or_eof(&mut self, expected: &str) -> Result<LexerToken, ParseError> {
        self.next().ok_or_else(|| {
            ParseErrorKind::UnexpectedEof {
                expected: expected.to_string(),
            }
            .at(self.eof)
        })
    }

    pub fn peek_or_eof(&self, expected: &str) -> Result<&LexerToken, ParseError> {
        self.peek().ok_or_else(|| {
            ParseErrorKind::UnexpectedEof {
                expected: expected.to_string(),
            }
            .at(self.eof)
        })
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

fn get_number(input: &mut Vec<char>) -> NumberType {
    let mut chars = vec![];

    while let Some(c) = input.last() {
        if !c.is_ascii_digit() {
            break;
        }

        chars.push(input.pop().unwrap());
    }

    let n = chars
        .into_iter()
        .collect::<String>()
        .parse::<i64>()
        .unwrap();

    // TODO: Parse as coercible for now, but may want to delegate out actual number type later
    // if we want to support 0u32 type syntax
    NumberType::Coercible(n)
}

fn get_identifier(input: &mut Vec<char>) -> String {
    let mut string_vec = vec![];

    while let Some(c) = &input.last() {
        if !c.is_alphanumeric() && **c != '_' {
            break;
        }

        string_vec.push(input.pop().unwrap());
    }

    string_vec.into_iter().collect::<String>()
}

fn get_string(input: &mut Vec<char>, span: Span) -> Result<String, LexError> {
    // Remove the leading quote
    input.pop().unwrap();

    let mut chars = vec![];
    loop {
        match input.pop() {
            None => return Err(LexErrorKind::MissingClosingQuote.at(span)),
            Some('"') => break,
            Some('\\') => match input.pop() {
                Some(c) => chars
                    .push(get_escape(c).ok_or_else(|| LexErrorKind::UnknownEscape(c).at(span))?),
                None => return Err(LexErrorKind::MissingClosingQuote.at(span)),
            },
            Some(c) => chars.push(c),
        }
    }

    Ok(chars.into_iter().collect())
}

fn get_char(input: &mut Vec<char>, span: Span) -> Result<char, LexError> {
    input.pop().unwrap();

    let c = match input.pop() {
        None => return Err(LexErrorKind::MissingClosingCharQuote.at(span)),
        Some('\'') => return Err(LexErrorKind::EmptyChar.at(span)),
        Some('\\') => match input.pop() {
            Some(c) => get_escape(c).ok_or_else(|| LexErrorKind::UnknownEscape(c).at(span))?,
            None => return Err(LexErrorKind::MissingClosingCharQuote.at(span)),
        },
        Some(c) => c,
    };

    match input.pop() {
        Some('\'') => Ok(c),
        _ => Err(LexErrorKind::MissingClosingCharQuote.at(span)),
    }
}

fn get_escape(c: char) -> Option<char> {
    match c {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        '0' => Some('\0'),
        '\\' | '\'' | '"' => Some(c),
        _ => None,
    }
}

/// Pops the longest operator at the front of `chars`, if there is one
fn get_operator(chars: &mut Vec<char>) -> Option<OperatorType> {
    // `chars` is reversed, so the upcoming characters are at the end
    let upcoming = chars.iter().rev();
    let (symbol, op) = OPERATORS
        .iter()
        .filter(|(symbol, _)| symbol.chars().zip(upcoming.clone()).all(|(s, c)| s == *c))
        .filter(|(symbol, _)| symbol.len() <= chars.len())
        .max_by_key(|(symbol, _)| symbol.len())?;

    chars.truncate(chars.len() - symbol.len());
    Some(op.clone())
}

impl Iterator for Lexer {
    type Item = LexerToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::OperatorType,
        lex::{LexErrorKind, Lexer, LexerAtomType},
        types::{NumberType, Token},
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
    fn lex_string_escaped_quotes() {
        lex_token!(
            r#""say \"hi\"""#,
            Token::Atom(LexerAtomType::String("say \"hi\"".to_string()))
        );
        lex_token!(
            r#""\"""#,
            Token::Atom(LexerAtomType::String("\"".to_string()))
        );
        lex_token!(
            r#""\"\"""#,
            Token::Atom(LexerAtomType::String("\"\"".to_string()))
        );
        lex_token!(
            r#""it's""#,
            Token::Atom(LexerAtomType::String("it's".to_string()))
        );
    }

    #[test]
    fn lex_string_escaped_backslash_before_quote() {
        lex_token!(
            r#""a\\""#,
            Token::Atom(LexerAtomType::String("a\\".to_string()))
        );
        lex_token!(
            r#""\\\"""#,
            Token::Atom(LexerAtomType::String("\\\"".to_string()))
        );
    }

    #[test]
    fn lex_string_escaped_quote_then_tokens() {
        let mut lexer = Lexer::new(r#"print("a\"b"); x"#).unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("print".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::OpenParen));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::String("a\"b".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::CloseParen));
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("x".to_string()))
        );
    }

    #[test]
    fn lex_adjacent_strings_with_escaped_quotes() {
        let mut lexer = Lexer::new(r#""\"""\"""#).unwrap();
        token_eq!(lexer, Token::Atom(LexerAtomType::String("\"".to_string())));
        token_eq!(lexer, Token::Atom(LexerAtomType::String("\"".to_string())));
    }

    #[test]
    fn lex_string_escaped_quote_columns() {
        let mut lexer = Lexer::new(r#""a\"b" + x"#).unwrap();
        let cols: Vec<u32> = std::iter::from_fn(|| lexer.next())
            .map(|token| token.span.col)
            .collect();
        assert_eq!(cols, vec![0, 7, 9]);
    }

    #[test]
    fn reject_string_ending_in_escaped_quote() {
        for input in [r#""\""#, r#""abc\""#, r#""a\"b"#] {
            let err = Lexer::new(input).unwrap_err();
            assert!(
                matches!(err.kind, LexErrorKind::MissingClosingQuote),
                "{input}"
            );
        }
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
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(123)))
        );
    }

    #[test]
    fn lex_indentifier() {
        let mut lexer = Lexer::new("abc123").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("abc123".to_string()))
        );
    }

    #[test]
    fn lex_identifier_with_underscores() {
        let mut lexer = Lexer::new("my_var _private __x1_").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("my_var".to_string()))
        );
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("_private".to_string()))
        );
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("__x1_".to_string()))
        );
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
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(1)))
        );
    }

    #[test]
    fn lex_operators() {
        use OperatorType::*;

        lex_token!("+", Token::Op(Add));
        lex_token!("-", Token::Op(Subtract));
        lex_token!("*", Token::Op(Multiply));
        lex_token!("/", Token::Op(Divide));
        lex_token!("%", Token::Op(Modulo));
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
    fn lex_modulo() {
        lex_token!("%", Token::Op(OperatorType::Modulo));
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
    fn lex_logical_or() {
        lex_token!("||", Token::Op(OperatorType::Or));
    }

    #[test]
    fn lex_logical_and() {
        lex_token!("&&", Token::Op(OperatorType::And));
    }

    #[test]
    fn lex_add_assign() {
        lex_token!("+=", Token::Op(OperatorType::AddAssign));
    }

    #[test]
    fn lex_sub_assign() {
        lex_token!("-=", Token::Op(OperatorType::SubAssign));
    }

    #[test]
    fn lex_mul_assign() {
        lex_token!("*=", Token::Op(OperatorType::MulAssign));
    }

    #[test]
    fn lex_div_assign() {
        lex_token!("/=", Token::Op(OperatorType::DivAssign));
    }

    #[test]
    fn lex_mod_assign() {
        lex_token!("%=", Token::Op(OperatorType::ModAssign));
    }

    #[test]
    fn lex_compound_assignment_statement() {
        let mut lexer = Lexer::new("a += 1; b -= 2;").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::AddAssign));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(1)))
        );
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("b".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::SubAssign));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(2)))
        );
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
    }

    #[test]
    fn lex_compound_assignment_without_spaces() {
        let mut lexer = Lexer::new("a+=b-=c").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::AddAssign));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("b".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::SubAssign));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("c".to_string()))
        );
    }

    #[test]
    fn lex_split_compound_assignment() {
        // Whitespace between the characters means two separate operators
        let mut lexer = Lexer::new("+ = - =").unwrap();
        token_eq!(lexer, Token::Op(OperatorType::Add));
        token_eq!(lexer, Token::Op(OperatorType::Assign));
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
        token_eq!(lexer, Token::Op(OperatorType::Assign));
    }

    #[test]
    fn lex_sub_assign_negative_number() {
        let mut lexer = Lexer::new("a -= -1").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::SubAssign));
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(1)))
        );
    }

    #[test]
    fn lex_increment() {
        lex_token!("++", Token::Op(OperatorType::Increment));
    }

    #[test]
    fn lex_decrement() {
        lex_token!("--", Token::Op(OperatorType::Decrement));
    }

    #[test]
    fn lex_increment_decrement_statements() {
        let mut lexer = Lexer::new("a++; b--;").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Increment));
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("b".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Decrement));
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
    }

    #[test]
    fn lex_split_increment_decrement() {
        // Whitespace between the characters means two separate operators
        let mut lexer = Lexer::new("+ + - -").unwrap();
        token_eq!(lexer, Token::Op(OperatorType::Add));
        token_eq!(lexer, Token::Op(OperatorType::Add));
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
    }

    #[test]
    fn lex_increment_is_greedy() {
        // Like C, `a+++b` is `a++ + b`
        let mut lexer = Lexer::new("a+++b").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("a".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Increment));
        token_eq!(lexer, Token::Op(OperatorType::Add));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("b".to_string()))
        );
    }

    #[test]
    fn lex_subtract_negative_needs_space() {
        let mut lexer = Lexer::new("5--3").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(5)))
        );
        token_eq!(lexer, Token::Op(OperatorType::Decrement));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(3)))
        );

        let mut lexer = Lexer::new("5 - -3").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(5)))
        );
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
        token_eq!(lexer, Token::Op(OperatorType::Subtract));
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(3)))
        );
    }

    #[test]
    fn lex_semicolon() {
        lex_token!(";", Token::Atom(LexerAtomType::Semicolon));
    }

    #[test]
    fn lex_number_in_whitespace() {
        lex_token!(
            "   \n\t  42  ",
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(42)))
        );
    }

    #[test]
    fn lex_line_numbers() {
        let mut lexer = Lexer::new("a;\n\nb;\r\nc").unwrap();
        let lines: Vec<u32> = std::iter::from_fn(|| lexer.next())
            .map(|token| token.span.line)
            .collect();
        assert_eq!(lines, vec![0, 0, 2, 2, 3]);
    }

    #[test]
    fn lex_columns() {
        let mut lexer = Lexer::new("let x = 10;\n  foo(\"hi\");").unwrap();
        let spans: Vec<(u32, u32)> = std::iter::from_fn(|| lexer.next())
            .map(|token| (token.span.line, token.span.col))
            .collect();
        assert_eq!(
            spans,
            vec![
                (0, 0),
                (0, 4),
                (0, 6),
                (0, 8),
                (0, 10),
                (1, 2),
                (1, 5),
                (1, 6),
                (1, 10),
                (1, 11),
            ]
        );
    }

    #[test]
    fn lex_char() {
        lex_token!("'a'", Token::Atom(LexerAtomType::Char('a')));
        lex_token!("'Z'", Token::Atom(LexerAtomType::Char('Z')));
        lex_token!("'7'", Token::Atom(LexerAtomType::Char('7')));
        lex_token!("' '", Token::Atom(LexerAtomType::Char(' ')));
        lex_token!("'\"'", Token::Atom(LexerAtomType::Char('"')));
        lex_token!("'é'", Token::Atom(LexerAtomType::Char('é')));
    }

    #[test]
    fn lex_char_operator_symbols() {
        lex_token!("'+'", Token::Atom(LexerAtomType::Char('+')));
        lex_token!("';'", Token::Atom(LexerAtomType::Char(';')));
        lex_token!("'/'", Token::Atom(LexerAtomType::Char('/')));
    }

    #[test]
    fn lex_char_escapes() {
        lex_token!(r"'\n'", Token::Atom(LexerAtomType::Char('\n')));
        lex_token!(r"'\t'", Token::Atom(LexerAtomType::Char('\t')));
        lex_token!(r"'\r'", Token::Atom(LexerAtomType::Char('\r')));
        lex_token!(r"'\0'", Token::Atom(LexerAtomType::Char('\0')));
        lex_token!(r"'\\'", Token::Atom(LexerAtomType::Char('\\')));
        lex_token!(r"'\''", Token::Atom(LexerAtomType::Char('\'')));
    }

    #[test]
    fn lex_char_in_expression() {
        let mut lexer = Lexer::new("let c = 'x';").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("let".to_string()))
        );
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Identifier("c".to_string()))
        );
        token_eq!(lexer, Token::Op(OperatorType::Assign));
        token_eq!(lexer, Token::Atom(LexerAtomType::Char('x')));
        token_eq!(lexer, Token::Atom(LexerAtomType::Semicolon));
    }

    #[test]
    fn lex_adjacent_chars() {
        let mut lexer = Lexer::new(r"'a''b'=='\''").unwrap();
        token_eq!(lexer, Token::Atom(LexerAtomType::Char('a')));
        token_eq!(lexer, Token::Atom(LexerAtomType::Char('b')));
        token_eq!(lexer, Token::Op(OperatorType::Equal));
        token_eq!(lexer, Token::Atom(LexerAtomType::Char('\'')));
    }

    #[test]
    fn lex_char_columns() {
        let mut lexer = Lexer::new(r"'a' + '\n'").unwrap();
        let cols: Vec<u32> = std::iter::from_fn(|| lexer.next())
            .map(|token| token.span.col)
            .collect();
        assert_eq!(cols, vec![0, 4, 6]);
    }

    #[test]
    fn reject_empty_char() {
        let err = Lexer::new("''").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::EmptyChar));
    }

    #[test]
    fn reject_unterminated_char() {
        for input in ["'", "'a", r"'\", r"'\n", "'ab'"] {
            let err = Lexer::new(input).unwrap_err();
            assert!(
                matches!(err.kind, LexErrorKind::MissingClosingCharQuote),
                "{input}"
            );
        }
    }

    #[test]
    fn reject_unknown_escape() {
        let err = Lexer::new(r"'\q'").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::UnknownEscape('q')));
    }

    #[test]
    fn reject_incomplete_operator() {
        assert!(Lexer::new("a | b").is_err());
        assert!(Lexer::new("a & b").is_err());
    }
}
