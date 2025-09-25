use udled2::{or, AsChar, AsStr, Buffer, Error, Item, Reader, Tokenizer, TokenizerExt};

/// Match 'true' or 'false'
#[derive(Debug, Clone, Copy, Default)]
pub struct Bool;

impl<'input, B> Tokenizer<'input, B> for Bool
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsStr<'input>,
{
    type Token = Item<bool>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let item = reader
            .parse(or(
                "true".map_ok(|m| m.map(|_| true)),
                "false".map_ok(|m| m.map(|_| false)),
            ))?
            .unify();

        Ok(item)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        reader.eat(or("true", "false"))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.peek(or("true", "false"))
    }
}

#[cfg(test)]
mod test {
    use udled2::{Input, EOF};

    use super::Bool;

    #[test]
    fn bool() {
        let mut input = Input::new("true false");

        let (a, _, b) = input.parse((Bool, ' ', Bool)).unwrap();

        assert_eq!(a.value, true);
        assert_eq!(b.value, false);
        assert!(input.peek(EOF))
    }
}
