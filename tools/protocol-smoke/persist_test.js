const WebSocket = require('ws');
const zlib = require('zlib');

let threshold = null;
function varint(v) { const out = []; v >>>= 0; while (true) { if ((v & ~0x7f) === 0) { out.push(v); break; } out.push((v & 0x7f) | 0x80); v >>>= 7; } return Buffer.from(out); }
function mcString(s) { const b = Buffer.from(s); return Buffer.concat([varint(b.length), b]); }
function payload(id, ...parts) { return Buffer.concat([varint(id), ...parts]); }
function frame(body) {
  if (threshold === null) return Buffer.concat([varint(body.length), body]);
  const inner = body.length < threshold ? Buffer.concat([varint(0), body]) : Buffer.concat([varint(body.length), zlib.deflateSync(body)]);
  return Buffer.concat([varint(inner.length), inner]);
}
function rv(buf, off) { let n = 0, s = 0, i = off; while (true) { const b = buf[i++]; n |= (b & 0x7f) << s; if (!(b & 0x80)) break; s += 7; } return [n >>> 0, i]; }

function nbtSkip(buf, off) {
  // buf[off] is a tag payload start? we implement entry skipping: tag byte + name + payload
  function payloadSize(tag, o) {
    switch (tag) {
      case 1: return o + 1;
      case 2: return o + 2;
      case 3: return o + 4;
      case 4: return o + 8;
      case 5: return o + 4;
      case 6: return o + 8;
      case 7: { const n = buf.readInt32BE(o); return o + 4 + n; }
      case 8: { const n = buf.readUInt16BE(o); return o + 2 + n; }
      case 9: { const t = buf[o]; const n = buf.readInt32BE(o + 1); let p = o + 5; for (let k = 0; k < n; k++) p = payloadSize(t, p); return p; }
      case 10: { let p = o; while (true) { const t = buf[p]; if (t === 0) return p + 1; const nl = buf.readUInt16BE(p + 1); p = payloadSize(t, p + 3 + nl); } }
      case 11: { const n = buf.readInt32BE(o); return o + 4 + n * 4; }
      case 12: { const n = buf.readInt32BE(o); return o + 4 + n * 8; }
      default: throw new Error('bad nbt tag ' + tag);
    }
  }
  const tag = buf[off];
  const nl = buf.readUInt16BE(off + 1);
  return payloadSize(tag, off + 3 + nl);
}

function unpackLongs(longsBuf, bits) {
  const perLong = Math.floor(64 / bits);
  const values = [];
  for (let li = 0; li < longsBuf.length / 8; li++) {
    let acc = 0n;
    for (let b = 0; b < 8; b++) acc = (acc << 8n) | BigInt(longsBuf[li * 8 + b]);
    for (let slot = 0; slot < perLong; slot++) {
      const v = Number((acc >> BigInt(slot * bits)) & ((1n << BigInt(bits)) - 1n));
      values.push(v);
    }
  }
  return values;
}

const ws = new WebSocket('ws://127.0.0.1:8787/');
ws.binaryType = 'nodebuffer';
let stage = 0;

ws.on('message', (data) => {
  let [, off] = rv(data, 0);
  let inner = data.subarray(off);
  if (threshold !== null) {
    const [dl, i2] = rv(inner, 0);
    inner = dl === 0 ? inner.subarray(i2) : zlib.inflateSync(inner.subarray(i2));
  }
  const [id] = rv(inner, 0);

  if (stage === 0 && id === 0x03) { threshold = 256; stage = 1; return; }
  if (stage === 1 && id === 0x02) { ws.send(frame(payload(0x03))); stage = 2; return; }
  if (stage === 2 && id === 0x0e) {
    ws.send(frame(payload(0x07, varint(1), mcString('minecraft'), mcString('core'), mcString('1.21.4'))));
    stage = 3; return;
  }
  if (stage === 3 && id === 0x03) { ws.send(frame(payload(0x03))); stage = 4; return; }
  if (stage === 4 && id === 0x28) {
    const x = inner.readInt32BE(1);
    const z = inner.readInt32BE(5);
    if (x !== 0 || z !== 0) return;
    // locate section 0 by signature: count i16le 1024 = 00 04, bits=04,
    // palette len varint 04, entries 55 0A 09 00 (bedrock,dirt,grass,air)
    const sig = Buffer.from([0x00, 0x04, 0x04, 0x04, 0x55, 0x0a, 0x09, 0x00]);
    const sigPos = inner.indexOf(sig);
    if (sigPos < 0) { console.log('SIGNATURE NOT FOUND'); process.exit(1); }
    const cd = inner.subarray(sigPos);
    const dataLen = cd.length;

    // parse section 0
    const count = cd.readInt16LE(0);
    let p = 2;
    const bits = cd[p++];
    let palette = [];
    if (bits === 0) { const [v, n] = rv(cd, p); palette = [v]; p = n; }
    else {
      const [n, np] = rv(cd, p); p = np;
      for (let k = 0; k < n; k++) { const [v, vp] = rv(cd, p); palette.push(v); p = vp; }
    }
    const [lc, lp] = rv(cd, p); p = lp;
    const longsBuf = cd.subarray(p, p + lc * 8);
    const indices = unpackLongs(longsBuf, bits);
    // block at x=3,y=-61,z=2 -> section-local index = ly*256 + z*16 + x ; -61-(-64)=3
    const idx = 3 * 256 + 2 * 16 + 3;
    const state = palette[indices[idx]];
    console.log('section0 count=' + count + ' bits=' + bits + ' palette=' + JSON.stringify(palette));
    console.log('block(3,-61,2) state=' + state);
    const idxAir = 5 * 256 + 2 * 16 + 3; // y=-59 should be air unless test placed there
    console.log('block(3,-59,2) state=' + palette[indices[idxAir]]);
    console.log((state === 10 && count >= 1024) ? 'PERSISTENCE-PASS' : 'PERSISTENCE-CHECK');
    process.exit(state === 10 ? 0 : 1);
  }
});

ws.on('open', () => {
  const hs = payload(0x00, varint(769), mcString('kitecraft.test'), (() => { const p = Buffer.alloc(2); p.writeUInt16BE(25565); return p; })(), varint(2));
  ws.send(Buffer.concat([varint(hs.length), hs]));
  setTimeout(() => {
    const ls = payload(0x00, mcString('Verifier'), Buffer.alloc(16));
    ws.send(Buffer.concat([varint(ls.length), ls]));
  }, 50);
});

setTimeout(() => { console.log('TIMEOUT'); process.exit(1); }, 20000);
