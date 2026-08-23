use crate::error::{DecodeError, DecodeResult};
use crate::varint::{read_varint, write_varint};

#[derive(Default)]
pub struct Encoder {
    pub buf: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    pub fn bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn varint(&mut self, v: i32) {
        write_varint(&mut self.buf, v);
    }

    pub fn varlong(&mut self, v: i64) {
        let mut value = v as u64;
        loop {
            if value & !0x7f == 0 {
                self.u8(value as u8);
                return;
            }
            self.u8((value as u8 & 0x7f) | 0x80);
            value >>= 7;
        }
    }

    pub fn string(&mut self, s: &str) {
        self.varint(s.len() as i32);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn uuid(&mut self, uuid: &[u8; 16]) {
        self.buf.extend_from_slice(uuid);
    }

    pub fn raw(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn byte_array(&mut self, bytes: &[u8]) {
        self.varint(bytes.len() as i32);
        self.raw(bytes);
    }
}

pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn take(&mut self, n: usize) -> DecodeResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(DecodeError::Eof);
        }
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    pub fn u8(&mut self) -> DecodeResult<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn i8(&mut self) -> DecodeResult<i8> {
        Ok(self.u8()? as i8)
    }

    pub fn bool(&mut self) -> DecodeResult<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(DecodeError::InvalidBool(other)),
        }
    }

    pub fn u16(&mut self) -> DecodeResult<u16> {
        let b = self.take(2)?;
        Ok(u16::from_be_bytes([b[0], b[1]]))
    }

    pub fn i16(&mut self) -> DecodeResult<i16> {
        Ok(self.u16()? as i16)
    }

    pub fn u32(&mut self) -> DecodeResult<u32> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn i32(&mut self) -> DecodeResult<i32> {
        Ok(self.u32()? as i32)
    }

    pub fn u64(&mut self) -> DecodeResult<u64> {
        let b = self.take(8)?;
        Ok(u64::from_be_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn i64(&mut self) -> DecodeResult<i64> {
        Ok(self.u64()? as i64)
    }

    pub fn f32(&mut self) -> DecodeResult<f32> {
        Ok(f32::from_bits(self.u32()?))
    }

    pub fn f64(&mut self) -> DecodeResult<f64> {
        Ok(f64::from_bits(self.u64()?))
    }

    pub fn varint(&mut self) -> DecodeResult<i32> {
        read_varint(self.data, &mut self.pos)
    }

    pub fn varlong(&mut self) -> DecodeResult<i64> {
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.u8()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value as i64);
            }
        }
        Err(DecodeError::Malformed("VarLong is too long"))
    }

    pub fn string(&mut self) -> DecodeResult<String> {
        let len = self.varint()?;
        if !(0..=262144).contains(&len) {
            return Err(DecodeError::Malformed("string length out of range"));
        }
        let bytes = self.take(len as usize)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    pub fn uuid(&mut self) -> DecodeResult<[u8; 16]> {
        let b = self.take(16)?;
        let mut out = [0u8; 16];
        out.copy_from_slice(b);
        Ok(out)
    }

    pub fn byte_array(&mut self) -> DecodeResult<Vec<u8>> {
        let len = self.varint()?;
        if len < 0 {
            return Err(DecodeError::Malformed("negative array length"));
        }
        Ok(self.take(len as usize)?.to_vec())
    }
}

pub fn pack_block_pos(x: i32, y: i32, z: i32) -> i64 {
    (((x as i64) & 0x3FF_FFFF) << 38) | (((z as i64) & 0x3FF_FFFF) << 12) | ((y as i64) & 0xFFF)
}

pub fn unpack_block_pos(packed: i64) -> (i32, i32, i32) {
    let x = (packed >> 38) as i32;
    let y = (packed << 52 >> 52) as i32;
    let z = (packed << 26 >> 38) as i32;
    (x, y, z)
}

pub fn pack_chunk_pos(x: i32, z: i32) -> i64 {
    ((x as i64 & 0xFFFF_FFFF) << 32) | (z as i64 & 0xFFFF_FFFF)
}

pub fn angle_byte(degrees: f32) -> i8 {
    ((degrees / 360.0) * 256.0).rem_euclid(256.0).round() as i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_roundtrip() {
        let mut e = Encoder::new();
        e.string("hello world");
        let bytes = e.into_bytes();
        assert_eq!(bytes[0], 11);
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.string().unwrap(), "hello world");
        assert!(d.is_empty());
    }

    #[test]
    fn mixed_roundtrip() {
        let mut e = Encoder::new();
        e.i32(-123456);
        e.f64(1.5);
        e.bool(true);
        e.uuid(&[7u8; 16]);
        let bytes = e.into_bytes();
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.i32().unwrap(), -123456);
        assert_eq!(d.f64().unwrap(), 1.5);
        assert!(d.bool().unwrap());
        assert_eq!(d.uuid().unwrap(), [7u8; 16]);
    }

    #[test]
    fn block_pos_roundtrip() {
        for (x, y, z) in [
            (0, 0, 0),
            (-1, -65, 1),
            (29_999_999, 319, -29_999_999),
            (100, -60, -200),
        ] {
            let packed = pack_block_pos(x, y, z);
            assert_eq!(unpack_block_pos(packed), (x, y, z));
        }
    }

    #[test]
    fn eof_on_truncated() {
        let mut d = Decoder::new(&[0x80]);
        assert_eq!(d.i32(), Err(DecodeError::Eof));
    }
}
