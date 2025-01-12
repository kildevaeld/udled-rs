use udled::{token::Spanned, Either, Error, Lex, Reader, Span, Tokenizer};

pub const fn cstyle_line_comment() -> RawLineComment<&'static str> {
    RawLineComment("//")
}

pub const fn cstyle_multiline_comment(
    nested: bool,
) -> Either<RawMultiLine<&'static str, &'static str>, RawMultiLineNested<&'static str, &'static str>>
{
    if nested {
        Either::Right(RawMultiLineNested("/*", "*/"))
    } else {
        Either::Left(RawMultiLine("/*", "*/"))
    }
}

pub const fn rust_doc_comment() -> RawLineComment<&'static str> {
    RawLineComment("///")
}

pub const fn python_line_comment() -> RawLineComment<&'static str> {
    RawLineComment("#")
}

pub const fn python_multiline_comment() -> RawMultiLine<&'static str, &'static str> {
    RawMultiLine("'''", "'''")
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LineComment;

impl Tokenizer for LineComment {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        reader.parse(cstyle_line_comment())
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("//")
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MultiLineComment;

impl Tokenizer for MultiLineComment {
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        reader
            .parse(cstyle_multiline_comment(true))
            .map(|m| m.unify())
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("/*")
    }
}

pub struct RawLineComment<T>(T);

impl<T> Tokenizer for RawLineComment<T>
where
    T: Tokenizer,
{
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let _ = reader.parse(&self.0)?;

        loop {
            let Some(ch) = reader.peek_ch() else {
                break;
            };

            if ch == "\0" || ch == "\n" {
                break;
            }

            let _ = reader.eat_ch()?;
        }

        let span = Span::new(start, reader.position());

        Ok(Lex {
            value: span.slice(reader.input()).expect("slice"),
            span,
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(&self.0)
    }
}

pub struct RawMultiLine<O, C>(O, C);

impl<O, C> Tokenizer for RawMultiLine<O, C>
where
    O: Tokenizer,
    C: Tokenizer,
{
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.parse(Spanned(&self.0))?;

        let end = loop {
            if reader.eof() {
                return Err(reader.error("unexpected end of input inside multi-line comment"));
            } else if let Ok(end) = reader.parse(Spanned(&self.1)) {
                break end;
            } else {
                reader.eat_ch()?;
            }
        };

        let span = start + end;

        Ok(Lex {
            value: span.slice(reader.input()).expect("slice"),
            span,
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("/*")
    }
}

pub struct RawMultiLineNested<O, C>(O, C);

impl<O, C> Tokenizer for RawMultiLineNested<O, C>
where
    O: Tokenizer,
    C: Tokenizer,
{
    type Token<'a> = Lex<'a>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let start = reader.position();

        let _ = reader.parse(&self.0)?;

        let mut depth = 1;

        loop {
            if reader.eof() {
                return Err(reader.error("unexpected end of input inside multi-line comment"));
            } else if reader.parse(&self.0).is_ok() {
                depth += 1;
            } else if reader.parse(&self.1).is_ok() {
                depth -= 1;

                if depth == 0 {
                    break;
                }
            } else {
                reader.eat_ch()?;
            }
        }

        let span = Span::new(start, reader.position());

        Ok(Lex {
            value: span.slice(reader.input()).expect("slice"),
            span,
        })
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek("/*")
    }
}

#[cfg(test)]
mod test {
    use udled::Input;

    use super::*;

    #[test]
    fn line_comment() {
        let mut input = Input::new("//");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new("//", Span::new(0, 2))
        );

        let mut input = Input::new("// Some tekst");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new("// Some tekst", Span::new(0, 13))
        );
        let mut input = Input::new("// Some tekst\n test");
        assert_eq!(
            input.parse(LineComment).unwrap(),
            Lex::new("// Some tekst", Span::new(0, 13))
        );
    }
}
