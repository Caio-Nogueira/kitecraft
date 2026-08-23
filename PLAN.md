# KiteCraft — Minecraft server on Cloudflare Workers + Durable Objects

A Minecraft Java Edition server that runs entirely on Cloudflare Workers, using
Durable Objects for stateful game logic and WebSocket transport via [wsmc]
(one packet per WS binary message).

## Locked decisions

- **Strategy**: new workspace; reuse Pumpkin's portable crates from a pinned fork.
- **MVP milestone**: "walkable world" — login/ping over WS, superflat worldgen,
  block place/break, chat, movement sync, chunk persistence.
- **Auth**: offline mode (offline UUIDs; no Mojang session validation).
- **Topology**: one Durable Object = one server instance (`idFromName("primary")`).
- **Platform**: Workers Paid required (free tier's 10 ms CPU/invocation cannot
  sustain gameplay bursts; paid gives 30 s CPU per invocation by default).
- **License**: GPL-3.0-only, matching the pinned Pumpkin 26.2 upstream.

## Architecture

### Transport (wsmc)

Standard Java protocol framed by wsmc: each WS binary message = exactly one MC
packet. Clients connect with the wsmc Fabric mod, or deathcap/wsmc standalone
TCP↔WS proxy for unmodded clients. TLS comes free from Cloudflare edge (wss://),
so offline-mode encryption is skipped; zlib compression via flate2 (sync).

**Open item**: confirm against wsmc source whether messages include the VarInt
length prefix, and where compression sits relative to framing.

### Execution model — player-scoped fixed-rate tick loop

DO alarms are not a clock (no sub-second delivery guarantee, at-least-once +
2 s retry backoff, every reschedule is a durable storage write). Instead:

| Concern | Mechanism |
|---|---|
| Movement / block / attack events | validated semantic inputs queued for the 20 TPS core |
| Chat / protocol-only events | inline per `websocket_message` |
| World time / daylight | derived arithmetically: `age = base + (now − base_wallclock)/50` |
| Time Update packet | vanilla expects every 20 ticks = 1 s → 1 Hz alarm |
| Keep Alive (~10–15 s) | same alarm |
| Chunk autosave | dirty-set batch flush in same alarm |

`alarm()` @ ~1 Hz: send Time Updates, keep-alives to quiet clients, flush dirty
chunks, LRU sweep, and if zero sockets remain → final flush → `deleteAlarm()` →
full hibernation eligibility (zero cost while idle-connected).

Gameplay runs through a deterministic input → tick → effect core. A 50 ms timer
is armed while players exist, with bounded catch-up after late callbacks. The
loop stops after the last player leaves; alarms remain responsible for periodic
maintenance and durability. Player positions are finite/bounds/delta checked,
swept through Pumpkin-derived block collision shapes, and corrected only to the
actor when the submitted state is invalid or intersects the world.
Attacks use the same ordered input stream: target identity, hitbox reach, world
occlusion, recharge, hurt protection, health, and knockback are deterministic
core state; server-do only translates successful effects to protocol packets.

### Session lifecycle

WS upgrade → Worker fetch handler → forwarded to Server DO → hibernation API
(`accept_web_socket` + tags). Handshaking → status ping OR offline login
(set-compression → login success → play packets). Disconnect → save player data,
despawn, broadcast; last disconnect triggers idle shutdown above.

### Storage (DO SQLite KV, ≤2 MB/item, 10 GB/DO)

- `chunk:{x}:{z}` → compressed chunk column bytes
- `player:{uuid}` → position/inventory/state
- `meta:*` → seed, spawn, world-time base, weather

Writes batched at ~1 Hz + forced flush before hibernation. In-memory chunk LRU
capped ~64 MB (128 MB isolate limit).

## Crate reuse

| Upstream crate | Disposition |
|---|---|
| pumpkin-data | compact pinned generated tables; exclude native-only dependency graph |
| pumpkin-nbt / util | import incrementally behind WASM-safe feature gates when needed |
| pumpkin-protocol | packet defs + ser/de only; skip AsyncRead/Write codec layer |
| pumpkin-config | struct defs; load from embedded defaults/env/KV |
| pumpkin-world | First density-only slice lifted into WASM-safe `worldgen-core`; later systems remain incremental |
| rest of Pumpkin | not carried over (bedrock, plugins, query/rcon/console) |

## Workspace layout

```
kitecraft/
├── crates/
│   ├── block-data/    # compact pinned Pumpkin block/state/item/shape tables
│   ├── game-core/     # deterministic inputs, ticks, collision, and effects
│   ├── server-do/     # workers-rs entry + MinecraftServer DO (fetch/websocket_*/alarm)
│   ├── net-ws/        # wsmc framing adapter: WS msg ↔ packet Bytes, sink/source traits
│   ├── world/         # generator/backing traits, DO-SQLite impl, LRU, superflat fallback
│   └── worldgen-core/ # pinned Pumpkin seed/noise/density, stone-air occupancy
└── tools/pumpkin-import/ # reproducible import from the pinned Pumpkin revision
```

Target: wasm32-unknown-unknown via workers-rs (`#[durable_object]`, hibernation
API merged upstream).

## Phases

0. **Spikes**: (a) document wsmc exact framing; (b) hello-world workers-rs deploy:
   DO + hibernating binary WS echo + 1 Hz alarm counter.
1. **Transport**: fork/vendor crates compiling on wasm; framing adapter;
   server-list ping through wsmc end-to-end.
2. **Walkable world (MVP)**: offline login → join → superflat chunks at view
   distance → movement sync → chat → persisted block break/place.
3. **Robustness**: idle shutdown/hibernate, autosave batching, LRU eviction,
   multi-player soak test under `wrangler dev`.
4. **Stretch**: Pumpkin density-only worldgen landed; add surface rules, biomes,
   caves, and later stages one golden-tested system at a time. Player persistence
   across restarts, HTTP admin endpoint, real-time simulation loop if entities land.

## Risks / constraints

- Deploys kick all players (DO restart drops WebSockets) — accepted for MVP.
- Script size limits (~10–15 MB gzipped): pumpkin-data registries are large;
  measure early, trim via codegen features.
- workers-rs rough edges may need raw worker-sys/wasm-bindgen calls.
- Unmodded clients require the deathcap-style local proxy; wsmc mod is primary.
- No inbound TCP/UDP on Workers → Bedrock permanently out of scope here.
