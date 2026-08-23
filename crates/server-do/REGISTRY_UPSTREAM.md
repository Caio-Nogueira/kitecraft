# Configuration registry provenance

KiteCraft's Minecraft 26.2 synchronized registry stream is imported from
Pumpkin revision `beb6947dfc21a1a781523bf207a3c2740f4928f9`
(`0.1.0-dev+26.2-26.40`). The source is Pumpkin's generated
`crates/pumpkin-data/src/generated/registry.rs` table for `REGISTRY_V_26_2`.

From the repository root, reproduce the Rust index, NBT blob, and manifest with:

```sh
python3 tools/pumpkin-import/import_registries.py /path/to/pinned/Pumpkin
```

The importer refuses another Git revision and writes:

- `src/vanilla_registries.rs`, including semantic wire-index constants;
- `src/vanilla_registries.bin`, containing the concatenated entry NBT;
- `registry-26.2-manifest.json`, containing source/blob hashes and counts.

The generated Rust tests bounds-check all 398 slices, verify that every entry
starts with an unnamed compound tag, and prove the consumed plains, overworld,
player-attack, and overworld-clock indices against registry order.

The configuration tag stream comes from the same pinned revision's generated
`crates/pumpkin-data/src/generated/tag.rs`. Reproduce it with:

```sh
python3 tools/pumpkin-import/import_tags.py /path/to/pinned/Pumpkin
```

This writes `src/vanilla_tags.rs`, `src/vanilla_tags.bin`, and
`tags-26.2-manifest.json`. The generated test fully decodes the blob and checks
all registry, tag, and entry-reference counts plus representative tags required
by dimensions, enchantments, and sulfur-cube archetypes.
