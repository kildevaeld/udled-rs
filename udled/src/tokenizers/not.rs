use alloc::format;

use crate::{AsChar, Buffer, Reader, Result, Tokenizer};

/// Match anything but T
#[derive(Debug, Clone, Copy)]
pub struct Not<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Not<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = ();

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token> {
        if reader.is(&self.0) {
            let ch = reader.peek_ch().ok_or_else(|| reader.error("EOF"))?;
            return Err(reader.error(format!("unexpected token: {:?}", ch.as_char())));
        }
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        !reader.is(&self.0)
    }
}
