use core::fmt;

use crate::span::{Span, WithSpan};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Eq, PartialOrd, Ord)]
pub struct Lex<'a> {
    pub value: &'a str,
    pub span: Span,
}

impl<'a> PartialEq for Lex<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<'a> Lex<'a> {
    pub const fn new(value: &'a str, span: Span) -> Lex<'a> {
        Lex { value, span }
    }

    pub fn as_str(&self) -> &'a str {
        self.value
    }
}

impl<'a, 'b> PartialEq<&'b str> for Lex<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> AsRef<str> for Lex<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> WithSpan for Lex<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> fmt::Display for Lex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Item<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Item<T> {
    pub const fn new(value: T, span: Span) -> Item<T> {
        Item { value, span }
    }
}

impl<T> AsRef<T> for Item<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T: PartialEq> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Item<T> {}

impl<T> WithSpan for Item<T> {
    fn span(&self) -> Span {
        self.span
    }
}
