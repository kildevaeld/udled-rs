use alloc::string::ToString;

use crate::{AsBytes, AsChar, Buffer, Item, Reader, Result, Span, Tokenizer};

pub struct IgnoreCase<T>(pub T);

impl<'lit, T, B> Tokenizer<'lit, B> for IgnoreCase<T>
where
    T: AsRef<str>,
    B: Buffer<'lit>,
    B::Item: AsChar,
    B::Source: AsBytes<'lit>,
{
    type Token = Item<&'lit str>;
    fn to_token(&self, reader: &mut Reader<'_, 'lit, B>) -> Result<Self::Token> {
        let tokens = self.0.as_ref().chars();

        let start = reader.position();

        for token in tokens {
            let Some(next) = reader.read()?.as_char() else {
                return Err(reader.error(self.0.as_ref().to_string()));
            };
            if token != next {
                return Err(reader.error(self.0.as_ref().to_string()));
            }
        }

        if start == reader.position() {
            return Err(reader.error(self.0.as_ref().to_string()));
        }

        let span = Span {
            start,
            end: reader.position(),
        };

        let string = reader.buffer().source().as_bytes();
        let string = unsafe { core::str::from_utf8_unchecked(string) };

        Ok(Item {
            value: span.slice(string).unwrap(),
            span,
        })
    }

    fn peek(&self, reader: &mut Reader<'_, 'lit, B>) -> bool {
        let tokens = self.0.as_ref().chars();
        for (idx, next) in tokens.enumerate() {
            if Some(next) == reader.peek_chn(idx).and_then(|m| m.as_char()) {
                continue;
            }
            return false;
        }

        true
    }
}

#[cfg(test)]
mod test {
    use crate::Input;

    use super::IgnoreCase;

    macro_rules! parse {
        ($parser: literal, $($input:literal),+) => {
          $(
            let mut input = Input::new($input);
            let ret = input.parse(IgnoreCase($parser)).expect("parse");

            assert_eq!($input,ret.value);
          )+
        };
    }

    #[test]

    fn ignore_case() {
        parse!("DOCTYPE", "docType", "DOCTYPE", "DocType");
        parse!("ÆæpÅLLÆ", "ææpållæ");
    }
}
