use net_ws::packets::{play_chunk_data, HeightmapPayload, LightPayload};
use world::chunk::{ChunkColumn, SECTIONS_PER_CHUNK, SECTION_VOLUME};

use crate::vanilla_registries::BIOME_PLAINS;

fn ceil_log2(n: usize) -> u8 {
    let mut bits = 0u8;
    let mut v = n.saturating_sub(1).max(1);
    while v > 1 {
        v >>= 1;
        bits += 1;
    }
    bits + 1
}

pub fn encode_section_blocks(out: &mut Vec<u8>, col: &ChunkColumn, si: usize) {
    use net_ws::Encoder;
    let count = col.non_air_count(si);
    out.extend_from_slice(&count.to_be_bytes());
    // Fluid accounting was added to chunk sections in 26.1. The current
    // The current generators emit no fluid states yet.
    out.extend_from_slice(&0i16.to_be_bytes());

    let mut e = Encoder::new();
    let Some(blocks) = col.sections[si].blocks.as_ref() else {
        e.u8(0);
        e.varint(0); // minecraft:air
        e.u8(0);
        e.varint(BIOME_PLAINS);
        out.extend_from_slice(&e.into_bytes());
        return;
    };
    let mut palette: Vec<u16> = Vec::new();
    let mut indices = Vec::with_capacity(SECTION_VOLUME);
    for &state in blocks.iter() {
        let idx = match palette.iter().position(|&p| p == state) {
            Some(i) => i,
            None => {
                palette.push(state);
                palette.len() - 1
            }
        };
        indices.push(idx as u32);
    }

    if palette.len() == 1 {
        e.u8(0);
        e.varint(palette[0] as i32);
    } else {
        let bits = ceil_log2(palette.len()).max(4);
        if bits <= 8 {
            e.u8(bits);
            e.varint(palette.len() as i32);
            for &p in &palette {
                e.varint(p as i32);
            }
            write_packed_longs(&mut e, &indices, bits as u32);
        } else {
            const DIRECT_BITS: u32 = 15;
            e.u8(DIRECT_BITS as u8);
            let direct: Vec<u32> = blocks.iter().map(|&s| s as u32).collect();
            write_packed_longs(&mut e, &direct, DIRECT_BITS);
        }
    }

    e.u8(0);
    e.varint(BIOME_PLAINS);

    out.extend_from_slice(&e.into_bytes());
}

fn write_packed_longs(e: &mut net_ws::Encoder, values: &[u32], bits: u32) {
    use std::io::Write;
    let per_long = (64 / bits) as usize;
    let long_count = values.len().div_ceil(per_long);
    let mut value_iter = values.iter().copied();
    for _ in 0..long_count {
        let mut acc: u64 = 0;
        for slot in 0..per_long {
            match value_iter.next() {
                Some(v) => {
                    acc |= ((v as u64) & mask(bits)) << (u32::try_from(slot).unwrap_or(0) * bits)
                }
                None => break,
            }
        }
        let _ = e.buf.write_all(&acc.to_be_bytes());
    }
}

fn mask(bits: u32) -> u64 {
    if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

fn column_heights(col: &ChunkColumn) -> Vec<i64> {
    let mut heights = vec![0i64; 256];
    for si in (0..SECTIONS_PER_CHUNK).rev() {
        if let Some(blocks) = &col.sections[si].blocks {
            for idx in 0..SECTION_VOLUME {
                if blocks[idx] != 0 {
                    let y_local = (idx / 256) as i32;
                    let z = (idx % 256) / 16;
                    let x = idx % 16;
                    let abs_y = world::MIN_Y + si as i32 * 16 + y_local;
                    let entry = &mut heights[z * 16 + x];
                    *entry = (*entry).max((abs_y - world::MIN_Y + 1) as i64);
                }
            }
        }
    }
    heights
}

fn pack_heightmap(heights: &[i64]) -> Vec<i64> {
    const BITS: u32 = 9;
    let per_long = (64 / BITS) as usize;
    let long_count = heights.len().div_ceil(per_long);
    let mut packed = Vec::with_capacity(long_count);
    for li in 0..long_count {
        let mut acc: u64 = 0;
        for slot in 0..per_long {
            let vi = li * per_long + slot;
            if vi < heights.len() {
                acc |= ((heights[vi] as u64) & mask(BITS)) << (slot as u32 * BITS);
            }
        }
        packed.push(acc as i64);
    }
    packed
}

pub fn heightmaps(col: &ChunkColumn) -> Vec<HeightmapPayload> {
    let h = column_heights(col);
    let packed = pack_heightmap(&h);
    vec![
        HeightmapPayload {
            id: 1,
            data: packed.clone(),
        }, // WORLD_SURFACE
        HeightmapPayload {
            id: 4,
            data: packed.clone(),
        }, // MOTION_BLOCKING
        HeightmapPayload {
            id: 5,
            data: packed,
        }, // MOTION_BLOCKING_NO_LEAVES
    ]
}

pub fn full_sky_light() -> LightPayload {
    // Light bit 0 is below the world, bits 1..=24 are sections, and bit 25 is
    // above the world.
    let section_mask: i64 = ((1i64 << SECTIONS_PER_CHUNK) - 1) << 1;
    let boundary_mask: i64 = 1 | (1i64 << (SECTIONS_PER_CHUNK + 1));
    let section = vec![0xFFu8; 2048];
    LightPayload {
        sky_light_mask: vec![section_mask],
        block_light_mask: vec![0],
        empty_sky_light_mask: vec![boundary_mask],
        empty_block_light_mask: vec![section_mask | boundary_mask],
        sky_light: vec![section.clone(); SECTIONS_PER_CHUNK],
        block_light: Vec::new(),
    }
}

pub fn encode_chunk_packet(x: i32, z: i32, col: &ChunkColumn) -> Vec<u8> {
    let mut data = Vec::new();
    for si in 0..SECTIONS_PER_CHUNK {
        encode_section_blocks(&mut data, col, si);
    }
    let maps = heightmaps(col);
    let light = full_sky_light();
    play_chunk_data(x, z, &maps, &data, &light)
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::superflat::Superflat;

    #[test]
    fn flat_chunk_section_layout() {
        let gen = Superflat::classic();
        let col = gen.generate(2, 3);
        let mut data = Vec::new();
        encode_section_blocks(&mut data, &col, 0);

        assert_eq!(&data[..2], &1024u16.to_be_bytes());
        assert_eq!(&data[2..4], &0i16.to_be_bytes());
        let mut p = 4;
        assert_eq!(data[p], 4);
        p += 1;
        assert_eq!(data[p], 4);
        p += 1;
        for expected in [85u32, 10, 9, 0] {
            assert_eq!(data[p], expected as u8);
            p += 1;
        }
        p += 256 * 8;

        assert_eq!(data[p], 0);
        p += 1;
        assert_eq!(data[p], BIOME_PLAINS as u8);
        p += 1;
        assert_eq!(p, data.len());

        let before = data.len();
        encode_section_blocks(&mut data, &col, 1);
        assert_eq!(data.len() - before, 8);
    }

    #[test]
    fn packed_longs_no_spanning() {
        let values: Vec<u32> = (0..4096).map(|i| i as u32 & 0xF).collect();
        let mut encoder = net_ws::Encoder::new();
        write_packed_longs(&mut encoder, &values, 4);
        let out = encoder.into_bytes();
        assert_eq!(out.len(), (4096 / 16) * 8);
    }

    #[test]
    fn heightmaps_shape() {
        let gen = Superflat::classic();
        let col = gen.generate(0, 0);
        let maps = heightmaps(&col);
        assert_eq!(maps.len(), 3);
        assert_eq!([maps[0].id, maps[1].id, maps[2].id], [1, 4, 5]);
        assert_eq!(maps[0].data.len(), 37);
        assert_eq!(maps[0].data[0] & 0x1FF, 4);
    }

    #[test]
    fn chunk_packet_builds() {
        let gen = Superflat::classic();
        let col = gen.generate(5, -5);
        let pkt = encode_chunk_packet(5, -5, &col);
        assert_eq!(pkt[0], net_ws::ids::cb::PLAY_CHUNK_DATA as u8);
        assert!(pkt.len() < 60_000);
    }
}
