use core::marker::PhantomData;

use alloc::fmt;

use crate::{Buffer, Error, Reader, Span, Tokenizer};

pub const fn spanned<T, B>(tokenizer: T) -> Spanned<T, B> {
    Spanned::new(tokenizer)
}

pub struct Spanned<T, B> {
    tokenizer: T,
    buffer: PhantomData<fn(B)>,
}

impl<T, B> Spanned<T, B> {
    pub const fn new(tokenizer: T) -> Spanned<T, B> {
        Spanned {
            tokenizer,
            buffer: PhantomData,
        }
    }
}

impl<T: Clone, B> Clone for Spanned<T, B> {
    fn clone(&self) -> Self {
        Spanned {
            tokenizer: self.tokenizer.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, B> Copy for Spanned<T, B> {}

impl<T: fmt::Debug, B> fmt::Debug for Spanned<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Spanned")
            .field("tokenizer", &self.tokenizer)
            .finish()
    }
}

impl<'input, T, B> Tokenizer<'input, B> for Spanned<T, B>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = Span;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        self.tokenizer.eat(reader)?;
        let end = reader.position();
        Ok(Span::new(start, end))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        self.tokenizer.eat(reader)
    }
}
