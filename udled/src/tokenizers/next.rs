use crate::{Buffer, Tokenizer, EOF};

#[derive(Debug, Clone, Copy, Default)]
pub struct Next;

impl<'input, B> Tokenizer<'input, B> for Next
where
    B: Buffer<'input>,
{
    type Token = B::Item;

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        reader.read()
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        !reader.is(EOF)
    }
}
