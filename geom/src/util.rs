use byteorder::{BigEndian, ByteOrder, LittleEndian};
use udled::bytes::Endian;

pub fn read_f64(buf: &[u8], endian: Endian) -> f64 {
    match endian {
        Endian::Big => BigEndian::read_f64(buf),
        Endian::Lt => LittleEndian::read_f64(buf),
    }
}

pub fn write_f64(buf: &mut [u8], n: f64, endian: Endian) {
    match endian {
        Endian::Big => BigEndian::write_f64(buf, n),
        Endian::Lt => LittleEndian::write_f64(buf, n),
    }
}

pub fn read_u32(buf: &[u8], endian: Endian) -> u32 {
    match endian {
        Endian::Big => BigEndian::read_u32(buf),
        Endian::Lt => LittleEndian::read_u32(buf),
    }
}

pub fn write_u32(buf: &mut [u8], n: u32, endian: Endian) {
    match endian {
        Endian::Big => BigEndian::write_u32(buf, n),
        Endian::Lt => LittleEndian::write_u32(buf, n),
    }
}

pub fn get_endian(i: u8) -> Option<Endian> {
    match i {
        0 => Some(Endian::Big),
        1 => Some(Endian::Lt),
        _ => return None,
    }
}
