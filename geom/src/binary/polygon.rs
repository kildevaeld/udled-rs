use alloc::vec::Vec;
use udled::{
    bytes::{Endian, FromBytes, FromBytesExt},
    AsBytes, AsSlice, Buffer, TokenizerExt,
};

use super::line_string::LineString;

pub type MultiLineString<'a> = Polygon<'a>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Polygon<'a> {
    lines: Vec<LineString<'a>>,
}

impl<'a> Polygon<'a> {
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn get(&self, idx: usize) -> Option<&LineString<'a>> {
        self.lines.get(idx)
    }

    pub fn rings(&self) -> core::slice::Iter<'_, LineString<'a>> {
        self.lines.iter()
    }
}

impl<'input, B> FromBytes<'input, B> for Polygon<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let num = reader.parse(u32::byteorder(byteorder))?;
        let slice = reader.parse(
            LineString::byteorder(byteorder)
                .map_ok(|v| v.value)
                .repeat(num.value as _),
        )?;
        Ok(Polygon { lines: slice.value })
    }

    fn eat(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<()> {
        let num = reader.parse(u32::byteorder(byteorder))?;
        reader.eat(
            LineString::byteorder(byteorder)
                .map_ok(|v| v.value)
                .repeat(num.value as _),
        )?;
        Ok(())
    }
}
