#[derive(Debug, Clone, PartialEq)]
pub enum Nbt {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Nbt>),
    Compound(Vec<(String, Nbt)>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Nbt {
    pub fn compound(entries: Vec<(&str, Nbt)>) -> Self {
        Nbt::Compound(
            entries
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn str(s: impl Into<String>) -> Self {
        Nbt::String(s.into())
    }

    fn tag_id(&self) -> u8 {
        match self {
            Nbt::Byte(_) => 1,
            Nbt::Short(_) => 2,
            Nbt::Int(_) => 3,
            Nbt::Long(_) => 4,
            Nbt::Float(_) => 5,
            Nbt::Double(_) => 6,
            Nbt::ByteArray(_) => 7,
            Nbt::String(_) => 8,
            Nbt::List(_) => 9,
            Nbt::Compound(_) => 10,
            Nbt::IntArray(_) => 11,
            Nbt::LongArray(_) => 12,
        }
    }
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn write_payload(out: &mut Vec<u8>, value: &Nbt) {
    match value {
        Nbt::Byte(v) => out.push(*v as u8),
        Nbt::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Nbt::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Nbt::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Nbt::ByteArray(items) => {
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            out.extend_from_slice(items);
        }
        Nbt::String(s) => write_string(out, s),
        Nbt::List(items) => {
            let elem_tag = items.first().map(Nbt::tag_id).unwrap_or(0);
            out.push(elem_tag);
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for item in items {
                write_payload(out, item);
            }
        }
        Nbt::Compound(entries) => {
            for (name, value) in entries {
                out.push(value.tag_id());
                write_string(out, name);
                write_payload(out, value);
            }
            out.push(0);
        }
        Nbt::IntArray(items) => {
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for v in items {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
        Nbt::LongArray(items) => {
            out.extend_from_slice(&(items.len() as i32).to_be_bytes());
            for v in items {
                out.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
}

pub fn write_named(out: &mut Vec<u8>, name: &str, value: &Nbt) {
    out.push(value.tag_id());
    write_string(out, name);
    write_payload(out, value);
}

pub fn write_root_unnamed(out: &mut Vec<u8>, value: &Nbt) {
    out.push(value.tag_id());
    write_payload(out, value);
}

pub fn encode_root_unnamed(value: &Nbt) -> Vec<u8> {
    let mut out = Vec::new();
    write_root_unnamed(&mut out, value);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_compound_layout() {
        let value = Nbt::compound(vec![("text", Nbt::str("hi"))]);
        let bytes = encode_root_unnamed(&value);
        assert_eq!(
            bytes,
            vec![0x0A, 0x08, 0x00, 0x04, b't', b'e', b'x', b't', 0x00, 0x02, b'h', b'i', 0x00]
        );
    }

    #[test]
    fn list_of_ints() {
        let value = Nbt::List(vec![Nbt::Int(1), Nbt::Int(2)]);
        let mut out = Vec::new();
        write_named(&mut out, "l", &value);
        assert_eq!(out[0], 9);
        assert_eq!(out[4], 3);
        assert_eq!(
            &out[out.len() - 12..],
            &[0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2][..]
        );
    }

    #[test]
    fn empty_list_is_end_typed() {
        let value = Nbt::List(vec![]);
        let mut out = Vec::new();
        write_named(&mut out, "e", &value);
        assert_eq!(out, vec![9, 0, 1, b'e', 0, 0, 0, 0, 0]);
    }

    #[test]
    fn long_array() {
        let value = Nbt::LongArray(vec![1, -1]);
        let mut out = Vec::new();
        write_named(&mut out, "h", &value);
        assert_eq!(out[0], 12);
        assert_eq!(
            out[out.len() - 16..],
            {
                let mut e = Vec::new();
                e.extend_from_slice(&1i64.to_be_bytes());
                e.extend_from_slice(&(-1i64).to_be_bytes());
                e
            }[..]
        );
    }

    #[test]
    fn nested() {
        let value = Nbt::compound(vec![(
            "effects",
            Nbt::compound(vec![("sky_color", Nbt::Int(7907327))]),
        )]);
        let bytes = encode_root_unnamed(&value);
        assert_eq!(bytes[0], 10);
        assert_eq!(7907327i32.to_be_bytes(), [0x00, 0x78, 0xA7, 0xFF]);
        assert!(bytes.windows(4).any(|w| w == [0x00, 0x78, 0xA7, 0xFF]));
    }
}
