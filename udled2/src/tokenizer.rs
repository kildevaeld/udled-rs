use alloc::{fmt, format, string::ToString, vec, vec::Vec};

use crate::{
    buffer::{Buffer, StringBuffer},
    error::Error,
    item::Item,
    reader::Reader,
    span::Span,
    AsBytes, AsChar, AsSlice, AsStr, Either, WithSpan,
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

#[derive(Debug, Clone, Copy)]
pub struct Spanned<T>(pub T);

impl<'input, B, T> Tokenizer<'input, B> for Spanned<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = Span;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        self.0.eat(reader)?;
        let end = reader.position();
        Ok(Span::new(start, end))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.0.peek(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        self.0.eat(reader)
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
                span: Span::new(start, start + ret.len_utf8()),
                value: ret,
            }),
            None => Err(reader.error("char")),
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

/// Match anything but T
#[derive(Debug, Clone, Copy)]
pub struct Not<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Not<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Item: fmt::Display,
{
    type Token = ();

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        if reader.peek(&self.0) {
            let ch = reader.peek_ch().ok_or_else(|| reader.error("EOF"))?;
            return Err(reader.error(format!("unexpected token: {ch}")));
        }
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        !reader.peek(&self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Test<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Test<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = T::Token;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        reader.parse(&self.0)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(&self.0)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.to_token(reader).is_ok()
    }
}

// /// Match either L or R
// #[derive(Debug, Clone, Copy, Default)]
// pub struct Or<L, R>(pub L, pub R);

// impl<'input, L, R, B> Tokenizer<'input, B> for Or<L, R>
// where
//     L: Tokenizer<'input, B>,
//     R: Tokenizer<'input, B>,
//     B: Buffer<'input>,
// {
//     type Token = Either<L::Token, R::Token>;
//     fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
//         let left_err = match reader.parse(&self.0) {
//             Ok(ret) => return Ok(Either::Left(ret)),
//             Err(err) => err,
//         };

//         let right_err = match reader.parse(&self.1) {
//             Ok(ret) => return Ok(Either::Right(ret)),
//             Err(err) => err,
//         };

//         Err(reader.error_with("either", vec![left_err, right_err]))
//     }

//     fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
//         let left_err = match reader.eat(&self.0) {
//             Ok(_) => return Ok(()),
//             Err(err) => err,
//         };

//         let right_err = match reader.eat(&self.1) {
//             Ok(_) => return Ok(()),
//             Err(err) => err,
//         };

//         Err(reader.error_with("either", vec![left_err, right_err]))
//     }

//     fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
//         reader.peek(&self.0) || reader.peek(&self.1)
//     }
// }

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

#[derive(Debug, Clone, Copy, Default)]
pub struct Sliced<T>(pub T);

impl<'input, T, B> Tokenizer<'input, B> for Sliced<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Source: AsSlice<'input>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        self.0.eat(reader)?;
        let end = reader.position();
        let span = Span::new(start, end);
        match reader.buffer().source().sliced(span) {
            Some(slice) => Ok(Item::new(span, slice)),
            None => Err(reader.error("Could not compute slice")),
        }
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        self.0.eat(reader)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        self.0.peek(reader)
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
        self.0.eat(reader)?;
        self.1.eat(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        if reader.parse(&self.0).is_err() {
            return false;
        }

        reader.peek(&self.1)
    }
}

// #[derive(Debug, Clone, Copy)]
// pub enum PuntuatedItem<T, P> {
//     Item(T),
//     Punct(P),
// }

// impl<T: WithSpan, P: WithSpan> WithSpan for PuntuatedItem<T, P> {
//     fn span(&self) -> Span {
//         match self {
//             PuntuatedItem::Item(item) => item.span(),
//             PuntuatedItem::Punct(punct) => punct.span(),
//         }
//     }
// }

// #[derive(Debug, Clone, Copy)]
// pub struct Puntuated<T, P> {
//     item: T,
//     punct: P,
//     non_empty: bool,
// }

// impl<T, P> Puntuated<T, P> {
//     pub fn new(item: T, punct: P) -> Puntuated<T, P> {
//         Puntuated {
//             item,
//             punct,
//             non_empty: false,
//         }
//     }
// }

// impl<'input, T, P, B> Tokenizer<'input, B> for Puntuated<T, P>
// where
//     B: Buffer<'input>,
//     T: Tokenizer<'input, B>,
//     P: Tokenizer<'input, B>,
// {
//     type Token = Item<Vec<PuntuatedItem<T::Token, P::Token>>>;

//     fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
//         let start = reader.position();
//         let mut output = Vec::new();

//         if self.non_empty {
//             let item = reader.parse(&self.item)?;
//             output.push(PuntuatedItem::Item(item));
//             if reader.peek(Prefix(&self.punct, &self.item)) {
//                 let punct = reader.parse(&self.punct)?;
//                 output.push(PuntuatedItem::Punct(punct));
//             }
//         }

//         loop {
//             if !reader.peek(&self.item) {
//                 break;
//             }

//             let item = reader.parse(&self.item)?;

//             output.push(PuntuatedItem::Item(item));

//             if reader.peek(Prefix(&self.punct, &self.item)) {
//                 let punct = reader.parse(&self.punct)?;
//                 output.push(PuntuatedItem::Punct(punct));
//             }
//         }

//         let end = reader.position();

//         Ok(Item::new(Span::new(start, end), output))
//     }
// }

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
