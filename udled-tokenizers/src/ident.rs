use udled::{any, token::Spanned, Lex, Span, StringExt, Tokenizer};

/// Match a unicode identifier
#[derive(Debug, Clone, Copy, Default)]
pub struct Ident;

impl Tokenizer for Ident {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let start_idx = reader.position();

        let mut end_idx = start_idx;

        let Some(first) = reader.peek_ch() else {
            return Err(reader.error("expected identifier"));
        };

        if !first.is_alphabetic() && first != "_" {
            return Err(reader.error("expected identifier"));
        }

        loop {
            let Some(ch) = reader.peek_ch() else {
                break;
            };

            if ch == "\0" {
                break;
            }

            if !ch.is_ascii_alphanumeric() && ch != "_" {
                break;
            }

            end_idx += 1;

            reader.eat_ch()?;
        }

        if start_idx == end_idx {
            return Err(reader.error("expected identifier"));
        }

        let ret = &reader.source()[start_idx..reader.position()];

        Ok(Lex::new(ret, Span::new(start_idx, reader.position())))
    }

    fn peek<'a>(&self, reader: &mut udled::Reader<'_, '_>) -> Result<bool, udled::Error> {
        let ch = reader.eat_ch()?;
        Ok(ch.is_alphabetic() || ch == "_")
    }
}

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

    use super::{Ident, XmlIdent};

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

    #[test]
    fn ident() {
        let mut input = Input::new("Ident other");
        assert_eq!(
            input.parse(Ident).unwrap(),
            Lex {
                value: "Ident",
                span: Span { start: 0, end: 5 }
            }
        );
    }
}
