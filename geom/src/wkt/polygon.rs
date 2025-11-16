use alloc::boxed::Box;
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, IntoTokenizer, Reader, TokenizerExt};

use crate::{
    wkt::{common::ws, multi_line_string::parse_multi_line_string_inner},
    writer::{BinaryWriter, ToBytes},
    GeoType,
};

pub fn parse_polyon<'input, B, W>(
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

    input.eat(("POLYGON", ws_opt))?;

    if write_type {
        GeoType::Polygon
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    parse_multi_line_string_inner(input, out, endian)?;

    Ok(())
}
