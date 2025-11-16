use geom::{text::parse, GeomB, Geometry};
use udled::bytes::Endian;

fn main() -> udled::Result<()> {
    let out = GeomB::from_text(
        "SRID=2022;POLYGON((12.2 20.1, 12112.2323 20202.101),(12.2 20.1, 12112.2323 20202.101))",
    )?;

    println!("{:?}", out.geometry());

    Ok(())
}
