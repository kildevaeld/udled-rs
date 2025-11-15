use byteorder::{BigEndian, ByteOrder, LittleEndian};
use udled::bytes::Endian;

pub fn read_f64(buf: &[u8], endian: Endian) -> f64 {
    match endian {
        Endian::Big => BigEndian::read_f64(buf),
        Endian::Lt => LittleEndian::read_f64(buf),
    }
}

pub fn read_u32(buf: &[u8], endian: Endian) -> u32 {
    match endian {
        Endian::Big => BigEndian::read_u32(buf),
        Endian::Lt => LittleEndian::read_u32(buf),
    }
}
