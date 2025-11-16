use core::fmt;

use udled::{
    bytes::{Endian, FromBytes, FromBytesExt},
    AsBytes, AsSlice, Buffer, Next, TokenizerExt,
};

use crate::binary::coords::Coord;

pub type MultiPoint<'a> = LineString<'a>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineString<'a> {
    slice: &'a [u8],
    num: u32,
    endian: Endian,
}

impl<'a> fmt::Debug for LineString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = f.debug_list();

        for i in 0..self.len() {
            let Some(m) = self.get(i) else {
                return Err(fmt::Error);
            };

            v.entry(&m);
        }

        v.finish()?;

        Ok(())
    }
}

impl<'a> LineString<'a> {
    pub fn len(&self) -> usize {
        self.num as _
    }

    pub fn get(&self, idx: usize) -> Option<Coord<'a>> {
        if idx >= self.len() {
            return None;
        }

        let buf_idx = idx * Coord::SIZE;

        Some(Coord {
            slice: &self.slice[buf_idx..(buf_idx + Coord::SIZE)],
            endian: self.endian,
        })
    }

    pub fn coords(&self) -> LineStringIter<'a> {
        LineStringIter {
            line: *self,
            idx: 0,
        }
    }
}

pub struct LineStringIter<'a> {
    line: LineString<'a>,
    idx: usize,
}

impl<'a> Iterator for LineStringIter<'a> {
    type Item = Coord<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.line.get(self.idx)?;
        self.idx += 1;
        Some(next)
    }
}

impl<'input, B> FromBytes<'input, B> for LineString<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let num = reader.parse(u32::byteorder(byteorder))?;
        let byte_len = (num.value as usize) * Coord::SIZE;
        let slice = reader.parse(Next.repeat(byte_len as _).slice())?;
        Ok(LineString {
            slice: slice.value.as_bytes(),
            endian: byteorder,
            num: num.value as _,
        })
    }
}
