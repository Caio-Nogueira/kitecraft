# Protocol notes — Minecraft Java 1.21.4 (protocol 769)

Target locked: **1.21.4 / protocol 769**. Sources of truth used during
implementation: PrismarineJS/minecraft-data `data/pc/1.21.4/protocol.json` +
`blocks.json`, minecraft.wiki "Java Edition protocol/Registries",
misode/mcmeta tag `1.21.4-registries` and `1.21.4-data`.

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
    S→C login success 0x02 {uuid, name, properties[] = 0}
    C→S login acknowledged 0x03           → both switch to CONFIGURATION state
CONFIGURATION:
    S→C known packs 0x0e [{namespace,id,version}]   ("minecraft","core","1.21.4")
    C→S known packs 0x07                  (client echoes; may be empty)
    S→C registry data 0x07 ×13            (see below)
    S→C custom payload 0x01 {"minecraft:brand", "kitecraft"}
    S→C finish configuration 0x03 {}
    C→S finish configuration 0x03 {}      → both switch to PLAY state
PLAY:
    S→C login (join game) 0x2c …          (SpawnInfo, gamemode creative)
    S→C game event 0x23 {reason=13 (start waiting for chunks), value=0}
    S→C spawn position 0x5b, S→C position 0x42 (teleport id)
    S→C chunk batch start 0x0d, chunk data 0x28 ×N, chunk batch finished 0x0c
    ... gameplay ...
disconnect: save player, S→C player remove 0x3f broadcast, close WS
```

## Packet IDs used (1.21.4)

Handshaking S→C: — | C→S: handshake 0x00.
Status C→S: request 0x00, ping 0x01. Status S→C: response 0x00, ping 0x01.
Login C→S: start 0x00, acknowledged 0x03. Login S→C: disconnect 0x00,
success 0x02, compress 0x03.
Configuration C→S: known packs 0x07, custom payload 0x02, finish 0x03,
keep alive 0x04, pong 0x05, settings 0x00.
Configuration S→C: custom payload 0x01, disconnect 0x02, finish 0x03,
keep alive 0x04, ping 0x05, registry data 0x07, known packs 0x0e.

Play S→C:
bundle_delimiter 0x00, animation 0x03, ack_player_digging 0x05,
block_change 0x09, chunk_batch_finished 0x0c, chunk_batch_start 0x0d,
sync_entity_position 0x20, unload_chunk 0x22 (z i32 THEN x i32),
game_event 0x23, keep_alive 0x27, chunk_data 0x28, update_light 0x2b,
login 0x2c, rel_entity_move 0x2f, entity_move_look 0x30, entity_look 0x32,
player_remove 0x3f, player_info 0x40, position 0x42, entity_head_rotation 0x4d,
spawn_position 0x5b, abilities 0x3a, update_time 0x6b, system_chat 0x73,
entity_teleport 0x77, kick_disconnect 0x1d, custom_payload 0x19.

Play C→S:
teleport_confirm 0x00, chat_message 0x07, settings 0x0c,
configuration_acknowledged 0x0e, custom_payload 0x14, keep_alive 0x1a,
position 0x1c, position_look 0x1d, look 0x1e, flying 0x1f,
pong 0x2b, block_dig 0x27, arm_animation 0x3a, block_place 0x3c,
use_item 0x3d, ping_request 0x24 (reply play ping_response 0x38).

## Configuration-phase registries

Known-packs negotiation lets us send entry *names* without NBT; the vanilla
client sources NBT from its built-in `minecraft:core` pack. Empirically
(tachyne-world PROTOCOL.md, 1.21.5): dimension_type / worldgen/biome /
damage_type must carry inline NBT even when the pack matches; Update Tags can
be skipped. KiteCraft sends:

- inline NBT: `minecraft:dimension_type` (single custom flat entry
  `kitecraft:flat`, overworld-like), `minecraft:worldgen/biome`
  (`minecraft:plains` only), `minecraft:damage_type` (all 49 vanilla entries).
- names only: banner_pattern(43), cat_variant(11), chat_type(7),
  enchantment(42), instrument(8), jukebox_song(19), painting_variant(50),
  trim_material(11), trim_pattern(18), wolf_variant(9).

Entry order within a packet defines numeric IDs; biome index 0 = plains is
assumed by chunk biomes palettes. Not present in 1.21.4: cow_variant,
wolf_sound_variant.

## Chunk format (1.18+, unchanged in 1.21.4)

Chunk Data packet 0x28: x i32, z i32, heightmaps NBT, chunkData buffer,
block entities array (0), sky/block light masks + arrays inline.

chunkData = 24 sections (y −64…320). Per section: non-air count i16; if 0,
section ends. Else paletted container: bits-per-entry u8, palette, data array
(varint long count + packed i64s, entries never span longs, little-endian bit
order within each long). Palette modes: bits=0 single value (varint);
1–8 indirect (varint count + varints); ≥9 direct (no palette, raw global
state ids; 15 bits for blocks in 1.21.4? we never emit direct). Biomes
container follows with same layout (64 entries/section; single-value plains).

Heightmaps NBT: compound of long-arrays; MOTION_BLOCKING + WORLD_SURFACE,
9 bits/entry, 37 columns per long array (256 entries * 9 bits = 36 longs).

Superflat column (KiteCraft default): y=-64 bedrock(85), y=-63/-62 dirt(10),
y=-61 grass_block[snowy=false](9); everything else air(0). Section 0 holds all
non-air blocks (count 1024, indirect palette [bedrock,dirt,grass,air] 2bpp);
sections 1..23 are empty (count 0). Sky light sent as full-brightness 0xFF
arrays for all sections.

## Block state IDs (worldgen-relevant)

air=0, stone=1, grass_block[snowy=false]=9, dirt=10, bedrock=85.
(BooleanProperty orders its values [true, false], so snowy=true=8,
snowy=false=9 — matches prismarine defaultState.)

## Misc encodings

- Position (block pos): i64: x[25:42]26 bits, z[11:37]? — canonical layout:
  `(x & 0x3FFFFFF) << 38 | (z & 0x3FFFFFF) << 12 | (y & 0xFFF)` signed fields.
- UUID: 16 raw bytes big-endian.
- Offline UUID: UUID v3 of `OfflinePlayer:{name}` with MD5.
- Text components: NBT-wrapped JSON strings? No — 1.20.3+ text components are
  serialized as SNBT-formatted NBT compounds on the wire ("anonymousNbt" =
  unnamed compound tag). We emit `{text:"..."}` compounds.
- Chat from players is echoed via System Chat (0x73) — avoids signed-chat
  boilerplate entirely; legal in all versions.
