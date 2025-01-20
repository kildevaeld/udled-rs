use alloc::string::ToString;
use udled::{
    any,
    token::{Digit, Opt, Spanned},
    Error, Item, Reader, Span, StringExt, Tokenizer,
};

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

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
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

pub struct Integer;

impl Tokenizer for Integer {
    type Token<'a> = Item<i128>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let mut val: i128 = 0;
        let base = 10;

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

        return Ok(Item::new(val, Span::new(start, reader.position())));
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Float;

impl Tokenizer for Float {
    type Token<'a> = Item<f64>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.parse(Spanned(Integer))?;
        reader.eat('.')?;
        let mut end = reader.parse(Spanned(Integer))?;

        if reader.peek(any!('E', 'e'))? {
            end = reader.parse(Spanned(('E', 'e', Opt('-'), Integer)))?;
        }

        let input = (start + end)
            .slice(reader.input())
            .ok_or_else(|| reader.error("Invalid range"))?;

        let float: f64 = input
            .parse()
            .map_err(|err: core::num::ParseFloatError| reader.error(err.to_string()))?;

        Ok(Item::new(float, start + end))
    }
}
