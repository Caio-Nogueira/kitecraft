// Minimal multi-client TCP <-> WebSocket bridge: 127.0.0.1:25565 <-> ws://127.0.0.1:8787/
const net = require('net');
const { WebSocket } = require('ws');

const server = net.createServer((tcp) => {
  const ws = new WebSocket('ws://127.0.0.1:8787/', { perMessageDeflate: false });
  ws.binaryType = 'nodebuffer';
  const queue = [];

  ws.on('open', () => {
    for (const chunk of queue.splice(0)) ws.send(chunk, { binary: true });
  });
  ws.on('message', (data) => {
    if (tcp.writable) tcp.write(Buffer.from(data));
  });
  const bye = () => { try { tcp.destroy(); } catch {} try { ws.close(); } catch {} };
  ws.on('close', bye);
  ws.on('error', bye);
  tcp.on('error', bye);

  tcp.on('data', (chunk) => {
    if (ws.readyState === WebSocket.OPEN) ws.send(chunk, { binary: true });
    else if (ws.readyState === WebSocket.CONNECTING) queue.push(chunk);
  });
  tcp.on('close', () => { try { ws.close(); } catch {} });
});

server.listen(25565, '127.0.0.1', () => console.log('bridge listening on 127.0.0.1:25565'));
