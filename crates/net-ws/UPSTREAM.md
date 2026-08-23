# Protocol data provenance

KiteCraft's Minecraft 26.2 packet IDs are imported from Pumpkin revision
`beb6947dfc21a1a781523bf207a3c2740f4928f9` (`0.1.0-dev+26.2-26.40`). The
source is Pumpkin's generated
`crates/pumpkin-data/src/generated/packet.rs` table.

From the repository root, reproduce both consumers and the audit manifest with:

```sh
python3 tools/pumpkin-import/import_protocol_ids.py /path/to/pinned/Pumpkin
```

The importer refuses any other Git revision and writes:

- `crates/net-ws/src/ids.rs` for the Rust server;
- `tools/protocol-smoke/protocol_ids.js` for the wire-level clients;
- `crates/net-ws/protocol-776-manifest.json` with source hash and mappings.

Packet layouts remain small, local codecs in `src/packets.rs`; their protocol
776 forms are compared against the matching Pumpkin packet implementations and
covered by Rust plus live Worker smoke tests.
