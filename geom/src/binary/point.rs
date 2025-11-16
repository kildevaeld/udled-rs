use core::fmt;

use udled::{
    bytes::{Endian, FromBytes, FromBytesExt},
    AsBytes, AsSlice, Buffer,
};

use crate::binary::coords::Coord;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Point<'a>(pub(crate) Coord<'a>);

impl<'a> fmt::Debug for Point<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl<'a> Point<'a> {
    pub const SIZE: usize = 16;
    pub fn x(&self) -> f64 {
        self.0.x()
    }

    pub fn y(&self) -> f64 {
        self.0.y()
    }

    pub fn coord(&self) -> Coord<'a> {
        self.0
    }
}

impl<'input, B> FromBytes<'input, B> for Point<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let coord = reader.parse(Coord::byteorder(byteorder))?;
        Ok(Point(coord.value))
    }
}
