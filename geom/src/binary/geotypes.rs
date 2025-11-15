use super::{
    geometry::{GeoKind, GeoType},
    GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use crate::writer::{BinaryWriter, ToBytes};
use geo_traits::{
    CoordTrait, GeometryCollectionTrait, LineStringTrait, MultiLineStringTrait, MultiPointTrait,
    MultiPolygonTrait, PointTrait, PolygonTrait, UnimplementedLine, UnimplementedRect,
    UnimplementedTriangle,
};
use udled::bytes::Endian;

pub fn process<T: geo_traits::GeometryTrait<T = f64>, W: BinaryWriter>(
    geo: &T,
    srid: i32,
    endian: Endian,
    output: &mut W,
) -> Result<(), W::Error> {
    let bo = match endian {
        Endian::Big => 0u8,
        Endian::Lt => 1u8,
    };

    output.write_u8(bo)?;

    srid.write(output, endian)?;

    process_inner(geo, output, endian, true)?;

    Ok(())
}

fn process_inner<T: geo_traits::GeometryTrait<T = f64>, W: BinaryWriter>(
    geo: &T,
    output: &mut W,
    endian: Endian,
    top: bool,
) -> Result<(), W::Error> {
    match geo.as_type() {
        geo_traits::GeometryType::Point(point) => {
            if top {
                GeoType::Point.write(output, endian)?;
            }

            let (x, y) = point.coord().unwrap().x_y();
            x.write(output, endian)?;
            y.write(output, endian)?;
        }
        geo_traits::GeometryType::LineString(line) => {
            if top {
                GeoType::LineString.write(output, endian)?;
            }
            (line.num_coords() as u32).write(output, endian)?;

            for c in line.coords() {
                let (x, y) = c.x_y();
                x.write(output, endian)?;
                y.write(output, endian)?;
            }
        }
        geo_traits::GeometryType::Polygon(polygon) => {
            if top {
                GeoType::Polygon.write(output, endian)?;
            }

            let has_ext = polygon.exterior().is_some();
            let num = if has_ext { 1 } else { 0 } + polygon.num_interiors();

            (num as u32).write(output, endian)?;

            if let Some(ext) = polygon.exterior() {
                process_inner(&ext, output, endian, false)?;
            }

            for i in polygon.interiors() {
                process_inner(&i, output, endian, false)?;
            }
        }
        geo_traits::GeometryType::MultiPoint(mp) => {
            if top {
                GeoType::MultiPoint.write(output, endian)?;
            }

            (mp.num_points() as u32).write(output, endian)?;

            for point in mp.points() {
                process_inner(&point, output, endian, false)?;
            }
        }
        geo_traits::GeometryType::MultiLineString(ml) => {
            if top {
                GeoType::MultiLineString.write(output, endian)?;
            }

            (ml.num_line_strings() as u32).write(output, endian)?;

            for point in ml.line_strings() {
                process_inner(&point, output, endian, false)?;
            }
        }
        geo_traits::GeometryType::MultiPolygon(mp) => {
            if top {
                GeoType::MultiPolygon.write(output, endian)?;
            }

            (mp.num_polygons() as u32).write(output, endian)?;

            for point in mp.polygons() {
                process_inner(&point, output, endian, false)?;
            }
        }
        geo_traits::GeometryType::GeometryCollection(col) => {
            if top {
                GeoType::MultiPolygon.write(output, endian)?;
            }

            (col.num_geometries() as u32).write(output, endian)?;

            for point in col.geometries() {
                process_inner(&point, output, endian, true)?;
            }
        }
        geo_traits::GeometryType::Rect(_) => todo!(),
        geo_traits::GeometryType::Triangle(_) => todo!(),
        geo_traits::GeometryType::Line(_) => todo!(),
    };

    Ok(())
}

impl<'input> CoordTrait for Point<'input> {
    type T = f64;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x()
    }

    fn y(&self) -> Self::T {
        self.y()
    }

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("invalid range"),
        }
    }
}

macro_rules! geo {
    ($ty: ident => $variant: ident) => {
        impl<'input> geo_traits::GeometryTrait for $ty<'input> {
            type T = f64;

            type PointType<'a>
                = Point<'a>
            where
                Self: 'a;

            type LineStringType<'a>
                = LineString<'a>
            where
                Self: 'a;

            type PolygonType<'a>
                = Polygon<'a>
            where
                Self: 'a;

            type MultiPointType<'a>
                = MultiPoint<'a>
            where
                Self: 'a;

            type MultiLineStringType<'a>
                = MultiLineString<'a>
            where
                Self: 'a;

            type MultiPolygonType<'a>
                = MultiPolygon<'a>
            where
                Self: 'a;

            type GeometryCollectionType<'a>
                = GeometryCollection<'a>
            where
                Self: 'a;

            type RectType<'a>
                = UnimplementedRect<Self::T>
            where
                Self: 'a;

            type TriangleType<'a>
                = UnimplementedTriangle<Self::T>
            where
                Self: 'a;

            type LineType<'a>
                = UnimplementedLine<Self::T>
            where
                Self: 'a;

            fn dim(&self) -> geo_traits::Dimensions {
                geo_traits::Dimensions::Xy
            }

            fn as_type(
                &self,
            ) -> geo_traits::GeometryType<
                '_,
                Self::PointType<'_>,
                Self::LineStringType<'_>,
                Self::PolygonType<'_>,
                Self::MultiPointType<'_>,
                Self::MultiLineStringType<'_>,
                Self::MultiPolygonType<'_>,
                Self::GeometryCollectionType<'_>,
                Self::RectType<'_>,
                Self::TriangleType<'_>,
                Self::LineType<'_>,
            > {
                geo_traits::GeometryType::$variant(self)
            }
        }
    };
}

impl<'input> geo_traits::PointTrait for Point<'input> {
    type CoordType<'a>
        = Self
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(*self)
    }
}

geo!(Point => Point);

impl<'input> geo_traits::LineStringTrait for LineString<'input> {
    type CoordType<'a>
        = Point<'a>
    where
        Self: 'a;

    fn num_coords(&self) -> usize {
        self.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.get(i).unwrap()
    }
}

geo!(LineString => LineString);

impl<'input> geo_traits::PolygonTrait for Polygon<'input> {
    type RingType<'a>
        = LineString<'a>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.get(0).copied()
    }

    fn num_interiors(&self) -> usize {
        let len = self.len();
        if len == 0 {
            0
        } else {
            len - 1
        }
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.get(i + 1).copied().unwrap()
    }
}

geo!(Polygon => Polygon);

impl<'input> geo_traits::MultiPolygonTrait for MultiPolygon<'input> {
    type InnerPolygonType<'a>
        = Polygon<'input>
    where
        Self: 'a;

    fn num_polygons(&self) -> usize {
        self.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
        self.get(i).cloned().unwrap()
    }
}

geo!(MultiPolygon => MultiPolygon);

impl<'input> geo_traits::MultiPointTrait for MultiPoint<'input> {
    type InnerPointType<'a>
        = Point<'a>
    where
        Self: 'a;

    fn num_points(&self) -> usize {
        self.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
        self.get(i).unwrap()
    }
}

impl<'input> geo_traits::MultiLineStringTrait for MultiLineString<'input> {
    type InnerLineStringType<'a>
        = LineString<'a>
    where
        Self: 'a;

    fn num_line_strings(&self) -> usize {
        self.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::InnerLineStringType<'_> {
        self.get(i).copied().unwrap()
    }
}

impl<'input> geo_traits::GeometryCollectionTrait for GeometryCollection<'input> {
    type GeometryType<'a>
        = GeoKind<'a>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        todo!()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        todo!()
    }
}

geo!(GeometryCollection => GeometryCollection);

impl<'input> geo_traits::GeometryTrait for GeoKind<'input> {
    type T = f64;

    type PointType<'a>
        = Point<'a>
    where
        Self: 'a;

    type LineStringType<'a>
        = LineString<'a>
    where
        Self: 'a;

    type PolygonType<'a>
        = Polygon<'a>
    where
        Self: 'a;

    type MultiPointType<'a>
        = MultiPoint<'a>
    where
        Self: 'a;

    type MultiLineStringType<'a>
        = MultiLineString<'a>
    where
        Self: 'a;

    type MultiPolygonType<'a>
        = MultiPolygon<'a>
    where
        Self: 'a;

    type GeometryCollectionType<'a>
        = GeometryCollection<'a>
    where
        Self: 'a;

    type RectType<'a>
        = UnimplementedRect<Self::T>
    where
        Self: 'a;

    type TriangleType<'a>
        = UnimplementedTriangle<Self::T>
    where
        Self: 'a;

    type LineType<'a>
        = UnimplementedLine<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> geo_traits::Dimensions {
        geo_traits::Dimensions::Xy
    }

    fn as_type(
        &self,
    ) -> geo_traits::GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        match self {
            GeoKind::Point(p) => geo_traits::GeometryType::Point(p),
            GeoKind::Path(ls) => geo_traits::GeometryType::LineString(ls),
            GeoKind::Polygon(poly) => geo_traits::GeometryType::Polygon(poly),
            GeoKind::MultiPoint(mp) => geo_traits::GeometryType::MultiPoint(mp),
            GeoKind::MultiLineString(mls) => geo_traits::GeometryType::MultiLineString(mls),
            GeoKind::MultiPolygon(mpoly) => geo_traits::GeometryType::MultiPolygon(mpoly),
            GeoKind::Collection(gc) => geo_traits::GeometryType::GeometryCollection(gc),
        }
    }
}
