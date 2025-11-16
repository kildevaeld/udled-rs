use core::fmt;

use udled::{
    bytes::{Endian, FromBytesExt},
    Input,
};

type FmtResult = Result<usize, fmt::Error>;

use crate::{
    util::{read_f64, read_u32},
    Coord, GeoType, GeomB,
};

pub fn display_geometry(geo: &GeomB, f: &mut fmt::Formatter) -> fmt::Result {
    let output = geo.slice();

    let endian = match output[0] {
        0 => Endian::Big,
        1 => Endian::Lt,
        _ => return Err(fmt::Error),
    };

    let srid = read_u32(&output[1..], endian);

    write!(f, "SRID={srid};")?;

    display_inner(&output[5..], endian, f)?;

    Ok(())
}

fn display_inner(out: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let ty = Input::new(&*out).parse(GeoType::byteorder(endian)).unwrap();

    let len = match ty.value {
        GeoType::Point => {
            write!(f, "POINT(")?;
            let ret = display_coords(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
        GeoType::LineString => {
            write!(f, "LINESTRING(")?;
            let ret = display_line_string(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
        GeoType::Polygon => {
            write!(f, "POLYGON(")?;
            let ret = display_polygon(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
        GeoType::MultiPoint => {
            write!(f, "MULTIPOINT(")?;
            let ret = display_line_string(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
        GeoType::MultiLineString => {
            write!(f, "MULTILINESTRING(")?;
            let ret = display_polygon(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }

        GeoType::MultiPolygon => {
            write!(f, "MULTIPOLYGON(")?;
            let ret = display_multipolygon(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
        GeoType::Collection => {
            write!(f, "GEOMETRYCOLLECTION(")?;
            let ret = display_collection(&out[1..], endian, f)?;
            write!(f, ")")?;
            ret
        }
    };

    Ok(1 + len)
}

fn display_coords(buf: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let x = read_f64(&*buf, endian);
    let y = read_f64(&buf[8..], endian);

    write!(f, "{} {}", x, y)?;

    Ok(16)
}

pub fn display_line_string(buf: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    for i in 0..num {
        if i > 0 {
            write!(f, ", ")?;
        }
        let offset = offset + (i * Coord::SIZE);
        display_coords(&buf[offset..(offset + Coord::SIZE)], endian, f)?;
    }

    Ok(offset + num * Coord::SIZE)
}

fn display_polygon(buf: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for i in 0..num {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "(")?;
        size += display_line_string(&buf[size..], endian, f)?;
        write!(f, ")")?;
    }

    Ok(size)
}

fn display_multipolygon(buf: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for i in 0..num {
        if i > 0 {
            write!(f, ", ")?;
        }
        write!(f, "(")?;
        size += display_polygon(&buf[size..], endian, f)?;
        write!(f, ")")?;
    }

    Ok(size)
}

fn display_collection(buf: &[u8], endian: Endian, f: &mut fmt::Formatter) -> FmtResult {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for i in 0..num {
        if i > 0 {
            write!(f, ", ")?;
        }
        size += display_inner(&buf[size..], endian, f)?;
    }

    Ok(size)
}
