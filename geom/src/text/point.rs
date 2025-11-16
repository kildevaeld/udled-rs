use byteorder::{BigEndian, LittleEndian};
use udled::{
    bytes::Endian, AsBytes, AsChar, AsSlice, AsStr, Buffer, Input, IntoTokenizer, Reader,
    Tokenizer, TokenizerExt,
};
use udled_tokenizers::Float;

use crate::{
    text::common::ws,
    writer::{BinaryWriter, ToBytes},
    GeoType,
};

pub fn parse_point<'input, B, W>(
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

    input.eat(("POINT", ws_opt, '('))?;
    let (x, y) = input.parse((Float, &ws, Float).map_ok(|(x, _, y)| (x.value, y.value)))?;
    input.eat((ws_opt, ')'))?;

    if write_type {
        GeoType::Point
            .write(out, endian)
            .map_err(|err| input.error(err))?;
    }

    x.write(out, endian).map_err(|err| input.error(err))?;
    y.write(out, endian).map_err(|err| input.error(err))?;

    Ok(())
}

pub fn parse_coord<'input, B, W>(
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
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    let ws = ws.into_tokenizer();
    let ws_opt = ws.optional();

    let (x, y) = input.parse((Float, &ws, Float).map_ok(|(x, _, y)| (x.value, y.value)))?;

    x.write(out, endian).map_err(|err| input.error(err))?;
    y.write(out, endian).map_err(|err| input.error(err))?;

    Ok(())
}

pub fn parse_coords<'input, B, W>(
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
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    let ws = ws.into_tokenizer();
    let ws_opt = ws.optional();

    input.eat('(')?;

    let pos = out.position();

    0u32.write(out, endian).map_err(|err| input.error(err))?;

    let mut first = 0u32;

    loop {
        input.eat(&ws_opt)?;
        if input.is(')') {
            break;
        }

        if first > 0 {
            input.eat((',', &ws_opt))?;
        }

        first += 1;

        parse_coord(input, out, endian)?;

        input.eat(&ws_opt)?;
    }
    input.eat((ws_opt, ')'))?;

    match endian {
        Endian::Big => out.write_u32_at::<BigEndian>(pos, first),
        Endian::Lt => out.write_u32_at::<LittleEndian>(pos, first),
    }
    .map_err(|err| input.error(err))?;

    Ok(())
}
