use geom::{wkt::parse, GeomB, Geometry};
use udled::bytes::Endian;

fn main() -> udled::Result<()> {
    let mut out = GeomB::from_text(
        "SRID=2230;MULTILINESTRING((12.2 20.1, 12112.2323 20202.101),(12.2 20.1, 12112.2323 20202.101))",
    )?;

    println!("{:?}", out.kind());

    // out.project(26946);

    // println!("{:?}", out.geometry());

    // out.project(2230);

    println!("{}", out);

    Ok(())
}
