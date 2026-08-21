use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Eof,
    VarIntTooLong,
    InvalidUtf8,
    InvalidBool(u8),
    BadFrame(&'static str),
    UnknownPacket { state: &'static str, id: i32 },
    Malformed(&'static str),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Eof => write!(f, "unexpected end of data"),
            DecodeError::VarIntTooLong => write!(f, "varint too long"),
            DecodeError::InvalidUtf8 => write!(f, "invalid utf-8 string"),
            DecodeError::InvalidBool(b) => write!(f, "invalid bool byte {b}"),
            DecodeError::BadFrame(m) => write!(f, "bad frame: {m}"),
            DecodeError::UnknownPacket { state, id } => {
                write!(f, "unknown packet in {state} state: 0x{id:02x}")
            }
            DecodeError::Malformed(m) => write!(f, "malformed packet: {m}"),
        }
    }
}

impl std::error::Error for DecodeError {}

pub type DecodeResult<T> = Result<T, DecodeError>;
