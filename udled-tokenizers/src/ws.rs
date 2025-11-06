use udled::{AsChar, AsSlice, AsStr, Buffer, Char, Item, Tokenizer, TokenizerExt};

#[derive(Debug, Clone, Copy)]
pub struct Whitespace;

impl<'input, T> Tokenizer<'input, T> for Whitespace
where
    T: Buffer<'input>,
    T::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, T>,
    ) -> Result<Self::Token, udled::Error> {
        let char = reader.parse(Char)?;
        if char.value.is_whitespace() {
            Ok(char)
        } else {
            Err(reader.error("Whitespace"))
        }
    }

    fn eat(&self, reader: &mut udled::Reader<'_, 'input, T>) -> Result<(), udled::Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut udled::Reader<'_, 'input, T>) -> bool {
        let Ok(ret) = reader.parse(Char) else {
            return false;
        };

        ret.value.is_whitespace()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AsciiWhitespace;

impl<'input, T> Tokenizer<'input, T> for AsciiWhitespace
where
    T: Buffer<'input>,
    T::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, T>,
    ) -> Result<Self::Token, udled::Error> {
        let char = reader.parse(Char)?;
        if char.value.is_whitespace() {
            Ok(char)
        } else {
            Err(reader.error("Whitespace"))
        }
    }

    fn eat(&self, reader: &mut udled::Reader<'_, 'input, T>) -> Result<(), udled::Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut udled::Reader<'_, 'input, T>) -> bool {
        let Ok(ret) = reader.parse(Char) else {
            return false;
        };

        ret.value.is_whitespace()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LineFeed;

impl<'input, T> Tokenizer<'input, T> for LineFeed
where
    T: Buffer<'input>,
    T::Item: AsChar,
    T::Source: AsStr<'input> + AsSlice<'input>,
{
    type Token = Item<<T::Source as AsSlice<'input>>::Slice>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, T>,
    ) -> Result<Self::Token, udled::Error> {
        let Ok(char) = reader.parse('\n'.or('\r').or('\u{2028}').or('\u{2029}').slice()) else {
            return Err(reader.error("Line Feed"));
        };
        Ok(char)
    }

    fn eat(&self, reader: &mut udled::Reader<'_, 'input, T>) -> Result<(), udled::Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut udled::Reader<'_, 'input, T>) -> bool {
        reader.is('\n'.or('\r').or('\u{2028}').or('\u{2029}'))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Space;

impl<'input, T> Tokenizer<'input, T> for Space
where
    T: Buffer<'input>,
    T::Item: AsChar,
{
    type Token = Item<char>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, T>,
    ) -> Result<Self::Token, udled::Error> {
        let char = reader.parse(' '.or('\t'))?.unify();
        Ok(char)
    }

    fn eat(&self, reader: &mut udled::Reader<'_, 'input, T>) -> Result<(), udled::Error> {
        let _ = self.to_token(reader)?;
        Ok(())
    }

    fn peek(&self, reader: &mut udled::Reader<'_, 'input, T>) -> bool {
        reader.is(' '.or('\t'))
    }
}
