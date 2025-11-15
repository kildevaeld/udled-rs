use alloc::{vec, vec::Vec};
use udled::{Error, Item, Reader, Span, Tokenizer};

/// Match a list of T's separated by P's.
/// Possible to allow trailing P's
#[derive(Debug, Clone, Copy, Default)]
pub struct Punctuated<T, P> {
    item: T,
    punct: P,
    trailing: bool,
}

impl<T, P> Punctuated<T, P> {
    pub const fn new(item: T, punct: P) -> Punctuated<T, P> {
        Punctuated {
            item,
            punct,
            trailing: false,
        }
    }

    pub const fn with_trailing(mut self, trailing: bool) -> Punctuated<T, P> {
        self.trailing = trailing;
        self
    }
}

impl<T, P> Tokenizer for Punctuated<T, P>
where
    T: Tokenizer,
    P: Tokenizer,
{
    type Token<'a> = Item<Vec<T::Token<'a>>>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();
        let item = reader.parse(&self.item)?;

        let mut output = vec![item];
        loop {
            if reader.eof() || !reader.peek(&self.punct)? {
                break;
            }

            reader.eat(&self.punct)?;

            if self.trailing && (reader.eof() || !reader.peek(&self.item)?) {
                break;
            }

            let item = reader.parse(&self.item)?;
            output.push(item);
        }

        let end = reader.position();

        Ok(Item::new(output, Span::new(start, end)))
    }

    fn eat(&self, reader: &mut Reader<'_, '_>) -> Result<(), Error> {
        reader.eat(&self.item)?;

        loop {
            if reader.eof() || !reader.peek(&self.punct)? {
                break;
            }

            reader.eat(&self.punct)?;

            if self.trailing && (reader.eof() || !reader.peek(&self.item)?) {
                break;
            }

            reader.eat(&self.item)?;
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(&self.item)
    }
}

#[cfg(test)]
mod test {
    use udled::{Input, Lex};

    use crate::Ident;

    use super::*;

    #[test]
    fn punctuated() {
        let mut input = Input::new("ident,identto,");

        let ret = input
            .parse(Punctuated::new(Ident, ',').with_trailing(true))
            .unwrap();

        assert_eq!(
            ret.value,
            vec![
                Lex::new("ident", Span::default()),
                Lex::new("identto", Span::default())
            ]
        )
    }
}
