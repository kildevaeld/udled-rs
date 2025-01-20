use alloc::{vec, vec::Vec};
use udled::{Error, Item, Reader, Span, Tokenizer};

/// Match a list of T's separated by P's.
/// Possible to allow trailing P's
#[derive(Debug, Clone, Copy, Default)]
pub struct Punctuated<T, P>(pub T, pub P, pub bool);

impl<T, P> Punctuated<T, P> {
    pub fn new(item: T, punct: P) -> Punctuated<T, P> {
        Punctuated(item, punct, false)
    }

    pub fn with_trailing(mut self, trailing: bool) -> Punctuated<T, P> {
        self.2 = trailing;
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

/// Match a group of O T C
/// Match a Item<T> with a span covering the full match
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

#[cfg(test)]
mod test {
    use udled::{
        token::{Opt, Ws},
        Input, Lex,
    };

    use crate::Ident;

    use super::*;

    #[test]
    fn punctuated() {
        let mut input = Input::new("ident ,identto, 202,");

        assert_eq!(
            input
                .parse(Punctuated(Ident, Group(Opt(Ws), ',', Opt(Ws)), true))
                .unwrap(),
            Item {
                value: vec![
                    Lex::new("ident", Span::new(0, 5)),
                    Lex::new("identto", Span::new(7, 12))
                ],
                span: Span::new(0, 13)
            }
        );
        assert!(input.parse(Punctuated(Ident, ',', false)).is_err())
    }
}
