//! WASM-safe, deterministic Pumpkin 26.2 overworld terrain density.
//!
//! This first slice intentionally stops before cave noise, aquifers, ores, and
//! surface rules, and emits only stone and air.

#[allow(dead_code)]
mod generated;

use generated::{
    overworld_noise_evaluator, sample_overworld_node, DoublePerlinNoiseParameters,
    DoublePerlinNoiseSampler, NoiseEvaluationContext, OctavePerlinNoiseSampler, SplineRepr,
    WrapperType,
};
use std::collections::HashMap;

pub const PUMPKIN_REVISION: &str = "beb6947dfc21a1a781523bf207a3c2740f4928f9";
pub const MINECRAFT_VERSION: &str = "26.2";
pub const MIN_Y: i32 = -64;
pub const WORLD_HEIGHT: i32 = 384;
const OCCUPANCY_BYTES: usize = 16 * WORLD_HEIGHT as usize * 16 / 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

pub struct Gradient {
    x: f64,
    y: f64,
    z: f64,
}

impl Gradient {
    #[inline]
    pub const fn dot(&self, x: f64, y: f64, z: f64) -> f64 {
        self.x * x + self.y * y + self.z * z
    }
}

pub const GRADIENTS: [Gradient; 16] = [
    Gradient {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    },
    Gradient {
        x: -1.0,
        y: 1.0,
        z: 0.0,
    },
    Gradient {
        x: 1.0,
        y: -1.0,
        z: 0.0,
    },
    Gradient {
        x: -1.0,
        y: -1.0,
        z: 0.0,
    },
    Gradient {
        x: 1.0,
        y: 0.0,
        z: 1.0,
    },
    Gradient {
        x: -1.0,
        y: 0.0,
        z: 1.0,
    },
    Gradient {
        x: 1.0,
        y: 0.0,
        z: -1.0,
    },
    Gradient {
        x: -1.0,
        y: 0.0,
        z: -1.0,
    },
    Gradient {
        x: 0.0,
        y: 1.0,
        z: 1.0,
    },
    Gradient {
        x: 0.0,
        y: -1.0,
        z: 1.0,
    },
    Gradient {
        x: 0.0,
        y: 1.0,
        z: -1.0,
    },
    Gradient {
        x: 0.0,
        y: -1.0,
        z: -1.0,
    },
    Gradient {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    },
    Gradient {
        x: 0.0,
        y: -1.0,
        z: 1.0,
    },
    Gradient {
        x: -1.0,
        y: 1.0,
        z: 0.0,
    },
    Gradient {
        x: 0.0,
        y: -1.0,
        z: -1.0,
    },
];

#[inline]
fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

#[inline]
fn lerp2(dx: f64, dy: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(dy, lerp(dx, x0y0, x1y0), lerp(dx, x0y1, x1y1))
}

#[allow(clippy::too_many_arguments)]
fn lerp3(
    dx: f64,
    dy: f64,
    dz: f64,
    x0y0z0: f64,
    x1y0z0: f64,
    x0y1z0: f64,
    x1y1z0: f64,
    x0y0z1: f64,
    x1y0z1: f64,
    x0y1z1: f64,
    x1y1z1: f64,
) -> f64 {
    lerp(
        dz,
        lerp2(dx, dy, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(dx, dy, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

trait RandomImpl {
    fn next_bounded_i32(&mut self, bound: i32) -> i32;
    fn next_f64(&mut self) -> f64;
    fn next_splitter(&mut self) -> XoroshiroSplitter;
    fn skip(&mut self, count: i32);
}

struct Xoroshiro {
    lo: u64,
    hi: u64,
}

impl Xoroshiro {
    const fn from_seed(seed: u64) -> Self {
        let lo = seed ^ 0x6A09_E667_F3BC_C909;
        let hi = lo.wrapping_add(0x9E37_79B9_7F4A_7C15);
        Self::new(mix_stafford_13(lo), mix_stafford_13(hi))
    }

    const fn new(lo: u64, hi: u64) -> Self {
        let (lo, hi) = if (lo | hi) == 0 {
            (0x9E37_79B9_7F4A_7C15, 0x6A09_E667_F3BC_C909)
        } else {
            (lo, hi)
        };
        Self { lo, hi }
    }

    const fn next_random(&mut self) -> u64 {
        let lo = self.lo;
        let hi = self.hi;
        let result = lo.wrapping_add(hi).rotate_left(17).wrapping_add(lo);
        let hi = hi ^ lo;
        self.lo = lo.rotate_left(49) ^ hi ^ (hi << 21);
        self.hi = hi.rotate_left(28);
        result
    }
}

impl RandomImpl for Xoroshiro {
    fn next_bounded_i32(&mut self, bound: i32) -> i32 {
        let mut value = (self.next_random() as i32 as u64) & 0xFFFF_FFFF;
        let mut product = value.wrapping_mul(bound as u64);
        let mut low = product & 0xFFFF_FFFF;
        if low < bound as u64 {
            let threshold = ((!bound).wrapping_add(1) as u64) % bound as u64;
            while low < threshold {
                value = (self.next_random() as i32 as u64) & 0xFFFF_FFFF;
                product = value.wrapping_mul(bound as u64);
                low = product & 0xFFFF_FFFF;
            }
        }
        (product >> 32) as i32
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_random() >> 11) as f64 * f64::from(1.110_223E-16f32)
    }

    fn next_splitter(&mut self) -> XoroshiroSplitter {
        XoroshiroSplitter {
            lo: self.next_random(),
            hi: self.next_random(),
        }
    }

    fn skip(&mut self, count: i32) {
        for _ in 0..count {
            self.next_random();
        }
    }
}

#[derive(Clone, Copy)]
struct XoroshiroSplitter {
    lo: u64,
    hi: u64,
}

impl XoroshiroSplitter {
    fn split_string(&self, seed: &str) -> Xoroshiro {
        let bytes = md5::compute(seed.as_bytes());
        let lo = u64::from_be_bytes(bytes[0..8].try_into().unwrap_or([0; 8]));
        let hi = u64::from_be_bytes(bytes[8..16].try_into().unwrap_or([0; 8]));
        Xoroshiro::new(lo ^ self.lo, hi ^ self.hi)
    }

    const fn from_lo_and_hi(&self, lo: u64, hi: u64) -> Xoroshiro {
        Xoroshiro::new(lo ^ self.lo, hi ^ self.hi)
    }
}

const fn mix_stafford_13(value: u64) -> u64 {
    let value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

struct InterpolatedNoiseSampler {
    lower_noise: OctavePerlinNoiseSampler,
    upper_noise: OctavePerlinNoiseSampler,
    noise: OctavePerlinNoiseSampler,
    fractions: [f64; 16],
    y_multiplier: f64,
}

impl InterpolatedNoiseSampler {
    fn new(random: &mut impl RandomImpl) -> Self {
        let lower_noise = OctavePerlinNoiseSampler::new(random, -15, &[1.0; 16], true);
        let upper_noise = OctavePerlinNoiseSampler::new(random, -15, &[1.0; 16], true);
        let noise = OctavePerlinNoiseSampler::new(random, -7, &[1.0; 8], true);
        let mut fraction = 1.0;
        let fractions = std::array::from_fn(|_| {
            let result = fraction;
            fraction /= 2.0;
            result
        });
        Self {
            lower_noise,
            upper_noise,
            noise,
            fractions,
            y_multiplier: 0.25 * 80.0 / 160.0 * 684.412,
        }
    }

    fn sample(&self, pos: &Vector3<i32>) -> f64 {
        let xz_multiplier = 0.25 * 684.412;
        let x = pos.x as f64 * xz_multiplier;
        let y = pos.y as f64 * self.y_multiplier;
        let z = pos.z as f64 * xz_multiplier;
        let scaled_x = x / 80.0;
        let scaled_y = y / 160.0;
        let scaled_z = z / 80.0;
        let smear = self.y_multiplier * 8.0;
        let scaled_smear = smear / 160.0;

        let blend: f64 = self
            .noise
            .samplers
            .iter()
            .rev()
            .zip(self.fractions)
            .map(|(data, fraction)| {
                data.sampler.sample_no_fade(
                    OctavePerlinNoiseSampler::maintain_precision(scaled_x * fraction),
                    OctavePerlinNoiseSampler::maintain_precision(scaled_y * fraction),
                    OctavePerlinNoiseSampler::maintain_precision(scaled_z * fraction),
                    scaled_smear * fraction,
                    scaled_y * fraction,
                ) / fraction
            })
            .sum();
        let blend = f64::midpoint(blend / 10.0, 1.0);

        let sample_octaves = |sampler: &OctavePerlinNoiseSampler| {
            sampler
                .samplers
                .iter()
                .rev()
                .zip(self.fractions)
                .map(|(data, fraction)| {
                    data.sampler.sample_no_fade(
                        OctavePerlinNoiseSampler::maintain_precision(x * fraction),
                        OctavePerlinNoiseSampler::maintain_precision(y * fraction),
                        OctavePerlinNoiseSampler::maintain_precision(z * fraction),
                        smear * fraction,
                        y * fraction,
                    ) / fraction
                })
                .sum::<f64>()
        };
        let lower = if blend >= 1.0 {
            0.0
        } else {
            sample_octaves(&self.lower_noise)
        };
        let upper = if blend <= 0.0 {
            0.0
        } else {
            sample_octaves(&self.upper_noise)
        };
        clamped_lerp(lower / 512.0, upper / 512.0, blend) / 128.0
    }
}

fn clamped_lerp(start: f64, end: f64, delta: f64) -> f64 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        lerp(delta, start, end)
    }
}

struct DensityContext {
    splitter: XoroshiroSplitter,
    samplers: Vec<Option<DoublePerlinNoiseSampler>>,
    interpolated: InterpolatedNoiseSampler,
    wrapper_cache: HashMap<(usize, i32, i32, i32), f64>,
}

impl DensityContext {
    fn new(seed: u64) -> Self {
        let splitter = Xoroshiro::from_seed(seed).next_splitter();
        let mut terrain_random = splitter.split_string("minecraft:terrain");
        Self {
            splitter,
            samplers: (0..DoublePerlinNoiseParameters::COUNT)
                .map(|_| None)
                .collect(),
            interpolated: InterpolatedNoiseSampler::new(&mut terrain_random),
            wrapper_cache: HashMap::new(),
        }
    }

    fn begin_chunk(&mut self) {
        self.wrapper_cache.clear();
    }

    fn sampler(&mut self, parameters: &DoublePerlinNoiseParameters) -> &DoublePerlinNoiseSampler {
        let id = parameters.id;
        if self.samplers[id].is_none() {
            let mut random = self.splitter.from_lo_and_hi(parameters.lo, parameters.hi);
            self.samplers[id] = Some(DoublePerlinNoiseSampler::from_params(
                &mut random,
                parameters,
                false,
            ));
        }
        self.samplers[id].as_ref().expect("inserted above")
    }

    fn eval_spline(
        &mut self,
        spline: &'static SplineRepr,
        location: Option<f32>,
        pos: &Vector3<i32>,
    ) -> f32 {
        match spline {
            SplineRepr::Fixed { value } => *value,
            SplineRepr::Standard {
                location_function_index,
                points,
            } => {
                let location = location.unwrap_or_else(|| {
                    sample_overworld_node(*location_function_index, pos, self) as f32
                });
                let index = points.partition_point(|point| location >= point.location);
                if index == 0 {
                    let point = &points[0];
                    let value = self.eval_spline(point.value, None, pos);
                    return point.derivative * (location - point.location) + value;
                }
                if index == points.len() {
                    let point = &points[points.len() - 1];
                    let value = self.eval_spline(point.value, None, pos);
                    return point.derivative * (location - point.location) + value;
                }
                let lower = &points[index - 1];
                let upper = &points[index];
                let lower_value = self.eval_spline(lower.value, None, pos);
                let upper_value = self.eval_spline(upper.value, None, pos);
                let distance = upper.location - lower.location;
                let progress = (location - lower.location) / distance;
                let delta = upper_value - lower_value;
                let lower_curve = lower.derivative * distance - delta;
                let upper_curve = -upper.derivative * distance + delta;
                let cubic = progress
                    * (1.0 - progress)
                    * (lower_curve + progress * (upper_curve - lower_curve));
                cubic + lower_value + progress * delta
            }
        }
    }
}

impl NoiseEvaluationContext for DensityContext {
    fn sample_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        x: f64,
        y: f64,
        z: f64,
    ) -> f64 {
        self.sampler(&noise_id).sample(x, y, z)
    }

    fn sample_shift_a(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &Vector3<i32>) -> f64 {
        self.sample_noise(noise_id, pos.x as f64 * 0.25, 0.0, pos.z as f64 * 0.25) * 4.0
    }

    fn sample_shift_b(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &Vector3<i32>) -> f64 {
        self.sample_noise(noise_id, pos.z as f64 * 0.25, pos.x as f64 * 0.25, 0.0) * 4.0
    }

    fn sample_shifted_noise(
        &mut self,
        noise_id: DoublePerlinNoiseParameters,
        shift_x: f64,
        shift_y: f64,
        shift_z: f64,
        xz_scale: f64,
        y_scale: f64,
    ) -> f64 {
        // Generated callers pass only shifts/scales, so recover the current
        // sample position through the thread-local-free field set by the caller.
        let pos = CURRENT_POS.with(|value| value.get());
        self.sample_noise(
            noise_id,
            pos.x as f64 * xz_scale + shift_x,
            pos.y as f64 * y_scale + shift_y,
            pos.z as f64 * xz_scale + shift_z,
        )
    }

    fn sample_interpolated_noise(&mut self, pos: &Vector3<i32>) -> f64 {
        self.interpolated.sample(pos)
    }

    fn sample_beardifier(&mut self, _pos: &Vector3<i32>) -> f64 {
        0.0
    }
    fn sample_blend_alpha(&mut self, _pos: &Vector3<i32>) -> f64 {
        1.0
    }
    fn sample_blend_offset(&mut self, _pos: &Vector3<i32>) -> f64 {
        0.0
    }
    fn sample_blend_density(&mut self, input_val: f64, _pos: &Vector3<i32>) -> f64 {
        input_val
    }
    fn sample_end_islands(&mut self, _pos: &Vector3<i32>) -> f64 {
        0.0
    }

    fn sample_wrapper(
        &mut self,
        wrapper_index: usize,
        wrapper_type: WrapperType,
        pos: &Vector3<i32>,
        eval_input: &dyn Fn(&Vector3<i32>, &mut Self) -> f64,
    ) -> f64 {
        match wrapper_type {
            WrapperType::Interpolated => {
                let x0 = pos.x.div_euclid(4) * 4;
                let y0 = pos.y.div_euclid(8) * 8;
                let z0 = pos.z.div_euclid(4) * 4;
                let corner = |x: i32, y: i32, z: i32, context: &mut Self| {
                    let key = (wrapper_index, x, y, z);
                    if let Some(value) = context.wrapper_cache.get(&key) {
                        *value
                    } else {
                        let point = Vector3::new(x, y, z);
                        let value = with_current_pos(point, || eval_input(&point, context));
                        context.wrapper_cache.insert(key, value);
                        value
                    }
                };
                let dx = pos.x.rem_euclid(4) as f64 / 4.0;
                let dy = pos.y.rem_euclid(8) as f64 / 8.0;
                let dz = pos.z.rem_euclid(4) as f64 / 4.0;
                lerp3(
                    dx,
                    dy,
                    dz,
                    corner(x0, y0, z0, self),
                    corner(x0 + 4, y0, z0, self),
                    corner(x0, y0 + 8, z0, self),
                    corner(x0 + 4, y0 + 8, z0, self),
                    corner(x0, y0, z0 + 4, self),
                    corner(x0 + 4, y0, z0 + 4, self),
                    corner(x0, y0 + 8, z0 + 4, self),
                    corner(x0 + 4, y0 + 8, z0 + 4, self),
                )
            }
            WrapperType::CacheFlat => {
                let point = Vector3::new(pos.x.div_euclid(4) * 4, 0, pos.z.div_euclid(4) * 4);
                let key = (wrapper_index, point.x, point.y, point.z);
                if let Some(value) = self.wrapper_cache.get(&key) {
                    *value
                } else {
                    let value = with_current_pos(point, || eval_input(&point, self));
                    self.wrapper_cache.insert(key, value);
                    value
                }
            }
            WrapperType::Cache2D => {
                let key = (wrapper_index, pos.x, 0, pos.z);
                if let Some(value) = self.wrapper_cache.get(&key) {
                    *value
                } else {
                    let value = eval_input(pos, self);
                    self.wrapper_cache.insert(key, value);
                    value
                }
            }
            WrapperType::CacheOnce | WrapperType::CellCache => eval_input(pos, self),
        }
    }

    fn sample_spline(
        &mut self,
        spline_index: usize,
        location_value: f64,
        pos: &Vector3<i32>,
    ) -> f64 {
        let spline = match spline_index {
            27 => &generated::SPLINE_27,
            34 => &generated::SPLINE_34,
            43 => &generated::SPLINE_43,
            _ => panic!("unknown overworld spline {spline_index}"),
        };
        self.eval_spline(spline, Some(location_value as f32), pos) as f64
    }

    fn sample_find_top_surface(
        &mut self,
        density_fn: &dyn Fn(&Vector3<i32>, &mut Self) -> f64,
        upper_bound_fn: &dyn Fn(&Vector3<i32>, &mut Self) -> f64,
        lower_bound: i32,
        cell_height: i32,
        pos: &Vector3<i32>,
    ) -> f64 {
        let upper = upper_bound_fn(pos, self).floor() as i32;
        let mut y = upper.div_euclid(cell_height) * cell_height;
        while y >= lower_bound {
            let point = Vector3::new(pos.x, y, pos.z);
            if density_fn(&point, self) > 0.390625 {
                return y as f64;
            }
            y -= cell_height;
        }
        lower_bound as f64
    }
}

thread_local! {
    static CURRENT_POS: std::cell::Cell<Vector3<i32>> = const { std::cell::Cell::new(Vector3::new(0, 0, 0)) };
}

fn with_current_pos<T>(pos: Vector3<i32>, callback: impl FnOnce() -> T) -> T {
    CURRENT_POS.with(|current| {
        let old = current.replace(pos);
        let result = callback();
        current.set(old);
        result
    })
}

pub struct PumpkinTerrain {
    seed: u64,
    context: DensityContext,
}

impl PumpkinTerrain {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            context: DensityContext::new(seed),
        }
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub fn density(&mut self, x: i32, y: i32, z: i32) -> f64 {
        let pos = Vector3::new(x, y, z);
        with_current_pos(pos, || {
            overworld_noise_evaluator::sample_terrain_density(&pos, &mut self.context)
        })
    }

    pub fn generate_occupancy(&mut self, chunk_x: i32, chunk_z: i32) -> TerrainColumn {
        self.context.begin_chunk();
        let mut column = TerrainColumn::default();
        for x in 0..16 {
            for y in MIN_Y..MIN_Y + WORLD_HEIGHT {
                for z in 0..16 {
                    let world_x = chunk_x.wrapping_mul(16).wrapping_add(x);
                    let world_z = chunk_z.wrapping_mul(16).wrapping_add(z);
                    if self.density(world_x, y, world_z) > 0.0 {
                        column.set_stone(x, y, z);
                    }
                }
            }
        }
        column
    }
}

/// Compact, registry-independent stone/air output from the density pipeline.
pub struct TerrainColumn {
    occupancy: Box<[u8; OCCUPANCY_BYTES]>,
}

impl Default for TerrainColumn {
    fn default() -> Self {
        Self {
            occupancy: Box::new([0; OCCUPANCY_BYTES]),
        }
    }
}

impl TerrainColumn {
    fn index(x: i32, y: i32, z: i32) -> usize {
        debug_assert!((0..16).contains(&x));
        debug_assert!((MIN_Y..MIN_Y + WORLD_HEIGHT).contains(&y));
        debug_assert!((0..16).contains(&z));
        ((x * WORLD_HEIGHT + (y - MIN_Y)) * 16 + z) as usize
    }

    fn set_stone(&mut self, x: i32, y: i32, z: i32) {
        let index = Self::index(x, y, z);
        self.occupancy[index / 8] |= 1 << (index % 8);
    }

    #[must_use]
    pub fn is_stone(&self, x: i32, y: i32, z: i32) -> bool {
        let index = Self::index(x, y, z);
        self.occupancy[index / 8] & (1 << (index % 8)) != 0
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.occupancy.as_slice()
    }
}
