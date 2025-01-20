use udled::{token::Char, Error, Lex, Reader, Span, Tokenizer};

#[derive(Debug, Clone, Copy, Default)]
pub struct Str;

impl Tokenizer for Str {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.parse('"')?;

        let end = loop {
            if reader.eof() {
                return Err(reader.error("Unexpected end of input while parsing string literal"));
            }

            let ch = reader.parse(Char)?;

            if ch == r#"""# {
                break ch.span;
            }

            if ch == "\\" {
                match reader.eat_ch()? {
                    "\\" | "\'" | "\"" | "t" | "r" | "n" | "0" => {
                        continue;
                    }
                    _ => return Err(reader.error("Unknown escape sequence")),
                }
            }
        };

        let span = start + end;

        let str = if span.len() == 2 {
            Span::new(span.start + 1, span.end).slice(reader.source())
        } else {
            Span::new(span.start + 1, span.end - 1).slice(reader.source())
        };

        Ok(Lex::new(str.unwrap(), span))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek('"')
    }
}
