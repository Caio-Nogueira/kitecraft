const {
  MinecraftClient,
  decodeBlockChange,
  decodePositionCorrection,
  readVarint,
  sleep,
  tickStats,
  cb,
} = require('./harness');

const checks = [];
function check(name, condition, details = '') {
  checks.push(Boolean(condition));
  console.log(condition ? 'PASS' : 'FAIL', '-', name, details);
}

async function main() {
  const alice = new MinecraftClient('TickAlice');
  const bob = new MinecraftClient('TickBob');

  try {
    await alice.connect();
    await bob.connect();
    check('two protocol clients reach play state', true);

    const before = await tickStats();
    check('tick loop starts once players enter', before.active && before.players === 2, JSON.stringify(before));

    await sleep(650);
    const after = await tickStats();
    const tickDelta = after.ticks - before.ticks;
    check('timer advances near 20 TPS', tickDelta >= 8 && tickDelta <= 20, `delta=${tickDelta}`);

    const movementCursor = bob.history.length;
    alice.move({ x: 2.5, y: -59, z: 2.5, yaw: 90 });
    await bob.waitForPacket(cb.PLAY_SYNC_ENTITY_POSITION, () => true, 4000, movementCursor);
    check('movement is synchronized between real connections', true);

    const collisionCursor = alice.history.length;
    const collisionObserverCursor = bob.history.length;
    alice.move({ x: 2.5, y: -60.5, z: 2.5, yaw: 90 });
    const [collisionPacket] = await Promise.all([
      alice.waitForPacket(cb.PLAY_POSITION, () => true, 4000, collisionCursor),
      bob.waitForPacket(cb.PLAY_SYNC_ENTITY_POSITION, () => true, 4000, collisionObserverCursor),
    ]);
    const collision = decodePositionCorrection(collisionPacket);
    check(
      'world collision resolves a player to the ground surface',
      collision.x === 2.5 && collision.y === -60 && collision.z === 2.5,
      JSON.stringify(collision),
    );

    const correctionCursor = alice.history.length;
    const rejectedObserverCursor = bob.history.length;
    alice.move({ x: 100, y: -60, z: 100, yaw: 90 });
    const correctionPacket = await alice.waitForPacket(cb.PLAY_POSITION, () => true, 4000, correctionCursor);
    const correction = decodePositionCorrection(correctionPacket);
    check(
      'implausible movement receives the authoritative position',
      correction.x === 2.5 && correction.y === -60 && correction.z === 2.5,
      JSON.stringify(correction),
    );
    await sleep(100);
    check(
      'rejected movement is not broadcast to observers',
      !bob.history.slice(rejectedObserverCursor).some((packet) => packet.id === cb.PLAY_SYNC_ENTITY_POSITION),
    );

    const clearCursor = alice.history.length;
    const clearSequence = alice.digBlock({ x: 6, y: -61, z: 6 });
    await alice.waitForPacket(
      cb.PLAY_ACK_DIGGING,
      (packet) => readVarint(packet.body, packet.idOffset)[0] === clearSequence,
      4000,
      clearCursor,
    );
    await sleep(100);

    alice.setCreativeSlot({ itemId: 55 });
    alice.selectHotbar(0);
    await sleep(100);
    const beforePlace = await tickStats();
    const alicePlaceCursor = alice.history.length;
    const bobPlaceCursor = bob.history.length;
    const placeSequence = alice.placeBlock({ x: 6, y: -62, z: 6, face: 1 });
    const [placeAck, blockPacket] = await Promise.all([
      alice.waitForPacket(
        cb.PLAY_ACK_DIGGING,
        (packet) => readVarint(packet.body, packet.idOffset)[0] === placeSequence,
        4000,
        alicePlaceCursor,
      ),
      bob.waitForPacket(
        cb.PLAY_BLOCK_CHANGE,
        (packet) => {
          const change = decodeBlockChange(packet);
          return change.x === 6 && change.y === -61 && change.z === 6;
        },
        4000,
        bobPlaceCursor,
      ),
    ]);
    const change = decodeBlockChange(blockPacket);
    check('block placement is broadcast between connections', change.state !== 0, `state=${change.state}`);
    check('placement acknowledgement returns to its actor', readVarint(placeAck.body, placeAck.idOffset)[0] === placeSequence);
    const afterPlace = await tickStats();
    check('placement is applied by a logical tick', afterPlace.ticks > beforePlace.ticks, `before=${beforePlace.ticks} after=${afterPlace.ticks}`);
    check(
      'placement acknowledgement is not leaked to observers',
      !bob.history.slice(bobPlaceCursor).some(
        (packet) => packet.id === cb.PLAY_ACK_DIGGING && readVarint(packet.body, packet.idOffset)[0] === placeSequence,
      ),
    );

    const occupiedCursor = bob.history.length;
    const occupiedAckCursor = alice.history.length;
    const occupiedSequence = alice.placeBlock({ x: 6, y: -62, z: 6, face: 1 });
    await alice.waitForPacket(
      cb.PLAY_ACK_DIGGING,
      (packet) => readVarint(packet.body, packet.idOffset)[0] === occupiedSequence,
      4000,
      occupiedAckCursor,
    );
    await sleep(100);
    check(
      'world query rejects placement into an occupied block',
      !bob.history.slice(occupiedCursor).some((packet) => {
        if (packet.id !== cb.PLAY_BLOCK_CHANGE) return false;
        const attempted = decodeBlockChange(packet);
        return attempted.x === 6 && attempted.y === -61 && attempted.z === 6;
      }),
    );

    const aliceDigCursor = alice.history.length;
    const bobDigCursor = bob.history.length;
    const digSequence = alice.digBlock({ x: 6, y: -61, z: 6 });
    const [digAck, breakPacket] = await Promise.all([
      alice.waitForPacket(
        cb.PLAY_ACK_DIGGING,
        (packet) => readVarint(packet.body, packet.idOffset)[0] === digSequence,
        4000,
        aliceDigCursor,
      ),
      bob.waitForPacket(
        cb.PLAY_BLOCK_CHANGE,
        (packet) => {
          const broken = decodeBlockChange(packet);
          return broken.x === 6 && broken.y === -61 && broken.z === 6 && broken.state === 0;
        },
        4000,
        bobDigCursor,
      ),
    ]);
    check('dig acknowledgement returns to its actor', readVarint(digAck.body, digAck.idOffset)[0] === digSequence);
    check('block breaking is broadcast between connections', decodeBlockChange(breakPacket).state === 0);
    check(
      'dig acknowledgement is not leaked to observers',
      !bob.history.slice(bobDigCursor).some(
        (packet) => packet.id === cb.PLAY_ACK_DIGGING && readVarint(packet.body, packet.idOffset)[0] === digSequence,
      ),
    );

    await alice.close();
    await bob.close();
    await sleep(200);

    const stopped = await tickStats();
    check('tick loop stops after the last player leaves', !stopped.active && stopped.players === 0, JSON.stringify(stopped));
    check('final tick count survives object recreation', stopped.ticks >= after.ticks, `ticks=${stopped.ticks}`);
    await sleep(250);
    const stillStopped = await tickStats();
    check('ticks remain stopped while the server is empty', stillStopped.ticks === stopped.ticks);
  } finally {
    await Promise.allSettled([alice.close(), bob.close()]);
  }

  if (checks.some((passed) => !passed)) process.exitCode = 1;
}

main().catch((error) => {
  console.error('FAIL - tick scenario:', error);
  process.exitCode = 1;
});
