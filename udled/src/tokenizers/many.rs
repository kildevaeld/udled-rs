use core::marker::PhantomData;

use alloc::{fmt, vec, vec::Vec};

use crate::{Buffer, Error, Item, Reader, Span, Tokenizer, EOF};

pub const fn many<T, B>(tokenizer: T) -> Many<T, B> {
    Many::new(tokenizer)
}

pub struct Many<T, B> {
    tokenizer: T,
    buffer: PhantomData<fn(B)>,
}

impl<T, B> Many<T, B> {
    pub const fn new(tokenizer: T) -> Many<T, B> {
        Many {
            tokenizer,
            buffer: PhantomData,
        }
    }
}

impl<T: Clone, B> Clone for Many<T, B> {
    fn clone(&self) -> Self {
        Many {
            tokenizer: self.tokenizer.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, B> Copy for Many<T, B> {}

impl<T: fmt::Debug, B> fmt::Debug for Many<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Many")
            .field("tokenizer", &self.tokenizer)
            .finish()
    }
}

impl<'input, T, B> Tokenizer<'input, B> for Many<T, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
    type Token = Item<Vec<T::Token>>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let first = reader.parse(&self.tokenizer)?;
        let mut output = vec![first];

        loop {
            if reader.is(EOF) {
                break;
            }

            let Ok(next) = reader.parse(&self.tokenizer) else {
                break;
            };
            output.push(next);
        }

        let end = reader.position();

        Ok(Item::new(Span::new(start, end), output))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(&self.tokenizer)?;

        loop {
            if reader.eat(&self.tokenizer).is_err() {
                break;
            }
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }
}
