use udled2::{
    AsChar, AsSlice, Buffer, Either, Error, Exclude, Item, Reader, Span, Tokenizer, TokenizerExt,
    EOF,
};

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

pub const fn javascript_doc_comment() -> RawMultiLine<&'static str, &'static str> {
    RawMultiLine("/**", "*/")
}

pub const fn html_comment() -> RawMultiLine<&'static str, &'static str> {
    RawMultiLine("<!--", "-->")
}

#[derive(Debug, Clone, Copy)]
pub struct RawLineComment<T>(T);

impl<'input, B, T> Tokenizer<'input, B> for RawLineComment<T>
where
    T: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Source: AsSlice<'input>,
    B::Item: AsChar,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let item = reader.parse(
            (
                &self.0,
                Exclude::new('\n').many().optional().spanned(),
                '\n'.optional(),
            )
                .slice(),
        )?;

        Ok(item)
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.peek(&self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RawMultiLine<O, C>(O, C);

impl<'input, O, C, B> Tokenizer<'input, B> for RawMultiLine<O, C>
where
    O: Tokenizer<'input, B>,
    C: Tokenizer<'input, B>,
    B: Buffer<'input>,
    B::Source: AsSlice<'input>,
    B::Item: AsChar,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let item =
            reader.parse((&self.0, Exclude::new(&self.1).many().optional(), &self.1).slice())?;

        Ok(item)
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.peek(&self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RawMultiLineNested<O, C>(O, C);

impl<'input, O, C, B> Tokenizer<'input, B> for RawMultiLineNested<O, C>
where
    B: Buffer<'input>,
    B::Source: AsSlice<'input>,
    B::Item: AsChar,
    O: Tokenizer<'input, B>,
    C: Tokenizer<'input, B>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;
    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let start = reader.position();

        reader.eat(&self.0)?;

        let mut depth = 1;

        loop {
            if reader.peek(EOF) {
                return Err(reader.error("unexpected end of input inside multi-line comment"));
            } else if reader.eat(&self.0).is_ok() {
                depth += 1;
            } else if reader.eat(&self.1).is_ok() {
                depth -= 1;

                if depth == 0 {
                    break;
                }
            } else {
                reader.eat_ch()?;
            }
        }

        let span = Span::new(start, reader.position());

        let Some(value) = reader.buffer().source().sliced(span) else {
            return Err(reader.error("slice"));
        };

        Ok(Item::new(span, value))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.peek(&self.0)
    }
}

#[cfg(test)]
mod test {
    use udled2::Input;

    use super::*;

    #[test]
    fn line_comment() {
        let mut input = Input::new("//");
        assert_eq!(
            input.parse(cstyle_line_comment()).unwrap(),
            Item::new(Span::new(0, 2), "//")
        );

        let mut input = Input::new("// Some tekst");
        assert_eq!(
            input.parse(cstyle_line_comment()).unwrap(),
            Item::new(Span::new(0, 13), "// Some tekst")
        );
        let mut input = Input::new("// Some tekst\n test");
        assert_eq!(
            input.parse(cstyle_line_comment()).unwrap(),
            Item::new(Span::new(0, 14), "// Some tekst\n")
        );
    }
}
