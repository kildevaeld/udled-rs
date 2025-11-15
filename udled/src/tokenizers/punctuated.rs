use alloc::vec::Vec;

use crate::{tokenizers::Peek, Buffer, Error, Reader, Span, Tokenizer, WithSpan};

#[derive(Debug, Clone, Copy)]
pub enum PuntuatedItem<T, P> {
    Item(T),
    Punct(P),
}

impl<T: WithSpan, P: WithSpan> WithSpan for PuntuatedItem<T, P> {
    fn span(&self) -> Span {
        match self {
            PuntuatedItem::Item(item) => item.span(),
            PuntuatedItem::Punct(punct) => punct.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PunctuatedList<T, P> {
    list: Vec<PuntuatedItem<T, P>>,
    span: Span,
}

impl<T, P> PunctuatedList<T, P> {
    pub fn items(&self) -> impl Iterator<Item = &T> {
        self.list.iter().filter_map(|m| match m {
            PuntuatedItem::Item(item) => Some(item),
            _ => None,
        })
    }

    pub fn into_items(self) -> impl Iterator<Item = T> {
        self.list.into_iter().filter_map(|m| match m {
            PuntuatedItem::Item(item) => Some(item),
            _ => None,
        })
    }

    pub fn iter(&self) -> core::slice::Iter<'_, PuntuatedItem<T, P>> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

impl<T, P> WithSpan for PunctuatedList<T, P> {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Puntuated<T, P> {
    item: T,
    punct: P,
    non_empty: bool,
}

impl<T, P> Puntuated<T, P> {
    pub const fn new(item: T, punct: P) -> Puntuated<T, P> {
        Puntuated {
            item,
            punct,
            non_empty: false,
        }
    }
}

impl<'input, T, P, B> Tokenizer<'input, B> for Puntuated<T, P>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
    P: Tokenizer<'input, B>,
{
    type Token = PunctuatedList<T::Token, P::Token>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let mut output = Vec::new();

        if self.non_empty {
            let item = reader.parse(&self.item)?;
            output.push(PuntuatedItem::Item(item));
            if reader.is(Peek((&self.punct, &self.item))) {
                let punct = reader.parse(&self.punct)?;
                output.push(PuntuatedItem::Punct(punct));
            }
        }

        loop {
            if !reader.is(&self.item) {
                break;
            }

            let item = reader.parse(&self.item)?;

            output.push(PuntuatedItem::Item(item));

            if reader.is(Peek((&self.punct, &self.item))) {
                let punct = reader.parse(&self.punct)?;
                output.push(PuntuatedItem::Punct(punct));
            }
        }

        let end = reader.position();

        Ok(PunctuatedList {
            list: output,
            span: Span::new(start, end),
        })
    }
}
