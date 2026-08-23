const WebSocket = require('ws');
const { PROTOCOL_VERSION: PINNED_PROTOCOL_VERSION, cb, sb } = require('./protocol_ids');
const SERVER_URL = process.env.KITECRAFT_URL || 'ws://127.0.0.1:8787/';
const PROTOCOL_VERSION = Number(process.env.MC_PROTOCOL || PINNED_PROTOCOL_VERSION);

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

function packet(id, ...parts) {
  const body = Buffer.concat([varint(id), ...parts]);
  return Buffer.concat([varint(body.length), body]);
}

const ws = new WebSocket(SERVER_URL);
ws.binaryType = 'nodebuffer';

let compressed = false;
const zlib = require('zlib');

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

ws.on('message', (data) => {
  let [len, off] = readVarint(data, 0);
  if (len !== data.length - off) { console.log('BAD LENGTH', len, data.length - off); }
  let inner = data.subarray(off);

  if (compressed) {
    const [dataLen, i2] = readVarint(inner, 0);
    inner = dataLen === 0 ? inner.subarray(i2) : zlib.inflateSync(inner.subarray(i2));
  }

  const [id] = readVarint(inner, 0);
  console.log('S->C id=0x' + id.toString(16), 'len=' + inner.length);

  if (id === cb.STATUS_RESPONSE && !handshakeDone2) {
    // status response
    const [slen, so] = readVarint(inner, 1);
    const json = JSON.parse(inner.subarray(so, so + slen).toString());
    console.log('STATUS:', JSON.stringify(json.version), 'online=', json.players.online);
    ws.send(packet(sb.STATUS_PING, Buffer.from([0,0,0,0,0,0,0,0]))); // ping
  } else if (id === cb.STATUS_PING && !loginPhase) {
    console.log('PONG ok');
    ws.close();
    console.log('SMOKE-PASS');
    process.exit(0);
  }
});

let handshakeDone2 = false;
let loginPhase = false;

ws.on('open', () => {
  handshakeDone2 = false;
  const hs = packet(sb.HANDSHAKE, varint(PROTOCOL_VERSION), mcString('kitecraft.test'), (() => { const p = Buffer.alloc(2); p.writeUInt16BE(25565); return p; })(), varint(1));
  ws.send(hs);
  setTimeout(() => ws.send(packet(sb.STATUS_REQUEST)), 50); // status request
});

setTimeout(() => { console.log('TIMEOUT'); process.exit(1); }, 15000);
