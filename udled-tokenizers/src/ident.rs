use udled::{
    any, AlphaNumeric, Alphabetic, AsChar, AsSlice, Buffer, Error, Item, Reader, Tokenizer,
    TokenizerExt,
};

/// Match a unicode identifier
#[derive(Debug, Clone, Copy, Default)]
pub struct Ident;

impl<'input, B> Tokenizer<'input, B> for Ident
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let item =
            reader.parse((Alphabetic.or('_'), AlphaNumeric.or('_').many().optional()).slice())?;

        Ok(item)
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Alphabetic.or('_'))
    }
}

/// Match a xml style tag or attribute
pub struct XmlIdent;

impl XmlIdent {}

impl<'input, B> Tokenizer<'input, B> for XmlIdent
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
{
    type Token = Item<<B::Source as AsSlice<'input>>::Slice>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
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
            '.',
            '_',
            '\u{00B7}',
            '\u{0300}'..='\u{036F}',
            '\u{203F}'..='\u{2040}'
        );

        let all = any!(&start_tokenizer, rest_tokenizer);

        reader.parse((&start_tokenizer, all.many()).slice())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(any!(
            ':',
            'a'..='z',
            'A'..='Z',
            '\u{2070}'..='\u{218F}',
            '\u{2C00}'..='\u{2FEF}',
            '\u{3001}'..='\u{D7FF}',
            '\u{F900}'..='\u{FDCF}',
            '\u{FDF0}'..='\u{FFFD}'
        ))
    }
}

// #[cfg(test)]
// mod test {
//     use udled::{Input, Item, Span};

//     use super::{Ident, XmlIdent};

//     #[test]
//     fn xml_ident() {
//         let mut input = Input::new("div custom-tag data-id2");

//         assert_eq!(
//             input
//                 .parse((XmlIdent, ' ', XmlIdent, ' ', XmlIdent))
//                 .unwrap(),
//             (
//                 Item::new(, Span::new(0, 3), "div"),
//                 Span::new(3, 4),
//                 Item::new("custom-tag", Span::new(4, 14)),
//                 Span::new(14, 15),
//                 Item::new("data-id2", Span::new(15, 23))
//             )
//         );
//     }

//     #[test]
//     fn ident() {
//         let mut input = Input::new("Ident other");
//         assert_eq!(
//             input.parse(Ident).unwrap(),
//             Lex {
//                 value: "Ident",
//                 span: Span { start: 0, end: 5 }
//             }
//         );
//     }
// }
