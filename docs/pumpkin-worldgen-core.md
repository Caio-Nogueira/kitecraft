# Pumpkin terrain-density slice

KiteCraft's first real world-generation slice is a deterministic, WASM-safe
port of Pumpkin 26.2's overworld density router. Fresh worlds use it to choose
stone versus air; worlds that already contain chunks and lack generator
metadata retain the classic superflat generator so an upgrade cannot silently
mix two terrain algorithms.

## Boundary

`worldgen-core` owns the pinned seed derivation, Xoroshiro random stream,
Perlin and double-Perlin samplers, interpolated noise, density wrappers, and
overworld splines. Its output is a registry-independent 12,288-byte occupancy
map for a 16 x 384 x 16 column. `server-do` adapts occupied cells to KiteCraft's
stone block state through the existing `world::ChunkGenerator` interface.

The slice deliberately excludes biomes, surface rules, aquifers, caves, ore
veins, structures and their beardifier, decorations, lighting, Tokio, Rayon,
and storage. The reference contract is simply `terrain_density > 0 => stone`,
otherwise air.

The import is pinned to Pumpkin revision
`beb6947dfc21a1a781523bf207a3c2740f4928f9`. The native reference harness,
source hashes, fixture-coordinate hashes, occupancy layout, and five golden
SHA-256 hashes are recorded in
`crates/worldgen-core/pumpkin-worldgen-26.2-manifest.json`. Reproduction
instructions are in `crates/worldgen-core/UPSTREAM.md`.

## Verification

Run the same golden suite on the host and in a Node WebAssembly runtime:

```sh
cargo test -p worldgen-core --test golden
wasm-pack test --node crates/worldgen-core
```

The fixtures cover seeds `0` and `13579`, positive and negative chunk
coordinates, the origin, and distant terrain. A mismatch reports the first
different block coordinate and its computed density.

## Local Durable Object measurements

Measured with Wrangler 4.125.0's local Workers runtime, a release build, and
fresh Durable Object storage on 2026-08-22:

| Measurement | Result |
|---|---:|
| Spawn chunk `(0, 0)`, cold world initialization | 27 ms |
| Full login and 289-chunk stream (original unpaced baseline) | 5.74 s wall time |
| Worker WASM before this slice | 1,963,499 bytes |
| Worldgen-slice Worker WASM | 2,051,255 bytes |
| Worker WASM after chunk-window pacing | 2,070,827 bytes |
| Current change from pre-worldgen | +107,328 bytes (+5.5%) |

Chunk generation is synchronous and compute-only, so the spawn-chunk wall time
is a useful local CPU proxy, but it is not Cloudflare's billed CPU metric.
A deployed Worker trace is required for authoritative production CPU time.

The server now sends the view nearest-first in client-acknowledged batches of at
most eight chunks. It also announces the chunk-cache center and, whenever a
player crosses a chunk boundary, unloads the old edge and queues the new edge.
A local wire-level test received all 289 initial chunks as 36 batches (35 of
eight and one of one), then verified symmetric recentering across positive and
negative X boundaries. These pacing changes bound each generation burst; they
do not make an individual chunk asynchronous.

The current release WASM SHA-256 is
`96e39d5f932aedecf15262a5e84f3136ed228106c0c6a1081cd659ae660b9817`.
