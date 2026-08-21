# wsmc framing — resolved (Phase 0 spike a)

This resolves the open item in PLAN.md ("confirm against wsmc source whether
messages include the VarInt length prefix, and where compression sits relative
to framing").

## Which wsmc

PLAN.md refers to the actively maintained mod by rikka0w0:
<https://github.com/rikka0w0/wsmc> (Fabric / Forge / NeoForge). The older
`deathcap/wsmc` proxy is a different implementation with *different* framing
(it strips the length prefix in one direction); do not confuse them.

## Answer

**Each WebSocket binary message contains exactly one Minecraft packet in full
vanilla TCP wire format, including the VarInt length prefix.**

The mod injects its `WebSocketHandler` into the Netty pipeline immediately
after the `timeout` handler (`pipeline.addAfter("timeout", ...)` in
`WebSocketClientHandler.hookPipeline` and `MixinServerConnectionListener`),
i.e. *outside* all vanilla codec handlers (frame decoder/prepender,
compression). Outbound, whatever ByteBuf vanilla's pipeline emits for one
packet write is wrapped verbatim into one `BinaryWebSocketFrame`
(`WebSocketHandler.write`). Inbound, the frame content is handed back into the
vanilla pipeline unchanged (`WebSocketHandler.channelRead`).

Therefore, per WS binary message:

- Before Set Compression:
  `[packet_length varint][packet_id varint][payload]`
- After Set Compression (threshold T):
  `[packet_length varint][data_length varint][zlib(packet_id + payload)]`
  where `data_length = 0` means the inner payload is uncompressed (used when
  uncompressed size < T); otherwise it is the uncompressed byte count and the
  remainder of the message is zlib-compressed.
- No permessage-deflate: the client offers it but the server side never
  negotiates the extension.

## Constraints worth knowing

- Default max frame payload: 65536 bytes (`wsmc.maxFramePayloadLength`,
  client-side system property). Keep chunk packets comfortably below this;
  enable server→client compression early (threshold 256).
- The client may connect to any WS endpoint path unless the server sets
  `wsmc.wsmcEndpoint`; our Worker accepts upgrades on any path.
- TLS terminates at Cloudflare edge (wss://); offline-mode encryption skipped.
- Supported MC versions (README, 2026): 26.x, 1.21.5–1.21.10, 1.20.5–1.21.4,
  1.20.2–1.20.4, 1.20.1, 1.18.2–1.20. KiteCraft targets **1.21.4
  (protocol 769)** — squarely inside the explicitly supported band.

## Unmodded clients

Unmodded clients need a local TCP↔WS bridge. Any generic bridge works since
framing is plain vanilla-over-WS; e.g. run `websocat`
(`websocat -b ws://127.0.0.1:25566/ tcp:127.0.0.1:25565`) or the deathcap-style
standalone proxy from PLAN.md.
