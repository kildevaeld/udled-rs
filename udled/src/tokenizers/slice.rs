use crate::{AsSlice, Buffer, Error, Item, Reader, Spanned, Tokenizer};
use alloc::fmt;
use core::marker::PhantomData;

pub struct Sliced<T, B> {
    tokenizer: T,
    buffer: PhantomData<fn(B)>,
}

impl<T, B> Sliced<T, B> {
    pub fn new(tokenizer: T) -> Sliced<T, B> {
        Self {
            tokenizer,
            buffer: PhantomData,
        }
    }
}

impl<T: fmt::Debug, B> fmt::Debug for Sliced<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Sliced")
            .field("tokenizer", &self.tokenizer)
            .finish()
    }
}

impl<T: Clone, B> Clone for Sliced<T, B> {
    fn clone(&self) -> Self {
        Self {
            tokenizer: self.tokenizer.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, B> Copy for Sliced<T, B> {}

impl<'input, T, B> Tokenizer<'input, B> for Sliced<T, B>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Source: AsSlice<'input>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let span = reader.parse(Spanned::new(&self.tokenizer))?;

        match reader.buffer().source().sliced(span) {
            Some(slice) => Ok(Item::new(span, slice)),
            None => Err(reader.error("Could not compute slice")),
        }
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        self.tokenizer.eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }
}
