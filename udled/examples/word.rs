use udled::{
    prelude::*,
    tokenizers::{Alphabetic, AsciiWhiteSpace, Punct},
    AsChar, AsSlice, Buffer, Error, Input, Item, Reader, Tokenizer,
};

struct Word;

impl<'input, B> Tokenizer<'input, B> for Word
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        reader.parse(Alphabetic.many().slice())
    }
}

fn main() -> udled::Result<()> {
    let mut input = Input::new("Hello, World!");

    let (greeting, _, _, subject, _) = input.parse((Word, Punct, AsciiWhiteSpace, Word, Punct))?;

    assert_eq!(greeting.value, "Hello");
    assert_eq!(subject.value, "World");

    Ok(())
}
