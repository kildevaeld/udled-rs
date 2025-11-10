use crate::{AsChar, Buffer, Char, Item, Tokenizer};

macro_rules! impls {
    ($($name: ident => $method: ident),+) => {
      $(
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<'input, B> Tokenizer<'input, B> for $name
        where
            B: Buffer<'input>,
            B::Item: AsChar,
        {
            type Token = Item<char>;

            fn to_token(
                &self,
                reader: &mut crate::Reader<'_, 'input, B>,
            ) -> Result<Self::Token, crate::Error> {
                let char = reader.parse(Char)?;
                if !char.value.$method() {
                    return Err(reader.error(stringify!($name)));
                }

                Ok(char)
            }
        }
      )+
    };
}

impls!(
  Alphabetic => is_alphabetic,
  AlphaNumeric => is_alphanumeric,
  Punct => is_ascii_punctuation,
  Numeric => is_numeric,
  AsciiWhiteSpace => is_ascii_whitespace,
  WhiteSpace => is_whitespace,
  LineFeed => is_linefeed
);
