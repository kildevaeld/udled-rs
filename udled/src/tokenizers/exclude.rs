use core::marker::PhantomData;

use alloc::fmt;

use crate::{tokenizers::next::Next, Buffer, Tokenizer};

/// Matches everything but [T]
pub struct Exclude<T, B> {
    tokenizer: T,
    buffer: PhantomData<fn(B)>,
}

impl<T: fmt::Debug, B> fmt::Debug for Exclude<T, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Exclude")
            .field("tokenizer", &self.tokenizer)
            .finish()
    }
}

impl<T, B> Exclude<T, B> {
    pub fn new(tokenizer: T) -> Exclude<T, B> {
        Exclude {
            tokenizer,
            buffer: PhantomData,
        }
    }
}

impl<T: Clone, B> Clone for Exclude<T, B> {
    fn clone(&self) -> Self {
        Exclude {
            tokenizer: self.tokenizer.clone(),
            buffer: PhantomData,
        }
    }
}

impl<T: Copy, B> Copy for Exclude<T, B> {}

impl<'input, T, B> Tokenizer<'input, B> for Exclude<T, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
    type Token = B::Item;

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        if reader.is(&self.tokenizer) {
            return Err(reader.error("unexpected"));
        }

        reader.parse(Next)
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        !self.tokenizer.peek(reader)
    }
}
