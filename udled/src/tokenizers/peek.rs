use crate::{Buffer, Reader, Result, Tokenizer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Peek<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Peek<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = T::Token;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token> {
        self.0.to_token(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<()> {
        self.0.eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.eat(reader).is_ok()
    }
}
