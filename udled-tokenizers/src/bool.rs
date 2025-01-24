use udled::{any, token::Or, Either, Error, Item, Reader, Tokenizer};

/// Match 'true' or 'false'
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

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(Or("true", "false"))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(any!("true", "false"))
    }
}

#[cfg(test)]
mod test {
    use udled::{token::Ws, Input};

    use super::Bool;

    #[test]
    fn bool() {
        let mut input = Input::new("true false");

        let (a, _, b) = input.parse((Bool, Ws, Bool)).unwrap();

        assert_eq!(a.value, true);
        assert_eq!(b.value, false);
        assert!(input.eos())
    }
}
