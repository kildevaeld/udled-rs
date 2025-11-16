use alloc::vec::Vec;
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, Input, Tokenizer};
use udled_tokenizers::Integer;

use crate::{wkt::geometry::parse_geometry, writer::ToBytes, GeomB};

mod collection;
mod common;
mod display;
mod geometry;
mod line_string;
mod multi_line_string;
mod point;
mod polygon;

pub use self::display::display_geometry;

pub fn parse(input: &str, endian: Endian) -> udled::Result<GeomB> {
    Input::new(input.as_bytes())
        .parse(Parser(endian))
        .map(GeomB::new)
}

struct Parser(Endian);

impl<'input, B> Tokenizer<'input, B> for Parser
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsBytes<'input>,
{
    type Token = Vec<u8>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, udled::Error> {
        let mut output = Vec::<u8>::default();

        let endian = self.0;

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

        parse_geometry(reader, &mut output, endian)?;

        Ok(output)
    }
}
