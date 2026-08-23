const WebSocket = require('ws');
const zlib = require('zlib');
const { PROTOCOL_VERSION: PINNED_PROTOCOL_VERSION, cb, sb } = require('./protocol_ids');
const registryManifest = require('../../crates/server-do/registry-26.2-manifest.json');
const tagManifest = require('../../crates/server-do/tags-26.2-manifest.json');
const SERVER_URL = process.env.KITECRAFT_URL || 'ws://127.0.0.1:8787/';
const PROTOCOL_VERSION = Number(process.env.MC_PROTOCOL || PINNED_PROTOCOL_VERSION);
const MINECRAFT_VERSION = process.env.MC_VERSION || '26.2';

let threshold = null;

function varint(v) {
  const out = [];
  v = v >>> 0;
  while (true) {
    if ((v & ~0x7f) === 0) { out.push(v); break; }
    out.push((v & 0x7f) | 0x80);
    v >>>= 7;
  }
  return Buffer.from(out);
}

function mcString(s) {
  const b = Buffer.from(s, 'utf8');
  return Buffer.concat([varint(b.length), b]);
}

function payload(id, ...parts) {
    return Buffer.concat([varint(id), ...parts]);
}

function f32(value) {
  const out = Buffer.alloc(4);
  out.writeFloatBE(value);
  return out;
}

function assertLoginSuccess(packet, idOffset) {
  let cursor = idOffset + 16; // profile UUID
  let length;
  [length, cursor] = readVarint(packet, cursor);
  cursor += length; // username

  let propertyCount;
  [propertyCount, cursor] = readVarint(packet, cursor);
  for (let index = 0; index < propertyCount; index++) {
    [length, cursor] = readVarint(packet, cursor);
    cursor += length; // property name
    [length, cursor] = readVarint(packet, cursor);
    cursor += length; // property value
    const signed = packet[cursor++];
    if (signed) {
      [length, cursor] = readVarint(packet, cursor);
      cursor += length;
    }
  }

  cursor += 16; // 26.2 session UUID
  if (cursor !== packet.length) {
    throw new Error(`invalid login success payload: consumed ${cursor}/${packet.length} bytes`);
  }
}

function decodeTags(packet, idOffset) {
  let cursor = idOffset;
  let registryCount;
  [registryCount, cursor] = readVarint(packet, cursor);
  const registries = [];
  const required = new Set([
    'minecraft:infiniburn_overworld',
    'minecraft:enchantable/armor',
    'minecraft:sulfur_cube_archetype/regular',
  ]);

  for (let registryIndex = 0; registryIndex < registryCount; registryIndex++) {
    let length;
    [length, cursor] = readVarint(packet, cursor);
    const id = packet.subarray(cursor, cursor + length).toString();
    cursor += length;
    let tagCount;
    [tagCount, cursor] = readVarint(packet, cursor);
    let entryReferenceCount = 0;
    for (let tagIndex = 0; tagIndex < tagCount; tagIndex++) {
      [length, cursor] = readVarint(packet, cursor);
      const name = packet.subarray(cursor, cursor + length).toString();
      cursor += length;
      required.delete(name);
      let entryCount;
      [entryCount, cursor] = readVarint(packet, cursor);
      entryReferenceCount += entryCount;
      for (let entryIndex = 0; entryIndex < entryCount; entryIndex++) {
        [, cursor] = readVarint(packet, cursor);
      }
    }
    registries.push({ id, tag_count: tagCount, entry_reference_count: entryReferenceCount });
  }

  if (cursor !== packet.length) {
    throw new Error(`invalid tags payload: consumed ${cursor}/${packet.length} bytes`);
  }
  if (required.size) {
    throw new Error(`required tags missing: ${Array.from(required).join(', ')}`);
  }
  return registries;
}

// wsmc frame: [len][data_len?][body] honoring current threshold
function frame(body) {
  if (threshold === null) return Buffer.concat([varint(body.length), body]);
  let inner;
  if (body.length < threshold) {
    inner = Buffer.concat([varint(0), body]);
  } else {
    inner = Buffer.concat([varint(body.length), zlib.deflateSync(body)]);
  }
  return Buffer.concat([varint(inner.length), inner]);
}

const ws = new WebSocket(SERVER_URL);
ws.binaryType = 'nodebuffer';

const seen = [];
const expect = (id) => seen.push(id);

ws.on('message', (data) => {
  let [, off] = readVarint(data, 0);
  let inner = data.subarray(off);
  if (threshold !== null) {
    const [dataLen, i2] = readVarint(inner, 0);
    inner = dataLen === 0 ? inner.subarray(i2) : zlib.inflateSync(inner.subarray(i2));
  }
  const [id, idOffset] = readVarint(inner, 0);
  seen.push(id);

  switch (seen.length) {
    case 1:
      console.log('login_compress id=0x' + id.toString(16));
      threshold = 256;
      // login success comes next; nothing to send yet
      break;
    case 2:
      console.log('login_success id=0x' + id.toString(16));
      assertLoginSuccess(inner, idOffset);
      ws.send(frame(payload(sb.LOGIN_ACKNOWLEDGED))); // login acknowledged
      break;
    case 3:
      console.log('cfg known_packs S->C id=0x' + id.toString(16));
      // reply known packs
      ws.send(frame(payload(sb.CFG_KNOWN_PACKS, varint(1), mcString('minecraft'), mcString('core'), mcString(MINECRAFT_VERSION))));
      break;
    default: {
      if (id === cb.CFG_REGISTRY_DATA && !playStarted) {
        let [, cursor] = readVarint(inner, 0);
        let stringLength;
        [stringLength, cursor] = readVarint(inner, cursor);
        const registryId = inner.subarray(cursor, cursor + stringLength).toString();
        cursor += stringLength;
        const [entryCount] = readVarint(inner, cursor);
        receivedRegistries.push({ id: registryId, entry_count: entryCount });
      } else if (id === cb.CFG_CUSTOM_PAYLOAD && !playStarted) {
        let cursor;
        let channelLength;
        [, cursor] = readVarint(inner, 0);
        [channelLength, cursor] = readVarint(inner, cursor);
        const channel = inner.subarray(cursor, cursor + channelLength).toString();
        if (channel !== 'minecraft:brand') {
          throw new Error(`unexpected configuration custom payload: ${channel}`);
        }
      } else if (id === cb.CFG_TAGS && !playStarted) {
        receivedTags = decodeTags(inner, idOffset);
      } else if (id === cb.CFG_FINISH && !playStarted) {
        console.log('cfg finish received after', seen.length - 4, 'packets; registries+extras done');
        registriesMatch = receivedRegistries.length === registryManifest.registries.length
          && receivedRegistries.every((registry, index) =>
            registry.id === registryManifest.registries[index].id
            && registry.entry_count === registryManifest.registries[index].entry_count
          );
        console.log(registriesMatch ? 'REGISTRY-SYNC-PASS' : 'REGISTRY-SYNC-MISMATCH');
        if (!registriesMatch) console.log('received registries:', JSON.stringify(receivedRegistries));
        tagsMatch = receivedTags.length === tagManifest.registries.length
          && receivedTags.every((registry, index) =>
            registry.id === tagManifest.registries[index].id
            && registry.tag_count === tagManifest.registries[index].tag_count
            && registry.entry_reference_count === tagManifest.registries[index].entry_reference_count
          );
        console.log(tagsMatch ? 'TAG-SYNC-PASS' : 'TAG-SYNC-MISMATCH');
        if (!tagsMatch) console.log('received tags:', JSON.stringify(receivedTags));
        playStarted = true;
        ws.send(frame(payload(sb.CFG_FINISH))); // finish ack -> play state
      } else if (playStarted) {
        handlePlay(id, inner);
      } else {
        throw new Error(`unexpected configuration packet id 0x${id.toString(16)}`);
      }
    }
  }
});

let playStarted = false;
let chunkCount = 0;
let gotLogin = false;
let gotPosition = false;
let gotChunkCenter = false;
let registriesMatch = false;
let tagsMatch = false;
const receivedRegistries = [];
let receivedTags = [];

function handlePlay(id, inner) {
  switch (id) {
    case cb.PLAY_LOGIN:
      gotLogin = true;
      console.log('PLAY login (join game)');
      break;
    case cb.PLAY_GAME_EVENT:
      console.log('PLAY game_event reason=' + inner[1]);
      break;
    case cb.PLAY_POSITION:
      gotPosition = true;
      console.log('PLAY position teleport');
      break;
    case cb.PLAY_SET_CHUNK_CACHE_CENTER: {
      let cursor = 1;
      let chunkX;
      let chunkZ;
      [chunkX, cursor] = readVarint(inner, cursor);
      [chunkZ] = readVarint(inner, cursor);
      gotChunkCenter = (chunkX | 0) === 0 && (chunkZ | 0) === 0;
      console.log(`PLAY chunk cache center x=${chunkX | 0} z=${chunkZ | 0}`);
      break;
    }
    case cb.PLAY_CHUNK_BATCH_START:
      console.log('PLAY chunk batch start');
      break;
    case cb.PLAY_CHUNK_DATA:
      chunkCount++;
      break;
    case cb.PLAY_CHUNK_BATCH_FINISHED: {
      const [, o] = readVarint(inner, 0);
      const n = readVarint(inner, o)[0];
      console.log('PLAY chunk batch finished n=' + n + ' chunks_received=' + chunkCount);
      ws.send(frame(payload(sb.PLAY_CHUNK_BATCH_RECEIVED, f32(8))));
      if (chunkCount >= 289) {
        const passed = gotLogin && gotPosition && gotChunkCenter && registriesMatch && tagsMatch;
        console.log(passed ? 'FULL-LOGIN-PASS' : 'FULL-LOGIN-INCOMPLETE');
        process.exit(passed ? 0 : 1);
      }
      break;
    }
    default:
      break;
  }
}

function readVarint(buf, off) {
  let num = 0, shift = 0, i = off;
  while (true) {
    const b = buf[i++];
    num |= (b & 0x7f) << shift;
    if (!(b & 0x80)) break;
    shift += 7;
  }
  return [num >>> 0, i];
}

ws.on('open', () => {
  const hs = payload(sb.HANDSHAKE, varint(PROTOCOL_VERSION), mcString('kitecraft.test'),
    (() => { const p = Buffer.alloc(2); p.writeUInt16BE(25565); return p; })(), varint(2));
  ws.send(Buffer.concat([varint(hs.length), hs]));
  setTimeout(() => {
    // login start: name "Tester", uuid zeros
    const ls = payload(sb.LOGIN_START, mcString('Tester'), Buffer.alloc(16));
    ws.send(Buffer.concat([varint(ls.length), ls]));
  }, 50);
});

setTimeout(() => { console.log('TIMEOUT seen=', seen.map(x => x.toString(16)).join(',')); process.exit(1); }, 30000);
