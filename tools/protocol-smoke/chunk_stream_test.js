const {
  MinecraftClient,
  cb,
  decodeChunkCacheCenter,
  decodeChunkCoordinates,
  decodePositionCorrection,
  decodeUnloadChunk,
  readVarint,
  sleep,
} = require('./harness');

function check(name, condition) {
  console.log(condition ? 'PASS' : 'FAIL', '-', name);
  if (!condition) process.exitCode = 1;
}

async function waitUntil(description, predicate, timeoutMs = 30000) {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const result = predicate();
    if (result) return result;
    await sleep(25);
  }
  throw new Error(`timed out waiting for ${description}`);
}

function packetsSince(client, fromIndex, id) {
  return client.history.slice(fromIndex).filter((packet) => packet.id === id);
}

function hasCoordinate(packets, decode, expected) {
  return packets.some((packet) => {
    const coordinate = decode(packet);
    return coordinate.x === expected.x && coordinate.z === expected.z;
  });
}

async function moveInSteps(client, positions, y) {
  for (const x of positions) {
    client.move({ x, y, z: 0.5, onGround: false });
    await sleep(500);
  }
}

(async () => {
  const client = new MinecraftClient(`Chunk${process.pid}`);
  try {
    await client.connect(60000);

    const initialPositionPacket = client.history.find((packet) => packet.id === cb.PLAY_POSITION);
    const initialPosition = decodePositionCorrection(initialPositionPacket);
    const flightY = initialPosition.y + 15;

    await waitUntil('the complete initial 17x17 window', () => {
      const unique = new Set(
        client.history
          .filter((packet) => packet.id === cb.PLAY_CHUNK_DATA)
          .map((packet) => {
            const { x, z } = decodeChunkCoordinates(packet);
            return `${x},${z}`;
          }),
      );
      return unique.size >= 289;
    }, 60000);
    await sleep(1000);
    for (const y of [initialPosition.y + 5, initialPosition.y + 10, flightY]) {
      client.move({ x: initialPosition.x, y, z: 0.5, onGround: false });
      await sleep(500);
    }

    const initialCenters = client.history
      .filter((packet) => packet.id === cb.PLAY_SET_CHUNK_CACHE_CENTER)
      .map(decodeChunkCacheCenter);
    check('login announces chunk cache center (0, 0)',
      initialCenters.some(({ x, z }) => x === 0 && z === 0));

    const batchSizes = client.history
      .filter((packet) => packet.id === cb.PLAY_CHUNK_BATCH_FINISHED)
      .map((packet) => readVarint(packet.body, packet.idOffset)[0]);
    check('initial generation is split into bounded batches',
      batchSizes.length > 1 && batchSizes.every((size) => size >= 1 && size <= 8));

    const positiveStart = client.history.length;
    await moveInSteps(client, [8.5, 16.5], flightY);
    try {
      await waitUntil('positive chunk center and replacement edge', () => {
        const centers = packetsSince(client, positiveStart, cb.PLAY_SET_CHUNK_CACHE_CENTER);
        const chunks = packetsSince(client, positiveStart, cb.PLAY_CHUNK_DATA);
        const unloads = packetsSince(client, positiveStart, cb.PLAY_UNLOAD_CHUNK);
        return hasCoordinate(centers, decodeChunkCacheCenter, { x: 1, z: 0 })
          && hasCoordinate(chunks, decodeChunkCoordinates, { x: 9, z: 0 })
          && hasCoordinate(unloads, decodeUnloadChunk, { x: -8, z: 0 });
      });
    } catch (error) {
      const centers = packetsSince(client, positiveStart, cb.PLAY_SET_CHUNK_CACHE_CENTER)
        .map(decodeChunkCacheCenter);
      const chunks = packetsSince(client, positiveStart, cb.PLAY_CHUNK_DATA)
        .map(decodeChunkCoordinates);
      const unloads = packetsSince(client, positiveStart, cb.PLAY_UNLOAD_CHUNK)
        .map(decodeUnloadChunk);
      const corrections = packetsSince(client, positiveStart, cb.PLAY_POSITION)
        .map(decodePositionCorrection);
      console.error('positive streaming diagnostics', { centers, chunks, unloads, corrections });
      throw error;
    }
    check('crossing +X recenters, unloads the old edge, and sends the new edge', true);

    const negativeStart = client.history.length;
    await moveInSteps(client, [8.5, 0.5, -0.5], flightY);
    await waitUntil('negative chunk center and replacement edge', () => {
      const centers = packetsSince(client, negativeStart, cb.PLAY_SET_CHUNK_CACHE_CENTER);
      const chunks = packetsSince(client, negativeStart, cb.PLAY_CHUNK_DATA);
      const unloads = packetsSince(client, negativeStart, cb.PLAY_UNLOAD_CHUNK);
      return hasCoordinate(centers, decodeChunkCacheCenter, { x: -1, z: 0 })
        && hasCoordinate(chunks, decodeChunkCoordinates, { x: -9, z: 0 })
        && hasCoordinate(unloads, decodeUnloadChunk, { x: 8, z: 0 });
    });
    check('crossing -X handles negative chunk coordinates symmetrically', true);

    console.log('CHUNK-STREAM-PASS');
  } finally {
    await client.close();
  }
  process.exit(process.exitCode || 0);
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
