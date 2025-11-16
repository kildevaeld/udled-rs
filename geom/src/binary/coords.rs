use core::fmt;

use udled::{
    bytes::{Endian, FromBytes},
    AsBytes, AsSlice, Buffer, Next, TokenizerExt,
};

use crate::util::read_f64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord<'a> {
    pub(crate) slice: &'a [u8],
    pub(crate) endian: Endian,
}

impl<'a> fmt::Debug for Coord<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Coord")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl<'a> Coord<'a> {
    pub const SIZE: usize = 16;
    pub fn x(&self) -> f64 {
        read_f64(self.slice, self.endian)
    }

    pub fn y(&self) -> f64 {
        read_f64(&self.slice[8..], self.endian)
    }
}

impl<'input, B> FromBytes<'input, B> for Coord<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let slice = reader.parse(Next.repeat(Coord::SIZE as _).slice())?;
        Ok(Coord {
            slice: slice.value.as_bytes(),
            endian: byteorder,
        })
    }
}
