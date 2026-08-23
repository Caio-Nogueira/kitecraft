#!/usr/bin/env python3
"""Import compact Minecraft 26.2 block metadata from a pinned Pumpkin checkout."""

import argparse
import hashlib
import json
import struct
import subprocess
from pathlib import Path


PUMPKIN_REVISION = "beb6947dfc21a1a781523bf207a3c2740f4928f9"
PUMPKIN_VERSION = "0.1.0-dev+26.2-26.40"
INVALID_ID = 0xFFFF


def checked_revision(pumpkin: Path) -> str:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=pumpkin,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if revision != PUMPKIN_REVISION:
        raise SystemExit(
            f"expected Pumpkin {PUMPKIN_REVISION}, found {revision}; refusing an unpinned import"
        )
    return revision


def write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("pumpkin", type=Path, help="path to the pinned Pumpkin checkout")
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("crates/block-data/data"),
        help="output directory relative to the current working directory",
    )
    args = parser.parse_args()

    pumpkin = args.pumpkin.resolve()
    revision = checked_revision(pumpkin)
    source = pumpkin / "assets" / "blocks.json"
    source_bytes = source.read_bytes()
    payload = json.loads(source_bytes)
    blocks = sorted(payload["blocks"], key=lambda block: block["id"])
    states = sorted(
        ((block["id"], state) for block in blocks for state in block["states"]),
        key=lambda pair: pair[1]["id"],
    )

    if [block["id"] for block in blocks] != list(range(len(blocks))):
        raise SystemExit("Pumpkin block IDs are not dense")
    if [state["id"] for _, state in states] != list(range(len(states))):
        raise SystemExit("Pumpkin block-state IDs are not dense")

    state_records = bytearray()
    collision_refs = bytearray()
    for block_id, state in states:
        collision_start = len(collision_refs) // 2
        collision_ids = state["collision_shapes"]
        if len(collision_ids) > 0xFF:
            raise SystemExit(f"state {state['id']} has too many collision shapes")
        for shape_id in collision_ids:
            collision_refs += struct.pack("<H", shape_id)
        state_records += struct.pack(
            "<HHBBBBIf",
            block_id,
            state["state_flags"],
            state["side_flags"],
            state["luminance"],
            state.get("opacity") or 0,
            len(collision_ids),
            collision_start,
            state["hardness"],
        )

    names = bytearray()
    block_records = bytearray()
    max_item_id = max(block["item_id"] for block in blocks)
    item_states = [INVALID_ID] * (max_item_id + 1)
    for block in blocks:
        name = block["name"].encode("utf-8")
        if len(name) > 0xFF:
            raise SystemExit(f"block name is too long: {block['name']}")
        name_start = len(names)
        names += name
        shape_offset = block.get("shape_offset")
        offset_type = {None: 0, "xz": 1, "xyz": 2}[
            None if shape_offset is None else shape_offset["type"]
        ]
        block_records += struct.pack(
            "<HHIB3x5fB3x2f",
            block["default_state_id"],
            block["item_id"],
            name_start,
            len(name),
            block["hardness"],
            block["blast_resistance"],
            block["slipperiness"],
            block["velocity_multiplier"],
            block["jump_velocity_multiplier"],
            offset_type,
            0.0 if shape_offset is None else shape_offset["max_horizontal"],
            0.0 if shape_offset is None else shape_offset["max_vertical"],
        )
        if block["item_id"] and item_states[block["item_id"]] == INVALID_ID:
            item_states[block["item_id"]] = block["default_state_id"]

    shape_records = bytearray()
    for shape in payload["shapes"]:
        shape_records += struct.pack("<6d", *(shape["min"] + shape["max"]))

    output = args.output.resolve()
    write(output / "states.bin", state_records)
    write(output / "blocks.bin", block_records)
    write(output / "block_names.bin", names)
    write(output / "collision_refs.bin", collision_refs)
    write(output / "collision_shapes.bin", shape_records)
    write(
        output / "item_default_states.bin",
        b"".join(struct.pack("<H", state) for state in item_states),
    )

    manifest = {
        "format": 1,
        "minecraft_version": "26.2",
        "pumpkin_revision": revision,
        "pumpkin_version": PUMPKIN_VERSION,
        "source": "assets/blocks.json",
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "block_count": len(blocks),
        "block_state_count": len(states),
        "collision_shape_count": len(payload["shapes"]),
        "collision_reference_count": len(collision_refs) // 2,
        "item_index_count": len(item_states),
    }
    write(
        output / "manifest.json",
        (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(),
    )


if __name__ == "__main__":
    main()
