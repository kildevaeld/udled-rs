use crate::{span::Span, WithSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Item<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Item<T> {
    pub fn new(span: Span, value: T) -> Item<T> {
        Item { span, value }
    }

    pub fn map<F, U>(self, map: F) -> Item<U>
    where
        F: FnOnce(T) -> U,
    {
        Item {
            span: self.span,
            value: map(self.value),
        }
    }
}

impl<T> WithSpan for Item<T> {
    fn span(&self) -> Span {
        self.span
    }
}
