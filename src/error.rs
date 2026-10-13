use std::{error::Error, fmt};

use crate::{type_check::TypeCheckErrorKind, types::Span};

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

#[derive(Debug)]
pub enum CompilerErrorKind {
    Parse(ParseErrorKind),
    TypeCheck(TypeCheckErrorKind),
    Codegen(CodegenErrorKind),
}

pub type CompilerError = Spanned<CompilerErrorKind>;
