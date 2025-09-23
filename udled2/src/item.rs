use crate::span::Span;

#[derive(Debug)]
pub struct Item<T> {
    pub span: Span,
    pub value: T,
}
