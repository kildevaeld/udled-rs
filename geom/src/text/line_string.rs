use byteorder::{BigEndian, LittleEndian};
use udled::{
    bytes::Endian, AsBytes, AsChar, AsSlice, AsStr, Buffer, Input, IntoTokenizer, Reader,
    Tokenizer, TokenizerExt,
};
use udled_tokenizers::Float;

use crate::{
    text::{
        common::ws,
        point::{parse_coord, parse_coords},
    },
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
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
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
