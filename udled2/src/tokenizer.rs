use alloc::{format, string::ToString};

use crate::{
    buffer::{Buffer, StringBuffer},
    error::Error,
    item::Item,
    reader::Reader,
    span::Span,
    AsChar,
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
    S: Buffer<'input, Item = char>,
{
    type Token = Item<char>;
    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let next = reader.eat_ch()?;
        if &next == self {
            Ok(Item {
                span: Span::new(start, next.len_utf8()),
                value: next,
            })
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

// impl<'input, F, U, B> Tokenizer<'input, B> for F
// where
//     F: Fn(&mut Reader<'_, 'input, B>) -> Result<U, Error>,
//     B: Buffer<'input>,
// {
//     type Token = U;
//     fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
//         (self)(reader)
//     }
// }

/// Match a literal string
impl<'lit> Tokenizer<'lit, StringBuffer<'lit>> for &'lit str {
    type Token = Item<&'lit str>;
    fn to_token(
        &self,
        reader: &mut Reader<'_, 'lit, StringBuffer<'lit>>,
    ) -> Result<Self::Token, Error> {
        let tokens = self.chars();

        let start = reader.position();

        for token in tokens {
            let next = reader.eat_ch()?;
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

        Ok(Item {
            value: span.slice(reader.buffer().source().as_ref()).unwrap(),
            span,
        })
    }

    fn peek(&self, reader: &mut Reader<'_, 'lit, StringBuffer<'lit>>) -> bool {
        let tokens = self.chars();
        for (idx, next) in tokens.enumerate() {
            if Some(next) == reader.peek_chn(idx) {
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
