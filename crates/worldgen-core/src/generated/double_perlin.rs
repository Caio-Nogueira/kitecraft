use crate::DoublePerlinNoiseParameters;
use crate::{OctavePerlinNoiseSampler, RandomImpl};

pub struct DoublePerlinNoiseSampler {
    first_sampler: OctavePerlinNoiseSampler,
    second_sampler: OctavePerlinNoiseSampler,
    amplitude: f64,
    max_value: f64,
}

impl DoublePerlinNoiseSampler {
    const fn create_amplitude(octaves: i32) -> f64 {
        0.1f64 * (1f64 + 1f64 / (octaves + 1) as f64)
    }

    #[must_use]
    pub const fn max_value(&self) -> f64 {
        self.max_value
    }

    pub fn from_params(
        rand: &mut impl RandomImpl,
        parameters: &DoublePerlinNoiseParameters,
        legacy: bool,
    ) -> Self {
        Self::new(
            rand,
            parameters.first_octave,
            parameters.amplitudes,
            parameters.amplitude,
            legacy,
        )
    }

    #[must_use]
    pub fn get_amplitude(amplitudes: &[f64]) -> f64 {
        let mut j = i32::MAX;
        let mut k = i32::MIN;

        for (index, amplitude) in amplitudes.iter().enumerate() {
            if *amplitude != 0f64 {
                j = i32::min(j, index as i32);
                k = i32::max(k, index as i32);
            }
        }

        0.16666666666666666f64 / Self::create_amplitude(k - j)
    }

    pub fn new(
        rand: &mut impl RandomImpl,
        first_octave: i32,
        amplitudes: &[f64],
        amplitude: f64,
        legacy: bool,
    ) -> Self {
        let first_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);
        let second_sampler = OctavePerlinNoiseSampler::new(rand, first_octave, amplitudes, legacy);

        let max_value = (first_sampler.max_value() + second_sampler.max_value()) * amplitude;

        Self {
            first_sampler,
            second_sampler,
            amplitude,
            max_value,
        }
    }

    #[must_use]
    pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = x * 1.0181268882175227f64;
        let e = y * 1.0181268882175227f64;
        let f = z * 1.0181268882175227f64;

        (self.first_sampler.sample(x, y, z) + self.second_sampler.sample(d, e, f)) * self.amplitude
    }
}
