pub const SECTIONS_PER_CHUNK: usize = 24;
pub const SECTION_VOLUME: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub struct Section {
    pub blocks: Option<Box<[u16; SECTION_VOLUME]>>,
}


impl Section {
    pub fn is_empty(&self) -> bool {
        self.blocks.is_none()
    }

    pub fn get(&self, index: usize) -> u16 {
        match &self.blocks {
            None => 0,
            Some(arr) => arr[index],
        }
    }

    pub fn set(&mut self, index: usize, state: u16) {
        if state == 0 && self.blocks.is_none() {
            return;
        }
        let arr = self.blocks.get_or_insert_with(|| Box::new([0; SECTION_VOLUME]));
        arr[index] = state;
        if arr.iter().all(|&b| b == 0) {
            self.blocks = None;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkColumn {
    pub sections: Vec<Section>,
}

impl Default for ChunkColumn {
    fn default() -> Self {
        Self {
            sections: vec![Section::default(); SECTIONS_PER_CHUNK],
        }
    }
}

pub fn section_index(y: i32) -> Option<usize> {
    let rel = y - crate::MIN_Y;
    if (0..crate::WORLD_HEIGHT).contains(&rel) {
        Some((rel / 16) as usize)
    } else {
        None
    }
}

fn local_index(x: i32, y: i32, z: i32) -> Option<usize> {
    if !(0..16).contains(&x) || !(0..16).contains(&z) {
        return None;
    }
    let ly = y - crate::MIN_Y;
    if !(0..16).contains(&(ly % 16)) {
        return None;
    }
    Some((ly % 16) as usize * 256 + (z as usize) * 16 + x as usize)
}

impl ChunkColumn {
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<u16> {
        let si = section_index(y)?;
        Some(self.sections[si].get(local_index(x, y, z)?))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, state: u16) -> bool {
        match section_index(y) {
            Some(si) => {
                self.sections[si].set(local_index(x, y, z).unwrap(), state);
                true
            }
            None => false,
        }
    }

    pub fn non_air_count(&self, section: usize) -> u16 {
        match &self.sections[section].blocks {
            None => 0,
            Some(arr) => arr.iter().filter(|&&b| b != 0).count() as u16,
        }
    }

    pub fn highest_non_air_y(&self) -> Option<i32> {
        for si in (0..SECTIONS_PER_CHUNK).rev() {
            if let Some(arr) = &self.sections[si].blocks {
                for idx in (0..SECTION_VOLUME).rev() {
                    if arr[idx] != 0 {
                        let within = idx / 256;
                        return Some(crate::MIN_Y + (si as i32) * 16 + within as i32);
                    }
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn from_block(bx: i32, bz: i32) -> Self {
        Self { x: bx >> 4, z: bz >> 4 }
    }
}

pub fn serialize_column(col: &ChunkColumn) -> Vec<u8> {
    let mut raw = Vec::with_capacity(SECTIONS_PER_CHUNK * SECTION_VOLUME * 2);
    raw.push(1u8);
    for s in &col.sections {
        match &s.blocks {
            None => raw.extend_from_slice(&[0u8; SECTION_VOLUME * 2]),
            Some(arr) => {
                for b in arr.iter() {
                    raw.extend_from_slice(&b.to_le_bytes());
                }
            }
        }
    }
    zlib_compress(&raw)
}

pub fn deserialize_column(data: &[u8]) -> Option<ChunkColumn> {
    let buf = zlib_decompress(data, SECTIONS_PER_CHUNK * SECTION_VOLUME * 2 + 64)?;
    if buf.first() != Some(&1) || buf.len() < SECTIONS_PER_CHUNK * SECTION_VOLUME * 2 + 1 {
        return None;
    }
    let mut col = ChunkColumn::default();
    for (si, section) in col.sections.iter_mut().enumerate() {
        let start = 1 + si * SECTION_VOLUME * 2;
        let bytes = &buf[start..start + SECTION_VOLUME * 2];
        let mut any = false;
        let mut arr = [0u16; SECTION_VOLUME];
        for (i, chunk) in bytes.as_chunks::<2>().0.iter().enumerate() {
            let v = u16::from_le_bytes([chunk[0], chunk[1]]);
            arr[i] = v;
            if v != 0 {
                any = true;
            }
        }
        section.blocks = if any { Some(Box::new(arr)) } else { None };
    }
    Some(col)
}

fn zlib_compress(data: &[u8]) -> Vec<u8> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::fast());
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

fn zlib_decompress(data: &[u8], max_out: usize) -> Option<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    let mut dec = ZlibDecoder::new(data);
    let mut out = Vec::new();
    dec.by_ref().take(max_out as u64).read_to_end(&mut out).ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks;

    #[test]
    fn flat_column_layout() {
        let gen = crate::superflat::Superflat::classic();
        let col = gen.generate(3, -7);
        assert_eq!(col.get_block(5, -64, 5), Some(blocks::BEDROCK));
        assert_eq!(col.get_block(5, -63, 5), Some(blocks::DIRT));
        assert_eq!(col.get_block(5, -62, 5), Some(blocks::DIRT));
        assert_eq!(col.get_block(5, -61, 5), Some(blocks::GRASS_BLOCK));
        assert_eq!(col.get_block(5, -60, 5), Some(blocks::AIR));
        assert_eq!(col.get_block(15, 100, 15), Some(blocks::AIR));
        assert_eq!(col.non_air_count(0), 1024);
        assert_eq!(col.highest_non_air_y(), Some(-61));
    }

    #[test]
    fn out_of_bounds() {
        let mut col = ChunkColumn::default();
        assert_eq!(col.get_block(0, -65, 0), None);
        assert_eq!(col.get_block(0, 320, 0), None);
        assert!(!col.set_block(0, 9999, 0, blocks::STONE));
    }

    #[test]
    fn serialize_roundtrip() {
        let gen = crate::superflat::Superflat::classic();
        let mut col = gen.generate(-12, 900);
        col.set_block(3, 200, 4, 12345);
        col.set_block(0, -64, 0, blocks::BEDROCK);
        let blob = serialize_column(&col);
        assert!(blob.len() < 2048);
        let back = deserialize_column(&blob).unwrap();
        assert_eq!(back.get_block(3, 200, 4), Some(12345));
        assert_eq!(back.get_block(5, -61, 5), Some(blocks::GRASS_BLOCK));
        assert_eq!(back.sections[1].is_empty(), true);
    }

    #[test]
    fn section_auto_clears() {
        let mut col = ChunkColumn::default();
        col.set_block(1, 2, 3, 42);
        assert!(col.sections[section_index(2).unwrap()].blocks.is_some());
        col.set_block(1, 2, 3, 0);
        assert!(col.sections[section_index(2).unwrap()].blocks.is_none());
    }
}
