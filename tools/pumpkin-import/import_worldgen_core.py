#!/usr/bin/env python3
"""Import the minimal Pumpkin 26.2 overworld density data used by worldgen-core."""

from __future__ import annotations

import hashlib
import json
import pathlib
import re
import subprocess
import sys

REVISION = "beb6947dfc21a1a781523bf207a3c2740f4928f9"
ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "crates" / "worldgen-core" / "src" / "generated"
FIXTURES = ROOT / "crates" / "worldgen-core" / "tests" / "fixtures"
REFERENCE_PATCH = ROOT / "tools" / "pumpkin-import" / "worldgen-reference.patch"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def matching(text: str, start: int, opening: str, closing: str) -> int:
    depth = 0
    in_string = False
    escaped = False
    for index in range(start, len(text)):
        char = text[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return index
    raise ValueError(f"unmatched {opening!r} at {start}")


def split_top_level(text: str) -> list[str]:
    items: list[str] = []
    start = 0
    depths = {"{": 0, "[": 0, "(": 0}
    pairs = {"}": "{", "]": "[", ")": "("}
    in_string = False
    escaped = False
    for index, char in enumerate(text):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char in depths:
            depths[char] += 1
        elif char in pairs:
            depths[pairs[char]] -= 1
        elif char == "," and all(depth == 0 for depth in depths.values()):
            item = text[start:index].strip()
            if item:
                items.append(item)
            start = index + 1
    tail = text[start:].strip()
    if tail:
        items.append(tail)
    return items


def extract_module(text: str, declaration: str) -> str:
    start = text.index(declaration)
    brace = text.index("{", start)
    end = matching(text, brace, "{", "}")
    return text[start : end + 1]


def extract_overworld_stack(text: str) -> list[str]:
    router = text.index("pub const OVERWORLD_BASE_NOISE_ROUTER")
    noise = text.index("noise: BaseNoiseRouter", router)
    stack = text.index("full_component_stack: &[", noise)
    bracket = text.index("[", stack)
    end = matching(text, bracket, "[", "]")
    return split_top_level(text[bracket + 1 : end])


def write(path: pathlib.Path, text: str | bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if isinstance(text, str):
        path.write_text(text)
    else:
        path.write_bytes(text)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: import_worldgen_core.py /path/to/Pumpkin")
    upstream = pathlib.Path(sys.argv[1]).resolve()
    revision = subprocess.check_output(
        ["git", "-C", str(upstream), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != REVISION:
        raise SystemExit(f"expected Pumpkin {REVISION}, got {revision}")

    parameter_path = upstream / "crates/pumpkin-data/src/generated/noise_parameter.rs"
    router_path = upstream / "crates/pumpkin-data/src/generated/noise_router.rs"
    perlin_path = upstream / "crates/pumpkin-util/src/noise/perlin.rs"
    double_perlin_path = upstream / "crates/pumpkin-world/src/generation/noise/perlin.rs"

    all_parameters = parameter_path.read_text()
    router = router_path.read_text()
    overworld = extract_module(router, "pub mod overworld_noise_evaluator")
    cave_start = overworld.index(
        "    pub fn overworld_node_55<"
    )
    cave_attribute = overworld.rfind("    #[inline(always)]", 0, cave_start)
    overworld = overworld[:cave_attribute] + r'''    #[inline(always)]
    fn sample_terrain_before_interpolation<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let with_top_slide = overworld_node_1(pos, ctx)
            * (overworld_node_54(pos, ctx) + 0.078125f64)
            - 0.078125f64;
        let with_bottom_slide = overworld_node_0(pos, ctx)
            * (with_top_slide - 0.1171875f64)
            + 0.1171875f64;
        ctx.sample_blend_density(with_bottom_slide, pos) * 0.64f64
    }

    #[inline(always)]
    pub fn sample_terrain_density<C: NoiseEvaluationContext>(
        pos: &pumpkin_util::math::vector3::Vector3<i32>,
        ctx: &mut C,
    ) -> f64 {
        let value = ctx.sample_wrapper(
            55usize,
            WrapperType::Interpolated,
            pos,
            &sample_terrain_before_interpolation,
        );
        let clamped = value.clamp(-1.0, 1.0);
        clamped / 2.0 - clamped * clamped * clamped / 24.0
    }
}'''
    overworld = overworld.replace(
        "pumpkin_util::math::vector3::Vector3<i32>", "crate::Vector3<i32>"
    )
    node_indices = sorted(
        {int(value) for value in re.findall(r"pub fn overworld_node_(\d+)<", overworld)}
    )
    parameter_names = sorted(
        set(re.findall(r"DoublePerlinNoiseParameters::([A-Z0-9_]+)", overworld))
    )
    struct_end = all_parameters.index("impl DoublePerlinNoiseParameters")
    new_start = all_parameters.index("    pub const fn new(", struct_end)
    new_brace = all_parameters.index("{", new_start)
    new_end = matching(all_parameters, new_brace, "{", "}") + 1
    parameter_constants = []
    for name in parameter_names:
        start = all_parameters.index(f"    pub const {name}:", new_end)
        equals = all_parameters.index("=", start)
        opening = all_parameters.index("(", equals)
        end = matching(all_parameters, opening, "(", ")") + 1
        parameter_constants.append(all_parameters[start : end + 1])
    parameter_count = 1 + max(
        int(re.search(r"new\(\s*(\d+)usize", item).group(1))
        for item in parameter_constants
    )
    parameters = (
        all_parameters[:struct_end]
        + "impl DoublePerlinNoiseParameters {\n"
        + f"    pub const COUNT: usize = {parameter_count};\n"
        + all_parameters[new_start:new_end]
        + "\n"
        + "\n".join(parameter_constants)
        + "\n}\n"
    )
    node_dispatch = "\n".join(
        f"        {index} => overworld_noise_evaluator::overworld_node_{index}(pos, ctx),"
        for index in node_indices
    )
    router_out = """/* Generated from pinned Pumpkin. Do not edit manually. */
use crate::DoublePerlinNoiseParameters;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WrapperType {
    Interpolated,
    CacheFlat,
    Cache2D,
    CacheOnce,
    CellCache,
}

pub trait NoiseEvaluationContext {
    fn sample_noise(&mut self, noise_id: DoublePerlinNoiseParameters, x: f64, y: f64, z: f64) -> f64;
    fn sample_shift_a(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &crate::Vector3<i32>) -> f64;
    fn sample_shift_b(&mut self, noise_id: DoublePerlinNoiseParameters, pos: &crate::Vector3<i32>) -> f64;
    fn sample_shifted_noise(&mut self, noise_id: DoublePerlinNoiseParameters, shift_x: f64, shift_y: f64, shift_z: f64, xz_scale: f64, y_scale: f64) -> f64;
    fn sample_interpolated_noise(&mut self, pos: &crate::Vector3<i32>) -> f64;
    fn sample_beardifier(&mut self, pos: &crate::Vector3<i32>) -> f64;
    fn sample_blend_alpha(&mut self, pos: &crate::Vector3<i32>) -> f64;
    fn sample_blend_offset(&mut self, pos: &crate::Vector3<i32>) -> f64;
    fn sample_blend_density(&mut self, input_val: f64, pos: &crate::Vector3<i32>) -> f64;
    fn sample_end_islands(&mut self, pos: &crate::Vector3<i32>) -> f64;
    fn sample_wrapper(&mut self, wrapper_index: usize, wrapper_type: WrapperType, pos: &crate::Vector3<i32>, eval_input: &dyn Fn(&crate::Vector3<i32>, &mut Self) -> f64) -> f64;
    fn sample_spline(&mut self, spline_index: usize, location_value: f64, pos: &crate::Vector3<i32>) -> f64;
    fn sample_find_top_surface(&mut self, density_fn: &dyn Fn(&crate::Vector3<i32>, &mut Self) -> f64, upper_bound_fn: &dyn Fn(&crate::Vector3<i32>, &mut Self) -> f64, lower_bound: i32, cell_height: i32, pos: &crate::Vector3<i32>) -> f64;
}

""" + overworld + f"""

pub fn sample_overworld_node<C: NoiseEvaluationContext>(index: usize, pos: &crate::Vector3<i32>, ctx: &mut C) -> f64 {{
    match index {{
{node_dispatch}
        _ => panic!("invalid overworld density node {{index}}"),
    }}
}}
"""

    perlin = perlin_path.read_text().split("/// Tests for the perlin noise implementations.", 1)[0]
    perlin = re.sub(
        r"use crate::\{\s*math::lerp3,\s*random::\{RandomDeriverImpl, RandomImpl\},\s*\};",
        "use crate::{lerp3, RandomImpl};",
        perlin,
    )
    perlin = perlin.replace("use super::GRADIENTS;", "use crate::GRADIENTS;")
    perlin = perlin.replace(
        "samplers\n            .into_iter()",
        "samplers\n            .into_vec()\n            .into_iter()",
    )
    double_perlin = double_perlin_path.read_text().split("#[cfg(test)]", 1)[0]
    double_perlin = double_perlin.replace(
        "use pumpkin_data::chunk::DoublePerlinNoiseParameters;",
        "use crate::DoublePerlinNoiseParameters;",
    ).replace(
        "use pumpkin_util::{noise::perlin::OctavePerlinNoiseSampler, random::RandomImpl};",
        "use crate::{OctavePerlinNoiseSampler, RandomImpl};",
    )

    stack = extract_overworld_stack(router)
    spline_indices = [27, 34, 43]
    spline_defs = []
    for index in spline_indices:
        item = stack[index]
        marker = "BaseNoiseFunctionComponent::Spline {"
        if marker not in item:
            raise SystemExit(f"overworld component {index} is no longer a spline")
        expression = item[item.index("spline: &") + len("spline: &") :].strip()
        if expression.endswith("}"):
            expression = expression[:-1].rstrip()
        expression = expression.removesuffix(",").rstrip()
        spline_defs.append(f"pub static SPLINE_{index}: SplineRepr = {expression};")
    splines = """/* Generated from pinned Pumpkin. Do not edit manually. */
pub struct SplinePoint {
    pub location: f32,
    pub value: &'static SplineRepr,
    pub derivative: f32,
}

pub enum SplineRepr {
    Standard { location_function_index: usize, points: &'static [SplinePoint] },
    Fixed { value: f32 },
}

""" + "\n\n".join(spline_defs) + "\n"

    write(OUT / "noise_parameter.rs", parameters)
    write(OUT / "overworld_router.rs", router_out)
    write(OUT / "perlin.rs", perlin)
    write(OUT / "double_perlin.rs", double_perlin)
    write(OUT / "splines.rs", splines)

    fixture_specs = [
        (0, 0, 0, "noise_no_blend_no_beard_0_0.chunk"),
        (0, 7, 4, "noise_no_blend_no_beard_7_4.chunk"),
        (0, -595, 544, "noise_no_blend_no_beard_-595_544.chunk"),
        (13579, -6, 11, "noise_no_blend_no_beard_13579_-6_11.chunk"),
        (13579, -2, 15, "noise_no_blend_no_beard_13579_-2_15.chunk"),
    ]
    fixture_manifest = []
    for seed, chunk_x, chunk_z, name in fixture_specs:
        source = upstream / "assets/tests" / name
        out_name = name.removesuffix(".chunk") + ".occupancy"
        fixture_path = FIXTURES / out_name
        if not fixture_path.exists():
            raise SystemExit(
                f"missing native reference {fixture_path}; apply "
                "tools/pumpkin-import/worldgen-reference.patch to the pinned "
                "Pumpkin checkout and run its dump_kitecraft_stone_air_references test"
            )
        bits = fixture_path.read_bytes()
        if len(bits) != 16 * 384 * 16 // 8:
            raise SystemExit(f"invalid native reference length: {fixture_path}")
        fixture_manifest.append(
            {
                "seed": seed,
                "chunk_x": chunk_x,
                "chunk_z": chunk_z,
                "height": 384,
                "layout": "x-major, then local-y, then z; bits are little-endian within each byte",
                "coordinate_source": f"assets/tests/{name}",
                "coordinate_source_sha256": sha256(source.read_bytes()),
                "reference_mode": {
                    "aquifers": False,
                    "noise_caves": False,
                    "ore_veins": False,
                    "beardifier": False,
                    "density_pipeline": "sloped_cheese + slides + blend + interpolation + squeeze",
                    "projection": "terrain_density > 0 => stone; otherwise air",
                },
                "occupancy": out_name,
                "occupancy_sha256": sha256(bits),
            }
        )

    manifest = {
        "format": 1,
        "minecraft_version": "26.2",
        "pumpkin_revision": REVISION,
        "reference_harness": {
            "path": str(REFERENCE_PATCH.relative_to(ROOT)),
            "sha256": sha256(REFERENCE_PATCH.read_bytes()),
        },
        "sources": {
            str(path.relative_to(upstream)): sha256(path.read_bytes())
            for path in [parameter_path, router_path, perlin_path, double_perlin_path]
        },
        "fixtures": fixture_manifest,
    }
    write(
        ROOT / "crates/worldgen-core/pumpkin-worldgen-26.2-manifest.json",
        json.dumps(manifest, indent=2) + "\n",
    )


if __name__ == "__main__":
    main()
