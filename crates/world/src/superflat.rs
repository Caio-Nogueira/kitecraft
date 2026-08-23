use crate::blocks;
use crate::chunk::ChunkColumn;
use crate::store::ChunkGenerator;

pub struct Superflat {
    pub layers: Vec<(u16, u32)>,
}

impl Default for Superflat {
    fn default() -> Self {
        Self::classic()
    }
}

impl Superflat {
    pub fn classic() -> Self {
        Self {
            layers: vec![
                (blocks::BEDROCK, 1),
                (blocks::DIRT, 2),
                (blocks::GRASS_BLOCK, 1),
            ],
        }
    }

    pub fn surface_y(&self) -> i32 {
        crate::MIN_Y + self.layers.iter().map(|(_, h)| *h as i32).sum::<i32>() - 1
    }

    pub fn generate(&self, _x: i32, _z: i32) -> ChunkColumn {
        let mut col = ChunkColumn::default();
        let mut y = crate::MIN_Y;
        for (state, height) in &self.layers {
            for _ in 0..*height {
                for z in 0..16i32 {
                    for x in 0..16i32 {
                        col.set_block(x, y, z, *state);
                    }
                }
                y += 1;
            }
        }
        col
    }
}

impl ChunkGenerator for Superflat {
    fn generate(&mut self, x: i32, z: i32) -> ChunkColumn {
        Self::generate(self, x, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let g = Superflat::classic();
        assert_eq!(g.generate(0, 0), g.generate(123, -456));
        assert_eq!(g.surface_y(), -61);
    }
}
