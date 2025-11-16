use proj::Proj;
use udled::{
    bytes::{Endian, FromBytesExt},
    Input,
};

use crate::{
    util::{read_f64, read_u32, write_f64, write_u32},
    Coord, GeoType, GeomB,
};

impl GeomB {
    pub fn project(&mut self, to: u32) {
        let proj = Proj::new_known_crs(
            &format!("EPSG:{}", self.geometry().srid()),
            &format!("EPSG:{}", to),
            None,
        )
        .unwrap();

        project(&proj, self, to);
    }

    pub fn project_into(&self, to: u32) -> GeomB {
        let mut this = self.clone();

        this.project(to);

        this
    }
}

struct ProjCoord(f64, f64);

impl proj::Coord<f64> for ProjCoord {
    fn x(&self) -> f64 {
        self.0
    }

    fn y(&self) -> f64 {
        self.1
    }

    fn from_xy(x: f64, y: f64) -> Self {
        Self(x, y)
    }
}

fn project(proj: &Proj, geo: &mut GeomB, to: u32) {
    let endian = geo.geometry().endian();
    let output = geo.slice_mut();

    write_u32(&mut output[1..], to, endian);

    project_inner(proj, &mut output[5..], endian);
}

fn project_inner(proj: &Proj, out: &mut [u8], endian: Endian) -> usize {
    let ty = Input::new(&*out).parse(GeoType::byteorder(endian)).unwrap();

    let len = match ty.value {
        GeoType::Point => project_coords(proj, &mut out[1..], endian),
        GeoType::LineString => project_line_string(proj, &mut out[1..], endian),
        GeoType::Polygon => project_polygon(proj, &mut out[1..], endian),
        GeoType::MultiPoint => project_line_string(proj, &mut out[1..], endian),
        GeoType::MultiLineString => project_polygon(proj, &mut out[1..], endian),
        GeoType::MultiPolygon => project_multipolygon(proj, &mut out[1..], endian),
        GeoType::Collection => project_collection(proj, &mut out[1..], endian),
    };

    1 + len
}

fn project_coords(proj: &Proj, buf: &mut [u8], endian: Endian) -> usize {
    let x = read_f64(&*buf, endian);
    let y = read_f64(&buf[8..], endian);

    let coords = proj.convert(ProjCoord(x, y)).unwrap();

    write_f64(buf, coords.0, endian);
    write_f64(&mut buf[8..], coords.1, endian);

    16
}

pub fn project_line_string(proj: &Proj, buf: &mut [u8], endian: Endian) -> usize {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    for i in 0..num {
        let offset = offset + (i * Coord::SIZE);
        project_coords(proj, &mut buf[offset..(offset + Coord::SIZE)], endian);
    }

    offset + num * Coord::SIZE
}

fn project_polygon(proj: &Proj, buf: &mut [u8], endian: Endian) -> usize {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for _ in 0..num {
        size += project_line_string(proj, &mut buf[size..], endian);
    }

    size
}

fn project_multipolygon(proj: &Proj, buf: &mut [u8], endian: Endian) -> usize {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for _ in 0..num {
        size += project_polygon(proj, &mut buf[size..], endian);
    }

    size
}

fn project_collection(proj: &Proj, buf: &mut [u8], endian: Endian) -> usize {
    let num = read_u32(buf, endian) as usize;
    let offset = 4;

    let mut size = offset;

    for _ in 0..num {
        size += project_inner(proj, &mut buf[size..], endian);
    }

    size
}
