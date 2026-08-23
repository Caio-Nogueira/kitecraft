# Pumpkin world-generation provenance

`worldgen-core` contains a WASM-safe port of Pumpkin's deterministic overworld
density computation. It exposes only a compact stone/air occupancy column; the
server's chunk model and block registry stay outside this crate.

- Repository: <https://github.com/Pumpkin-MC/Pumpkin>
- Revision: `beb6947dfc21a1a781523bf207a3c2740f4928f9`
- Pumpkin version: `0.1.0-dev+26.2-26.40`
- Minecraft version: `26.2`
- Upstream license: GPL-3.0

Generate the native-Pumpkin references from a clean checkout at that exact
revision, then regenerate the imported Rust and manifest:

```sh
git -C /path/to/Pumpkin apply \
  /path/to/KiteCraft/tools/pumpkin-import/worldgen-reference.patch
KITECRAFT_REFERENCE_DIR=/path/to/KiteCraft/crates/worldgen-core/tests/fixtures \
  cargo test --manifest-path /path/to/Pumpkin/Cargo.toml \
  -p pumpkin-world dump_kitecraft_stone_air_references -- --nocapture
python3 tools/pumpkin-import/import_worldgen_core.py /path/to/Pumpkin
```

The reference harness builds a native Pumpkin router that stops before cave
noise while retaining sloped-cheese terrain, vertical slides, blending,
interpolation, and squeeze. It also disables aquifers, ore veins, and the
structure beardifier, then writes a 1-bit stone/air projection for each selected
chunk. The importer rejects other revisions and records the harness, source,
coordinate-source, and occupancy hashes in
`pumpkin-worldgen-26.2-manifest.json`.

The runtime crate does not depend on Pumpkin's Tokio, Rayon, filesystem,
storage, networking, biome, surface-rule, cave, structure, decoration,
lighting, or chunk-system code.
