const WebSocket = require('ws');
const zlib = require('zlib');

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

const ws = new WebSocket('ws://127.0.0.1:8787/');
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
  const [id] = readVarint(inner, 0);
  seen.push(id);

  switch (seen.length) {
    case 1:
      console.log('login_compress id=0x' + id.toString(16));
      threshold = 256;
      // login success comes next; nothing to send yet
      break;
    case 2:
      console.log('login_success id=0x' + id.toString(16));
      ws.send(frame(payload(0x03))); // login acknowledged
      break;
    case 3:
      console.log('cfg known_packs S->C id=0x' + id.toString(16));
      // reply known packs
      ws.send(frame(payload(0x07, varint(1), mcString('minecraft'), mcString('core'), mcString('1.21.4'))));
      break;
    default: {
      // registry data flood, then brand(0x01 cfg custom payload) and finish(0x03)
      if (id === 0x03 && !playStarted) {
        console.log('cfg finish received after', seen.length - 4, 'packets; registries+extras done');
        playStarted = true;
        ws.send(frame(payload(0x03))); // finish ack -> play state
      } else if (playStarted) {
        handlePlay(id, inner);
      }
    }
  }
});

let playStarted = false;
let chunkCount = 0;
let gotLogin = false;
let gotPosition = false;

function handlePlay(id, inner) {
  switch (id) {
    case 0x2c:
      gotLogin = true;
      console.log('PLAY login (join game)');
      break;
    case 0x23:
      console.log('PLAY game_event reason=' + inner[1]);
      break;
    case 0x42:
      gotPosition = true;
      console.log('PLAY position teleport');
      break;
    case 0x0d:
      console.log('PLAY chunk batch start');
      break;
    case 0x28:
      chunkCount++;
      break;
    case 0x0c: {
      const [, o] = readVarint(inner, 0);
      const n = readVarint(inner, o)[0];
      console.log('PLAY chunk batch finished n=' + n + ' chunks_received=' + chunkCount);
      console.log(gotLogin && gotPosition ? 'FULL-LOGIN-PASS' : 'FULL-LOGIN-INCOMPLETE');
      process.exit((gotLogin && gotPosition && chunkCount > 0) ? 0 : 1);
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
  const hs = payload(0x00, varint(769), mcString('kitecraft.test'),
    (() => { const p = Buffer.alloc(2); p.writeUInt16BE(25565); return p; })(), varint(2));
  ws.send(Buffer.concat([varint(hs.length), hs]));
  setTimeout(() => {
    // login start: name "Tester", uuid zeros
    const ls = payload(0x00, mcString('Tester'), Buffer.alloc(16));
    ws.send(Buffer.concat([varint(ls.length), ls]));
  }, 50);
});

setTimeout(() => { console.log('TIMEOUT seen=', seen.map(x => x.toString(16)).join(',')); process.exit(1); }, 30000);
