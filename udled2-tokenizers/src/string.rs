use udled2::{
    any, AsChar, AsSlice, AsStr, Buffer, Char, Error, Exclude, Item, Reader, Span, Tokenizer,
    TokenizerExt, EOF,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Str;

impl<'input, B> Tokenizer<'input, B> for Str
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    type Token = Item<&'input str>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        reader
            .parse(
                (
                    '"',
                    ('\\', any!('\\', '\'', '"', 'r', 'n', '0'))
                        .or(Exclude::new('\\'.or('"')))
                        .until('"'),
                    '"'.map_err(|_, _| format!("Expected unicode string")),
                )
                    .slice(),
            )
            .map(|m| m.map(|m| m.as_str()))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is('"')
    }
}

#[cfg(test)]
mod test {

    use udled2::Input;

    use super::*;

    #[test]
    fn empty_string() {
        let mut input = Input::new(r#""""#);
        let str = input.parse(Str).unwrap();
        assert_eq!(str.value, "");
        assert_eq!(str.span, Span::new(0, 2));
    }

    #[test]
    fn string() {
        let mut input = Input::new(r#""Hello, World!""#);
        let str = input.parse(Str).unwrap();
        assert_eq!(str.value, "Hello, World!");
        assert_eq!(str.span, Span::new(0, 15));
    }
}
