use core::marker::PhantomData;

use alloc::{fmt, vec::Vec};

use crate::{Buffer, Error, Item, Reader, Span, Tokenizer, EOF};

pub const fn until<T, U, B>(tokenizer: T, until: U) -> Until<T, U, B> {
    Until::new(tokenizer, until)
}

pub struct Until<T, U, B> {
    tokenizer: T,
    until: U,
    buffer: PhantomData<fn(B)>,
}

impl<T, U, B> Until<T, U, B> {
    pub const fn new(tokenizer: T, until: U) -> Until<T, U, B> {
        Until {
            tokenizer,
            until,
            buffer: PhantomData,
        }
    }
}

impl<T: Clone, U: Clone, B> Clone for Until<T, U, B> {
    fn clone(&self) -> Self {
        Until {
            tokenizer: self.tokenizer.clone(),
            until: self.until.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, U: Copy, B> Copy for Until<T, U, B> {}

impl<T: fmt::Debug, U: fmt::Debug, B> fmt::Debug for Until<T, U, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Until")
            .field("tokenizer", &self.tokenizer)
            .field("until", &self.until)
            .finish()
    }
}

impl<'input, T, U, B> Tokenizer<'input, B> for Until<T, U, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
    U: Tokenizer<'input, B>,
{
    type Token = Item<Vec<T::Token>>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let mut output = Vec::new();

        loop {
            if reader.is(EOF) {
                return Err(reader.error("unexepted eof"));
            }

            if reader.is(&self.until) {
                break;
            }

            let next = reader.parse(&self.tokenizer)?;
            output.push(next);
        }

        let end = reader.position();

        Ok(Item::new(Span::new(start, end), output))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(&self.tokenizer)?;

        loop {
            if reader.is(&self.until) {
                break;
            }

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
