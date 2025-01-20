use udled::{any, token::Spanned, Lex, Tokenizer};

pub struct XmlIdent;

impl Tokenizer for XmlIdent {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let start_tokenizer = any!(
            ':',
            'a'..='z',
            'A'..='Z',
            '\u{2070}'..='\u{218F}',
            '\u{2C00}'..='\u{2FEF}',
            '\u{3001}'..='\u{D7FF}',
            '\u{F900}'..='\u{FDCF}',
            '\u{FDF0}'..='\u{FFFD}'
        );
        let rest_tokenizer = any!(
            '0'..='9',
            '-',
            ".",
            '_',
            '\u{00B7}',
            '\u{0300}'..='\u{036F}',
            '\u{203F}'..='\u{2040}'
        );

        let all = any!(&start_tokenizer, rest_tokenizer);

        let start = reader.parse(Spanned(&start_tokenizer))?;
        let mut end = start;

        loop {
            if reader.eof() {
                break;
            }

            if !reader.peek(&all)? {
                break;
            }

            end = reader.parse(Spanned(&all))?;
        }

        let span = start + end;

        if let Some(content) = span.slice(reader.source()) {
            Ok(Lex::new(content, span))
        } else {
            Err(reader.error("Invalid range"))
        }
    }
}

#[cfg(test)]
mod test {
    use udled::{token::Ws, Input, Lex, Span};

    use crate::XmlIdent;

    #[test]
    fn xml_ident() {
        let mut input = Input::new("div custom-tag data-id2");

        assert_eq!(
            input.parse((XmlIdent, Ws, XmlIdent, Ws, XmlIdent)).unwrap(),
            (
                Lex::new("div", Span::new(0, 3)),
                Span::new(3, 4),
                Lex::new("custom-tag", Span::new(4, 14)),
                Span::new(14, 15),
                Lex::new("data-id2", Span::new(15, 23))
            )
        );
    }
}
