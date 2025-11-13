use byteorder::{ByteOrder, NativeEndian};
use udled::{bytes::*, Input};

fn main() -> udled::Result<()> {
    let mut bytes = [0; 12];

    NativeEndian::write_i32(&mut bytes[0..4], 42);
    NativeEndian::write_f64(&mut bytes[4..], 84.2);

    let mut input = Input::new(&bytes[..]);

    let ans = input.parse(i32::native())?;
    let w = input.parse(f64::native())?;

    println!("{:?},{:?}", ans, w);
    Ok(())
}
