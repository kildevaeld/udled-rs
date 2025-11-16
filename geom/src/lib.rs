mod binary;
pub mod text;
mod util;
mod writer;

use std::sync::Arc;

use udled::bytes::Endian;

use crate::text::parse;

pub use self::binary::*;

pub struct GeomB(Arc<[u8]>);

impl GeomB {
    pub(crate) fn new(bytes: Vec<u8>) -> GeomB {
        GeomB(bytes.into())
    }

    pub fn from_text(input: &str) -> udled::Result<GeomB> {
        parse(input, Endian::native())
    }

    pub fn geometry(&self) -> Geometry<'_> {
        Geometry::from_bytes(&self.0).unwrap()
    }
}
