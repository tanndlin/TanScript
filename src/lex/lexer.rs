use std::str::Chars;

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
    pub depth: u32,
}

pub const MAX_DEPTH: u32 = 50;

impl Lexer {
    #[cfg(test)]
    pub fn new(input: &str) -> Result<Lexer, LexError> {
        Self::for_file(input, 0)
    }

    pub fn for_file(input: &str, file: u32) -> Result<Lexer, LexError> {
        let mut chars = Cursor::new(input);

        let mut tokens = vec![];
        while let Some(cur) = chars.peek() {
            if cur.is_whitespace() {
                chars.next();
                continue;
            }

            let span = Span::new(file, chars.line, chars.col);

            let token = if cur.is_ascii_digit() {
                Token::Atom(LexerAtomType::Number(get_number(&mut chars, span)?))
            } else if cur.is_alphabetic() || cur == '_' {
                let identifier = get_identifier(&mut chars);
                if identifier == OperatorType::Cast.symbol() {
                    Token::Op(OperatorType::Cast)
                } else {
                    Token::Atom(LexerAtomType::Identifier(identifier))
                }
            } else if let Some(op) = get_operator(&mut chars) {
                Token::Op(op)
            } else {
                chars.next();
                Token::Atom(match cur {
                    '"' => LexerAtomType::String(get_string(&mut chars, span)?),
                    '\'' => LexerAtomType::Char(get_char(&mut chars, span)?),
                    ';' => LexerAtomType::Semicolon,
                    ':' => LexerAtomType::Colon,
                    ',' => LexerAtomType::Comma,
                    _ => return Err(LexErrorKind::UnknownCharacter(cur).at(span)),
                })
            };

            tokens.push(LexerToken::new(token, span));
        }

        tokens.reverse();
        let eof = Span::new(file, chars.line, chars.col);
        Ok(Lexer {
            tokens,
            eof,
            depth: 0,
        })
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
        self.tokens.last()
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

struct Cursor<'a> {
    chars: Chars<'a>,
    line: u32,
    col: u32,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        Cursor {
            chars: input.chars(),
            line: 0,
            col: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn rest(&self) -> &'a str {
        self.chars.as_str()
    }

    fn next_if(&mut self, f: impl Fn(char) -> bool) -> Option<char> {
        self.peek().filter(|&c| f(c))?;
        self.next()
    }

    fn eat_while(&mut self, f: impl Fn(char) -> bool) -> String {
        std::iter::from_fn(|| self.next_if(&f)).collect()
    }
}

impl Iterator for Cursor<'_> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Some(c)
    }
}

fn get_number(input: &mut Cursor, span: Span) -> Result<NumberType, LexError> {
    let digits = input.eat_while(|c| c.is_ascii_digit());
    let n = digits
        .parse::<i64>()
        .map_err(|_| LexErrorKind::NumberTooLarge(digits).at(span))?;

    // TODO: Parse as coercible for now, but may want to delegate out actual number type later
    // if we want to support 0u32 type syntax
    Ok(NumberType::Coercible(n))
}

fn get_identifier(input: &mut Cursor) -> String {
    input.eat_while(|c| c.is_alphanumeric() || c == '_')
}

fn get_string(input: &mut Cursor, span: Span) -> Result<String, LexError> {
    let mut string = String::new();
    loop {
        match input.next() {
            None => return Err(LexErrorKind::MissingClosingQuote.at(span)),
            Some('"') => break,
            Some('\\') => {
                string.push(get_escaped(input, span, LexErrorKind::MissingClosingQuote)?);
            }
            Some(c) => string.push(c),
        }
    }

    Ok(string)
}

fn get_char(input: &mut Cursor, span: Span) -> Result<char, LexError> {
    let c = match input.next() {
        None => return Err(LexErrorKind::MissingClosingCharQuote.at(span)),
        Some('\'') => return Err(LexErrorKind::EmptyChar.at(span)),
        Some('\\') => get_escaped(input, span, LexErrorKind::MissingClosingCharQuote)?,
        Some(c) => c,
    };

    match input.next() {
        Some('\'') => Ok(c),
        _ => Err(LexErrorKind::MissingClosingCharQuote.at(span)),
    }
}

fn get_escaped(input: &mut Cursor, span: Span, missing: LexErrorKind) -> Result<char, LexError> {
    let c = input.next().ok_or_else(|| missing.at(span))?;
    get_escape(c).ok_or_else(|| LexErrorKind::UnknownEscape(c).at(span))
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

/// Consumes the longest operator at the front of `chars`, if there is one
fn get_operator(chars: &mut Cursor) -> Option<OperatorType> {
    let (symbol, op) = OPERATORS
        .iter()
        .filter(|(symbol, _)| chars.rest().starts_with(symbol))
        .max_by_key(|(symbol, _)| symbol.len())?;

    chars.by_ref().take(symbol.chars().count()).for_each(drop);
    Some(op.clone())
}

impl Iterator for Lexer {
    type Item = LexerToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.tokens.pop()
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
        let cols: Vec<u32> = Lexer::new(r#""a\"b" + x"#)
            .unwrap()
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
    fn lex_number() {
        let mut lexer = Lexer::new("123").unwrap();
        token_eq!(
            lexer,
            Token::Atom(LexerAtomType::Number(NumberType::Coercible(123)))
        );
    }

    #[test]
    fn lex_identifier() {
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
        lex_token!("||", Token::Op(Or));
        lex_token!("&&", Token::Op(And));
        lex_token!("+=", Token::Op(AddAssign));
        lex_token!("-=", Token::Op(SubAssign));
        lex_token!("*=", Token::Op(MulAssign));
        lex_token!("/=", Token::Op(DivAssign));
        lex_token!("%=", Token::Op(ModAssign));
        lex_token!("++", Token::Op(Increment));
        lex_token!("--", Token::Op(Decrement));

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
        let lines: Vec<u32> = Lexer::new("a;\n\nb;\r\nc")
            .unwrap()
            .map(|token| token.span.line)
            .collect();
        assert_eq!(lines, vec![0, 0, 2, 2, 3]);
    }

    #[test]
    fn lex_line_numbers_after_multiline_string() {
        let spans: Vec<(u32, u32)> = Lexer::new("\"a\nb\n\" x;")
            .unwrap()
            .map(|token| (token.span.line, token.span.col))
            .collect();
        assert_eq!(spans, vec![(0, 0), (2, 2), (2, 3)]);
    }

    #[test]
    fn lex_columns() {
        let spans: Vec<(u32, u32)> = Lexer::new("let x = 10;\n  foo(\"hi\");")
            .unwrap()
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
        let cols: Vec<u32> = Lexer::new(r"'a' + '\n'")
            .unwrap()
            .map(|token| token.span.col)
            .collect();
        assert_eq!(cols, vec![0, 4, 6]);
    }

    #[test]
    fn reject_number_too_large() {
        let err = Lexer::new("99999999999999999999").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::NumberTooLarge(_)));
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
    }

    fn lex_symbols(input: &str) -> Vec<String> {
        Lexer::new(input)
            .unwrap()
            .map(|token| token.token_type.to_string())
            .collect()
    }

    #[test]
    fn lex_address_of() {
        assert_eq!(lex_symbols("&x"), ["&", "x"]);
        assert_eq!(lex_symbols("& x"), ["&", "x"]);
        assert_eq!(
            lex_symbols("f(&x, &y)"),
            ["f", "(", "&", "x", ",", "&", "y", ")"]
        );
    }

    #[test]
    fn lex_logical_and_is_greedy_over_address_of() {
        assert_eq!(lex_symbols("a && b"), ["a", "&&", "b"]);
        assert_eq!(lex_symbols("a && &b"), ["a", "&&", "&", "b"]);
    }

    #[test]
    fn lex_deref() {
        assert_eq!(lex_symbols("*p"), ["*", "p"]);
        assert_eq!(lex_symbols("**pp"), ["*", "*", "pp"]);
        assert_eq!(lex_symbols("(*p).age"), ["(", "*", "p", ")", ".", "age"]);
    }
}
