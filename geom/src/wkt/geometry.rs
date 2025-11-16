use alloc::boxed::Box;
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, Reader};

use crate::{
    wkt::{
        collection::parse_collection, line_string::parse_line_string,
        multi_line_string::parse_multi_line_string, point::parse_point, polygon::parse_polyon,
    },
    writer::BinaryWriter,
};

pub fn parse_geometry<'input, B, W>(
    reader: &mut Reader<'_, 'input, B>,
    output: &mut W,
    endian: Endian,
) -> udled::Result<()>
where
    W: BinaryWriter,
    W::Error: Into<Box<dyn core::error::Error + Send + Sync>>,
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsBytes<'input>,
{
    if reader.is("POINT") {
        parse_point(reader, output, endian, true)?;
    } else if reader.is("LINESTRING") {
        parse_line_string(reader, output, endian, true)?;
    } else if reader.is("POLYGON") {
        parse_polyon(reader, output, endian, true)?;
    } else if reader.is("MULTILINESTRING") {
        parse_multi_line_string(reader, output, endian, true)?;
    } else if reader.is("GEOMETRYCOLLECTION") {
        parse_collection(reader, output, endian, true)?;
    } else {
        return Err(reader.error("Geometry"));
    }

    Ok(())
}
