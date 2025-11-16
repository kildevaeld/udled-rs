use alloc::vec::Vec;
use udled::{
    bytes::{Endian, FromBytes, FromBytesExt},
    AsBytes, AsSlice, Buffer, TokenizerExt,
};

use super::geometry::GeoKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeometryCollection<'a> {
    lines: Vec<GeoKind<'a>>,
}

impl<'a> GeometryCollection<'a> {
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn get(&self, idx: usize) -> Option<&GeoKind<'a>> {
        self.lines.get(idx)
    }

    pub fn shapes(&self) -> core::slice::Iter<'_, GeoKind<'a>> {
        self.lines.iter()
    }
}

impl<'input, B> FromBytes<'input, B> for GeometryCollection<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let num = reader.parse(u32::byteorder(byteorder))?;
        let slice = reader.parse(
            GeoKind::byteorder(byteorder)
                .map_ok(|v| v.value)
                .repeat(num.value as _),
        )?;
        Ok(GeometryCollection { lines: slice.value })
    }

    fn eat(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<()> {
        let num = reader.parse(u32::byteorder(byteorder))?;
        reader.eat(
            GeoKind::byteorder(byteorder)
                .map_ok(|v| v.value)
                .repeat(num.value as _),
        )?;
        Ok(())
    }
}
