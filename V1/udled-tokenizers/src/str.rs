use alloc::string::ToString;
use udled::{Lex, Span, Tokenizer};
use unicode_segmentation::UnicodeSegmentation;

pub struct IgnoreCase<T>(pub T);

impl<T> Tokenizer for IgnoreCase<T>
where
    T: AsRef<str>,
{
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let tokens = self.0.as_ref().graphemes(true);

        let start = reader.position();

        for token in tokens {
            let next = reader.eat_ch()?;
            if token.to_lowercase() != next.to_lowercase() {
                return Err(reader.error(self.0.as_ref().to_string()));
            }
        }

        if start == reader.position() {
            return Err(reader.error(self.0.as_ref().to_string()));
        }

        let span = Span::new(start, reader.position());
        let lex = span.slice(reader.source()).expect("Slice");

        Ok(Lex::new(lex, span))
    }
}

#[cfg(test)]
mod test {
    use udled::Input;

    use super::IgnoreCase;

    macro_rules! parse {
        ($parser: literal, $($input:literal),+) => {
          $(
            let mut input = Input::new($input);
            let ret = input.parse(IgnoreCase($parser)).expect("parse");

            assert_eq!($input,ret.as_str());
          )+
        };
    }

    #[test]

    fn ignore_case() {
        parse!("DOCTYPE", "docType", "DOCTYPE", "DocType");
        parse!("ÆæpÅLLÆ", "ææpållæ");
    }
}
