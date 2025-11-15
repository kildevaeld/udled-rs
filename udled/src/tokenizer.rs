use alloc::{format, string::ToString};

use crate::{
    buffer::Buffer, error::Error, item::Item, reader::Reader, span::Span, AsBytes, AsChar,
};

pub trait Tokenizer<'input, B: Buffer<'input>> {
    type Token;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error>;

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.eat(reader).is_ok()
    }
}

impl<'a, 'input, B, T> Tokenizer<'input, B> for &'a T
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
    type Token = T::Token;

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        (**self).eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        (**self).peek(reader)
    }

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        (**self).to_token(reader)
    }
}

impl<'input, S> Tokenizer<'input, S> for char
where
    S: Buffer<'input>,
    S::Item: AsChar,
{
    type Token = Item<char>;
    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        let next = reader.parse(Char)?;
        if &next.value == self {
            Ok(next)
        } else {
            Err(reader.error(format!("{}", self)))
        }
    }
}

/// Match a literal string
impl<'lit, 'input, B> Tokenizer<'lit, B> for &'input str
where
    B: Buffer<'lit>,
    B::Item: AsChar,
    B::Source: AsBytes<'lit>,
{
    type Token = Item<&'lit str>;
    fn to_token(&self, reader: &mut Reader<'_, 'lit, B>) -> Result<Self::Token, Error> {
        let tokens = self.chars();

        let start = reader.position();

        for token in tokens {
            let Some(next) = reader.read()?.as_char() else {
                return Err(reader.error(self.to_string()));
            };
            if token != next {
                return Err(reader.error(self.to_string()));
            }
        }

        if start == reader.position() {
            return Err(reader.error(self.to_string()));
        }

        let span = Span {
            start,
            end: reader.position(),
        };

        let string = reader.buffer().source().as_bytes();
        let string = unsafe { core::str::from_utf8_unchecked(string) };

        Ok(Item {
            value: span.slice(string).unwrap(),
            span,
        })
    }

    fn peek(&self, reader: &mut Reader<'_, 'lit, B>) -> bool {
        let tokens = self.chars();
        for (idx, next) in tokens.enumerate() {
            if Some(next) == reader.peek_chn(idx).and_then(|m| m.as_char()) {
                continue;
            }
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Char;

impl<'input, S> Tokenizer<'input, S> for Char
where
    S: Buffer<'input>,
    S::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        let start = reader.position();
        match reader.read()?.as_char() {
            Some(ret) => Ok(Item {
                span: Span::new(start, start + ret.len_utf8()),
                value: ret,
            }),
            None => Err(reader.error("char")),
        }
    }
}

impl<'input, B> Tokenizer<'input, B> for core::ops::Range<char>
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let char = reader.parse(Char)?;

        if !self.contains(&char.value) {
            return Err(reader.error(format!("Expected char in range: {:?}", self)));
        }
        Ok(char)
    }
}

impl<'input, B> Tokenizer<'input, B> for core::ops::RangeInclusive<char>
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let char = reader.parse(Char)?;

        if !self.contains(&char.value) {
            return Err(reader.error(format!("Expected char in range: {:?}", self)));
        }

        Ok(char)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EOF;

impl<'input, S> Tokenizer<'input, S> for EOF
where
    S: Buffer<'input>,
{
    type Token = usize;

    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        if reader.peek_ch().is_some() {
            return Err(reader.error("EOF"));
        }
        Ok(reader.position())
    }
}

macro_rules! tuples {
    ($first: ident) => {
        impl<'input, $first, B> Tokenizer<'input, B> for ($first,)
        where
            B: Buffer<'input>,
            $first: Tokenizer<'input, B>,
        {
            type Token = ($first::Token, );

            fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
                Ok((self.0.to_token(reader)?,))
            }

            fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
                self.0.peek(reader)
            }

            fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
                self.0.eat(reader)
            }
        }
    };
    ($first: ident, $($rest:ident),+) => {

        tuples!($($rest),+);

        #[allow(non_snake_case)]
        impl<'input, $first, $($rest),+, B> Tokenizer<'input, B> for ($first, $($rest),+)
        where
            B: Buffer<'input>,
            $first: Tokenizer<'input, B>,
            $(
                $rest: Tokenizer<'input, B>
            ),+
        {

            type Token = ($first::Token, $($rest::Token),+);

            fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
                let ($first, $($rest),+) = self;
                Ok((
                    reader.parse($first)?,
                    $(
                        reader.parse($rest)?
                    ),+
                ))
            }

            fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
                self.0.peek(reader)
            }

            fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
                let ($first, $($rest),+) = self;
                reader.parse($first)?;
                $(
                    reader.parse($rest)?;
                )+
                Ok(())
            }
        }
    };
}

tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
