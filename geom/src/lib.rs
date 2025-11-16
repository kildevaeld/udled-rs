#![no_std]

extern crate alloc;

mod binary;
mod util;
pub mod wkt;
mod writer;

#[cfg(feature = "geo-types")]
mod geotypes;
#[cfg(feature = "proj")]
mod projection;

use alloc::{sync::Arc, vec::Vec};
use core::fmt;

use udled::{
    bytes::{Endian, FromBytesExt},
    Input,
};

use crate::{
    util::{get_endian, read_u32},
    wkt::parse,
};

pub use self::binary::*;

#[derive(Clone)]
pub struct GeomB(Arc<[u8]>);

impl fmt::Debug for GeomB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GeomB").field(&self.geometry()).finish()
    }
}

impl fmt::Display for GeomB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        wkt::display_geometry(self, f)
    }
}

impl GeomB {
    pub fn from_text(input: &str) -> udled::Result<GeomB> {
        parse(input, Endian::native())
    }

    pub fn from_bytes(input: Vec<u8>) -> Option<GeomB> {
        if !Geometry::validate(&input) {
            None
        } else {
            Some(GeomB::new(input))
        }
    }

    pub fn srid(&self) -> u32 {
        let Some(endian) = get_endian(self.0[0]) else {
            panic!("Could not get endian")
        };

        read_u32(&self.0[1..], endian)
    }

    pub fn kind(&self) -> GeoType {
        let Some(endian) = get_endian(self.0[0]) else {
            panic!("Could not get endian")
        };

        Input::new(&self.0[5..])
            .parse(GeoType::byteorder(endian))
            .unwrap()
            .value
    }

    pub fn geometry(&self) -> Geometry<'_> {
        Geometry::from_bytes(&self.0).unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl GeomB {
    #[allow(unused)]
    pub(crate) fn slice_mut(&mut self) -> &mut [u8] {
        Arc::make_mut(&mut self.0)
    }

    pub(crate) fn slice(&self) -> &[u8] {
        &self.0
    }

    pub(crate) fn new(bytes: Vec<u8>) -> GeomB {
        GeomB(bytes.into())
    }
}
