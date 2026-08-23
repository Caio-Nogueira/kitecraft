//! Compact, WASM-safe Minecraft 26.2 block metadata imported from Pumpkin.

pub const PUMPKIN_REVISION: &str = "beb6947dfc21a1a781523bf207a3c2740f4928f9";
pub const PUMPKIN_VERSION: &str = "0.1.0-dev+26.2-26.40";
pub const MINECRAFT_VERSION: &str = "26.2";

pub const AIR: u16 = 0;
pub const STONE: u16 = 1;
pub const GRASS_BLOCK: u16 = 9;
pub const DIRT: u16 = 10;
pub const BEDROCK: u16 = 85;

const INVALID_ID: u16 = u16::MAX;
const STATE_STRIDE: usize = 16;
const BLOCK_STRIDE: usize = 44;
const SHAPE_STRIDE: usize = 48;

const IS_AIR: u16 = 1 << 0;
const BURNABLE: u16 = 1 << 1;
const TOOL_REQUIRED: u16 = 1 << 2;
const SIDED_TRANSPARENCY: u16 = 1 << 3;
const REPLACEABLE: u16 = 1 << 4;
const IS_LIQUID: u16 = 1 << 5;
const IS_SOLID: u16 = 1 << 6;
const IS_FULL_CUBE: u16 = 1 << 7;
const IS_SOLID_BLOCK: u16 = 1 << 8;
const HAS_RANDOM_TICKS: u16 = 1 << 9;

static STATES: &[u8] = include_bytes!("../data/states.bin");
static BLOCKS: &[u8] = include_bytes!("../data/blocks.bin");
static BLOCK_NAMES: &[u8] = include_bytes!("../data/block_names.bin");
static COLLISION_REFS: &[u8] = include_bytes!("../data/collision_refs.bin");
static COLLISION_SHAPES: &[u8] = include_bytes!("../data/collision_shapes.bin");
static ITEM_DEFAULT_STATES: &[u8] = include_bytes!("../data/item_default_states.bin");

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockStateMetadata {
    pub id: u16,
    pub block_id: u16,
    pub hardness: f32,
    pub luminance: u8,
    pub opacity: u8,
    pub side_flags: u8,
    flags: u16,
    collision_start: u32,
    collision_count: u8,
}

impl BlockStateMetadata {
    #[must_use]
    pub const fn is_air(self) -> bool {
        self.flags & IS_AIR != 0
    }

    #[must_use]
    pub const fn burnable(self) -> bool {
        self.flags & BURNABLE != 0
    }

    #[must_use]
    pub const fn tool_required(self) -> bool {
        self.flags & TOOL_REQUIRED != 0
    }

    #[must_use]
    pub const fn sided_transparency(self) -> bool {
        self.flags & SIDED_TRANSPARENCY != 0
    }

    #[must_use]
    pub const fn replaceable(self) -> bool {
        self.flags & REPLACEABLE != 0
    }

    #[must_use]
    pub const fn is_liquid(self) -> bool {
        self.flags & IS_LIQUID != 0
    }

    #[must_use]
    pub const fn is_solid(self) -> bool {
        self.flags & IS_SOLID != 0
    }

    #[must_use]
    pub const fn is_full_cube(self) -> bool {
        self.flags & IS_FULL_CUBE != 0
    }

    #[must_use]
    pub const fn is_solid_block(self) -> bool {
        self.flags & IS_SOLID_BLOCK != 0
    }

    #[must_use]
    pub const fn has_random_ticks(self) -> bool {
        self.flags & HAS_RANDOM_TICKS != 0
    }

    #[must_use]
    pub const fn collision_shape_count(self) -> usize {
        self.collision_count as usize
    }

    #[must_use]
    pub const fn collision_shapes(self) -> CollisionShapeIter {
        CollisionShapeIter {
            cursor: self.collision_start as usize,
            end: self.collision_start as usize + self.collision_count as usize,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockMetadata {
    pub id: u16,
    pub default_state_id: u16,
    pub item_id: Option<u16>,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    name_start: u32,
    name_len: u8,
    shape_offset_type: u8,
    shape_offset_horizontal: f32,
    shape_offset_vertical: f32,
}

impl BlockMetadata {
    #[must_use]
    pub fn name(self) -> &'static str {
        let start = self.name_start as usize;
        let end = start + self.name_len as usize;
        std::str::from_utf8(&BLOCK_NAMES[start..end]).expect("generated block names are UTF-8")
    }

    /// Returns Pumpkin's vanilla position-dependent collision-shape offset.
    #[must_use]
    pub fn collision_offset(self, x: i32, z: i32) -> [f64; 3] {
        if self.shape_offset_type == 0 {
            return [0.0; 3];
        }
        let seed = hash_block_pos(x, 0, z) as u64;
        let horizontal = f64::from(self.shape_offset_horizontal);
        let x = (f64::from((seed & 15) as u8) / 15.0 - 0.5) * 0.5;
        let x = x.clamp(-horizontal, horizontal) + horizontal;
        let z = (f64::from(((seed >> 8) & 15) as u8) / 15.0 - 0.5) * 0.5;
        let z = z.clamp(-horizontal, horizontal) + horizontal;
        let y = if self.shape_offset_type == 2 {
            let vertical = f64::from(self.shape_offset_vertical);
            (f64::from(((seed >> 4) & 15) as u8) / 15.0 - 1.0) * vertical + vertical
        } else {
            0.0
        };
        [x, y, z]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

pub struct CollisionShapeIter {
    cursor: usize,
    end: usize,
}

impl Iterator for CollisionShapeIter {
    type Item = CollisionBox;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            return None;
        }
        let shape_id = read_u16(COLLISION_REFS, self.cursor * 2) as usize;
        self.cursor += 1;
        collision_shape(shape_id)
    }
}

#[must_use]
pub fn state(id: u16) -> Option<BlockStateMetadata> {
    let offset = id as usize * STATE_STRIDE;
    let record = STATES.get(offset..offset + STATE_STRIDE)?;
    Some(BlockStateMetadata {
        id,
        block_id: read_u16(record, 0),
        flags: read_u16(record, 2),
        side_flags: record[4],
        luminance: record[5],
        opacity: record[6],
        collision_count: record[7],
        collision_start: read_u32(record, 8),
        hardness: read_f32(record, 12),
    })
}

#[must_use]
pub fn block(id: u16) -> Option<BlockMetadata> {
    let offset = id as usize * BLOCK_STRIDE;
    let record = BLOCKS.get(offset..offset + BLOCK_STRIDE)?;
    let item_id = read_u16(record, 2);
    Some(BlockMetadata {
        id,
        default_state_id: read_u16(record, 0),
        item_id: (item_id != 0).then_some(item_id),
        name_start: read_u32(record, 4),
        name_len: record[8],
        hardness: read_f32(record, 12),
        blast_resistance: read_f32(record, 16),
        slipperiness: read_f32(record, 20),
        velocity_multiplier: read_f32(record, 24),
        jump_velocity_multiplier: read_f32(record, 28),
        shape_offset_type: record[32],
        shape_offset_horizontal: read_f32(record, 36),
        shape_offset_vertical: read_f32(record, 40),
    })
}

#[must_use]
pub fn block_by_name(name: &str) -> Option<BlockMetadata> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    (0..block_count())
        .filter_map(|id| block(id as u16))
        .find(|block| block.name() == name)
}

#[must_use]
pub fn default_state_for_item(item_id: u16) -> Option<u16> {
    let offset = item_id as usize * 2;
    let state = read_u16(ITEM_DEFAULT_STATES.get(offset..offset + 2)?, 0);
    (state != INVALID_ID).then_some(state)
}

#[must_use]
pub const fn state_count() -> usize {
    STATES.len() / STATE_STRIDE
}

#[must_use]
pub const fn block_count() -> usize {
    BLOCKS.len() / BLOCK_STRIDE
}

fn collision_shape(id: usize) -> Option<CollisionBox> {
    let offset = id * SHAPE_STRIDE;
    let record = COLLISION_SHAPES.get(offset..offset + SHAPE_STRIDE)?;
    Some(CollisionBox {
        min: [
            read_f64(record, 0),
            read_f64(record, 8),
            read_f64(record, 16),
        ],
        max: [
            read_f64(record, 24),
            read_f64(record, 32),
            read_f64(record, 40),
        ],
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("generated u16"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("generated u32"))
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("generated f32"))
}

fn read_f64(bytes: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("generated f64"))
}

const fn hash_block_pos(x: i32, y: i32, z: i32) -> i64 {
    let value =
        ((x.wrapping_mul(3_129_871)) as i64) ^ ((z as i64).wrapping_mul(116_129_781)) ^ (y as i64);
    value
        .wrapping_mul(value)
        .wrapping_mul(42_317_861)
        .wrapping_add(value.wrapping_mul(11))
        >> 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_dataset_shape_matches_pumpkin() {
        assert_eq!(PUMPKIN_REVISION.len(), 40);
        assert_eq!(block_count(), 1_196);
        assert_eq!(state_count(), 32_366);
    }

    #[test]
    fn representative_states_match_pumpkin_26_2() {
        let air = state(AIR).unwrap();
        assert!(air.is_air());
        assert!(air.replaceable());
        assert_eq!(air.opacity, 0);

        let stone = state(STONE).unwrap();
        assert_eq!(stone.hardness, 1.5);
        assert!(stone.is_full_cube());
        assert_eq!(
            stone.collision_shapes().collect::<Vec<_>>(),
            vec![CollisionBox {
                min: [0.0, 0.0, 0.0],
                max: [1.0, 1.0, 1.0],
            }]
        );

        let short_grass = state(2_248).unwrap();
        assert!(short_grass.replaceable());
        assert!(!short_grass.is_air());

        let torch = state(3_370).unwrap();
        assert_eq!(torch.luminance, 14);
        assert_eq!(torch.collision_shape_count(), 0);
    }

    #[test]
    fn blocks_and_items_match_pumpkin_26_2() {
        let dirt = block_by_name("minecraft:dirt").unwrap();
        assert_eq!(dirt.id, 9);
        assert_eq!(dirt.default_state_id, DIRT);
        assert_eq!(dirt.item_id, Some(55));
        assert_eq!(default_state_for_item(55), Some(DIRT));
        assert_eq!(default_state_for_item(350), Some(3_370));
        assert_eq!(block_by_name("bedrock").unwrap().hardness, -1.0);
    }

    #[test]
    fn collision_offsets_match_pumpkin_coordinate_hash() {
        let bamboo = block_by_name("bamboo").unwrap();
        assert_eq!(bamboo.collision_offset(0, 0), [0.0, 0.0, 0.0]);
        assert_eq!(bamboo.collision_offset(-18, -7), [0.5, 0.0, 0.5]);

        let short_grass = block_by_name("short_grass").unwrap();
        let offset = short_grass.collision_offset(-18, -7);
        assert_eq!(offset[0], 0.5);
        assert!((offset[1] - 0.08).abs() < 1.0e-6);
        assert_eq!(offset[2], 0.5);
    }

    #[test]
    fn invalid_ids_are_rejected() {
        assert_eq!(state(u16::MAX), None);
        assert_eq!(block(u16::MAX), None);
        assert_eq!(default_state_for_item(u16::MAX), None);
    }
}
