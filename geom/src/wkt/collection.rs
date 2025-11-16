use alloc::boxed::Box;
use byteorder::{BigEndian, LittleEndian};
use udled::{bytes::Endian, AsBytes, AsChar, Buffer, IntoTokenizer, Reader, TokenizerExt};

use crate::{
    wkt::{common::ws, parse_geometry},
    writer::{BinaryWriter, ToBytes},
    GeoType,
};

pub fn parse_collection<'input, B, W>(
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

    input.eat(("GEOMETRYCOLLECTION", &ws_opt, '('))?;
    let mut count = 0u32;

    let pos = out.position();

    count.write(out, endian).map_err(|err| input.error(err))?;

    if write_type {
        GeoType::Collection
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    loop {
        input.eat(&ws_opt)?;
        if input.is(')') {
            break;
        }

        if count > 0 {
            input.eat((',', &ws_opt))?;
        }

        count += 1;

        parse_geometry(input, out, endian)?;
    }
    input.eat((ws_opt, ')'))?;

    match endian {
        Endian::Big => out.write_u32_at::<BigEndian>(pos, count),
        Endian::Lt => out.write_u32_at::<LittleEndian>(pos, count),
    }
    .map_err(|err| input.error(err))?;

    Ok(())
}
