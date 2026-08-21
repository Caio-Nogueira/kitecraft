use crate::DecodeError;

pub fn write_varint(buf: &mut Vec<u8>, value: i32) {
    let mut v = value as u32;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            buf.push(byte);
            return;
        }
        buf.push(byte | 0x80);
    }
}

pub fn varint_len(value: i32) -> usize {
    let mut len = 1;
    let mut v = (value as u32) >> 7;
    while v != 0 {
        len += 1;
        v >>= 7;
    }
    len
}

pub fn read_varint(data: &[u8], pos: &mut usize) -> Result<i32, DecodeError> {
    let mut result: u32 = 0;
    for shift in [0, 7, 14, 21, 28] {
        let byte = *data.get(*pos).ok_or(DecodeError::Eof)?;
        *pos += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if byte & 0x80 == 0 {
            return Ok(result as i32);
        }
    }
    Err(DecodeError::VarIntTooLong)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(v: i32) {
        let mut buf = Vec::new();
        write_varint(&mut buf, v);
        assert_eq!(varint_len(v), buf.len());
        let mut pos = 0;
        assert_eq!(read_varint(&buf, &mut pos).unwrap(), v);
        assert_eq!(pos, buf.len());
    }

    #[test]
    fn known_encodings() {
        let mut b = Vec::new();
        write_varint(&mut b, 0);
        assert_eq!(b, vec![0x00]);
        b.clear();
        write_varint(&mut b, 1);
        assert_eq!(b, vec![0x01]);
        b.clear();
        write_varint(&mut b, 127);
        assert_eq!(b, vec![0x7F]);
        b.clear();
        write_varint(&mut b, 128);
        assert_eq!(b, vec![0x80, 0x01]);
        b.clear();
        write_varint(&mut b, 255);
        assert_eq!(b, vec![0xFF, 0x01]);
        b.clear();
        write_varint(&mut b, 2097151);
        assert_eq!(b, vec![0xFF, 0xFF, 0x7F]);
        b.clear();
        write_varint(&mut b, 2147483647);
        assert_eq!(b, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]);
        b.clear();
        write_varint(&mut b, -1);
        assert_eq!(b, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn roundtrips() {
        for v in [0, 1, -1, 127, 128, -128, 255, 256, 65535, 2097152, i32::MAX, i32::MIN] {
            roundtrip(v);
        }
    }

    #[test]
    fn rejects_overlong() {
        let data = [0xFF; 6];
        let mut pos = 0;
        assert_eq!(read_varint(&data, &mut pos), Err(DecodeError::VarIntTooLong));
    }

    #[test]
    fn rejects_eof() {
        let data = [0x80];
        let mut pos = 0;
        assert_eq!(read_varint(&data, &mut pos), Err(DecodeError::Eof));
    }
}
