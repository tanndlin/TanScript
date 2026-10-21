use crate::{
    ast::{OPERATORS, OperatorType},
    error::At,
    lex::{LexError, LexErrorKind, LexerAtomType, LexerToken, Tokens, cursor::Cursor},
    types::{NumberType, Span, Token},
};

pub fn lex(input: &str, file: u32) -> Result<Tokens, LexError> {
    let mut lexer = Lexer::new(input, file);
    let tokens = lexer.by_ref().collect::<Result<_, _>>()?;
    Ok(Tokens {
        tokens,
        eof: lexer.span(),
    })
}

pub struct Lexer<'a> {
    chars: Cursor<'a>,
    file: u32,
    start: Span,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, file: u32) -> Self {
        Lexer {
            chars: Cursor::new(input),
            file,
            start: Span::new(file, 0, 0),
        }
    }

    pub fn span(&self) -> Span {
        self.chars.span(self.file)
    }

    fn token(&mut self, cur: char) -> Result<Token, LexError> {
        Ok(if cur.is_ascii_digit() {
            Token::Atom(LexerAtomType::Number(self.number()?))
        } else if cur.is_alphabetic() || cur == '_' {
            let identifier = self.identifier();
            if identifier == OperatorType::Cast.symbol() {
                Token::Op(OperatorType::Cast)
            } else {
                Token::Atom(LexerAtomType::Identifier(identifier.to_string()))
            }
        } else if let Some(op) = self.operator() {
            Token::Op(op)
        } else {
            self.chars.next();
            Token::Atom(match cur {
                '"' => LexerAtomType::String(self.string()?),
                '\'' => LexerAtomType::Char(self.char()?),
                ';' => LexerAtomType::Semicolon,
                ':' => LexerAtomType::Colon,
                ',' => LexerAtomType::Comma,
                _ => return Err(LexErrorKind::UnknownCharacter(cur).at(self.start)),
            })
        })
    }

    fn number(&mut self) -> Result<NumberType, LexError> {
        let digits = self.chars.eat_while(|c| c.is_ascii_digit());
        let n = digits
            .parse::<i64>()
            .map_err(|_| LexErrorKind::NumberTooLarge(digits.to_string()).at(self.start))?;

        // TODO: Parse as coercible for now, but may want to delegate out actual number type later
        // if we want to support 0u32 type syntax
        Ok(NumberType::Coercible(n))
    }

    fn identifier(&mut self) -> &'a str {
        self.chars.eat_while(|c| c.is_alphanumeric() || c == '_')
    }

    fn string(&mut self) -> Result<String, LexError> {
        let mut string = String::new();
        loop {
            match self.chars.next() {
                None => return Err(LexErrorKind::MissingClosingQuote.at(self.start)),
                Some('"') => break,
                Some('\\') => string.push(self.escaped(LexErrorKind::MissingClosingQuote)?),
                Some(c) => string.push(c),
            }
        }

        Ok(string)
    }

    fn char(&mut self) -> Result<char, LexError> {
        let c = match self.chars.next() {
            None => return Err(LexErrorKind::MissingClosingCharQuote.at(self.start)),
            Some('\'') => return Err(LexErrorKind::EmptyChar.at(self.start)),
            Some('\\') => self.escaped(LexErrorKind::MissingClosingCharQuote)?,
            Some(c) => c,
        };

        match self.chars.next() {
            Some('\'') => Ok(c),
            _ => Err(LexErrorKind::MissingClosingCharQuote.at(self.start)),
        }
    }

    fn escaped(&mut self, missing: LexErrorKind) -> Result<char, LexError> {
        match self.chars.next().ok_or_else(|| missing.at(self.start))? {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            '0' => Ok('\0'),
            c @ ('\\' | '\'' | '"') => Ok(c),
            c => Err(LexErrorKind::UnknownEscape(c).at(self.start)),
        }
    }

    /// Consumes the longest operator at the front of the input, if there is one
    fn operator(&mut self) -> Option<OperatorType> {
        let (symbol, op) = OPERATORS
            .iter()
            .filter(|(symbol, _)| self.chars.rest().starts_with(symbol))
            .max_by_key(|(symbol, _)| symbol.len())?;

        self.chars.eat(symbol).then(|| op.clone())
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<LexerToken, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chars.eat_while(char::is_whitespace);
        let cur = self.chars.peek()?;
        self.start = self.span();
        Some(
            self.token(cur)
                .map(|token| LexerToken::new(token, self.start)),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::OperatorType,
        lex::{LexError, LexErrorKind, LexerAtomType, LexerToken, lexer::Lexer},
        types::{NumberType, Token},
    };

    fn tokenize(input: &str) -> Result<Vec<LexerToken>, LexError> {
        Lexer::new(input, 0).collect()
    }

    fn lex(input: &str) -> Vec<Token> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|token| token.token_type)
            .collect()
    }

    fn ident(s: &str) -> Token {
        Token::Atom(LexerAtomType::Identifier(s.to_string()))
    }

    fn num(n: i64) -> Token {
        Token::Atom(LexerAtomType::Number(NumberType::Coercible(n)))
    }

    fn string(s: &str) -> Token {
        Token::Atom(LexerAtomType::String(s.to_string()))
    }

    fn chr(c: char) -> Token {
        Token::Atom(LexerAtomType::Char(c))
    }

    fn op(op: OperatorType) -> Token {
        Token::Op(op)
    }

    const SEMI: Token = Token::Atom(LexerAtomType::Semicolon);

    #[test]
    fn lex_string() {
        assert_eq!(
            lex("\"This is a string 123\""),
            [string("This is a string 123")]
        );
    }

    #[test]
    fn lex_string_escaped_quotes() {
        assert_eq!(lex(r#""say \"hi\"""#), [string("say \"hi\"")]);
        assert_eq!(lex(r#""\"""#), [string("\"")]);
        assert_eq!(lex(r#""\"\"""#), [string("\"\"")]);
        assert_eq!(lex(r#""it's""#), [string("it's")]);
    }

    #[test]
    fn lex_string_escaped_backslash_before_quote() {
        assert_eq!(lex(r#""a\\""#), [string("a\\")]);
        assert_eq!(lex(r#""\\\"""#), [string("\\\"")]);
    }

    #[test]
    fn lex_string_escaped_quote_then_tokens() {
        assert_eq!(
            lex(r#"print("a\"b"); x"#),
            [
                ident("print"),
                op(OperatorType::OpenParen),
                string("a\"b"),
                op(OperatorType::CloseParen),
                SEMI,
                ident("x"),
            ]
        );
    }

    #[test]
    fn lex_adjacent_strings_with_escaped_quotes() {
        assert_eq!(lex(r#""\"""\"""#), [string("\""), string("\"")]);
    }

    #[test]
    fn lex_string_escaped_quote_columns() {
        let cols: Vec<u32> = tokenize(r#""a\"b" + x"#)
            .unwrap()
            .into_iter()
            .map(|token| token.span.col)
            .collect();
        assert_eq!(cols, vec![0, 7, 9]);
    }

    #[test]
    fn reject_string_ending_in_escaped_quote() {
        for input in [r#""\""#, r#""abc\""#, r#""a\"b"#] {
            let err = tokenize(input).unwrap_err();
            assert!(
                matches!(err.kind, LexErrorKind::MissingClosingQuote),
                "{input}"
            );
        }
    }

    #[test]
    fn lex_number() {
        assert_eq!(lex("123"), [num(123)]);
    }

    #[test]
    fn lex_identifier() {
        assert_eq!(lex("abc123"), [ident("abc123")]);
    }

    #[test]
    fn lex_identifier_with_underscores() {
        assert_eq!(
            lex("my_var _private __x1_"),
            [ident("my_var"), ident("_private"), ident("__x1_")]
        );
    }

    #[test]
    fn lex_basic_math() {
        assert_eq!(
            lex("abc123+def456"),
            [ident("abc123"), op(OperatorType::Add), ident("def456")]
        );
    }

    #[test]
    fn lex_declaration() {
        assert_eq!(
            lex("let a = 1;"),
            [
                ident("let"),
                ident("a"),
                op(OperatorType::Assign),
                num(1),
                SEMI
            ]
        );
    }

    #[test]
    fn lex_operators() {
        use OperatorType::*;

        for (input, expected) in [
            ("+", Add),
            ("-", Subtract),
            ("*", Multiply),
            ("/", Divide),
            ("%", Modulo),
            ("<", LessThan),
            ("<=", LessOrEqual),
            (">", GreaterThan),
            (">=", GreaterOrEqual),
            ("==", Equal),
            ("!=", NotEqual),
            ("!", Not),
            ("||", Or),
            ("&&", And),
            ("+=", AddAssign),
            ("-=", SubAssign),
            ("*=", MulAssign),
            ("/=", DivAssign),
            ("%=", ModAssign),
            ("++", Increment),
            ("--", Decrement),
        ] {
            assert_eq!(lex(input), [op(expected)], "{input}");
        }

        assert_eq!(
            lex("< = > = ! = = <= >= != =="),
            [
                op(LessThan),
                op(Assign),
                op(GreaterThan),
                op(Assign),
                op(Not),
                op(Assign),
                op(Assign),
                op(LessOrEqual),
                op(GreaterOrEqual),
                op(NotEqual),
                op(Equal),
            ]
        );
    }

    #[test]
    fn lex_compound_assignment_statement() {
        assert_eq!(
            lex("a += 1; b -= 2;"),
            [
                ident("a"),
                op(OperatorType::AddAssign),
                num(1),
                SEMI,
                ident("b"),
                op(OperatorType::SubAssign),
                num(2),
                SEMI,
            ]
        );
    }

    #[test]
    fn lex_compound_assignment_without_spaces() {
        assert_eq!(
            lex("a+=b-=c"),
            [
                ident("a"),
                op(OperatorType::AddAssign),
                ident("b"),
                op(OperatorType::SubAssign),
                ident("c"),
            ]
        );
    }

    #[test]
    fn lex_split_compound_assignment() {
        // Whitespace between the characters means two separate operators
        assert_eq!(
            lex("+ = - ="),
            [
                op(OperatorType::Add),
                op(OperatorType::Assign),
                op(OperatorType::Subtract),
                op(OperatorType::Assign),
            ]
        );
    }

    #[test]
    fn lex_sub_assign_negative_number() {
        assert_eq!(
            lex("a -= -1"),
            [
                ident("a"),
                op(OperatorType::SubAssign),
                op(OperatorType::Subtract),
                num(1),
            ]
        );
    }

    #[test]
    fn lex_increment_decrement_statements() {
        assert_eq!(
            lex("a++; b--;"),
            [
                ident("a"),
                op(OperatorType::Increment),
                SEMI,
                ident("b"),
                op(OperatorType::Decrement),
                SEMI,
            ]
        );
    }

    #[test]
    fn lex_split_increment_decrement() {
        // Whitespace between the characters means two separate operators
        assert_eq!(
            lex("+ + - -"),
            [
                op(OperatorType::Add),
                op(OperatorType::Add),
                op(OperatorType::Subtract),
                op(OperatorType::Subtract),
            ]
        );
    }

    #[test]
    fn lex_increment_is_greedy() {
        // Like C, `a+++b` is `a++ + b`
        assert_eq!(
            lex("a+++b"),
            [
                ident("a"),
                op(OperatorType::Increment),
                op(OperatorType::Add),
                ident("b"),
            ]
        );
    }

    #[test]
    fn lex_subtract_negative_needs_space() {
        assert_eq!(lex("5--3"), [num(5), op(OperatorType::Decrement), num(3)]);
        assert_eq!(
            lex("5 - -3"),
            [
                num(5),
                op(OperatorType::Subtract),
                op(OperatorType::Subtract),
                num(3),
            ]
        );
    }

    #[test]
    fn lex_semicolon() {
        assert_eq!(lex(";"), [SEMI]);
    }

    #[test]
    fn lex_number_in_whitespace() {
        assert_eq!(lex("   \n\t  42  "), [num(42)]);
    }

    #[test]
    fn lex_line_numbers() {
        let lines: Vec<u32> = tokenize("a;\n\nb;\r\nc")
            .unwrap()
            .into_iter()
            .map(|token| token.span.line)
            .collect();
        assert_eq!(lines, vec![0, 0, 2, 2, 3]);
    }

    #[test]
    fn lex_line_numbers_after_multiline_string() {
        let spans: Vec<(u32, u32)> = tokenize("\"a\nb\n\" x;")
            .unwrap()
            .into_iter()
            .map(|token| (token.span.line, token.span.col))
            .collect();
        assert_eq!(spans, vec![(0, 0), (2, 2), (2, 3)]);
    }

    #[test]
    fn lex_columns() {
        let spans: Vec<(u32, u32)> = tokenize("let x = 10;\n  foo(\"hi\");")
            .unwrap()
            .into_iter()
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
        for c in ['a', 'Z', '7', ' ', '"', 'é'] {
            assert_eq!(lex(&format!("'{c}'")), [chr(c)]);
        }
    }

    #[test]
    fn lex_char_operator_symbols() {
        for c in ['+', ';', '/'] {
            assert_eq!(lex(&format!("'{c}'")), [chr(c)]);
        }
    }

    #[test]
    fn lex_char_escapes() {
        assert_eq!(lex(r"'\n'"), [chr('\n')]);
        assert_eq!(lex(r"'\t'"), [chr('\t')]);
        assert_eq!(lex(r"'\r'"), [chr('\r')]);
        assert_eq!(lex(r"'\0'"), [chr('\0')]);
        assert_eq!(lex(r"'\\'"), [chr('\\')]);
        assert_eq!(lex(r"'\''"), [chr('\'')]);
    }

    #[test]
    fn lex_char_in_expression() {
        assert_eq!(
            lex("let c = 'x';"),
            [
                ident("let"),
                ident("c"),
                op(OperatorType::Assign),
                chr('x'),
                SEMI,
            ]
        );
    }

    #[test]
    fn lex_adjacent_chars() {
        assert_eq!(
            lex(r"'a''b'=='\''"),
            [chr('a'), chr('b'), op(OperatorType::Equal), chr('\'')]
        );
    }

    #[test]
    fn lex_char_columns() {
        let cols: Vec<u32> = tokenize(r"'a' + '\n'")
            .unwrap()
            .into_iter()
            .map(|token| token.span.col)
            .collect();
        assert_eq!(cols, vec![0, 4, 6]);
    }

    #[test]
    fn reject_number_too_large() {
        let err = tokenize("99999999999999999999").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::NumberTooLarge(_)));
    }

    #[test]
    fn reject_empty_char() {
        let err = tokenize("''").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::EmptyChar));
    }

    #[test]
    fn reject_unterminated_char() {
        for input in ["'", "'a", r"'\", r"'\n", "'ab'"] {
            let err = tokenize(input).unwrap_err();
            assert!(
                matches!(err.kind, LexErrorKind::MissingClosingCharQuote),
                "{input}"
            );
        }
    }

    #[test]
    fn reject_unknown_escape() {
        let err = tokenize(r"'\q'").unwrap_err();
        assert!(matches!(err.kind, LexErrorKind::UnknownEscape('q')));
    }

    #[test]
    fn reject_incomplete_operator() {
        assert!(tokenize("a | b").is_err());
    }

    fn lex_symbols(input: &str) -> Vec<String> {
        tokenize(input)
            .unwrap()
            .into_iter()
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
