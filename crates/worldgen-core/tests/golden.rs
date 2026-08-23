use sha2::{Digest, Sha256};
use worldgen_core::PumpkinTerrain;

struct Fixture {
    seed: u64,
    chunk_x: i32,
    chunk_z: i32,
    expected: &'static [u8],
    expected_sha256: &'static str,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        seed: 0,
        chunk_x: 0,
        chunk_z: 0,
        expected: include_bytes!("fixtures/noise_no_blend_no_beard_0_0.occupancy"),
        expected_sha256: "45a889952333d75ae5cf3d2aafad2267d5b1d0e7da20f5467f03ac608a184f20",
    },
    Fixture {
        seed: 0,
        chunk_x: 7,
        chunk_z: 4,
        expected: include_bytes!("fixtures/noise_no_blend_no_beard_7_4.occupancy"),
        expected_sha256: "b86d4f377ae77c9363b4a1ce5c4e9376f80dc252fd8844c0f8a7ffffde21d12a",
    },
    Fixture {
        seed: 0,
        chunk_x: -595,
        chunk_z: 544,
        expected: include_bytes!("fixtures/noise_no_blend_no_beard_-595_544.occupancy"),
        expected_sha256: "ead482cedc31934593e89eafaaa64ae224945f5d1439b4bfee04006a6eae3106",
    },
    Fixture {
        seed: 13_579,
        chunk_x: -6,
        chunk_z: 11,
        expected: include_bytes!("fixtures/noise_no_blend_no_beard_13579_-6_11.occupancy"),
        expected_sha256: "f5f3ccc9d53a7d5faf1b828002a2fab2bfd2659e0e101d8a9ce7ff64ce06b059",
    },
    Fixture {
        seed: 13_579,
        chunk_x: -2,
        chunk_z: 15,
        expected: include_bytes!("fixtures/noise_no_blend_no_beard_13579_-2_15.occupancy"),
        expected_sha256: "b6926ec61c5c0dd3df0c3f6f0c5f545203649b5f9dd2c19e4d076d9e2fc78112",
    },
];

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn fixture_hashes_are_self_consistent() {
    for fixture in FIXTURES {
        let hash = format!("{:x}", Sha256::digest(fixture.expected));
        assert_eq!(hash, fixture.expected_sha256);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), test)]
fn pumpkin_terrain_matches_golden_occupancy() {
    for fixture in FIXTURES {
        let mut generator = PumpkinTerrain::new(fixture.seed);
        let column = generator.generate_occupancy(fixture.chunk_x, fixture.chunk_z);
        let actual = column.as_bytes();
        if actual != fixture.expected {
            let mismatch_count = actual
                .iter()
                .zip(fixture.expected)
                .map(|(actual, expected)| (actual ^ expected).count_ones() as usize)
                .sum::<usize>();
            let first_bit = actual
                .iter()
                .zip(fixture.expected)
                .enumerate()
                .find_map(|(byte_index, (actual, expected))| {
                    let difference = actual ^ expected;
                    (difference != 0).then(|| byte_index * 8 + difference.trailing_zeros() as usize)
                })
                .expect("different buffers have a differing bit");
            let z = first_bit % 16;
            let local_y = (first_bit / 16) % 384;
            let x = first_bit / (16 * 384);
            let density = generator.density(
                fixture.chunk_x * 16 + x as i32,
                local_y as i32 - 64,
                fixture.chunk_z * 16 + z as i32,
            );
            panic!(
                "occupancy mismatch for seed {} chunk ({}, {}): {mismatch_count} blocks; first at local ({x}, {}, {z}), actual={}, expected={}, density={density}",
                fixture.seed,
                fixture.chunk_x,
                fixture.chunk_z,
                local_y as i32 - 64,
                (actual[first_bit / 8] >> (first_bit % 8)) & 1,
                (fixture.expected[first_bit / 8] >> (first_bit % 8)) & 1,
            );
        }
        assert_eq!(
            format!("{:x}", Sha256::digest(actual)),
            fixture.expected_sha256
        );
    }
}
