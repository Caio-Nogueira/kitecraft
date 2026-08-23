# KiteCraft

A Minecraft Java Edition server that runs entirely on Cloudflare Workers, using
a Durable Object for stateful game logic and WebSocket transport via
[wsmc](https://github.com/rikka0w0/wsmc) (one Minecraft packet per WS binary
message). Targets **MC 26.2 (protocol 776)**.

Implements the PLAN.md MVP milestone — *walkable world*:

- server-list ping over wsmc end-to-end
- offline-mode login → full configuration phase (vanilla registry sync) → join
- Pumpkin 26.2 overworld terrain density for fresh worlds, currently projected
  to stone/air and streamed as real 1.18+ chunk sections (paletted containers +
  heightmaps + skylight); superflat remains the legacy-world fallback
- moving chunk window centered on each player, with client-acknowledged batches
  capped at eight chunks so generation is paced across the connection
- server-authoritative standing-player collision against Pumpkin's 26.2 block
  shapes, including actor corrections and validated movement sync
- deterministic bare-fist PvP with hitbox reach, block-shape line of sight,
  Pumpkin's recharge/hurt-protection rules, health, and knockback packets
- chat and creative block place/break
- chunk + player persistence in DO SQLite, batched ~1 Hz flushes
- player-scoped 20 TPS timer loop for deterministic gameplay; a 1 Hz DO alarm
  sends Time Updates / Keep-Alives, flushes dirty chunks, and deletes itself
  when the last player leaves (hibernation-eligible idle)

## Layout

```
crates/
├── block-data/ # compact Pumpkin 26.2 block/state/item/shape metadata
├── server-do/   # workers-rs entry + MinecraftServer DO (fetch/websocket_*/alarm)
│                #   protocol glue: registries, chunk wire format, sessions
│                #   src/vanilla_registries.* — generated Pumpkin 26.2
│                #   synchronized registry names, ordering, and inline NBT
├── game-core/   # platform-independent, deterministic player inputs/ticks/effects
├── net-ws/      # wsmc framing adapter + host-testable packet codecs:
│                #   VarInt, read/write buffers, zlib frame codec, NBT writer,
│                #   text components, generated Pumpkin 26.2 packet ID table,
│                #   and all packet builders/parsers used
├── world/       # ChunkColumn/Section model, generator/backing traits,
│                #   LRU cache, superflat fallback, compressed blobs
└── worldgen-core/# WASM-safe pinned Pumpkin seed, noise, and density computation
tools/
└── protocol-smoke/  # Node scripts driving raw MC-over-WS against `wrangler dev`
docs/
├── wsmc-framing.md  # Phase 0 spike: exact WS↔packet framing (resolved)
└── protocol-notes.md# packet IDs, registries, chunk format decisions
```

Movement and block interactions are decoded into semantic game-core input,
applied in FIFO order on the next logical 20 TPS tick, and returned to server-do
as effects for world mutation, packet broadcast, and persistence. Workers
timers, WebSockets, and SQLite remain outside the core.
The core reads block state through a storage-agnostic `WorldView`; server-do
adapts the cached SQLite-backed world without exposing mutation to game logic.
Movement coordinates and rotations must be finite and in vanilla's coordinate
bounds, and cumulative movement is capped per tick. A standing 0.6 × 1.8 player
AABB is swept through imported block shapes (Y, then X, then Z); collision,
invalid input, excessive movement, and void recovery send an authoritative
position packet only to the affected player. Observers see only accepted state.

Minecraft 26.2's dedicated Attack packet is also queued through `game-core`.
The core resolves targets by server entity ID, rejects self/dead/unknown,
out-of-reach, obstructed, and hurt-protected attacks, and applies the pinned
Pumpkin fist recharge curve and knockback. Only successful damage becomes
animation, damage, motion, and target-health packets.

`block-data` is the first pinned Pumpkin import. It compacts Pumpkin's generated
26.2 `blocks.json` into dependency-free runtime tables rather than linking the
full native server dependency graph. The import includes all block states,
physical/light properties, collision shapes, registry names, and item-to-default
state mappings; see `crates/block-data/UPSTREAM.md` for provenance and the
reproducible importer.

The packet IDs used by `net-ws` and the Node smoke clients are likewise
generated from the pinned Pumpkin 26.2 packet table. The importer records the
upstream revision, source hash, and every local-to-upstream name mapping in
`crates/net-ws/protocol-776-manifest.json`.

Configuration registry synchronization is also imported reproducibly from the
pinned Pumpkin table: 29 registries and 398 inline NBT entries. Generated
constants keep biome, dimension, damage-type, and world-clock wire indices tied
to upstream ordering; see `crates/server-do/REGISTRY_UPSTREAM.md`.

`worldgen-core` is the first Pumpkin terrain slice. It produces only a compact
stone/air occupancy map and has native-Pumpkin golden references that also run
under Node WebAssembly. Fresh Durable Objects use it with seed 0; existing
chunk stores without generator metadata remain superflat. See
`docs/pumpkin-worldgen-core.md` for the boundary, provenance, hashes, and local
Workers measurements.

## Run locally

Requirements: Rust with `wasm32-unknown-unknown`, Node, and Wrangler 4.125 or
newer (the current Rust WASM output needs exception-reference support).

```sh
npx -y wrangler@4.125.0 dev --port 8787
```

The worker answers plain HTTP GET `/` with an info page. Everything else is a
Minecraft connection.

### Connect a client

- **wsmc mod** for MC 26.2: add the mod to the
  client and connect to `ws://localhost:8787/` (see the mod's README for the
  address syntax).
- **Any unmodded client** through a local TCP↔WS bridge, e.g.
  [websocat](https://github.com/vi/websocat):

```sh
websocat -b --exit-on-unbuffered-close ws://127.0.0.1:8787/ tcp-listen:25565,reuseaddr
```

then point the vanilla 26.2 client at `127.0.0.1:25565`.

Offline mode: any username works; UUIDs are derived `OfflinePlayer:<name>` v3.

## Smoke tests

With `wrangler dev` running:

```sh
cd tools/protocol-smoke && npm install
node status_test.js    # handshake → status JSON → ping/pong
node login_test.js     # login → config → 289 chunks in acknowledged batches
node chunk_stream_test.js # recenter across +X/-X boundaries, unload/load edges
node play_test.js      # dig/place/chat/move/time/keepalive round-trips
node persist_test.js   # mutate, disconnect, reconnect: edit comes back from SQLite
node mp_test.js        # two players: tab info, entity spawn, movement sync
node tick_test.js      # two real clients: 20 TPS lifecycle + movement + blocks
node combat_test.js    # two players: damage, hurt protection, reach, knockback
```

All eight print PASS lines per check and exit non-zero on failure. Unit tests:
`cargo test --workspace`.

## Deploy

```sh
npx -y wrangler@4.125.0 deploy
```

Workers Paid is required (Durable Objects). Deploys restart the DO and drop
connected players (accepted for MVP).

## License

KiteCraft is licensed under GPL-3.0-only. Pumpkin-derived data retains its
upstream GPL-3.0 provenance.

## Current limitations

- Creative mode only; health is synchronized, but hunger and inventory beyond
  hotbar picks are not.
- Combat is currently bare-fist PvP: no armor, equipment attributes,
  enchantments, critical/sweeping attacks, blocking, death, or respawn flow.
- Collision currently covers the standing player shape; step-up, crouching,
  swimming, entity collision, gravity, and velocity integration are not yet
  simulated server-side.
- Chat from players is echoed as System Chat (no signed player chat).
- No mobs/redstone/daylight-affected lighting; skylight is full-brightness.
- Pumpkin terrain currently has no biomes, surface blocks, fluids/aquifers,
  caves, structures, decorations, or ores: density-positive cells are stone
  and all other cells are air.
- Chunk generation remains synchronous per chunk, but the 289-chunk view is
  nearest-first and split into client-acknowledged batches capped at eight.
- View distance fixed at 8; no respawn screen flow (void falls teleport home).
- Commands: only `/time set day|noon|night`.
