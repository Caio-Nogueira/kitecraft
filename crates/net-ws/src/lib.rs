pub mod buf;
pub mod error;
pub mod frame;
pub mod ids;
pub mod nbt;
pub mod packets;
pub mod text;
pub mod varint;

pub use buf::{Decoder, Encoder};
pub use error::{DecodeError, DecodeResult};
pub use frame::{decode_frame, encode_frame};
pub use varint::{read_varint, write_varint};

pub const PROTOCOL_VERSION: i32 = 776;
pub const MINECRAFT_VERSION: &str = "26.2";
