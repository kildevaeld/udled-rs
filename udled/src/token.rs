use alloc::{format, string::ToString, vec, vec::Vec};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    either::Either,
    input::Reader,
    lexeme::{Item, Lex},
    span::Span,
    string::StringExt,
    Error,
};

pub trait Tokenizer {
    type Token<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error>;
    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
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

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        (*self).peek(reader)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ws;

impl Tokenizer for Ws {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let first = reader.eat_ch()?;

        if !first.is_whitespace() {
            return Err(reader.error("whitespace"));
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

impl<'lit> Tokenizer for &'lit str {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let tokens = self.graphemes(true);

        let start = reader.position();

        let line_no = reader.line_no();
        let col_no = reader.col_no();

        for token in tokens {
            let next = reader.eat_ch()?;
            if token != next {
                return Err(Error::new(
                    self.to_string(),
                    reader.position(),
                    line_no,
                    col_no,
                ));
            }
        }

        if start == reader.position() {
            return Err(Error::new(
                self.to_string(),
                reader.position(),
                line_no,
                col_no,
            ));
        }

        Ok(Span {
            start,
            end: reader.position(),
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
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

impl Tokenizer for char {
    type Token<'a> = Span;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let next = reader.eat_ch()?;

        let start = reader.position();

        match next.chars().next() {
            Some(next) if next == *self => Ok(Span {
                start,
                end: reader.position(),
            }),
            _ => return Err(reader.error(format!("expected '{}'", self))),
        }
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let Some(next) = reader.peek_ch() else {
            return Ok(false);
        };
        match next.chars().next() {
            Some(next) if next == *self => Ok(true),
            _ => return Ok(false),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Str;

impl Tokenizer for Str {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let _ = reader.parse('"')?;

        let start_idx = reader.position();

        loop {
            if reader.eof() {
                return Err(reader.error("unexpected end of input while parsing string literal"));
            }

            let ch = reader.eat_ch()?;

            if ch == r#"""# {
                break;
            }

            if ch == "\\" {
                match reader.eat_ch()? {
                    "\\" | "\'" | "\"" | "t" | "r" | "n" | "0" => {
                        continue;
                    }

                    // // Hexadecimal escape sequence
                    // "x" => {
                    //     let digit0 = reader.eat_ch()?.to_digit(16);
                    //     let digit1 = reader.eat_ch()?.to_digit(16);

                    //     match (digit0, digit1) {
                    //         (Some(d0), Some(d1)) => {
                    //             // let byte_val = ((d0 << 4) + d1) as u8;
                    //             //out.push(byte_val as char);
                    //             continue;
                    //         }
                    //         _ => return Err(reader.error("invalid hexadecimal escape sequence")),
                    //     }
                    // }
                    _ => return Err(reader.error("unknown escape sequence")),
                }
            }
        }

        Ok(Lex::new(
            &reader.input()[(start_idx + 1)..reader.position()],
            Span::new(start_idx, reader.position() + 1),
        ))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek('"')
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ident;

impl Tokenizer for Ident {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start_idx = reader.position();

        let mut end_idx = start_idx;

        let Some(first) = reader.peek_ch() else {
            return Err(reader.error("expected identifier"));
        };

        if !first.is_alphabetic() && first != "_" {
            return Err(reader.error("expected identifier"));
        }

        loop {
            let Some(ch) = reader.peek_ch() else {
                break;
            };

            if ch == "\0" {
                break;
            }

            if !ch.is_ascii_alphanumeric() && ch != "_" {
                break;
            }

            end_idx += 1;

            reader.eat_ch()?;
        }

        if start_idx == end_idx {
            return Err(reader.error("expected identifier"));
        }

        let ret = &reader.input()[start_idx..reader.position()];

        Ok(Lex::new(ret, Span::new(start_idx, reader.position())))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let ch = reader.eat_ch()?;
        Ok(ch.is_alphabetic() || ch == "_")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LineComment;

impl Tokenizer for LineComment {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let _ = reader.parse("//")?;

        let mut lb = 0;

        loop {
            let Some(ch) = reader.peek_ch() else {
                break;
            };

            if ch == "\0" {
                break;
            }

            reader.eat_ch()?;

            if ch == "\n" {
                lb = 1;
                break;
            }
        }

        let end = reader.position() - lb;

        let value = if end > 2 {
            &reader.input()[(start + 2)..end]
        } else {
            ""
        };

        Ok(Lex {
            value,
            span: Span::new(start, end),
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("//")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MultiLineComment;

impl Tokenizer for MultiLineComment {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let _ = reader.parse("/*")?;

        let mut depth = 1;

        loop {
            if reader.eof() {
                return Err(reader.error("unexpected end of input inside multi-line comment"));
            } else if reader.parse("/*").is_ok() {
                depth += 1;
            } else if reader.parse("*/").is_ok() {
                depth -= 1;

                if depth == 0 {
                    break;
                }
            } else {
                reader.eat_ch()?;
            }
        }

        Ok(Lex {
            value: &reader.input()[(start + 2)..reader.position() - 2],
            span: Span::new(start, reader.position()),
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("/*")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Int;

impl Tokenizer for Int {
    type Token<'a> = Item<i128>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let mut val: i128 = 0;

        let start = reader.position();

        let sign = if reader.parse("-").is_ok() { -1 } else { 1 };

        let mut base = 10;
        if reader.parse("0x").is_ok() {
            base = 16
        };
        if reader.parse("0b").is_ok() {
            base = 2
        };

        loop {
            let ch = reader.parse(Digit(base))?;

            val = (base as i128) * val + (ch as i128);

            let Some(ch) = reader.peek_ch() else {
                break;
            };

            // Allow underscores as separators
            if ch == "_" {
                reader.eat_ch()?;
                continue;
            }

            if ch == "\0" {
                break;
            }

            if !ch.is_digit(base) {
                break;
            }
        }

        return Ok(Item::new(sign * val, Span::new(start, reader.position())));
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let Some(mut ch) = reader.peek_ch() else {
            return Ok(false);
        };

        if ch == "-" {
            let Some(next) = reader.peek_chn(1) else {
                return Ok(false);
            };

            ch = next;
        }

        Ok(ch.is_digit(10))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Bool;

impl Tokenizer for Bool {
    type Token<'a> = Item<bool>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let ret = reader.parse(Or("true", "false"))?;

        let item = match ret {
            Either::Left(span) => Item::new(true, span),
            Either::Right(span) => Item::new(false, span),
        };

        Ok(item)
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(reader.peek("true")? || reader.peek("false")?)
    }
}

// Helpers

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
    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(reader.eof())
    }
}

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

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        let Some(ch) = reader.peek_ch() else {
            return Ok(false);
        };

        Ok(ch.is_digit(self.0))
    }
}

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

    fn peek<'a>(&self, _reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(true)
    }
}

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

#[derive(Debug, Clone, Copy, Default)]
pub struct Punctuated<T, P>(pub T, pub P, pub bool);

impl<T, P> Tokenizer for Punctuated<T, P>
where
    T: Tokenizer,
    P: Tokenizer,
{
    type Token<'a> = Item<Vec<T::Token<'a>>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let item = reader.parse(&self.0)?;

        let mut output = vec![item];
        loop {
            if !reader.peek(&self.1)? {
                break;
            }

            reader.eat(&self.1)?;

            if self.2 && !reader.peek(&self.0)? {
                break;
            }

            let item = reader.parse(&self.0)?;
            output.push(item);
        }

        let end = reader.position();

        Ok(Item::new(output, Span::new(start, end)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Group<O, T, C>(pub O, pub T, pub C);

impl<O, T, C> Tokenizer for Group<O, T, C>
where
    O: Tokenizer,
    T: Tokenizer,
    C: Tokenizer,
{
    type Token<'a> = Item<T::Token<'a>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        reader.eat(&self.0)?;

        let item = reader.parse(&self.1)?;

        reader.eat(&self.2)?;

        let end = reader.position();

        Ok(Item::new(item, Span::new(start, end)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Prefix<P, T>(pub P, pub T);

impl<P, T> Tokenizer for Prefix<P, T>
where
    P: Tokenizer,
    T: Tokenizer,
{
    type Token<'a> = Item<T::Token<'a>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let (_, item) = reader.parse((&self.0, &self.1))?;
        Ok(Item::new(item, Span::new(start, reader.position())))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Suffix<T, S>(pub T, pub S);

impl<T, S> Tokenizer for Suffix<T, S>
where
    S: Tokenizer,
    T: Tokenizer,
{
    type Token<'a> = Item<T::Token<'a>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let (item, _) = reader.parse((&self.0, &self.1))?;
        Ok(Item::new(item, Span::new(start, reader.position())))
    }
}

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

    fn peek<'a>(&self, _reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(true)
    }
}

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

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(&self.0)
    }
}

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

    fn peek<'a>(&self, _reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
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

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(self.iter().any(|m| reader.peek(m).unwrap_or_default()))
    }
}

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
}

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

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        Ok(reader.peek(&self.0)?)
    }
}

#[macro_export]
macro_rules! any {
    [$one: expr] => {
        $one
    };
    [$first: expr, $($rest: expr),*] => {
        {
            let e = $first;
            $(
                let e = $crate::token::Or(e, $rest);
            )*
            e
        }
    }
}

macro_rules! tokenizer {
    ($first: ident) => {
        impl<$first: Tokenizer> Tokenizer for ($first,) {
            type Token<'a> = $first::Token<'a>;

            fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
                reader.parse(&self.0)
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

    #[test]
    fn line_comment() {
        let mut input = Input::new("//");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new("", Span::new(0, 2))
        );

        let mut input = Input::new("// Some tekst");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new(" Some tekst", Span::new(0, 13))
        );
        let mut input = Input::new("// Some tekst\n test");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new(" Some tekst", Span::new(0, 13))
        );
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
    fn ident() {
        let mut input = Input::new("Ident other");
        assert_eq!(
            input.parse(Ident).unwrap(),
            Lex {
                value: "Ident",
                span: Span { start: 0, end: 5 }
            }
        );

        let mut input = Input::new("-har");
        assert!(input.parse(AlphaNumeric).is_err());
    }

    #[test]
    fn spanned() {
        let mut input = Input::new("Test this string");
        assert_eq!(
            input.parse(Spanned(Ident)).unwrap(),
            Span { start: 0, end: 4 }
        );

        let mut input = Input::new("-har");
        assert!(input.parse(AlphaNumeric).is_err());
    }

    #[test]
    fn punctuated() {
        let mut input = Input::new("ident ,ident2, 202");

        assert_eq!(
            input
                .parse(Punctuated(Ident, Group(Opt(Ws), ',', Opt(Ws)), true))
                .unwrap(),
            Item {
                value: vec![
                    Lex::new("ident", Span::new(0, 5)),
                    Lex::new("ident2", Span::new(7, 12))
                ],
                span: Span::new(0, 13)
            }
        );
        assert!(input.parse(Punctuated(Ident, ',', false)).is_err())
    }
}
