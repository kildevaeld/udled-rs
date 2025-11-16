use alloc::boxed::Box;
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, IntoTokenizer, Reader, TokenizerExt};

use crate::{
    wkt::{common::ws, point::parse_coords},
    writer::{BinaryWriter, ToBytes},
    GeoType,
};

pub fn parse_line_string<'input, B, W>(
    input: &mut Reader<'_, 'input, B>,
    out: &mut W,
    endian: Endian,
    write_type: bool,
) -> udled::Result<()>
where
    W: BinaryWriter,
    W::Error: Into<Box<dyn core::error::Error + Send + Sync>>,
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsBytes<'input>,
{
    let ws = ws.into_tokenizer();
    let ws_opt = ws.optional();

    input.eat(("LINESTRING", ws_opt))?;

    if write_type {
        GeoType::LineString
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    parse_coords(input, out, endian)?;

    Ok(())
}
