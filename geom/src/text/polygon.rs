use byteorder::{BigEndian, LittleEndian};
use udled::{
    bytes::Endian, AsBytes, AsChar, AsSlice, AsStr, Buffer, Input, IntoTokenizer, Reader,
    Tokenizer, TokenizerExt,
};
use udled_tokenizers::Float;

use crate::{
    text::{
        common::ws,
        line_string::parse_line_string,
        point::{parse_coord, parse_coords},
    },
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
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    let ws = ws.into_tokenizer();
    let ws_opt = ws.optional();

    input.eat(("POLYGON", ws_opt, '('))?;

    if write_type {
        GeoType::Polygon
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    let mut count = 0u32;

    let pos = out.position();

    count.write(out, endian).map_err(|err| input.error(err))?;

    loop {
        input.eat(&ws_opt)?;
        if input.is(')') {
            break;
        }

        if count > 0 {
            input.eat((',', &ws_opt))?;
        }

        count += 1;

        parse_coords(input, out, endian)?;
    }
    input.eat((ws_opt, ')'))?;

    match endian {
        Endian::Big => out.write_u32_at::<BigEndian>(pos, count),
        Endian::Lt => out.write_u32_at::<LittleEndian>(pos, count),
    }
    .map_err(|err| input.error(err))?;

    Ok(())
}
