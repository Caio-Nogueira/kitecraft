use crate::error::{DecodeError, DecodeResult};
use crate::varint::{read_varint, write_varint};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{Read, Write};

pub const MAX_PACKET_SIZE: usize = 0x20_0000;
pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 256;

pub fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::with_capacity(data.len() / 4 + 16), Compression::fast());
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

pub fn zlib_decompress(data: &[u8], max_out: usize) -> DecodeResult<Vec<u8>> {
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.by_ref()
        .take((max_out as u64) + 1)
        .read_to_end(&mut out)
        .map_err(|_| DecodeError::BadFrame("zlib decompression failed"))?;
    if out.len() > max_out {
        return Err(DecodeError::BadFrame("decompressed packet too large"));
    }
    Ok(out)
}

fn encode_inner(payload: &[u8], threshold: Option<usize>) -> Vec<u8> {
    match threshold {
        None => payload.to_vec(),
        Some(t) => {
            let mut inner = Vec::with_capacity(payload.len() + 5);
            if payload.len() < t {
                write_varint(&mut inner, 0);
                inner.extend_from_slice(payload);
            } else {
                write_varint(&mut inner, payload.len() as i32);
                inner.extend_from_slice(&zlib_compress(payload));
            }
            inner
        }
    }
}

fn decode_inner(inner: &[u8], threshold: Option<usize>) -> DecodeResult<Vec<u8>> {
    if threshold.is_none() {
        return Ok(inner.to_vec());
    }
    let mut pos = 0;
    let data_len = read_varint(inner, &mut pos)?;
    if data_len == 0 {
        Ok(inner[pos..].to_vec())
    } else {
        if data_len as usize > MAX_PACKET_SIZE {
            return Err(DecodeError::BadFrame("declared size too large"));
        }
        zlib_decompress(&inner[pos..], data_len as usize)
    }
}

pub fn encode_frame(payload: &[u8], threshold: Option<usize>) -> Vec<u8> {
    let inner = encode_inner(payload, threshold);
    let mut msg = Vec::with_capacity(inner.len() + 5);
    write_varint(&mut msg, inner.len() as i32);
    msg.extend_from_slice(&inner);
    msg
}

pub fn decode_frame(msg: &[u8], threshold: Option<usize>) -> DecodeResult<Vec<u8>> {
    let mut pos = 0;
    let declared = read_varint(msg, &mut pos)? as usize;
    if declared != msg.len() - pos {
        return Err(DecodeError::BadFrame(
            "length prefix does not match message",
        ));
    }
    decode_inner(&msg[pos..], threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncompressed_roundtrip() {
        let payload = [1u8, 2, 3, 4, 5];
        let frame = encode_frame(&payload, None);
        assert_eq!(frame[0], payload.len() as u8);
        assert_eq!(decode_frame(&frame, None).unwrap(), payload);
    }

    #[test]
    fn compressed_roundtrip_small_below_threshold() {
        let payload = vec![42u8; 100];
        let frame = encode_frame(&payload, Some(256));
        let decoded = decode_frame(&frame, Some(256)).unwrap();
        assert_eq!(decoded, payload);
        assert_eq!(frame[1], 0x00);
        assert_eq!(frame[2], 42);
    }

    #[test]
    fn compressed_roundtrip_large_above_threshold() {
        let payload = vec![7u8; 4096];
        let frame = encode_frame(&payload, Some(256));
        assert!(frame.len() < 128);
        let decoded = decode_frame(&frame, Some(256)).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn rejects_bad_prefix() {
        let mut frame = encode_frame(&[1, 2, 3], None);
        frame[0] = 99;
        assert!(decode_frame(&frame, None).is_err());
    }

    #[test]
    fn rejects_bomb() {
        let zeros = vec![0u8; 100_000];
        let compressed = zlib_compress(&zeros);
        let mut inner = Vec::new();
        write_varint(&mut inner, 10_000_000);
        inner.extend_from_slice(&compressed);
        let mut msg = Vec::new();
        write_varint(&mut msg, inner.len() as i32);
        msg.extend_from_slice(&inner);
        assert!(decode_frame(&msg, Some(256)).is_err());
    }
}
