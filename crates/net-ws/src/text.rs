use crate::nbt::Nbt;

pub fn text_compound(text: &str) -> Nbt {
    Nbt::compound(vec![("text", Nbt::str(text))])
}

pub fn text_json(text: &str) -> String {
    format!("{{\"text\":\"{}\"}}", json_escape(text))
}

pub fn status_description(text: &str) -> String {
    text_json(text)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quotes() {
        assert_eq!(text_json("a\"b\\c\n"), "{\"text\":\"a\\\"b\\\\c\\n\"}");
    }

    #[test]
    fn compound_has_text_key() {
        let nbt = text_compound("hello");
        match nbt {
            Nbt::Compound(entries) => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "text");
                assert_eq!(entries[0].1, Nbt::String("hello".into()));
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
