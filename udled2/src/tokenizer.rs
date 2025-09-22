use crate::{buffer::Buffer, error::Error, reader::Reader};

pub trait Tokenizer<'input, B: Buffer<'input>> {
    type Token;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error>;

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        let _ = reader.eat_ch()?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.to_token(reader).is_ok()
    }
}

impl<'input> Tokenizer<'input, &'input str> for char {
    type Token = char;
    fn to_token(&self, reader: &mut Reader<'_, 'input, &'input str>) -> Result<Self::Token, Error> {
        let next = reader.eat_ch()?;
        if &next == self {
            Ok(next)
        } else {
            todo!()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Peek<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Peek<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = T::Token;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        self.0.to_token(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.to_token(reader).is_ok()
    }
}

impl<'input, F, U, B> Tokenizer<'input, B> for F
where
    F: Fn(&mut Reader<'_, 'input, B>) -> Result<U, Error>,
    B: Buffer<'input>,
{
    type Token = U;
    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        (self)(reader)
    }
}
