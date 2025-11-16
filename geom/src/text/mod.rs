use udled::{bytes::Endian, AsBytes, AsChar, AsSlice, AsStr, Buffer, Input, Tokenizer};
use udled_tokenizers::Integer;

use crate::{
    text::{line_string::parse_line_string, point::parse_point, polygon::parse_polyon},
    writer::ToBytes,
    GeomB,
};

mod common;
mod line_string;
mod point;
mod polygon;

pub fn parse(input: &str, endian: Endian) -> udled::Result<GeomB> {
    Input::new(input).parse(Parser).map(GeomB::new)
}

pub struct Parser;

impl<'input, B> Tokenizer<'input, B> for Parser
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsBytes<'input>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    type Token = Vec<u8>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, udled::Error> {
        let mut output = Vec::<u8>::default();

        let endian = Endian::native();

        match endian {
            Endian::Big => {
                output.push(0);
            }
            Endian::Lt => {
                output.push(1);
            }
        }

        let (_, srid, _) = reader.parse((("SRID", "="), Integer, ";"))?;

        (srid.value as u32)
            .write(&mut output, endian)
            .map_err(|err| reader.error(err))?;

        if reader.is("POINT") {
            parse_point(reader, &mut output, endian, true)?;
        } else if reader.is("LINESTRING") {
            parse_line_string(reader, &mut output, endian, true)?;
        } else if reader.is("POLYGON") {
            parse_polyon(reader, &mut output, endian, true)?;
        }

        Ok(output)
    }
}
