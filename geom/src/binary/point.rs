use core::fmt;

use udled::{
    bytes::{Endian, FromBytes},
    AsBytes, AsSlice, Buffer, Next, TokenizerExt,
};

use crate::util::read_f64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<'a> {
    pub(crate) slice: &'a [u8],
    pub(crate) endian: Endian,
}

impl<'a> fmt::Debug for Point<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .finish()
    }
}

impl<'a> Point<'a> {
    pub const SIZE: usize = 16;
    pub fn x(&self) -> f64 {
        read_f64(self.slice, self.endian)
    }

    pub fn y(&self) -> f64 {
        read_f64(&self.slice[8..], self.endian)
    }
}

impl<'input, B> FromBytes<'input, B> for Point<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let slice = reader.parse(Next.repeat(Point::SIZE as _).slice())?;
        Ok(Point {
            slice: slice.value.as_bytes(),
            endian: byteorder,
        })
    }
}

// impl<'input> geozero::GeozeroGeometry for Point<'input> {
//     fn process_geom<P: geozero::GeomProcessor>(
//         &self,
//         processor: &mut P,
//     ) -> geozero::error::Result<()>
//     where
//         Self: Sized,
//     {
//         processor.point_begin(0)?;
//         processor.xy(self.x(), self.y(), 0)?;
//         processor.point_end(0)?;

//         Ok(())
//     }
// }
