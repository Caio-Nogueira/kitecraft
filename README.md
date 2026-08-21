# KiteCraft

A Minecraft Java Edition server that runs entirely on Cloudflare Workers, using
a Durable Object for stateful game logic and WebSocket transport via
[wsmc](https://github.com/rikka0w0/wsmc) (one Minecraft packet per WS binary
message). Targets **MC 1.21.4 (protocol 769)**.

Implements the PLAN.md MVP milestone — *walkable world*:

- server-list ping over wsmc end-to-end
- offline-mode login → full configuration phase (vanilla registry sync) → join
- superflat worldgen streamed as real 1.18+ chunk sections (paletted
  containers + heightmaps + skylight)
- movement sync between players, chat, creative block place/break
- chunk + player persistence in DO SQLite, batched ~1 Hz flushes
- no fixed-rate tick loop: world time is derived arithmetically and a 1 Hz DO
  alarm sends Time Updates / Keep-Alives, flushes dirty chunks, and deletes
  itself when the last player leaves (hibernation-eligible idle)

## Layout

```
crates/
├── server-do/   # workers-rs entry + MinecraftServer DO (fetch/websocket_*/alarm)
│                #   protocol glue: registries, chunk wire format, sessions
│                #   src/vanilla_registries.bin — captured vanilla 1.21.4
│                #   registry sync NBT (extracted from Pumpkin's generated data)
├── net-ws/      # wsmc framing adapter + host-testable packet codecs:
│                #   VarInt, read/write buffers, zlib frame codec, NBT writer,
│                #   text components, all packet builders/parsers used
└── world/       # ChunkColumn/Section model, ChunkBacking trait,
                 #   LRU cache, superflat generator, compressed blobs
tools/
└── protocol-smoke/  # Node scripts driving raw MC-over-WS against `wrangler dev`
docs/
├── wsmc-framing.md  # Phase 0 spike: exact WS↔packet framing (resolved)
└── protocol-notes.md# packet IDs, registries, chunk format decisions
```

Note on PLAN.md's crate-reuse strategy: instead of forking Pumpkin's crates up
front, the needed portable pieces (VarInt/NBT/packet codecs, worldgen-lite) are
implemented clean-room in `net-ws`/`world` behind similar seams, so Pumpkin
crates can still be slotted in later. The one thing borrowed directly from
upstream is data: the configuration-phase registry NBT bytes are extracted from
Pumpkin's generated vanilla captures (`REGISTRY_V_1_21_4`).

## Run locally

Requirements: Rust with `wasm32-unknown-unknown`, Node, wrangler.

```sh
npm install            # (any package.json; wrangler runs via npx)
npx wrangler dev --port 8787
```

The worker answers plain HTTP GET `/` with an info page. Everything else is a
Minecraft connection.

### Connect a client

- **wsmc mod** (Fabric/Forge/NeoForge, MC 1.20.5–1.21.4): add the mod to the
  client and connect to `ws://localhost:8787/` (see the mod's README for the
  address syntax).
- **Any unmodded client** through a local TCP↔WS bridge, e.g.
  [websocat](https://github.com/vi/websocat):

```sh
websocat -b --exit-on-unbuffered-close ws://127.0.0.1:8787/ tcp-listen:25565,reuseaddr
```

then point the vanilla 1.21.4 client at `127.0.0.1:25565`.

Offline mode: any username works; UUIDs are derived `OfflinePlayer:<name>` v3.

## Smoke tests

With `wrangler dev` running:

```sh
cd tools/protocol-smoke && npm install
node status_test.js    # handshake → status JSON → ping/pong
node login_test.js     # offline login → config → join game → 289 chunks
node play_test.js      # dig/place/chat/move/time/keepalive round-trips
node persist_test.js   # reconnect: earlier edits must come back from SQLite
node mp_test.js        # two players: tab info, entity spawn, movement sync
```

All five print PASS lines per check and exit non-zero on failure. Unit tests:
`cargo test -p net-ws -p world`.

## Deploy

```sh
npx wrangler deploy
```

Workers Paid is required (Durable Objects). Deploys restart the DO and drop
connected players (accepted for MVP).

## Current limitations

- Creative mode only; no health/hunger/inventory sync beyond hotbar picks.
- Chat from players is echoed as System Chat (no signed player chat).
- No mobs/redstone/daylight-affected lighting; skylight is full-brightness.
- View distance fixed at 8; no respawn screen flow (void falls teleport home).
- Commands: only `/time set day|noon|night`.
