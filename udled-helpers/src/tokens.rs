use udled::{
    token::{Char, Or},
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

        let str = if span.len() == 2 {
            Span::new(span.start + 1, span.end).slice(reader.input())
        } else {
            Span::new(span.start + 1, span.end - 1).slice(reader.input())
        };

        Ok(Lex::new(str.unwrap(), span))
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
