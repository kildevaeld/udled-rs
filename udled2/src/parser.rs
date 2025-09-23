use core::marker::PhantomData;

use crate::{Buffer, Error, Reader, Tokenizer};

pub trait Parser<'a, B>
where
    B: Buffer<'a>,
{
    type Tokenizer: Tokenizer<'a, B>;

    fn parser(self) -> Self::Tokenizer;
}

impl<'a, T, U, B> Parser<'a, B> for T
where
    T: Fn(&mut Reader<'_, 'a, B>) -> Result<U, Error>,
    B: Buffer<'a>,
{
    type Tokenizer = Func<T, U>;

    fn parser(self) -> Self::Tokenizer {
        Func(self, PhantomData)
    }
}

pub struct Func<T, U>(T, PhantomData<U>);

impl<T, U> Func<T, U> {
    pub fn new(func: T) -> Func<T, U> {
        Func(func, PhantomData)
    }
}

impl<'input, T, U, B> Tokenizer<'input, B> for Func<T, U>
where
    B: Buffer<'input>,
    for<'a> T: Fn(&mut Reader<'a, 'input, B>) -> Result<U, Error>,
{
    type Token = U;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        (self.0)(reader)
    }
}
