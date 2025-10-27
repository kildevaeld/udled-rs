use core::marker::PhantomData;

use alloc::{format, string::ToString, vec, vec::Vec};
use unicode_segmentation::UnicodeSegmentation;

use crate::{either::Either, lexeme::Lex, span::Span, string::StringExt, Error, Reader};

pub trait Tokenizer {
    type Token<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error>;

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(self.to_token(reader).is_ok())
    }
}

impl<'b, T> Tokenizer for &'b T
where
    T: Tokenizer,
{
    type Token<'a> = T::Token<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        (*self).to_token(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        (*self).eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        (*self).peek(reader)
    }
}

impl<'b, T> Tokenizer for &'b mut T
where
    T: Tokenizer,
{
    type Token<'a> = T::Token<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        (**self).to_token(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        (**self).eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        (**self).peek(reader)
    }
}

pub struct Func<T, U>(T, PhantomData<U>);

impl<T, U> Func<T, U> {
    pub fn new(func: T) -> Func<T, U> {
        Func(func, PhantomData)
    }
}

impl<T, U> Tokenizer for Func<T, U>
where
    for<'a, 'b> T: Fn(&mut Reader<'a, 'b>) -> Result<U, Error>,
{
    type Token<'a> = U;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        (self.0)(reader)
    }
}

impl Tokenizer for core::ops::Range<char> {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let char = reader.parse(Char)?;

        for n in char.as_str().chars() {
            if !self.contains(&n) {
                return Err(reader.error(format!("Expected char in range: {:?}", self)));
            }
        }

        Ok(char)
    }
}

impl Tokenizer for core::ops::RangeInclusive<char> {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let char = reader.parse(Char)?;

        for n in char.as_str().chars() {
            if !self.contains(&n) {
                return Err(reader.error(format!("Expected char in range: {:?}", self)));
            }
        }

        Ok(char)
    }
}

impl<L, R> Tokenizer for Either<L, R>
where
    L: Tokenizer,
    R: Tokenizer,
{
    type Token<'a> = Either<L::Token<'a>, R::Token<'a>>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        match self {
            Self::Left(left) => Ok(Either::Left(left.to_token(reader)?)),
            Self::Right(right) => Ok(Either::Right(right.to_token(reader)?)),
        }
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        match self {
            Self::Left(left) => left.eat(reader),
            Self::Right(right) => right.eat(reader),
        }
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        match self {
            Self::Left(left) => left.peek(reader),
            Self::Right(right) => right.peek(reader),
        }
    }
}
/// Match any whitespace
#[derive(Debug, Clone, Copy, Default)]
pub struct Ws;

impl Tokenizer for Ws {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let first = reader.eat_ch()?;

        if !first.is_whitespace() {
            return Err(reader.error("Expected whitespace"));
        }

        loop {
            let Some(ch) = reader.peek_ch() else {
                break;
            };

            if !ch.is_whitespace() {
                break;
            }

            reader.eat_ch()?;
        }

        Ok(Span {
            start,
            end: reader.position(),
        })
    }
}

/// Match a literal string
impl<'lit> Tokenizer for &'lit str {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let tokens = self.graphemes(true);

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

        Ok(Span {
            start,
            end: reader.position(),
        })
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let tokens = self.graphemes(true);
        for (idx, next) in tokens.enumerate() {
            if Some(next) == reader.peek_chn(idx) {
                continue;
            }
            return Ok(false);
        }

        Ok(true)
    }
}

/// Match a literal char
impl Tokenizer for char {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let next = reader.eat_ch()?;

        match next.chars().next() {
            Some(next) if next == *self => Ok(Span {
                start,
                end: reader.position(),
            }),
            _ => return Err(reader.error(format!("expected '{}'", self))),
        }
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let Some(next) = reader.peek_ch() else {
            return Ok(false);
        };
        match next.chars().next() {
            Some(next) if next == *self => Ok(true),
            _ => return Ok(false),
        }
    }
}

// Helpers

/// Match EOF
#[derive(Debug, Clone, Copy, Default)]
pub struct EOF;

impl Tokenizer for EOF {
    type Token<'a> = ();

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        if reader.eof() {
            Ok(())
        } else {
            Err(reader.error("expected eof"))
        }
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(reader.eof())
    }
}

/// Match a digit with a given radix
#[derive(Debug, Clone, Copy)]
pub struct Digit(pub u32);

impl Default for Digit {
    fn default() -> Self {
        Digit(10)
    }
}

impl Tokenizer for Digit {
    type Token<'a> = u32;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let ch = reader.eat_ch()?;

        if !ch.is_digit(self.0) {
            return Err(reader.error("expected digit"));
        }

        Ok(ch.chars().next().unwrap().to_digit(self.0).unwrap())
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        let ch = reader.eat_ch()?;

        if !ch.is_digit(self.0) {
            return Err(reader.error("expected digit"));
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let Some(ch) = reader.peek_ch() else {
            return Ok(false);
        };

        Ok(ch.is_digit(self.0))
    }
}

/// Match any unnicode graphme
#[derive(Debug, Clone, Copy, Default)]
pub struct Char;

impl Tokenizer for Char {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let ch = reader.eat_ch()?;
        let end = reader.position();
        Ok(Lex {
            value: ch,
            span: Span { start, end },
        })
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        let _ = reader.eat_ch()?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(if reader.eof() { false } else { false })
    }
}

/// Match a alphabetic character
#[derive(Debug, Clone, Copy, Default)]
pub struct Alphabetic;

impl Tokenizer for Alphabetic {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let ch = reader.parse(Char)?;
        if ch.value.is_alphabetic() {
            Ok(ch)
        } else {
            Err(reader.error("expected alphabetic"))
        }
    }
}

/// Match any unicode alphanumeric character
#[derive(Debug, Clone, Copy, Default)]
pub struct AlphaNumeric;

impl Tokenizer for AlphaNumeric {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let ch = reader.parse(Char)?;
        if ch.value.is_alphanumeric() {
            Ok(ch)
        } else {
            Err(reader.error("expected alphanumeric"))
        }
    }
}

/// Match any ascii punctuation
/// Match a punctuation
#[derive(Debug, Clone, Copy, Default)]
pub struct Punctuation;

impl Tokenizer for Punctuation {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let ch = reader.parse(Char)?;
        if ch.value.is_ascii_punctuation() {
            Ok(ch)
        } else {
            Err(reader.error("expected punctuation"))
        }
    }
}

/// Optional match T
#[derive(Debug, Clone, Copy, Default)]
pub struct Opt<T>(pub T);

impl<T> Tokenizer for Opt<T>
where
    T: Tokenizer,
{
    type Token<'a> = Option<T::Token<'a>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        Ok(reader.parse(&self.0).ok())
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(&self.0).ok();
        Ok(())
    }

    fn peek(&self, _reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(true)
    }
}

/// Match either L or R
#[derive(Debug, Clone, Copy, Default)]
pub struct Or<L, R>(pub L, pub R);

impl<L, R> Tokenizer for Or<L, R>
where
    L: Tokenizer,
    R: Tokenizer,
{
    type Token<'a> = Either<L::Token<'a>, R::Token<'a>>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let line_no = reader.line_no();
        let col_no = reader.col_no();

        let left_err = match reader.parse(&self.0) {
            Ok(ret) => return Ok(Either::Left(ret)),
            Err(err) => err,
        };

        let right_err = match reader.parse(&self.1) {
            Ok(ret) => return Ok(Either::Right(ret)),
            Err(err) => err,
        };

        Err(Error::new_with(
            "either",
            reader.position(),
            line_no,
            col_no,
            vec![left_err, right_err],
        ))
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(reader.peek(&self.0)? || reader.peek(&self.1)?)
    }
}

/// One or many T's
#[derive(Debug, Clone, Copy)]
pub struct OneOrMany<T>(pub T);

impl<T> Tokenizer for OneOrMany<T>
where
    T: Tokenizer,
{
    type Token<'a> = Vec<T::Token<'a>>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let mut output = vec![reader.parse(&self.0)?];

        loop {
            let next = match reader.parse(&self.0) {
                Ok(next) => next,
                Err(_) => break,
            };

            output.push(next);
        }

        Ok(output)
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(&self.0)?;

        loop {
            match reader.eat(&self.0) {
                Ok(_) => continue,
                Err(_) => break,
            };
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(&self.0)
    }
}

/// Many of T's
#[derive(Debug, Clone, Copy)]
pub struct Many<T>(pub T);

impl<T> Tokenizer for Many<T>
where
    T: Tokenizer,
{
    type Token<'a> = Vec<T::Token<'a>>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let mut output = Vec::default();

        loop {
            let next = match reader.parse(&self.0) {
                Ok(next) => next,
                Err(_) => break,
            };

            output.push(next);
        }

        Ok(output)
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        loop {
            match reader.eat(&self.0) {
                Ok(_) => continue,
                Err(_) => break,
            };
        }
        Ok(())
    }

    fn peek(&self, _reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(true)
    }
}

impl<'b, T> Tokenizer for &'b [T]
where
    T: Tokenizer,
{
    type Token<'a> = T::Token<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let mut errors = Vec::default();
        for tokenizer in self.iter() {
            match tokenizer.to_token(reader) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        Err(reader.error_with("one of", errors))
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(self.iter().any(|m| reader.peek(m).unwrap_or_default()))
    }
}

/// Returns span of T
pub struct Spanned<T>(pub T);

impl<T> Tokenizer for Spanned<T>
where
    T: Tokenizer,
{
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        reader.eat(&self.0)?;
        let end = reader.position();
        Ok(Span { start, end })
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(&self.0)
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(&self.0)
    }
}

/// Match anything but T
#[derive(Debug, Clone, Copy)]
pub struct Not<T>(pub T);

impl<T> Tokenizer for Not<T>
where
    T: Tokenizer,
{
    type Token<'a> = ();

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        if reader.peek(&self.0)? {
            let ch = reader.peek_ch().unwrap_or("EOF");
            return Err(reader.error(format!("unexpected token: {ch}")));
        }
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(!reader.peek(&self.0)?)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Test<T>(pub T);

impl<T> Tokenizer for Test<T>
where
    T: Tokenizer,
{
    type Token<'a> = T::Token<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        reader.parse(&self.0)
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(&self.0)
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(self.to_token(reader).is_ok())
    }
}

#[macro_export]
macro_rules! any {
    [$one: expr] => {
        $one
    };
    [$first: expr, $($rest: expr),*] => {
        $crate::token::Or($first, $crate::any!($($rest),*))
    };

}

macro_rules! tokenizer {
    ($first: ident) => {
        impl<$first: Tokenizer> Tokenizer for ($first,) {
            type Token<'a> = $first::Token<'a>;

            fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
                reader.parse(&self.0)
            }

            fn eat<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<(), Error> {
                reader.eat(&self.0)
            }

            fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
                Ok(reader.peek(&self.0)?)
            }
        }
    };
    ($first:ident $($rest:ident)*) => {
        tokenizer!($($rest)*);

        impl<$first: Tokenizer, $($rest: Tokenizer),*> Tokenizer for ($first,$($rest),*) {
            type Token<'a> = ($first::Token<'a>, $($rest::Token<'a>),*);

            #[allow(non_snake_case)]
            fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
                let ($first, $($rest),*) = self;
                Ok((
                    reader.parse(&$first)?,
                    $(
                        reader.parse(&$rest)?
                    ),*
                ))
            }

            #[allow(non_snake_case)]
            fn eat<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<(), Error> {
                let ($first, $($rest),*) = self;

                reader.eat(&$first)?;
                $(
                    reader.eat(&$rest)?;
                )*

                Ok(())
            }

            fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
                Ok(reader.peek(&self.0)?)
            }
        }
    }
}

tokenizer!(T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12);

#[cfg(test)]
mod test {
    use crate::Input;

    use super::*;

    struct Word;

    impl Tokenizer for Word {
        type Token<'a> = Lex<'a>;

        fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
            if !reader.peek(Alphabetic)? {
                return Err(reader.error("expected alphabetic"));
            }

            let start = reader.position();

            loop {
                if reader.eof() {
                    break;
                }

                if !reader.peek(Alphabetic)? {
                    break;
                }

                reader.eat(Alphabetic)?;
            }

            let span = Span::new(start, reader.position());

            if !span.is_valid() {
                return Err(reader.error("no word"));
            }

            Ok(Lex::new(span.slice(reader.source()).unwrap(), span))
        }
    }

    #[test]
    fn opt() {
        let mut input = Input::new("WS");
        assert_eq!(input.parse(Opt("He")).unwrap(), None,);
        assert_eq!(input.position(), 0);
        assert_eq!(input.peek_ch(), Some("W"));
    }

    #[test]
    fn char() {
        let mut input = Input::new("char");
        assert_eq!(
            input.parse(Char).unwrap(),
            Lex {
                value: "c",
                span: Span { start: 0, end: 1 }
            }
        );
    }

    #[test]
    fn alphabetic() {
        let mut input = Input::new("char");
        assert_eq!(
            input.parse(Alphabetic).unwrap(),
            Lex {
                value: "c",
                span: Span { start: 0, end: 1 }
            }
        );

        let mut input = Input::new("-har");
        assert!(input.parse(Alphabetic).is_err());
    }

    #[test]
    fn alphabetic_numeric() {
        let mut input = Input::new("2char");
        assert_eq!(
            input.parse(AlphaNumeric).unwrap(),
            Lex {
                value: "2",
                span: Span { start: 0, end: 1 }
            }
        );

        let mut input = Input::new("-har");
        assert!(input.parse(AlphaNumeric).is_err());
    }

    #[test]
    fn spanned() {
        let mut input = Input::new("Test this string");
        assert_eq!(
            input.parse(Spanned(Word)).unwrap(),
            Span { start: 0, end: 4 }
        );
    }

    #[test]
    fn range() {
        let mut input = Input::new("b");
        assert_eq!(
            input.parse('a'..'z').unwrap(),
            Lex::new("b", Span::new(0, 1))
        )
    }

    #[test]
    fn not() {
        assert_eq!(
            Input::new("=-").parse(('=', Not('='))).unwrap(),
            (Span::new(0, 1), ())
        );

        assert!(Input::new("==").parse(('=', Not('='))).is_err())
    }

    #[test]
    fn func() {
        let mut input = Input::new("Hello");

        let ret = input
            .parse(Func::new(|ctx: &mut Reader| ctx.parse("Hello")))
            .unwrap();

        assert_eq!(ret, Span::new(0, 5));
    }
}
