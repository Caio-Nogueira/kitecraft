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
function mcString(s) { const b = Buffer.from(s); return Buffer.concat([varint(b.length), b]); }
function payload(id, ...parts) { return Buffer.concat([varint(id), ...parts]); }
function frame(body) {
  if (threshold === null) return Buffer.concat([varint(body.length), body]);
  const inner = body.length < threshold
    ? Buffer.concat([varint(0), body])
    : Buffer.concat([varint(body.length), zlib.deflateSync(body)]);
  return Buffer.concat([varint(inner.length), inner]);
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
function packPos(x, y, z) {
  let v = ((BigInt(x) & 0x3FFFFFFn) << 38n) | ((BigInt(z) & 0x3FFFFFFn) << 12n) | (BigInt(y) & 0xFFFn);
  const buf = Buffer.alloc(8);
  buf.writeBigInt64BE(BigInt.asIntN(64, v));
  return buf;
}
function f64(v) { const b = Buffer.alloc(8); b.writeDoubleBE(v); return b; }
function f32(v) { const b = Buffer.alloc(4); b.writeFloatBE(v); return b; }
function i64(v) { const b = Buffer.alloc(8); b.writeBigInt64BE(BigInt.asIntN(64, BigInt(v))); return b; }

const results = [];
function check(name, ok) { results.push(ok); console.log(ok ? 'PASS' : 'FAIL', '-', name); if (!ok) process.exitCode = 1; }

let ws;
let pendingResolve = null;
let lastChange = null;
let gotDigAck = false;
let gotChat = false;
let gotKeepAlive = false;
let gotTimeUpdate = false;

function waitFor(predMs) {
  return new Promise((res) => {
    const t0 = Date.now();
    const iv = setInterval(() => {
      if (predMs()) { clearInterval(iv); res(); }
      else if (Date.now() - t0 > 4000) { clearInterval(iv); res(); }
    }, 25);
  });
}

ws = new WebSocket('ws://127.0.0.1:8787/');
ws.binaryType = 'nodebuffer';

ws.on('open', () => {
  const hs = payload(0x00, varint(769), mcString('kitecraft.test'),
    (() => { const p = Buffer.alloc(2); p.writeUInt16BE(25565); return p; })(), varint(2));
  ws.send(Buffer.concat([varint(hs.length), hs]));
  setTimeout(() => {
    const ls = payload(0x00, mcString('Tester'), Buffer.alloc(16));
    ws.send(Buffer.concat([varint(ls.length), ls]));
  }, 60);
});

// implement config handshake inside the message handler instead:
let cfgStage = 0;
ws.removeAllListeners('message');
ws.on('message', async (data) => {
  let [outerLen, off] = readVarint(data, 0);
  let inner = data.subarray(off);
  if (threshold !== null) {
    const [dl, i2] = readVarint(inner, 0);
    try {
      inner = dl === 0 ? inner.subarray(i2) : zlib.inflateSync(inner.subarray(i2));
    } catch (e) {
      console.log('INFLATE FAIL dl=' + dl + ' outer=' + outerLen + ' avail=' + (data.length - off) + ' restlen=' + (inner.length - i2));
      console.log('head:', data.subarray(0, 24).toString('hex'));
      throw e;
    }
  }
  const [id] = readVarint(inner, 0);

  if (cfgStage === 0 && id === 0x03 && threshold === null) {
    threshold = 256; console.log('compress received'); return;
  }
  if (cfgStage === 0 && id === 0x02) { // login success
    cfgStage = 1; console.log('success received');
    ws.send(frame(payload(0x03))); // login acknowledged
    return;
  }
  if (cfgStage === 1 && id === 0x0e) { // known packs
    cfgStage = 2; console.log('known packs received');
    ws.send(frame(payload(0x07, varint(1), mcString('minecraft'), mcString('core'), mcString('1.21.4'))));
    return;
  }
  if (cfgStage === 2 && id === 0x03) { // finish config
    cfgStage = 3; console.log('finish config received -> entering play');
    ws.send(frame(payload(0x03)));
    onPlay();
    return;
  }
  if (cfgStage === 3) {
    handlePlay(id, inner);
  }
});

let playResolved = null;
const playReady = new Promise((r) => { playResolved = r; });

function handlePlay(id, inner) {
  if (id === 0x27) {
    gotKeepAlive = true;
    ws.send(frame(payload(0x1a, i64(inner.readBigInt64BE(1)))));
  } else if (id === 0x05) gotDigAck = true;
  else if (id === 0x09) {
    const packed = inner.readBigInt64BE(1);
    const x = Number(BigInt.asIntN(26, packed >> 38n));
    const z = Number(BigInt.asIntN(26, (packed << 26n) >> 38n));
    const y = Number(BigInt.asIntN(12, (packed << 52n) >> 52n));
    const state = readVarint(inner, 9)[0];
    lastChange = { x, y, z, state };
    console.log('block_change:', JSON.stringify(lastChange));
  } else if (id === 0x73) {
    gotChat = true;
    console.log('system_chat received');
  } else if (id === 0x6b) gotTimeUpdate = true;
}

async function onPlay() {
  playResolved();
  await new Promise((r) => setTimeout(r, 1500));

  console.log('-- step 1: break block (3,-61,2)');
  ws.send(frame(payload(0x27, varint(0), packPos(3, -61, 2), Buffer.from([0]), varint(101))));
  await new Promise((r) => setTimeout(r, 800));
  check('dig acknowledged', gotDigAck);
  check('break broadcast (air at 3,-61,2)',
    lastChange && lastChange.x === 3 && lastChange.y === -61 && lastChange.z === 2 && lastChange.state === 0);

  console.log('-- step 2: select dirt + place at (3,-60,2) clicking top of (3,-61,2)? using face up of bedrock layer block');
  ws.send(frame(payload(0x36, (()=>{const b=Buffer.alloc(2);b.writeInt16BE(36);return b;})(), varint(1), varint(28), varint(0), varint(0))));
  ws.send(frame(payload(0x33, (()=>{const b=Buffer.alloc(2);b.writeInt16BE(0);return b;})())));
  await new Promise((r) => setTimeout(r, 200));
  ws.send(frame(payload(0x3c, varint(1), packPos(3, -62, 2), varint(1), f32(0.5), f32(1.0), f32(0.5), Buffer.from([0]), Buffer.from([0]), varint(102))));
  await new Promise((r) => setTimeout(r, 800));
  check('place broadcast (dirt at 3,-61,2)',
    lastChange && lastChange.x === 3 && lastChange.y === -61 && lastChange.z === 2 && lastChange.state === 10);

  console.log('-- step 3: chat');
  ws.send(frame(payload(0x07, mcString('hello world'), i64(Date.now()), Buffer.alloc(8), Buffer.from([0]), varint(0), Buffer.alloc(3))));
  await new Promise((r) => setTimeout(r, 600));
  check('chat echoed as system chat', gotChat);

  console.log('-- step 4: movement packet accepted');
  ws.send(frame(payload(0x1d, f64(1.5), f64(-59.0), f64(1.5), f32(90), f32(0), Buffer.from([0]))));
  await new Promise((r) => setTimeout(r, 300));
  check('still connected after move', ws.readyState === 1);

  console.log('-- step 5: waiting for time update / keepalive (up to 12s)');
  const t0 = Date.now();
  while (!(gotTimeUpdate && gotKeepAlive) && Date.now() - t0 < 12000) await new Promise((r) => setTimeout(r, 200));
  check('time update received', gotTimeUpdate);
  check('keepalive roundtrip', gotKeepAlive);

  printResults();
  ws.close();
  process.exit(process.exitCode || 0);
}

function printResults() {
  console.log('--- results ---');
  ['dig acknowledged','break broadcast','place broadcast','chat echo','move ok','time update','keepalive'].forEach(()=>{});
}
