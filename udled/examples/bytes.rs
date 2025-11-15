use core::fmt;

use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian};
use geo_traits::{CoordTrait, GeometryTrait, LineStringTrait, PointTrait, PolygonTrait};
use geo_types::{line_string, polygon};
use udled::{
    bytes::{Endian, FromBytes},
    prelude::*,
    AsBytes, AsSlice, Buffer, Input, Next, Tokenizer,
};

pub trait ToBytes {
    fn write<W: BinaryWriter>(&self, output: &mut W, endian: Endian);
}

impl ToBytes for f64 {
    fn write<W: BinaryWriter>(&self, bytes: &mut W, endian: Endian) {
        bytes.write_f64::<NativeEndian>(*self);
    }
}

fn main() -> udled::Result<()> {
    let mut bytes = [0; 12];

    NativeEndian::write_i32(&mut bytes[0..4], 42);
    NativeEndian::write_f64(&mut bytes[4..], 84.2);

    let mut input = Input::new(&bytes[..]);

    let ans = input.parse(i32::native())?;
    let w = input.parse(f64::native())?;

    let lt = line_string![
         (x: -21.95156, y: 64.1446),
        (x: -21.951, y: 64.14479),
        (x: -21.95044, y: 64.14527),
        (x: -21.951445, y: 64.145508),
    ];

    let lt = polygon![
        (x: -111., y: 45.),
        (x: -111., y: 41.),
        (x: -104., y: 41.),
        (x: -104., y: 45.),
    ];

    let out = process(&lt, 2030, Endian::Lt);
    let geo = Input::new(&*out).parse(GeometryParser)?;

    println!("{:?},{:?}", ans, w);
    println!("{:?}", geo);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Geometry<'a> {
    srid: i32,
    kind: GeoKind<'a>,
}

pub struct GeometryParser;

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
        let endian = match reader.read()? {
            0 => Endian::Big,
            1 => Endian::Lt,
            _ => return Err(reader.error("byteorder")),
        };

        let srid = reader.parse(i32::byteorder(endian))?;

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
            _ => {
                todo!()
            }
        };

        Ok(Geometry {
            srid: srid.value,
            kind,
        })
    }
}

// https://libgeos.org/specifications/wkb/
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum GeoKind<'a> {
    Point(Point<'a>),
    Path(LineString<'a>),
    Polygon(Polygon<'a>),
}

fn read_f64(buf: &[u8], endian: Endian) -> f64 {
    match endian {
        Endian::Big => BigEndian::read_f64(buf),
        Endian::Lt => LittleEndian::read_f64(buf),
    }
}

fn read_u32(buf: &[u8], endian: Endian) -> u32 {
    match endian {
        Endian::Big => BigEndian::read_u32(buf),
        Endian::Lt => LittleEndian::read_u32(buf),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point<'a> {
    slice: &'a [u8],
    endian: Endian,
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
    const SIZE: usize = 16;
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineString<'a> {
    slice: &'a [u8],
    num: u32,
    endian: Endian,
}

impl<'a> fmt::Debug for LineString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

    pub fn get(&self, idx: usize) -> Option<Point<'a>> {
        if idx >= self.len() {
            return None;
        }

        let buf_idx = idx * Point::SIZE;

        Some(Point {
            slice: &self.slice[buf_idx..(buf_idx + Point::SIZE)],
            endian: self.endian,
        })
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
        let byte_len = (num.value as usize) * Point::SIZE;
        let slice = reader.parse(Next.repeat(byte_len as _).slice())?;
        Ok(LineString {
            slice: slice.value.as_bytes(),
            endian: byteorder,
            num: num.value as _,
        })
    }
}

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

pub trait BinaryWriter {
    fn write_all(&mut self, bytes: &[u8]);

    fn write_u8(&mut self, n: u8) {
        self.write_all(&[n]);
    }

    fn write_u16<T: ByteOrder>(&mut self, n: u16) {
        let mut v = [0; 2];
        T::write_u16(&mut v, n);
        self.write_all(&v);
    }

    fn write_u32<T: ByteOrder>(&mut self, n: u32) {
        let mut v = [0; 4];
        T::write_u32(&mut v, n);
        self.write_all(&v);
    }

    fn write_i32<T: ByteOrder>(&mut self, n: i32) {
        let mut v = [0; 4];
        T::write_i32(&mut v, n);
        self.write_all(&v);
    }

    fn write_f64<T: ByteOrder>(&mut self, n: f64) {
        let mut v = [0; 8];
        T::write_f64(&mut v, n);
        self.write_all(&v);
    }
}

impl BinaryWriter for Vec<u8> {
    fn write_all(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

fn process<T: geo_traits::GeometryTrait<T = f64>>(geo: &T, srid: i32, endian: Endian) -> Vec<u8> {
    let mut output = Vec::default();
    let bo = match endian {
        Endian::Big => 0u8,
        Endian::Lt => 1u8,
    };

    output.write_u8(bo);

    output.write_i32::<NativeEndian>(srid);

    process_inner(geo, &mut output, endian, true);

    output
}

fn process_inner<T: geo_traits::GeometryTrait<T = f64>>(
    geo: &T,
    output: &mut Vec<u8>,
    endian: Endian,
    top: bool,
) {
    match geo.as_type() {
        geo_traits::GeometryType::Point(point) => {
            if top {
                output.write_u8(1);
            }

            let (x, y) = point.coord().unwrap().x_y();
            x.write(output, endian);
            y.write(output, endian);
        }
        geo_traits::GeometryType::LineString(line) => {
            if top {
                output.write_u8(2);
            }
            output.write_u32::<NativeEndian>(line.num_coords() as _);
            for c in line.coords() {
                let (x, y) = c.x_y();
                x.write(output, endian);
                y.write(output, endian);
            }
        }
        geo_traits::GeometryType::Polygon(polygon) => {
            if top {
                output.write_u8(3);
            }

            let has_ext = polygon.exterior().is_some();
            let num = if has_ext { 1 } else { 0 } + polygon.num_interiors();

            output.write_u32::<NativeEndian>(num as _);

            if let Some(ext) = polygon.exterior() {
                process_inner(&ext, output, endian, false);
            }

            for i in polygon.interiors() {
                process_inner(&i, output, endian, false);
            }
        }
        geo_traits::GeometryType::MultiPoint(_) => todo!(),
        geo_traits::GeometryType::MultiLineString(_) => todo!(),
        geo_traits::GeometryType::MultiPolygon(_) => todo!(),
        geo_traits::GeometryType::GeometryCollection(_) => todo!(),
        geo_traits::GeometryType::Rect(_) => todo!(),
        geo_traits::GeometryType::Triangle(_) => todo!(),
        geo_traits::GeometryType::Line(_) => todo!(),
    }
}
