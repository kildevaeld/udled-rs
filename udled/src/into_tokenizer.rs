use core::marker::PhantomData;

use crate::{Buffer, Error, Reader, Tokenizer};

pub trait IntoTokenizer<'a, B>
where
    B: Buffer<'a>,
{
    type Tokenizer: Tokenizer<'a, B>;

    fn into_tokenizer(self) -> Self::Tokenizer;
}

impl<'a, T, U, B> IntoTokenizer<'a, B> for T
where
    T: Fn(&mut Reader<'_, 'a, B>) -> Result<U, Error>,
    B: Buffer<'a>,
{
    type Tokenizer = Func<T, U>;

    fn into_tokenizer(self) -> Self::Tokenizer {
        Func(self, PhantomData)
    }
}

pub struct Func<T, U>(T, PhantomData<U>);

impl<T, U> Func<T, U> {
    pub const fn new(func: T) -> Func<T, U> {
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
