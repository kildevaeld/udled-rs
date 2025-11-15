use std::convert::Infallible;

use byteorder::{BigEndian, ByteOrder, LittleEndian, NativeEndian};
use udled::bytes::Endian;

pub trait ToBytes {
    fn write<W: BinaryWriter>(&self, output: &mut W, endian: Endian) -> Result<(), W::Error>;
}

macro_rules! primitives {
    ($($ty: ty => $method: ident),*) => {
        $(
            impl ToBytes for $ty {
                fn write<W: BinaryWriter>(&self, bytes: &mut W, endian: Endian) -> Result<(), W::Error> {
                    match endian {
                        Endian::Big => bytes.$method::<BigEndian>(*self),
                        Endian::Lt => bytes.$method::<LittleEndian>(*self),
                    }
                }
            }
        )*
    };
}

primitives!(
    i16 => write_i16,
    u16 => write_u16,
    i32 => write_i32,
    u32 => write_u32,
    f32 => write_f32,
    f64 => write_f64
);

pub trait BinaryWriter {
    type Error;
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    fn write_u8(&mut self, n: u8) -> Result<(), Self::Error> {
        self.write_all(&[n])
    }

    fn write_i8(&mut self, n: i8) -> Result<(), Self::Error> {
        self.write_all(&[n as _])
    }

    fn write_u16<T: ByteOrder>(&mut self, n: u16) -> Result<(), Self::Error> {
        let mut v = [0; 2];
        T::write_u16(&mut v, n);
        self.write_all(&v)
    }

    fn write_i16<T: ByteOrder>(&mut self, n: i16) -> Result<(), Self::Error> {
        let mut v = [0; 2];
        T::write_i16(&mut v, n);
        self.write_all(&v)
    }

    fn write_u32<T: ByteOrder>(&mut self, n: u32) -> Result<(), Self::Error> {
        let mut v = [0; 4];
        T::write_u32(&mut v, n);
        self.write_all(&v)
    }

    fn write_i32<T: ByteOrder>(&mut self, n: i32) -> Result<(), Self::Error> {
        let mut v = [0; 4];
        T::write_i32(&mut v, n);
        self.write_all(&v)
    }

    fn write_u64<T: ByteOrder>(&mut self, n: u64) -> Result<(), Self::Error> {
        let mut v = [0; 8];
        T::write_u64(&mut v, n);
        self.write_all(&v)
    }

    fn write_i64<T: ByteOrder>(&mut self, n: i64) -> Result<(), Self::Error> {
        let mut v = [0; 8];
        T::write_i64(&mut v, n);
        self.write_all(&v)
    }

    fn write_f64<T: ByteOrder>(&mut self, n: f64) -> Result<(), Self::Error> {
        let mut v = [0; 8];
        T::write_f64(&mut v, n);
        self.write_all(&v)
    }

    fn write_f32<T: ByteOrder>(&mut self, n: f32) -> Result<(), Self::Error> {
        let mut v = [0; 4];
        T::write_f32(&mut v, n);
        self.write_all(&v)
    }
}

impl BinaryWriter for Vec<u8> {
    type Error = Infallible;
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}
