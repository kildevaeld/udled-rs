use core::marker::PhantomData;

use alloc::fmt;

use crate::{Buffer, Error, Reader, Tokenizer};

pub const fn opt<T, B>(tokenizer: T) -> Opt<T, B> {
    Opt::new(tokenizer)
}

pub struct Opt<T, B> {
    tokenizer: T,
    buffer: PhantomData<fn(B)>,
}

impl<T, B> Opt<T, B> {
    pub const fn new(tokenizer: T) -> Opt<T, B> {
        Opt {
            tokenizer,
            buffer: PhantomData,
        }
    }
}

impl<T: Clone, B> Clone for Opt<T, B> {
    fn clone(&self) -> Self {
        Opt {
            tokenizer: self.tokenizer.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, B> Copy for Opt<T, B> {}

impl<T: fmt::Debug, B> fmt::Debug for Opt<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Opt")
            .field("tokenizer", &self.tokenizer)
            .finish()
    }
}

impl<'input, T, B> Tokenizer<'input, B> for Opt<T, B>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = Option<T::Token>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        if reader.peek(&self.tokenizer) {
            Ok(Some(reader.parse(&self.tokenizer)?))
        } else {
            Ok(None)
        }
    }

    fn peek(&self, _reader: &mut Reader<'_, 'input, B>) -> bool {
        true
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        if reader.peek(&self.tokenizer) {
            reader.parse(&self.tokenizer)?;
        }
        Ok(())
    }
}
