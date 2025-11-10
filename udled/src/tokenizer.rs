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

pub struct IgnoreCase<T>(pub T);

impl<'lit, 'input, T, B> Tokenizer<'lit, B> for IgnoreCase<T>
where
    T: AsRef<str>,
    B: Buffer<'lit>,
    B::Item: AsChar,
    B::Source: AsBytes<'lit>,
{
    type Token = Item<&'lit str>;
    fn to_token(&self, reader: &mut Reader<'_, 'lit, B>) -> Result<Self::Token, Error> {
        let tokens = self.0.as_ref().chars();

        let start = reader.position();

        for token in tokens {
            let Some(next) = reader.read()?.as_char() else {
                return Err(reader.error(self.0.as_ref().to_string()));
            };
            if token != next {
                return Err(reader.error(self.0.as_ref().to_string()));
            }
        }

        if start == reader.position() {
            return Err(reader.error(self.0.as_ref().to_string()));
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
        let tokens = self.0.as_ref().chars();
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
            return Err(reader.error("Eof not reached"));
        }
        Ok(reader.position())
    }
}

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

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
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

pub struct Prefix<P, T>(pub P, pub T);

impl<'input, P, T, B> Tokenizer<'input, B> for Prefix<P, T>
where
    B: Buffer<'input>,
    P: Tokenizer<'input, B>,
    T: Tokenizer<'input, B>,
{
    type Token = Item<(P::Token, T::Token)>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let prefix = reader.parse(&self.0)?;
        let item = reader.parse(&self.1)?;
        let end = reader.position();
        Ok(Item::new(Span::new(start, end), (prefix, item)))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(&self.0)?;
        reader.eat(&self.1)?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        if reader.parse(&self.0).is_err() {
            return false;
        }

        reader.is(&self.1)
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

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        self.0.eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.eat(reader).is_ok()
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

#[cfg(test)]
mod test {
    use crate::Input;

    use super::IgnoreCase;

    macro_rules! parse {
        ($parser: literal, $($input:literal),+) => {
          $(
            let mut input = Input::new($input);
            let ret = input.parse(IgnoreCase($parser)).expect("parse");

            assert_eq!($input,ret.value);
          )+
        };
    }

    #[test]

    fn ignore_case() {
        parse!("DOCTYPE", "docType", "DOCTYPE", "DocType");
        parse!("ÆæpÅLLÆ", "ææpållæ");
    }
}
