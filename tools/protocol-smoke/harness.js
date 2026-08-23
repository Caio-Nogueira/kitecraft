const { EventEmitter } = require('events');
const WebSocket = require('ws');
const zlib = require('zlib');
const { PROTOCOL_VERSION: PINNED_PROTOCOL_VERSION, cb, sb } = require('./protocol_ids');
const tagManifest = require('../../crates/server-do/tags-26.2-manifest.json');

const PROTOCOL_VERSION = Number(process.env.MC_PROTOCOL || PINNED_PROTOCOL_VERSION);
const MINECRAFT_VERSION = process.env.MC_VERSION || '26.2';
const SERVER_URL = process.env.KITECRAFT_URL || 'ws://127.0.0.1:8787/';

function varint(value) {
  const out = [];
  let v = value >>> 0;
  while (true) {
    if ((v & ~0x7f) === 0) {
      out.push(v);
      return Buffer.from(out);
    }
    out.push((v & 0x7f) | 0x80);
    v >>>= 7;
  }
}

function readVarint(buffer, offset = 0) {
  let value = 0;
  let shift = 0;
  let cursor = offset;
  while (cursor < buffer.length) {
    const byte = buffer[cursor++];
    value |= (byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) return [value >>> 0, cursor];
    shift += 7;
    if (shift > 35) throw new Error('VarInt is too long');
  }
  throw new Error('truncated VarInt');
}

function mcString(value) {
  const bytes = Buffer.from(value, 'utf8');
  return Buffer.concat([varint(bytes.length), bytes]);
}

function payload(id, ...parts) {
  return Buffer.concat([varint(id), ...parts]);
}

function f64(value) {
  const out = Buffer.alloc(8);
  out.writeDoubleBE(value);
  return out;
}

function f32(value) {
  const out = Buffer.alloc(4);
  out.writeFloatBE(value);
  return out;
}

function i16(value) {
  const out = Buffer.alloc(2);
  out.writeInt16BE(value);
  return out;
}

function i64(value) {
  const out = Buffer.alloc(8);
  out.writeBigInt64BE(BigInt.asIntN(64, BigInt(value)));
  return out;
}

function packPosition(x, y, z) {
  const packed = ((BigInt(x) & 0x3ffffffn) << 38n)
    | ((BigInt(z) & 0x3ffffffn) << 12n)
    | (BigInt(y) & 0xfffn);
  const out = Buffer.alloc(8);
  out.writeBigInt64BE(BigInt.asIntN(64, packed));
  return out;
}

function decodeBlockChange(packet) {
  const packed = packet.body.readBigInt64BE(packet.idOffset);
  return {
    x: Number(BigInt.asIntN(26, packed >> 38n)),
    z: Number(BigInt.asIntN(26, (packed << 26n) >> 38n)),
    y: Number(BigInt.asIntN(12, (packed << 52n) >> 52n)),
    state: readVarint(packet.body, packet.idOffset + 8)[0],
  };
}

function decodePositionCorrection(packet) {
  const [teleportId, cursor] = readVarint(packet.body, packet.idOffset);
  return {
    teleportId,
    x: packet.body.readDoubleBE(cursor),
    y: packet.body.readDoubleBE(cursor + 8),
    z: packet.body.readDoubleBE(cursor + 16),
  };
}

function decodeSpawnEntity(packet) {
  const [entityId, cursor] = readVarint(packet.body, packet.idOffset);
  const [entityType] = readVarint(packet.body, cursor + 16);
  return { entityId, entityType };
}

function decodeHealth(packet) {
  return packet.body.readFloatBE(packet.idOffset);
}

function validateLoginSuccess(packet) {
  let cursor = packet.idOffset + 16; // profile UUID
  let length;
  [length, cursor] = readVarint(packet.body, cursor);
  cursor += length; // username

  let propertyCount;
  [propertyCount, cursor] = readVarint(packet.body, cursor);
  for (let index = 0; index < propertyCount; index++) {
    [length, cursor] = readVarint(packet.body, cursor);
    cursor += length; // property name
    [length, cursor] = readVarint(packet.body, cursor);
    cursor += length; // property value
    const signed = packet.body[cursor++];
    if (signed) {
      [length, cursor] = readVarint(packet.body, cursor);
      cursor += length;
    }
  }

  cursor += 16; // 26.2 session UUID
  if (cursor !== packet.body.length) {
    throw new Error(`invalid login success payload: consumed ${cursor}/${packet.body.length} bytes`);
  }
}

function validateTags(packet) {
  let cursor = packet.idOffset;
  let registryCount;
  [registryCount, cursor] = readVarint(packet.body, cursor);
  if (registryCount !== tagManifest.registry_count) {
    throw new Error(`expected ${tagManifest.registry_count} tag registries, received ${registryCount}`);
  }

  let tagCount = 0;
  let entryReferenceCount = 0;
  for (let registryIndex = 0; registryIndex < registryCount; registryIndex++) {
    let length;
    [length, cursor] = readVarint(packet.body, cursor);
    cursor += length;
    let registryTagCount;
    [registryTagCount, cursor] = readVarint(packet.body, cursor);
    tagCount += registryTagCount;
    for (let tagIndex = 0; tagIndex < registryTagCount; tagIndex++) {
      [length, cursor] = readVarint(packet.body, cursor);
      cursor += length;
      let entries;
      [entries, cursor] = readVarint(packet.body, cursor);
      entryReferenceCount += entries;
      for (let entryIndex = 0; entryIndex < entries; entryIndex++) {
        [, cursor] = readVarint(packet.body, cursor);
      }
    }
  }

  if (cursor !== packet.body.length
      || tagCount !== tagManifest.tag_count
      || entryReferenceCount !== tagManifest.entry_reference_count) {
    throw new Error('configuration tag payload does not match the pinned 26.2 manifest');
  }
}

function packedValue(buffer, offset, bits, index) {
  const valuesPerLong = Math.floor(64 / bits);
  const wordOffset = offset + Math.floor(index / valuesPerLong) * 8;
  const word = buffer.readBigUInt64BE(wordOffset);
  const shift = BigInt((index % valuesPerLong) * bits);
  return Number((word >> shift) & ((1n << BigInt(bits)) - 1n));
}

function decodeChunkBlock(packet, blockX, blockY, blockZ) {
  let cursor = packet.idOffset;
  const chunkX = packet.body.readInt32BE(cursor); cursor += 4;
  const chunkZ = packet.body.readInt32BE(cursor); cursor += 4;
  if ((blockX >> 4) !== chunkX || (blockZ >> 4) !== chunkZ) return undefined;

  let count;
  [count, cursor] = readVarint(packet.body, cursor);
  for (let index = 0; index < count; index++) {
    [, cursor] = readVarint(packet.body, cursor);
    let length;
    [length, cursor] = readVarint(packet.body, cursor);
    cursor += length * 8;
  }

  let sectionBytes;
  [sectionBytes, cursor] = readVarint(packet.body, cursor);
  const sectionEnd = cursor + sectionBytes;
  const targetSection = Math.floor((blockY + 64) / 16);
  const localIndex = ((blockY & 15) * 256) + ((blockZ & 15) * 16) + (blockX & 15);

  for (let section = 0; section < 24 && cursor < sectionEnd; section++) {
    cursor += 4; // non-air and fluid counts
    const bits = packet.body[cursor++];
    let palette = null;
    if (bits === 0) {
      let state;
      [state, cursor] = readVarint(packet.body, cursor);
      palette = [state];
    } else if (bits <= 8) {
      let paletteLength;
      [paletteLength, cursor] = readVarint(packet.body, cursor);
      palette = [];
      for (let index = 0; index < paletteLength; index++) {
        let state;
        [state, cursor] = readVarint(packet.body, cursor);
        palette.push(state);
      }
    }

    const wordCount = bits === 0 ? 0 : Math.ceil(4096 / Math.floor(64 / bits));
    if (section === targetSection) {
      if (bits === 0) return palette[0];
      const value = packedValue(packet.body, cursor, bits, localIndex);
      return palette ? palette[value] : value;
    }
    cursor += wordCount * 8;

    const biomeBits = packet.body[cursor++];
    if (biomeBits === 0) {
      [, cursor] = readVarint(packet.body, cursor);
    } else {
      if (biomeBits <= 3) {
        let paletteLength;
        [paletteLength, cursor] = readVarint(packet.body, cursor);
        for (let index = 0; index < paletteLength; index++) [, cursor] = readVarint(packet.body, cursor);
      }
      cursor += Math.ceil(64 / Math.floor(64 / biomeBits)) * 8;
    }
  }
  throw new Error('target section is missing from chunk payload');
}

function decodeChunkCoordinates(packet) {
  return {
    x: packet.body.readInt32BE(packet.idOffset),
    z: packet.body.readInt32BE(packet.idOffset + 4),
  };
}

function decodeChunkCacheCenter(packet) {
  let cursor = packet.idOffset;
  let x;
  let z;
  [x, cursor] = readVarint(packet.body, cursor);
  [z] = readVarint(packet.body, cursor);
  return { x: x | 0, z: z | 0 };
}

function decodeUnloadChunk(packet) {
  return {
    z: packet.body.readInt32BE(packet.idOffset),
    x: packet.body.readInt32BE(packet.idOffset + 4),
  };
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

class MinecraftClient extends EventEmitter {
  constructor(name, url = SERVER_URL) {
    super();
    this.name = name;
    this.url = url;
    this.threshold = null;
    this.phase = 'login';
    this.history = [];
    this.sequence = 1;
    this.ws = null;
  }

  frame(body) {
    if (this.threshold === null) return Buffer.concat([varint(body.length), body]);
    const inner = body.length < this.threshold
      ? Buffer.concat([varint(0), body])
      : Buffer.concat([varint(body.length), zlib.deflateSync(body)]);
    return Buffer.concat([varint(inner.length), inner]);
  }

  decodeFrame(data) {
    const [, bodyOffset] = readVarint(data, 0);
    let body = data.subarray(bodyOffset);
    if (this.threshold !== null) {
      const [uncompressedLength, compressedOffset] = readVarint(body, 0);
      body = uncompressedLength === 0
        ? body.subarray(compressedOffset)
        : zlib.inflateSync(body.subarray(compressedOffset));
    }
    const [id, idOffset] = readVarint(body, 0);
    return { id, idOffset, body };
  }

  async connect(timeoutMs = 30000) {
    if (this.ws) throw new Error(`${this.name} is already connected`);

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error(`${this.name} login timed out`)), timeoutMs);
      this.ws = new WebSocket(this.url);
      this.ws.binaryType = 'nodebuffer';

      this.ws.once('error', reject);
      this.ws.once('close', () => this.emit('close'));
      this.ws.once('open', () => {
        const port = Buffer.alloc(2);
        port.writeUInt16BE(25565);
        this.ws.send(this.frame(payload(
          sb.HANDSHAKE,
          varint(PROTOCOL_VERSION),
          mcString('kitecraft.test'),
          port,
          varint(2),
        )));
        this.ws.send(this.frame(payload(sb.LOGIN_START, mcString(this.name), Buffer.alloc(16))));
      });

      this.ws.on('message', (data) => {
        try {
          const packet = this.decodeFrame(data);
          if (this.phase === 'login') {
            if (packet.id === cb.LOGIN_COMPRESS && this.threshold === null) {
              this.threshold = 256;
            } else if (packet.id === cb.LOGIN_SUCCESS) {
              validateLoginSuccess(packet);
              this.ws.send(this.frame(payload(sb.LOGIN_ACKNOWLEDGED)));
              this.phase = 'configuration';
            }
            return;
          }

          if (this.phase === 'configuration') {
            if (packet.id === cb.CFG_KNOWN_PACKS) {
              this.ws.send(this.frame(payload(
                sb.CFG_KNOWN_PACKS,
                varint(1),
                mcString('minecraft'),
                mcString('core'),
                mcString(MINECRAFT_VERSION),
              )));
            } else if (packet.id === cb.CFG_TAGS) {
              validateTags(packet);
            } else if (packet.id === cb.CFG_REGISTRY_DATA || packet.id === cb.CFG_CUSTOM_PAYLOAD) {
              // These are valid while the client waits for configuration to finish.
            } else if (packet.id === cb.CFG_FINISH) {
              this.ws.send(this.frame(payload(sb.CFG_FINISH)));
              this.phase = 'await-play';
            } else {
              throw new Error(`unexpected configuration packet id 0x${packet.id.toString(16)}`);
            }
            return;
          }

          this.history.push(packet);
          this.emit('packet', packet);

          if (packet.id === cb.PLAY_KEEP_ALIVE) {
            this.ws.send(this.frame(payload(sb.PLAY_KEEP_ALIVE, packet.body.subarray(packet.idOffset))));
          }
          if (packet.id === cb.PLAY_POSITION) {
            const correction = decodePositionCorrection(packet);
            this.ws.send(this.frame(payload(sb.PLAY_TELEPORT_CONFIRM, varint(correction.teleportId))));
          }
          if (packet.id === cb.PLAY_CHUNK_BATCH_FINISHED) {
            this.ws.send(this.frame(payload(sb.PLAY_CHUNK_BATCH_RECEIVED, f32(8))));
            if (this.phase === 'await-play') {
              this.phase = 'play';
              clearTimeout(timeout);
              resolve(this);
            }
          }
        } catch (error) {
          clearTimeout(timeout);
          reject(error);
        }
      });
    });
  }

  sendPlay(id, ...parts) {
    if (this.phase !== 'play') throw new Error(`${this.name} is not in play state`);
    this.ws.send(this.frame(payload(id, ...parts)));
  }

  move({ x, y, z, yaw = 0, pitch = 0, onGround = true }) {
    this.sendPlay(sb.PLAY_POSITION_LOOK, f64(x), f64(y), f64(z), f32(yaw), f32(pitch), Buffer.from([onGround ? 1 : 0]));
  }

  attack(entityId) {
    this.sendPlay(sb.PLAY_ATTACK, varint(entityId));
  }

  setCreativeSlot({ slot = 36, itemId, count = 1 }) {
    this.sendPlay(sb.PLAY_SET_CREATIVE_SLOT, i16(slot), varint(count), varint(itemId), varint(0), varint(0));
  }

  selectHotbar(slot = 0) {
    this.sendPlay(sb.PLAY_HELD_ITEM_SLOT, i16(slot));
  }

  placeBlock({ x, y, z, face = 1 }) {
    const sequence = this.sequence++;
    this.sendPlay(
      sb.PLAY_BLOCK_PLACE,
      varint(1),
      packPosition(x, y, z),
      varint(face),
      f32(0.5), f32(0.5), f32(0.5),
      Buffer.from([0, 0]),
      varint(sequence),
    );
    return sequence;
  }

  digBlock({ x, y, z }) {
    const sequence = this.sequence++;
    this.sendPlay(sb.PLAY_BLOCK_DIG, varint(0), packPosition(x, y, z), Buffer.from([0]), varint(sequence));
    return sequence;
  }

  waitForPacket(id, predicate = () => true, timeoutMs = 5000, fromIndex = this.history.length) {
    const existing = this.history.slice(fromIndex).find((packet) => packet.id === id && predicate(packet));
    if (existing) return Promise.resolve(existing);

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.off('packet', onPacket);
        reject(new Error(`${this.name} timed out waiting for packet 0x${id.toString(16)}`));
      }, timeoutMs);
      const onPacket = (packet) => {
        if (packet.id !== id || !predicate(packet)) return;
        clearTimeout(timeout);
        this.off('packet', onPacket);
        resolve(packet);
      };
      this.on('packet', onPacket);
    });
  }

  async close() {
    if (!this.ws || this.ws.readyState >= WebSocket.CLOSING) return;
    await new Promise((resolve) => {
      let settled = false;
      const finish = () => {
        if (settled) return;
        settled = true;
        clearTimeout(timeout);
        resolve();
      };
      const timeout = setTimeout(() => {
        this.ws.terminate();
        finish();
      }, 1000);
      this.ws.once('close', finish);
      this.ws.close();
    });
  }
}

async function tickStats(url = SERVER_URL) {
  const endpoint = new URL('/__kitecraft/tick-stats', url.replace(/^ws/, 'http'));
  const response = await fetch(endpoint);
  if (!response.ok) throw new Error(`tick stats returned HTTP ${response.status}`);
  return response.json();
}

module.exports = {
  MinecraftClient,
  PROTOCOL_VERSION,
  MINECRAFT_VERSION,
  SERVER_URL,
  cb,
  sb,
  decodeBlockChange,
  decodeChunkBlock,
  decodeChunkCacheCenter,
  decodeChunkCoordinates,
  decodeHealth,
  decodePositionCorrection,
  decodeSpawnEntity,
  decodeUnloadChunk,
  readVarint,
  sleep,
  tickStats,
};
