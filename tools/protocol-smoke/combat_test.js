const {
  MinecraftClient,
  decodeHealth,
  decodeSpawnEntity,
  readVarint,
  sleep,
  cb,
} = require('./harness');

const checks = [];
function check(name, condition, details = '') {
  checks.push(Boolean(condition));
  console.log(condition ? 'PASS' : 'FAIL', '-', name, details);
}

async function main() {
  const alice = new MinecraftClient('CombatAlice');
  const bob = new MinecraftClient('CombatBob');

  try {
    await alice.connect();
    const spawnCursor = alice.history.length;
    await bob.connect();
    const spawnPacket = await alice.waitForPacket(
      cb.PLAY_SPAWN_ENTITY,
      (packet) => decodeSpawnEntity(packet).entityType === 147,
      4000,
      spawnCursor,
    );
    const bobEntity = decodeSpawnEntity(spawnPacket).entityId;
    check('attacker resolves the target entity from a real spawn packet', bobEntity > 0, `entity=${bobEntity}`);

    const hitCursor = bob.history.length;
    alice.attack(bobEntity);
    const [healthPacket, damagePacket, hurtPacket, motionPacket] = await Promise.all([
      bob.waitForPacket(cb.PLAY_SET_HEALTH, () => true, 4000, hitCursor),
      bob.waitForPacket(
        cb.PLAY_DAMAGE_EVENT,
        (packet) => readVarint(packet.body, packet.idOffset)[0] === bobEntity,
        4000,
        hitCursor,
      ),
      bob.waitForPacket(cb.PLAY_HURT_ANIMATION, () => true, 4000, hitCursor),
      bob.waitForPacket(cb.PLAY_SET_ENTITY_MOTION, () => true, 4000, hitCursor),
    ]);
    check('a recharged fist hit reduces target health', decodeHealth(healthPacket) === 19);
    check('damage event identifies the target', readVarint(damagePacket.body, damagePacket.idOffset)[0] === bobEntity);
    check('hurt animation is broadcast', Boolean(hurtPacket));
    check('Pumpkin-style knockback motion is broadcast', Boolean(motionPacket));

    const protectedCursor = bob.history.length;
    alice.attack(bobEntity);
    await sleep(150);
    check(
      'hurt protection suppresses an immediate weaker hit',
      !bob.history.slice(protectedCursor).some((packet) => packet.id === cb.PLAY_SET_HEALTH || packet.id === cb.PLAY_DAMAGE_EVENT),
    );

    const movementCursor = alice.history.length;
    bob.move({ x: 10.5, y: -60, z: 0.5 });
    await alice.waitForPacket(cb.PLAY_SYNC_ENTITY_POSITION, () => true, 4000, movementCursor);
    await sleep(600);
    const rangeCursor = bob.history.length;
    alice.attack(bobEntity);
    await sleep(150);
    check(
      'out-of-reach attacks do not damage the target',
      !bob.history.slice(rangeCursor).some((packet) => packet.id === cb.PLAY_SET_HEALTH || packet.id === cb.PLAY_DAMAGE_EVENT),
    );
  } finally {
    await Promise.allSettled([alice.close(), bob.close()]);
  }

  if (checks.some((passed) => !passed)) process.exitCode = 1;
}

main().catch((error) => {
  console.error('FAIL - combat scenario:', error);
  process.exitCode = 1;
});
