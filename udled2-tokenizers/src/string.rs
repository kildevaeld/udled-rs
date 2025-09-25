use udled2::{AsChar, AsSlice, AsStr, Buffer, Char, Error, Item, Reader, Span, Tokenizer, EOF};

#[derive(Debug, Clone, Copy, Default)]
pub struct Str;

impl<'input, B> Tokenizer<'input, B> for Str
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    type Token = Item<&'input str>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.parse('"')?;

        let end = loop {
            if reader.peek(EOF) {
                return Err(reader.error("Unexpected end of input while parsing string literal"));
            }

            // if !reader.peek(('\0'..'\x1f').or('\x22').or('\x5C')) {
            //     return;
            // }

            let ch = reader.parse(Char)?;

            if ch.value == '"' {
                break ch.span;
            }

            if ch.value == '\\' {
                match reader.parse(Char)?.value {
                    '\\' | '\'' | '"' | 't' | 'r' | 'n' | '0' => {
                        continue;
                    }
                    _ => return Err(reader.error("Unknown escape sequence")),
                }
            }
        };

        let span = start.span + end;

        let str = if span.len() == 2 {
            Some("")
        } else {
            reader
                .buffer()
                .source()
                .sliced(Span::new(span.start + 1, span.end - 1))
                .map(|m| m.as_str())
        };

        Ok(Item::new(span, str.unwrap()))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.peek('"')
    }
}

#[cfg(test)]
mod test {

    use udled2::Input;

    use super::*;

    #[test]
    fn empty_string() {
        let mut input = Input::new(r#""""#);
        let str = input.parse(Str).unwrap();
        assert_eq!(str.value, "");
        assert_eq!(str.span, Span::new(0, 2));
    }

    #[test]
    fn string() {
        let mut input = Input::new(r#""Hello, World!""#);
        let str = input.parse(Str).unwrap();
        assert_eq!(str.value, "Hello, World!");
        assert_eq!(str.span, Span::new(0, 15));
    }
}
