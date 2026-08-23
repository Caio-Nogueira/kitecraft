# Protocol notes — Minecraft Java 26.2 (protocol 776)

# Target

The currently tested target is **26.2 / protocol 776**. Packet IDs are generated
from Pumpkin revision `beb6947dfc21a1a781523bf207a3c2740f4928f9`; see
`crates/net-ws/protocol-776-manifest.json`. Run
`tools/pumpkin-import/import_protocol_ids.py <pinned-pumpkin-checkout>` to
reproduce the Rust and JavaScript tables. The embedded configuration registry
data is generated from the same pinned Pumpkin revision; see
`crates/server-do/registry-26.2-manifest.json`.

## Connection lifecycle

```
WS upgrade (any path) → DO fetch → accept_web_socket
C→S handshake      0x00 {protocolVersion varint, serverHost string, serverPort u16, nextState varint}
  nextState=1: status
    C→S status request 0x00 {}            S→C response 0x00 {json string}
    C→S ping 0x01 {i64}                   S→C ping 0x01 {i64}   then close
  nextState=2: login (offline mode)
    C→S login start 0x00 {name string, uuid [u8;16]}
    S→C set compression 0x03 {threshold varint}   (threshold 256)
    S→C login success 0x02 {uuid, name, properties[] = 0, sessionId uuid}
    C→S login acknowledged 0x03           → both switch to CONFIGURATION state
CONFIGURATION:
    S→C known packs 0x0e [{namespace,id,version}]   ("minecraft","core","26.2")
    C→S known packs 0x07                  (client echoes; may be empty)
    S→C registry data 0x07 ×29            (see below)
    S→C update tags 0x0d {17 registry tag sets}
    S→C custom payload 0x01 {"minecraft:brand", "kitecraft"}
    S→C finish configuration 0x03 {}
    C→S finish configuration 0x03 {}      → both switch to PLAY state
PLAY:
    S→C login (join game) 0x31 …          (SpawnInfo, gamemode creative)
    S→C game event 0x26 {reason=13 (start waiting for chunks), value=0}
    S→C spawn position 0x61, S→C position 0x48 (teleport id)
    S→C chunk batch start 0x0c, chunk data 0x2d ×N, chunk batch finished 0x0b
    ... gameplay ...
disconnect: save player, S→C player remove 0x45 broadcast, close WS
```

## Packet IDs used (verified with 26.2)

Handshaking S→C: — | C→S: handshake 0x00.
Status C→S: request 0x00, ping 0x01. Status S→C: response 0x00, ping 0x01.
Login C→S: start 0x00, acknowledged 0x03. Login S→C: disconnect 0x00,
success 0x02, compress 0x03.
Configuration C→S: known packs 0x07, custom payload 0x02, finish 0x03,
keep alive 0x04, pong 0x05, settings 0x00.
Configuration S→C: custom payload 0x01, disconnect 0x02, finish 0x03,
keep alive 0x04, ping 0x05, registry data 0x07, update tags 0x0d,
known packs 0x0e.

Play S→C: add entity `0x01`, animate `0x02`, block ack `0x04`, block update
`0x08`, chunk batch end/start `0x0b`/`0x0c`, custom payload `0x18`, damage
event `0x19`, disconnect `0x20`, entity position sync `0x23`, unload chunk
`0x25`, game event `0x26`, hurt animation `0x2a`, keep alive `0x2c`, chunk
with light `0x2d`, login `0x31`, relative move `0x35`, relative move+look
`0x36`, entity look `0x38`, pong `0x3e`, abilities `0x40`, player info
remove/update `0x45`/`0x46`, player position `0x48`, remove entities `0x4d`,
rotate head `0x53`, default spawn `0x61`, entity motion `0x65`, health `0x68`,
time `0x71`, system chat `0x79`, entity teleport `0x7d`.

Play C→S: teleport confirm `0x00`, attack `0x01`, chat `0x09`, settings
`0x0e`, custom payload `0x16`, keep alive `0x1c`, position `0x1e`,
position+look `0x1f`, look `0x20`, status-only movement `0x21`, ping `0x26`,
block action `0x29`, pong `0x2d`, held slot `0x35`, creative slot `0x38`, swing
`0x3f`, use item on `0x42`, use item `0x43`. Minecraft 26.1+ uses the
dedicated attack packet containing only the target entity VarInt.

The damage event uses damage-type registry index 34 (`minecraft:player_attack`).

## Configuration-phase registries

KiteCraft sends Pumpkin's complete 26.2 synchronized registry set: 29
registries and 398 entries, each with its inline unnamed-compound NBT. This
includes the 26.x variant, timeline, dialog, world-clock, test, and sulfur-cube
registries absent from the POC capture. The exact ordered registry list and
counts are recorded in `crates/server-do/registry-26.2-manifest.json`.

Entry order defines numeric wire IDs. Generated constants connect consumers to
that order: plains biome = 40, overworld dimension type = 0,
`player_attack` damage type = 34, and overworld clock = 0. Chunk biome palettes,
Join Game, Damage Event, and Time Update use those constants rather than
hand-maintained numbers.

Before finishing configuration, KiteCraft also sends Pumpkin's complete 26.2
network tag set: 17 registry categories, 1,207 tags, and 11,693 entry
references. This includes tags referenced by synchronized dimension,
enchantment, and sulfur-cube entries. The generated payload and counts are
recorded in `crates/server-do/tags-26.2-manifest.json`.

## Chunk format (protocol 776)

Chunk Data packet `0x2d`: x i32, z i32, registry-indexed heightmap map,
chunkData buffer,
block entities array (0), sky/block light masks + arrays inline.

chunkData = 24 sections (y −64…320). Every section contains non-air count i16,
fluid count i16, then its block and biome paletted containers. Packed-storage
long counts are implicit in 26.2 and are not written. Entries never span longs
and use little-endian bit order within each big-endian i64. Palette modes are
bits=0 single value, 1–8 indirect, and ≥9 direct global state IDs. Biomes
container follows with same layout (64 entries/section; single-value plains).

Heightmaps are keyed by synchronized registry IDs: WORLD_SURFACE=1,
MOTION_BLOCKING=4, and MOTION_BLOCKING_NO_LEAVES=5. Each uses 9 bits/entry and
37 i64s for 256 columns with non-spanning values.

Superflat column (KiteCraft default): y=-64 bedrock(85), y=-63/-62 dirt(10),
y=-61 grass_block[snowy=false](9); everything else air(0). Section 0 holds all
non-air blocks (count 1024, indirect palette [bedrock,dirt,grass,air] 2bpp);
sections 1..23 use explicit single-air containers. Sky light is full brightness
for section-mask bits 1..24; boundary bits 0 and 25 are marked empty.

## Block state IDs (worldgen-relevant)

air=0, stone=1, grass_block[snowy=false]=9, dirt=10, bedrock=85.
(BooleanProperty orders its values [true, false], so snowy=true=8,
snowy=false=9.) These and all other 26.2 block states now come from the compact
Pumpkin-derived `block-data` crate pinned in `crates/block-data/UPSTREAM.md`.

## Misc encodings

- Position (block pos): i64: x[25:42]26 bits, z[11:37]? — canonical layout:
  `(x & 0x3FFFFFF) << 38 | (z & 0x3FFFFFF) << 12 | (y & 0xFFF)` signed fields.
- UUID: 16 raw bytes big-endian.
- Offline UUID: UUID v3 of `OfflinePlayer:{name}` with MD5.
- Text components: NBT-wrapped JSON strings? No — 1.20.3+ text components are
  serialized as SNBT-formatted NBT compounds on the wire ("anonymousNbt" =
  unnamed compound tag). We emit `{text:"..."}` compounds.
- Chat from players is echoed via System Chat (`0x79`) — avoids signed-chat
  boilerplate entirely; legal in all versions.
