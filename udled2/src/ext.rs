use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use crate::{
    AsDigits, AsSlice, Buffer, Error, Item, Many, Opt, Or, Puntuated, Reader, Sliced, Span,
    Spanned, Tokenizer,
};

pub trait TokenizerExt<'input, B>: Tokenizer<'input, B>
where
    B: Buffer<'input>,
{
    fn map_ok<F, U>(self, func: F) -> MapOk<Self, F, B>
    where
        F: Fn(Self::Token) -> U,
        Self: Sized,
    {
        MapOk {
            tokenizer: self,
            func,
            ph: PhantomData,
        }
    }

    fn map_err<F, U>(self, func: F) -> MapErr<Self, F, B>
    where
        F: Fn(usize, &B) -> U,
        U: Into<Box<dyn core::error::Error + Send + Sync>>,
        Self: Sized,
    {
        MapErr {
            tokenizer: self,
            func,
            ph: PhantomData,
        }
    }

    fn repeat(self, count: i32) -> Repeat<Self, B>
    where
        Self: Sized,
    {
        Repeat {
            tokenizer: self,
            count,
            ph: PhantomData,
        }
    }

    fn many(self) -> Many<Self, B>
    where
        Self: Sized,
    {
        Many::new(self)
    }

    fn or<T>(self, other: T) -> Or<Self, T, B>
    where
        Self: Sized,
        T: Tokenizer<'input, B>,
    {
        Or::new(self, other)
    }

    fn optional(self) -> Opt<Self, B>
    where
        Self: Sized,
    {
        Opt::new(self)
    }

    fn spanned(self) -> Spanned<Self, B>
    where
        Self: Sized,
    {
        Spanned::new(self)
    }

    fn into_integer(self, base: u32) -> IntoInteger<Self, B>
    where
        Self: Sized,
        Self::Token: AsDigits,
    {
        IntoInteger {
            tokenizer: self,
            base,
            ph: PhantomData,
        }
    }

    fn punctuated<P>(self, punct: P) -> Puntuated<Self, P>
    where
        Self: Sized,
        P: Tokenizer<'input, B>,
    {
        Puntuated::new(self, punct)
    }

    fn slice(self) -> Sliced<Self, B>
    where
        Self: Sized,
        B: Buffer<'input>,
        B::Source: AsSlice<'input>,
    {
        Sliced::new(self)
    }

    fn parse(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error>
    where
        Self: Sized,
    {
        reader.parse(self)
    }
}

impl<'input, T, B> TokenizerExt<'input, B> for T
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
}

pub struct MapOk<T, F, B> {
    tokenizer: T,
    func: F,
    ph: PhantomData<fn(&B)>,
}

impl<'input, T, F, U, B> Tokenizer<'input, B> for MapOk<T, F, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
    F: Fn(T::Token) -> U,
{
    type Token = U;

    fn eat(&self, reader: &mut crate::Reader<'_, 'input, B>) -> Result<(), crate::Error> {
        self.tokenizer.eat(reader)
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        match self.tokenizer.to_token(reader) {
            Ok(ret) => Ok((self.func)(ret)),
            Err(err) => Err(err),
        }
    }
}

pub struct MapErr<T, F, B> {
    tokenizer: T,
    func: F,
    ph: PhantomData<fn(&B)>,
}

impl<'input, T, F, U, B> Tokenizer<'input, B> for MapErr<T, F, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
    F: Fn(usize, &B) -> U,
    U: Into<Box<dyn core::error::Error + Send + Sync>>,
{
    type Token = T::Token;

    fn eat(&self, reader: &mut crate::Reader<'_, 'input, B>) -> Result<(), crate::Error> {
        self.tokenizer
            .eat(reader)
            .map_err(|err| Error::new(err.position(), (self.func)(err.position(), reader.buffer())))
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        self.tokenizer
            .to_token(reader)
            .map_err(|err| Error::new(err.position(), (self.func)(err.position(), reader.buffer())))
    }
}

pub struct IntoInteger<T, B> {
    tokenizer: T,
    base: u32,
    ph: PhantomData<fn(&B)>,
}

impl<'input, T, B> Tokenizer<'input, B> for IntoInteger<T, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
    T::Token: AsDigits,
{
    type Token = Item<i128>;

    fn eat(&self, reader: &mut crate::Reader<'_, 'input, B>) -> Result<(), crate::Error> {
        self.tokenizer.eat(reader)
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        let start = reader.position();
        let digits = self.tokenizer.to_token(reader)?;
        let end = reader.position();
        let mut val = 0i128;

        for digit in digits.digits() {
            val = (self.base as i128) * val + (digit as i128);
        }

        Ok(Item::new(Span::new(start, end), val))
    }
}

pub struct Repeat<T, B> {
    tokenizer: T,
    count: i32,
    ph: PhantomData<fn(&B)>,
}

impl<'input, T, B> Tokenizer<'input, B> for Repeat<T, B>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
    type Token = Item<Vec<T::Token>>;

    fn eat(&self, reader: &mut crate::Reader<'_, 'input, B>) -> Result<(), crate::Error> {
        self.tokenizer.eat(reader)
    }

    fn peek(&self, reader: &mut crate::Reader<'_, 'input, B>) -> bool {
        self.tokenizer.peek(reader)
    }

    fn to_token(
        &self,
        reader: &mut crate::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, crate::Error> {
        let start = reader.position();

        let mut output = Vec::with_capacity(self.count as _);
        loop {
            let next = self.tokenizer.parse(reader)?;
            output.push(next);
            if output.len() == self.count as usize {
                break;
            }
        }

        let end = reader.position();

        Ok(Item::new(Span::new(start, end), output))
    }
}
