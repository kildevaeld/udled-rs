#![no_std]

use udled::{
    token::{Char, Digit, Or},
    Either, Error, Item, Lex, Reader, Span, StringExt, Tokenizer,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Str;

impl Tokenizer for Str {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.parse('"')?;

        let end = loop {
            if reader.eof() {
                return Err(reader.error("unexpected end of input while parsing string literal"));
            }

            let ch = reader.parse(Char)?;

            if ch == r#"""# {
                break ch.span;
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
        };

        let span = start + end;

        Ok(Lex::new(
            Span::new(span.start + 1, span.end - 1)
                .slice(reader.input())
                .unwrap(),
            span,
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

#[cfg(test)]
mod test {
    use udled::{token::AlphaNumeric, Input};

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
}
