use alloc::{format, string::ToString, vec, vec::Vec};

use crate::{
    buffer::{Buffer, StringBuffer},
    error::Error,
    item::Item,
    reader::Reader,
    span::Span,
    AsBytes, AsChar, AsStr,
};

pub trait Tokenizer<'input, B: Buffer<'input>> {
    type Token;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error>;

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.to_token(reader).is_ok()
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

#[derive(Debug, Clone, Copy)]
pub struct Many<T>(pub T);
impl<'input, T, B> Tokenizer<'input, B> for Many<T>
where
    B: Buffer<'input>,
    T: Tokenizer<'input, B>,
{
    type Token = Item<Vec<T::Token>>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let first = reader.parse(&self.0)?;
        let mut output = vec![first];

        loop {
            let Ok(next) = reader.parse(&self.0) else {
                break;
            };
            output.push(next);
        }

        let end = reader.position();

        Ok(Item::new(Span::new(start, end), output))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(&self.0)?;

        loop {
            if reader.eat(&self.0).is_err() {
                break;
            }
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.0.peek(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Opt<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Opt<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = Option<T::Token>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        Ok(reader.parse(&self.0).ok())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        true
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        let _ = self.0.eat(reader);
        Ok(())
    }
}

/// Match a literal string
impl<'lit, B> Tokenizer<'lit, B> for &'lit str
where
    B: Buffer<'lit> + 'lit,
    B::Item: AsChar,
    B::Source: AsBytes<'lit>,
{
    type Token = Item<&'lit str>;
    fn to_token(&self, reader: &mut Reader<'_, 'lit, B>) -> Result<Self::Token, Error> {
        let tokens = self.chars();

        let start = reader.position();

        for token in tokens {
            let Some(next) = reader.eat_ch()?.as_char() else {
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
        match reader.eat_ch()?.as_char() {
            Some(ret) => Ok(Item {
                span: Span::new(start, ret.len_utf8()),
                value: ret,
            }),
            None => Err(reader.error("char")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Digit(pub u32);

impl Default for Digit {
    fn default() -> Self {
        Digit(10)
    }
}

impl<'input, S> Tokenizer<'input, S> for Digit
where
    S: Buffer<'input>,
    S::Item: AsChar,
{
    type Token = Item<u32>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        let item = reader.parse(Char)?;

        item.value
            .to_digit(self.0)
            .map(|value| Item {
                span: item.span,
                value,
            })
            .ok_or_else(|| reader.error("digit"))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, S>) -> bool {
        match reader.peek_ch().and_then(|m| m.as_char()) {
            Some(char) => char.is_digit(self.0),
            None => false,
        }
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
            return Err(reader.error("Eof not reached"));
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
                    $first.to_token(reader)?,
                    $(
                        $rest.to_token(reader)?
                    ),+
                ))
            }

            fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
                self.0.peek(reader)
            }

            fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
                let ($first, $($rest),+) = self;
                $first.to_token(reader)?;
                $(
                    $rest.to_token(reader)?;
                )+
                Ok(())
            }
        }
    };
}

tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
