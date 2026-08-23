#!/usr/bin/env python3
"""Import the protocol 776 packet IDs used by KiteCraft from pinned Pumpkin."""

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path


PUMPKIN_REVISION = "beb6947dfc21a1a781523bf207a3c2740f4928f9"
PUMPKIN_VERSION = "0.1.0-dev+26.2-26.40"
PROTOCOL_VERSION = 776

# Local name, Pumpkin direction/state/name.
CLIENTBOUND = [
    ("STATUS_RESPONSE", "status", "STATUS_RESPONSE"),
    ("STATUS_PING", "status", "PONG_RESPONSE"),
    ("LOGIN_DISCONNECT", "login", "LOGIN_DISCONNECT"),
    ("LOGIN_SUCCESS", "login", "LOGIN_FINISHED"),
    ("LOGIN_COMPRESS", "login", "LOGIN_COMPRESSION"),
    ("CFG_CUSTOM_PAYLOAD", "config", "CUSTOM_PAYLOAD"),
    ("CFG_DISCONNECT", "config", "DISCONNECT"),
    ("CFG_FINISH", "config", "FINISH_CONFIGURATION"),
    ("CFG_KEEP_ALIVE", "config", "KEEP_ALIVE"),
    ("CFG_PING", "config", "PING"),
    ("CFG_REGISTRY_DATA", "config", "REGISTRY_DATA"),
    ("CFG_TAGS", "config", "UPDATE_TAGS"),
    ("CFG_KNOWN_PACKS", "config", "SELECT_KNOWN_PACKS"),
    ("PLAY_SPAWN_ENTITY", "play", "ADD_ENTITY"),
    ("PLAY_ANIMATION", "play", "ANIMATE"),
    ("PLAY_ACK_DIGGING", "play", "BLOCK_CHANGED_ACK"),
    ("PLAY_BLOCK_CHANGE", "play", "BLOCK_UPDATE"),
    ("PLAY_CHUNK_BATCH_FINISHED", "play", "CHUNK_BATCH_FINISHED"),
    ("PLAY_CHUNK_BATCH_START", "play", "CHUNK_BATCH_START"),
    ("PLAY_CUSTOM_PAYLOAD", "play", "CUSTOM_PAYLOAD"),
    ("PLAY_DAMAGE_EVENT", "play", "DAMAGE_EVENT"),
    ("PLAY_KICK_DISCONNECT", "play", "DISCONNECT"),
    ("PLAY_SYNC_ENTITY_POSITION", "play", "ENTITY_POSITION_SYNC"),
    ("PLAY_UNLOAD_CHUNK", "play", "FORGET_LEVEL_CHUNK"),
    ("PLAY_GAME_EVENT", "play", "GAME_EVENT"),
    ("PLAY_HURT_ANIMATION", "play", "HURT_ANIMATION"),
    ("PLAY_KEEP_ALIVE", "play", "KEEP_ALIVE"),
    ("PLAY_CHUNK_DATA", "play", "LEVEL_CHUNK_WITH_LIGHT"),
    ("PLAY_LOGIN", "play", "LOGIN"),
    ("PLAY_REL_ENTITY_MOVE", "play", "MOVE_ENTITY_POS"),
    ("PLAY_ENTITY_MOVE_LOOK", "play", "MOVE_ENTITY_POS_ROT"),
    ("PLAY_ENTITY_LOOK", "play", "MOVE_ENTITY_ROT"),
    ("PLAY_PING_RESPONSE", "play", "PONG_RESPONSE"),
    ("PLAY_ABILITIES", "play", "PLAYER_ABILITIES"),
    ("PLAY_PLAYER_REMOVE", "play", "PLAYER_INFO_REMOVE"),
    ("PLAY_PLAYER_INFO", "play", "PLAYER_INFO_UPDATE"),
    ("PLAY_POSITION", "play", "PLAYER_POSITION"),
    ("PLAY_ENTITY_DESTROY", "play", "REMOVE_ENTITIES"),
    ("PLAY_ENTITY_HEAD_ROTATION", "play", "ROTATE_HEAD"),
    ("PLAY_SPAWN_POSITION", "play", "SET_DEFAULT_SPAWN_POSITION"),
    ("PLAY_SET_ENTITY_MOTION", "play", "SET_ENTITY_MOTION"),
    ("PLAY_SET_HEALTH", "play", "SET_HEALTH"),
    ("PLAY_SET_CHUNK_CACHE_CENTER", "play", "SET_CHUNK_CACHE_CENTER"),
    ("PLAY_UPDATE_TIME", "play", "SET_TIME"),
    ("PLAY_SYSTEM_CHAT", "play", "SYSTEM_CHAT"),
    ("PLAY_ENTITY_TELEPORT", "play", "TELEPORT_ENTITY"),
]

SERVERBOUND = [
    ("HANDSHAKE", "handshake", "INTENTION"),
    ("STATUS_REQUEST", "status", "STATUS_REQUEST"),
    ("STATUS_PING", "status", "PING_REQUEST"),
    ("LOGIN_START", "login", "HELLO"),
    ("LOGIN_ACKNOWLEDGED", "login", "LOGIN_ACKNOWLEDGED"),
    ("CFG_SETTINGS", "config", "CLIENT_INFORMATION"),
    ("CFG_CUSTOM_PAYLOAD", "config", "CUSTOM_PAYLOAD"),
    ("CFG_FINISH", "config", "FINISH_CONFIGURATION"),
    ("CFG_KEEP_ALIVE", "config", "KEEP_ALIVE"),
    ("CFG_PONG", "config", "PONG"),
    ("CFG_KNOWN_PACKS", "config", "SELECT_KNOWN_PACKS"),
    ("PLAY_TELEPORT_CONFIRM", "play", "ACCEPT_TELEPORTATION"),
    ("PLAY_ATTACK", "play", "ATTACK"),
    ("PLAY_CHAT_MESSAGE", "play", "CHAT"),
    ("PLAY_CHUNK_BATCH_RECEIVED", "play", "CHUNK_BATCH_RECEIVED"),
    ("PLAY_SETTINGS", "play", "CLIENT_INFORMATION"),
    ("PLAY_CUSTOM_PAYLOAD", "play", "CUSTOM_PAYLOAD"),
    ("PLAY_KEEP_ALIVE", "play", "KEEP_ALIVE"),
    ("PLAY_POSITION", "play", "MOVE_PLAYER_POS"),
    ("PLAY_POSITION_LOOK", "play", "MOVE_PLAYER_POS_ROT"),
    ("PLAY_LOOK", "play", "MOVE_PLAYER_ROT"),
    ("PLAY_FLYING", "play", "MOVE_PLAYER_STATUS_ONLY"),
    ("PLAY_PING_REQUEST", "play", "PING_REQUEST"),
    ("PLAY_BLOCK_DIG", "play", "PLAYER_ACTION"),
    ("PLAY_PONG", "play", "PONG"),
    ("PLAY_HELD_ITEM_SLOT", "play", "SET_CARRIED_ITEM"),
    ("PLAY_SET_CREATIVE_SLOT", "play", "SET_CREATIVE_MODE_SLOT"),
    ("PLAY_ARM_ANIMATION", "play", "SWING"),
    ("PLAY_BLOCK_PLACE", "play", "USE_ITEM_ON"),
    ("PLAY_USE_ITEM", "play", "USE_ITEM"),
]


def checked_revision(pumpkin: Path) -> str:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=pumpkin, check=True, capture_output=True, text=True
    ).stdout.strip()
    if revision != PUMPKIN_REVISION:
        raise SystemExit(
            f"expected Pumpkin {PUMPKIN_REVISION}, found {revision}; refusing an unpinned import"
        )
    return revision


def packet_values(source: str) -> dict[tuple[str, str, str], int]:
    values: dict[tuple[str, str, str], int] = {}
    direction = None
    state = None
    lines = source.splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        module = re.match(r"\s*pub mod ([a-z_]+) \{", line)
        if module:
            name = module.group(1)
            if name in {"serverbound", "clientbound"}:
                direction = name
                state = None
            elif direction and name in {"handshake", "status", "login", "config", "play"}:
                state = name

        packet = re.match(
            r"\s*pub const ([A-Z0-9_]+): super::super::PacketId = super::super::PacketId \{",
            line,
        )
        if packet and direction and state:
            name = packet.group(1)
            while index < len(lines):
                version = re.search(r"v26_2: (-?\d+)i32", lines[index])
                if version:
                    values[(direction, state, name)] = int(version.group(1))
                    break
                index += 1
        index += 1
    return values


def resolve(
    values: dict[tuple[str, str, str], int], direction: str, mappings: list[tuple[str, str, str]]
) -> list[dict[str, object]]:
    result = []
    for local, state, upstream in mappings:
        key = (direction, state, upstream)
        if key not in values:
            raise SystemExit(f"Pumpkin packet constant not found: {'.'.join(key)}")
        packet_id = values[key]
        if packet_id < 0:
            raise SystemExit(f"Pumpkin packet does not exist in 26.2: {'.'.join(key)}")
        result.append(
            {"local": local, "state": state, "upstream": upstream, "id": packet_id}
        )
    return result


def rust_module(name: str, packets: list[dict[str, object]]) -> str:
    groups: list[list[dict[str, object]]] = []
    for packet in packets:
        if not groups or groups[-1][-1]["state"] != packet["state"]:
            groups.append([])
        groups[-1].append(packet)
    lines = [f"pub mod {name} {{"]
    for group_index, group in enumerate(groups):
        if group_index:
            lines.append("")
        for packet in group:
            lines.append(
                f"    pub const {packet['local']}: i32 = 0x{packet['id']:02x};"
                f" // {packet['state']}.{packet['upstream']}"
            )
    lines.append("}")
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("pumpkin", type=Path, help="path to the pinned Pumpkin checkout")
    parser.add_argument("--rust-output", type=Path, default=Path("crates/net-ws/src/ids.rs"))
    parser.add_argument(
        "--js-output", type=Path, default=Path("tools/protocol-smoke/protocol_ids.js")
    )
    parser.add_argument(
        "--manifest-output",
        type=Path,
        default=Path("crates/net-ws/protocol-776-manifest.json"),
    )
    args = parser.parse_args()

    pumpkin = args.pumpkin.resolve()
    revision = checked_revision(pumpkin)
    source_path = pumpkin / "crates" / "pumpkin-data" / "src" / "generated" / "packet.rs"
    source_bytes = source_path.read_bytes()
    values = packet_values(source_bytes.decode())
    clientbound = resolve(values, "clientbound", CLIENTBOUND)
    serverbound = resolve(values, "serverbound", SERVERBOUND)

    header = (
        "// Generated by tools/pumpkin-import/import_protocol_ids.py. Do not edit.\n"
        f"// Pumpkin {revision}, Minecraft 26.2, protocol {PROTOCOL_VERSION}.\n\n"
    )
    args.rust_output.parent.mkdir(parents=True, exist_ok=True)
    args.rust_output.write_text(
        header + rust_module("cb", clientbound) + "\n\n" + rust_module("sb", serverbound) + "\n"
    )

    js_payload = {
        "PROTOCOL_VERSION": PROTOCOL_VERSION,
        "cb": {packet["local"]: packet["id"] for packet in clientbound},
        "sb": {packet["local"]: packet["id"] for packet in serverbound},
    }
    args.js_output.parent.mkdir(parents=True, exist_ok=True)
    args.js_output.write_text(
        "// Generated by tools/pumpkin-import/import_protocol_ids.py. Do not edit.\n"
        f"// Pumpkin {revision}, Minecraft 26.2.\n"
        f"module.exports = Object.freeze({json.dumps(js_payload, indent=2)});\n"
    )

    manifest = {
        "format": 1,
        "minecraft_version": "26.2",
        "protocol_version": PROTOCOL_VERSION,
        "pumpkin_revision": revision,
        "pumpkin_version": PUMPKIN_VERSION,
        "source": "crates/pumpkin-data/src/generated/packet.rs",
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "clientbound": clientbound,
        "serverbound": serverbound,
    }
    args.manifest_output.parent.mkdir(parents=True, exist_ok=True)
    args.manifest_output.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
