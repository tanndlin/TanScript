use std::{error::Error, fmt};

use crate::{
    ast::{ParseError, ParseErrorKind},
    codegen::{CodegenError, CodegenErrorKind},
    lex::{LexError, LexErrorKind},
    type_check::{TypeCheckError, TypeCheckErrorKind},
    types::Span,
};

pub struct Spanned<K> {
    pub kind: K,
    pub span: Span,
}

impl<K> Spanned<K> {
    pub fn map<K2>(self, f: impl FnOnce(K) -> K2) -> Spanned<K2> {
        Spanned {
            kind: f(self.kind),
            span: self.span,
        }
    }
}

impl<K: fmt::Display> fmt::Display for Spanned<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.kind, self.span)
    }
}

impl<K: fmt::Display> fmt::Debug for Spanned<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<K: fmt::Display> Error for Spanned<K> {}

pub trait At: Sized {
    fn at(self, span: Span) -> Spanned<Self> {
        Spanned { kind: self, span }
    }
}

pub trait ResultAt<T, K> {
    fn at(self, span: Span) -> Result<T, Spanned<K>>;
}

impl<T, K: At> ResultAt<T, K> for Result<T, K> {
    fn at(self, span: Span) -> Result<T, Spanned<K>> {
        self.map_err(|kind| kind.at(span))
    }
}

#[derive(Debug)]
pub enum CompilerErrorKind {
    Lex(LexErrorKind),
    Parse(ParseErrorKind),
    TypeCheck(TypeCheckErrorKind),
    Codegen(CodegenErrorKind),
}

pub type CompilerError = Spanned<CompilerErrorKind>;

impl fmt::Display for CompilerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerErrorKind::Lex(kind) => write!(f, "{kind}"),
            CompilerErrorKind::Parse(kind) => write!(f, "{kind}"),
            CompilerErrorKind::TypeCheck(kind) => write!(f, "{kind}"),
            CompilerErrorKind::Codegen(kind) => write!(f, "{kind}"),
        }
    }
}

impl From<LexError> for CompilerError {
    fn from(e: LexError) -> Self {
        e.map(CompilerErrorKind::Lex)
    }
}

impl From<ParseError> for CompilerError {
    fn from(e: ParseError) -> Self {
        e.map(CompilerErrorKind::Parse)
    }
}

impl From<TypeCheckError> for CompilerError {
    fn from(e: TypeCheckError) -> Self {
        e.map(CompilerErrorKind::TypeCheck)
    }
}

impl From<CodegenError> for CompilerError {
    fn from(e: CodegenError) -> Self {
        e.map(CompilerErrorKind::Codegen)
    }
}
