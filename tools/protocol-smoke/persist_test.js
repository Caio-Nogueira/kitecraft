const {
  MinecraftClient,
  cb,
  decodeChunkBlock,
  readVarint,
  sleep,
} = require('./harness');

const TARGET = { x: 12, y: -61, z: 12 };
const DIRT_ITEM_ID = 55;
const DIRT_STATE_ID = 10;

async function main() {
  const writer = new MinecraftClient('PersistWriter');
  const verifier = new MinecraftClient('PersistVerifier');
  try {
    await writer.connect();

    const clearCursor = writer.history.length;
    const clearSequence = writer.digBlock(TARGET);
    await writer.waitForPacket(
      cb.PLAY_ACK_DIGGING,
      (packet) => readVarint(packet.body, packet.idOffset)[0] === clearSequence,
      4000,
      clearCursor,
    );

    writer.setCreativeSlot({ itemId: DIRT_ITEM_ID });
    writer.selectHotbar(0);
    await sleep(100);
    const placeCursor = writer.history.length;
    const placeSequence = writer.placeBlock({
      x: TARGET.x,
      y: TARGET.y - 1,
      z: TARGET.z,
      face: 1,
    });
    await writer.waitForPacket(
      cb.PLAY_ACK_DIGGING,
      (packet) => readVarint(packet.body, packet.idOffset)[0] === placeSequence,
      4000,
      placeCursor,
    );

    await writer.close();
    await sleep(250);
    await verifier.connect();
    const chunk = verifier.history.find((packet) =>
      packet.id === cb.PLAY_CHUNK_DATA
      && packet.body.readInt32BE(packet.idOffset) === 0
      && packet.body.readInt32BE(packet.idOffset + 4) === 0
    );
    if (!chunk) throw new Error('spawn chunk was not received');
    const state = decodeChunkBlock(chunk, TARGET.x, TARGET.y, TARGET.z);
    if (state !== DIRT_STATE_ID) {
      throw new Error(`persisted block state is ${state}, expected ${DIRT_STATE_ID}`);
    }
    console.log('PASS - a dirty chunk survives writer disconnect and a new protocol connection');
  } finally {
    await Promise.allSettled([writer.close(), verifier.close()]);
  }
}

main().catch((error) => {
  console.error('FAIL - persistence scenario:', error);
  process.exitCode = 1;
});
