/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy)]
pub struct DoublePerlinNoiseParameters {
    pub id: usize,
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
    pub lo: u64,
    pub hi: u64,
    pub amplitude: f64,
}
impl DoublePerlinNoiseParameters {
    pub const COUNT: usize = 44;
    pub const fn new(
        id: usize,
        first_octave: i32,
        amplitudes: &'static [f64],
        lo: u64,
        hi: u64,
        amplitude: f64,
    ) -> Self {
        Self {
            id,
            first_octave,
            amplitudes,
            lo,
            hi,
            amplitude,
        }
    }
    pub const CONTINENTALNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        12usize,
        -9i32,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        9477944837549565538u64,
        12656866088844454061u64,
        1.4999999999999998f64,
    );
    pub const EROSION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        14usize,
        -9i32,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        14998273076172386264u64,
        5157273775208757888u64,
        1.3888888888888888f64,
    );
    pub const JAGGED: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        22usize,
        -16i32,
        &[
            1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64,
            1f64, 1f64,
        ],
        17943115692276099476u64,
        8209175272455791875u64,
        1.568627450980392f64,
    );
    pub const OFFSET: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        32usize,
        -3i32,
        &[1f64, 1f64, 1f64, 0f64],
        577895406318539652u64,
        4557074653038767061u64,
        1.25f64,
    );
    pub const RIDGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        43usize,
        -7i32,
        &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64],
        17278323085305457460u64,
        2012804684704589034u64,
        1.25f64,
    );
}
