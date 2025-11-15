use alloc::string::ToString;
use udled::{
    tokenizers::{opt, Digit, Peek},
    AsChar, AsSlice, AsStr, Buffer, Error, Item, Reader, Span, Tokenizer, TokenizerExt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Integer;

impl<'input, B> Tokenizer<'input, B> for Integer
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = Item<i128>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();
        let mut val: i128 = 0;
        let base = 10;

        let sign = if reader.eat('-').is_ok() { -1 } else { 1 };

        loop {
            let ch = reader.parse(Digit(base))?;

            val = (base as i128) * val + (ch.value as i128);

            if !reader.is(Digit(base)) {
                break;
            }
        }

        return Ok(Item::new(Span::new(start, reader.position()), val * sign));
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek((opt('-'), Digit(10))))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Float;

impl<'input, B> Tokenizer<'input, B> for Float
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    type Token = Item<f64>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let slice = reader.parse(
            (
                Integer,
                '.',
                Digit(10).many(),
                ('e'.or('E'), opt('-'), Digit(10).many()).optional(),
            )
                .slice(),
        )?;

        let float: f64 = slice
            .value
            .as_str()
            .parse()
            .map_err(|err: core::num::ParseFloatError| reader.error(err.to_string()))?;

        Ok(Item::new(slice.span, float))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Peek((Integer, '.')))
    }
}

#[cfg(test)]
mod test {
    use udled::Input;

    use super::{Float, Integer};

    #[test]
    fn integer() {
        let mut input = Input::new("10203 0 42");

        let (a, _, b, _, c) = input.parse((Integer, ' ', Integer, ' ', Integer)).unwrap();

        assert_eq!(a.value, 10203);
        assert_eq!(b.value, 0);
        assert_eq!(c.value, 42);
    }

    // #[test]
    // fn int() {
    //     let mut input = Input::new("0x202 0b11 42");

    //     let (a, _, b, _, c) = input.parse((Int, Ws, Int, Ws, Int)).unwrap();

    //     assert_eq!(a.value, 0x202);
    //     assert_eq!(b.value, 0b11);
    //     assert_eq!(c.value, 42);
    // }

    #[test]
    fn float() {
        let mut input = Input::new("1.0000033 2003.303 12.03e-20");

        let (a, _, b, _, c) = input.parse((Float, ' ', Float, ' ', Float)).unwrap();

        assert_eq!(a.value, 1.0000033);
        assert_eq!(b.value, 2003.303);
        assert_eq!(c.value, 12.03e-20);
    }
}
