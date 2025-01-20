use udled::{any, token::Or, Either, Error, Item, Reader, Tokenizer};

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
        reader.peek(any!("true", "false"))
    }
}
