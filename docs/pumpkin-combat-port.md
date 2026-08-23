# Pumpkin combat port

The first portable PvP slice is derived from Pumpkin revision
`beb6947dfc21a1a781523bf207a3c2740f4928f9`, the same GPL-3.0 revision used by
`block-data` and targeting Minecraft 26.2.

Reference behavior:

- `crates/pumpkin-protocol/src/java/server/play/attack.rs`: Minecraft 26.2's
  dedicated Attack packet and target entity ID layout.
- `crates/pumpkin/src/entity/player.rs`: fist attack speed, recharge progress,
  damage multiplier, and successful-hit flow.
- `crates/pumpkin/src/entity/living.rs`: ten-tick hurt protection and health
  application.
- `crates/pumpkin/src/entity/combat.rs` and `entity/mod.rs`: yaw-based 0.5
  knockback and grounded vertical cap.
- Pumpkin's generated packet table and Java client packet implementations:
  damage, hurt animation, packed entity motion, and health packet IDs/layouts.

KiteCraft ports these as synchronous deterministic state transitions in
`game-core`; no Tokio task, lock, world handle, plugin event, registry singleton,
or RNG is retained. It additionally validates three-block hitbox reach and
block-shape line of sight before running Pumpkin's damage path. This is a
deliberate server-authoritative boundary that Pumpkin's current Java handler
does not itself enforce.

The current slice intentionally fixes combat to an empty-hand player: one base
damage, 1.6 attacks per second, no armor or enchantments. Equipment-derived
attributes, critical/sweeping attacks, shields, death, respawn, and server-side
velocity integration remain later slices.
