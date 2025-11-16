use core::mem::transmute;

use alloc::fmt;
use udled::{
    bytes::{Endian, FromBytes, FromBytesExt},
    AsBytes, AsSlice, Buffer, Input, Tokenizer,
};

use crate::{util::get_endian, writer::ToBytes};

use super::{
    collection::GeometryCollection,
    line_string::{LineString, MultiPoint},
    multi_polygon::MultiPolygon,
    point::Point,
    polygon::{MultiLineString, Polygon},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Geometry<'a> {
    srid: u32,
    kind: GeoKind<'a>,
    endian: Endian,
}

impl<'a> Geometry<'a> {
    pub fn srid(&self) -> u32 {
        self.srid
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn kind(&self) -> &GeoKind<'a> {
        &self.kind
    }

    pub fn from_bytes(bytes: &[u8]) -> udled::Result<Geometry<'_>> {
        Input::new(bytes).parse(GeometryParser)
    }

    pub fn validate(bytes: &[u8]) -> bool {
        Input::new(bytes).eat(GeometryParser).is_ok()
    }
}

struct GeometryParser;

impl<'input, B> Tokenizer<'input, B> for GeometryParser
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    type Token = Geometry<'input>;

    fn to_token(
        &self,
        reader: &mut udled::Reader<'_, 'input, B>,
    ) -> Result<Self::Token, udled::Error> {
        let Some(endian) = get_endian(reader.read()?) else {
            return Err(reader.error("byteorder"));
        };

        let srid = reader.parse(u32::byteorder(endian))?;

        let kind = reader.parse(GeoKind::byteorder(endian))?;

        Ok(Geometry {
            srid: srid.value,
            kind: kind.value,
            endian,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum GeoType {
    Point = 1,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    Collection,
}

impl fmt::Display for GeoType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            GeoType::Point => "POINT",
            GeoType::LineString => "LINESTRING",
            GeoType::Polygon => "POLYGON",
            GeoType::MultiPoint => "MULTIPOINT",
            GeoType::MultiLineString => "MULTILINESTRING",
            GeoType::MultiPolygon => "MULTIPOLYGON",
            GeoType::Collection => "GEOMETRYCOLLECTION",
        };
        f.write_str(s)
    }
}

impl<'input, B> FromBytes<'input, B> for GeoType
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, byteorder: Endian) -> udled::Result<Self> {
        let ty = reader.parse(u8::byteorder(byteorder))?;
        if ty.value > 0 && ty.value <= 7 {
            let ty = unsafe { transmute::<u8, GeoType>(ty.value) };
            Ok(ty)
        } else {
            Err(reader.error("GeoType"))
        }
    }
}

impl ToBytes for GeoType {
    fn write<W: crate::writer::BinaryWriter>(
        &self,
        output: &mut W,
        _endian: Endian,
    ) -> Result<(), W::Error> {
        output.write_u8(*self as _)
    }
}

// https://libgeos.org/specifications/wkb/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeoKind<'a> {
    Point(Point<'a>),
    MultiPoint(MultiPoint<'a>),
    Path(LineString<'a>),
    MultiLineString(MultiLineString<'a>),
    Polygon(Polygon<'a>),
    MultiPolygon(MultiPolygon<'a>),
    Collection(GeometryCollection<'a>),
}

impl<'input, B> FromBytes<'input, B> for GeoKind<'input>
where
    B: Buffer<'input, Item = u8>,
    B::Source: AsSlice<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
{
    fn parse(reader: &mut udled::Reader<'_, 'input, B>, endian: Endian) -> udled::Result<Self> {
        let ty = reader.read()?;

        let kind = match ty {
            1 => {
                let point = reader.parse(Point::byteorder(endian))?;
                GeoKind::Point(point.value)
            }
            2 => {
                let line = reader.parse(LineString::byteorder(endian))?;
                GeoKind::Path(line.value)
            }
            3 => {
                let line = reader.parse(Polygon::byteorder(endian))?;
                GeoKind::Polygon(line.value)
            }
            4 => {
                let line = reader.parse(LineString::byteorder(endian))?;
                GeoKind::MultiPoint(line.value)
            }
            5 => {
                let line = reader.parse(Polygon::byteorder(endian))?;
                GeoKind::MultiLineString(line.value)
            }
            6 => {
                let line = reader.parse(MultiPolygon::byteorder(endian))?;
                GeoKind::MultiPolygon(line.value)
            }
            7 => {
                let line = reader.parse(GeometryCollection::byteorder(endian))?;
                GeoKind::Collection(line.value)
            }
            _ => {
                todo!()
            }
        };

        Ok(kind)
    }

    fn eat(reader: &mut udled::Reader<'_, 'input, B>, endian: Endian) -> udled::Result<()> {
        let ty = reader.read()?;

        match ty {
            1 => {
                reader.eat(Point::byteorder(endian))?;
            }
            2 => {
                reader.eat(LineString::byteorder(endian))?;
            }
            3 => {
                reader.eat(Polygon::byteorder(endian))?;
            }
            4 => {
                reader.eat(LineString::byteorder(endian))?;
            }
            5 => {
                reader.eat(Polygon::byteorder(endian))?;
            }
            6 => {
                reader.eat(MultiPolygon::byteorder(endian))?;
            }
            7 => {
                reader.eat(GeometryCollection::byteorder(endian))?;
            }
            _ => {
                todo!()
            }
        };

        Ok(())
    }
}
