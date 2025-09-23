use crate::{span::Span, WithSpan};

#[derive(Debug)]
pub struct Item<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Item<T> {
    pub fn new(span: Span, value: T) -> Item<T> {
        Item { span, value }
    }
}

impl<T> WithSpan for Item<T> {
    fn span(&self) -> Span {
        self.span
    }
}
