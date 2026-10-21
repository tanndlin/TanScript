use std::str::Chars;

use crate::types::Span;

pub struct Cursor<'a> {
    chars: Chars<'a>,
    line: u32,
    col: u32,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Cursor {
            chars: input.chars(),
            line: 0,
            col: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    pub fn rest(&self) -> &'a str {
        self.chars.as_str()
    }

    pub fn next_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<char> {
        self.peek().filter(|&c| f(c))?;
        self.next()
    }

    pub fn eat_while(&mut self, f: impl Fn(char) -> bool) -> &'a str {
        let start = self.rest();
        while self.next_if(&f).is_some() {}
        &start[..start.len() - self.rest().len()]
    }

    pub fn eat(&mut self, prefix: &str) -> bool {
        if !self.rest().starts_with(prefix) {
            return false;
        }

        for _ in prefix.chars() {
            self.next();
        }
        true
    }

    pub fn span(&self, file: u32) -> Span {
        Span::new(file, self.line, self.col)
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
