use alloc::boxed::Box;
use byteorder::{BigEndian, LittleEndian};
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, IntoTokenizer, Reader, TokenizerExt};

use crate::{
    wkt::{common::ws, point::parse_coords},
    writer::{BinaryWriter, ToBytes},
    GeoType,
};

pub fn parse_multi_line_string<'input, B, W>(
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

    input.eat(("MULTILINESTRING", ws_opt))?;

    if write_type {
        GeoType::MultiLineString
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    parse_multi_line_string_inner(input, out, endian)
}

pub fn parse_multi_line_string_inner<'input, B, W>(
    input: &mut Reader<'_, 'input, B>,
    out: &mut W,
    endian: Endian,
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

    input.eat((&ws_opt, '('))?;
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
