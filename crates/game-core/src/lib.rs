use std::collections::{HashMap, VecDeque};

pub const VOID_FLOOR_Y: f64 = -70.0;
pub const AIR_BLOCK_STATE: u16 = block_data::AIR;
pub const PLAYER_WIDTH: f64 = 0.6;
pub const PLAYER_HEIGHT: f64 = 1.8;
pub const PLAYER_EYE_HEIGHT: f64 = 1.62;
pub const PLAYER_MAX_HEALTH: f32 = 20.0;
pub const PLAYER_ATTACK_REACH: f64 = 3.0;

const MAX_HORIZONTAL_COORDINATE: f64 = 30_000_000.0;
const MAX_VERTICAL_COORDINATE: f64 = 20_000_000.0;
const MAX_MOVEMENT_SQUARED: f64 = 100.0;
const COLLISION_EPSILON: f64 = 1.0e-7;
const GROUND_PROBE: f64 = 1.0e-5;
const FIST_ATTACK_DAMAGE: f32 = 1.0;
const FIST_ATTACK_SPEED: f64 = 1.6;
const TICKS_PER_SECOND: f64 = 20.0;
const HURT_PROTECTION_TICKS: u64 = 10;
const BASE_KNOCKBACK: f64 = 0.5;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlayerId(pub [u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerState {
    pub position: Position,
    pub rotation: Rotation,
    pub on_ground: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementInput {
    pub position: Option<Position>,
    pub rotation: Option<Rotation>,
    pub on_ground: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CombatState {
    pub health: f32,
    pub max_health: f32,
    pub velocity: Velocity,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            health: PLAYER_MAX_HEALTH,
            max_health: PLAYER_MAX_HEALTH,
            velocity: Velocity::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Position,
    pub max: Position,
}

impl Aabb {
    #[must_use]
    pub fn player(position: Position) -> Self {
        let radius = PLAYER_WIDTH / 2.0;
        Self {
            min: Position {
                x: position.x - radius,
                y: position.y,
                z: position.z - radius,
            },
            max: Position {
                x: position.x + radius,
                y: position.y + PLAYER_HEIGHT,
                z: position.z + radius,
            },
        }
    }

    #[must_use]
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    #[must_use]
    fn stretched(self, movement: Position) -> Self {
        Self {
            min: Position {
                x: self.min.x + movement.x.min(0.0),
                y: self.min.y + movement.y.min(0.0),
                z: self.min.z + movement.z.min(0.0),
            },
            max: Position {
                x: self.max.x + movement.x.max(0.0),
                y: self.max.y + movement.y.max(0.0),
                z: self.max.z + movement.z.max(0.0),
            },
        }
    }

    #[must_use]
    fn translated(self, movement: Position) -> Self {
        Self {
            min: add_position(self.min, movement),
            max: add_position(self.max, movement),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildHeight {
    pub min_y: i32,
    pub max_y_exclusive: i32,
}

impl BuildHeight {
    #[must_use]
    pub const fn contains(self, y: i32) -> bool {
        y >= self.min_y && y < self.max_y_exclusive
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPosition {
    #[must_use]
    pub const fn offset(self, face: BlockFace) -> Self {
        let (dx, dy, dz) = match face {
            BlockFace::Bottom => (0, -1, 0),
            BlockFace::Top => (0, 1, 0),
            BlockFace::North => (0, 0, -1),
            BlockFace::South => (0, 0, 1),
            BlockFace::West => (-1, 0, 0),
            BlockFace::East => (1, 0, 0),
        };
        Self {
            x: self.x + dx,
            y: self.y + dy,
            z: self.z + dz,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockFace {
    Bottom,
    Top,
    North,
    South,
    West,
    East,
}

impl TryFrom<i32> for BlockFace {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bottom),
            1 => Ok(Self::Top),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockActionId(pub i32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlockChange {
    pub position: BlockPosition,
    pub state: u16,
}

/// The gameplay core's read-only view of world state.
///
/// The mutable receiver permits adapters to populate caches while answering a
/// query; implementations cannot expose world mutation through this contract.
pub trait WorldView {
    fn block_state(&mut self, position: BlockPosition) -> Option<u16>;
}

fn observed_block_state(
    world: &mut impl WorldView,
    pending: &HashMap<BlockPosition, u16>,
    position: BlockPosition,
) -> Option<u16> {
    pending
        .get(&position)
        .copied()
        .or_else(|| world.block_state(position))
}

fn add_position(left: Position, right: Position) -> Position {
    Position {
        x: left.x + right.x,
        y: left.y + right.y,
        z: left.z + right.z,
    }
}

fn subtract_position(left: Position, right: Position) -> Position {
    Position {
        x: left.x - right.x,
        y: left.y - right.y,
        z: left.z - right.z,
    }
}

fn valid_position(position: Position) -> bool {
    position.x.is_finite()
        && position.y.is_finite()
        && position.z.is_finite()
        && position.x.abs() <= MAX_HORIZONTAL_COORDINATE
        && position.y.abs() <= MAX_VERTICAL_COORDINATE
        && position.z.abs() <= MAX_HORIZONTAL_COORDINATE
}

fn valid_rotation(rotation: Rotation) -> bool {
    rotation.yaw.is_finite() && rotation.pitch.is_finite()
}

fn movement_squared(movement: Position) -> f64 {
    movement.x * movement.x + movement.y * movement.y + movement.z * movement.z
}

fn block_collision_boxes(
    world: &mut impl WorldView,
    pending: &HashMap<BlockPosition, u16>,
    swept: Aabb,
) -> Vec<Aabb> {
    let min_x = swept.min.x.floor() as i32;
    let min_y = swept.min.y.floor() as i32;
    let min_z = swept.min.z.floor() as i32;
    let max_x = (swept.max.x - COLLISION_EPSILON).floor() as i32;
    let max_y = (swept.max.y - COLLISION_EPSILON).floor() as i32;
    let max_z = (swept.max.z - COLLISION_EPSILON).floor() as i32;
    let mut collisions = Vec::new();

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let position = BlockPosition { x, y, z };
                let Some(state) =
                    observed_block_state(world, pending, position).and_then(block_data::state)
                else {
                    continue;
                };
                let Some(block) = block_data::block(state.block_id) else {
                    continue;
                };
                let [offset_x, offset_y, offset_z] = block.collision_offset(x, z);
                for shape in state.collision_shapes() {
                    let collision = Aabb {
                        min: Position {
                            x: f64::from(x) + offset_x + shape.min[0],
                            y: f64::from(y) + offset_y + shape.min[1],
                            z: f64::from(z) + offset_z + shape.min[2],
                        },
                        max: Position {
                            x: f64::from(x) + offset_x + shape.max[0],
                            y: f64::from(y) + offset_y + shape.max[1],
                            z: f64::from(z) + offset_z + shape.max[2],
                        },
                    };
                    if collision.intersects(swept) {
                        collisions.push(collision);
                    }
                }
            }
        }
    }
    collisions
}

fn resolve_y(mut amount: f64, player: Aabb, collision: Aabb) -> f64 {
    if player.max.x <= collision.min.x
        || player.min.x >= collision.max.x
        || player.max.z <= collision.min.z
        || player.min.z >= collision.max.z
    {
        return amount;
    }
    if amount > 0.0 && player.max.y <= collision.min.y {
        amount = amount.min(collision.min.y - player.max.y);
    } else if amount < 0.0 && player.min.y >= collision.max.y {
        amount = amount.max(collision.max.y - player.min.y);
    }
    amount
}

fn resolve_x(mut amount: f64, player: Aabb, collision: Aabb) -> f64 {
    if player.max.y <= collision.min.y
        || player.min.y >= collision.max.y
        || player.max.z <= collision.min.z
        || player.min.z >= collision.max.z
    {
        return amount;
    }
    if amount > 0.0 && player.max.x <= collision.min.x {
        amount = amount.min(collision.min.x - player.max.x);
    } else if amount < 0.0 && player.min.x >= collision.max.x {
        amount = amount.max(collision.max.x - player.min.x);
    }
    amount
}

fn resolve_z(mut amount: f64, player: Aabb, collision: Aabb) -> f64 {
    if player.max.x <= collision.min.x
        || player.min.x >= collision.max.x
        || player.max.y <= collision.min.y
        || player.min.y >= collision.max.y
    {
        return amount;
    }
    if amount > 0.0 && player.max.z <= collision.min.z {
        amount = amount.min(collision.min.z - player.max.z);
    } else if amount < 0.0 && player.min.z >= collision.max.z {
        amount = amount.max(collision.max.z - player.min.z);
    }
    amount
}

fn resolve_movement(
    world: &mut impl WorldView,
    pending: &HashMap<BlockPosition, u16>,
    position: Position,
    desired: Position,
) -> Position {
    let mut player = Aabb::player(position);
    let collisions = block_collision_boxes(world, pending, player.stretched(desired));

    let mut resolved = desired;
    for collision in &collisions {
        resolved.y = resolve_y(resolved.y, player, *collision);
    }
    player = player.translated(Position {
        x: 0.0,
        y: resolved.y,
        z: 0.0,
    });
    for collision in &collisions {
        resolved.x = resolve_x(resolved.x, player, *collision);
    }
    player = player.translated(Position {
        x: resolved.x,
        y: 0.0,
        z: 0.0,
    });
    for collision in &collisions {
        resolved.z = resolve_z(resolved.z, player, *collision);
    }
    resolved
}

fn is_on_ground(
    world: &mut impl WorldView,
    pending: &HashMap<BlockPosition, u16>,
    position: Position,
) -> bool {
    let probe = Position {
        x: 0.0,
        y: -GROUND_PROBE,
        z: 0.0,
    };
    resolve_movement(world, pending, position, probe).y > probe.y
}

fn movement_differs(left: Position, right: Position) -> bool {
    (left.x - right.x).abs() > COLLISION_EPSILON
        || (left.y - right.y).abs() > COLLISION_EPSILON
        || (left.z - right.z).abs() > COLLISION_EPSILON
}

fn nearest_point(point: Position, bounds: Aabb) -> Position {
    Position {
        x: point.x.clamp(bounds.min.x, bounds.max.x),
        y: point.y.clamp(bounds.min.y, bounds.max.y),
        z: point.z.clamp(bounds.min.z, bounds.max.z),
    }
}

fn segment_intersects(bounds: Aabb, start: Position, end: Position) -> bool {
    let direction = subtract_position(end, start);
    let mut entry = 0.0_f64;
    let mut exit = 1.0_f64;
    for (origin, delta, min, max) in [
        (start.x, direction.x, bounds.min.x, bounds.max.x),
        (start.y, direction.y, bounds.min.y, bounds.max.y),
        (start.z, direction.z, bounds.min.z, bounds.max.z),
    ] {
        if delta.abs() <= COLLISION_EPSILON {
            if origin <= min || origin >= max {
                return false;
            }
            continue;
        }
        let first = (min - origin) / delta;
        let second = (max - origin) / delta;
        entry = entry.max(first.min(second));
        exit = exit.min(first.max(second));
        if entry > exit {
            return false;
        }
    }
    exit > COLLISION_EPSILON && entry < 1.0 - COLLISION_EPSILON
}

fn has_line_of_sight(
    world: &mut impl WorldView,
    pending: &HashMap<BlockPosition, u16>,
    start: Position,
    end: Position,
) -> bool {
    let ray_bounds = Aabb {
        min: Position {
            x: start.x.min(end.x) - COLLISION_EPSILON,
            y: start.y.min(end.y) - COLLISION_EPSILON,
            z: start.z.min(end.z) - COLLISION_EPSILON,
        },
        max: Position {
            x: start.x.max(end.x) + COLLISION_EPSILON,
            y: start.y.max(end.y) + COLLISION_EPSILON,
            z: start.z.max(end.z) + COLLISION_EPSILON,
        },
    };
    !block_collision_boxes(world, pending, ray_bounds)
        .into_iter()
        .any(|collision| segment_intersects(collision, start, end))
}

fn attack_damage(tick: u64, last_attack_tick: Option<u64>) -> f32 {
    let cooldown_ticks = TICKS_PER_SECOND / FIST_ATTACK_SPEED;
    let progress = last_attack_tick.map_or(1.0, |last| {
        (tick.saturating_sub(last) as f64 + 0.5) / cooldown_ticks
    });
    let progress = progress.clamp(0.0, 1.0);
    FIST_ATTACK_DAMAGE * (0.2 + progress.powi(2) as f32 * 0.8)
}

fn knockback(attacker: PlayerState, target: PlayerState, velocity: Velocity) -> Velocity {
    let yaw = f64::from(attacker.rotation.yaw).to_radians();
    Velocity {
        x: velocity.x / 2.0 - yaw.sin() * BASE_KNOCKBACK,
        y: if target.on_ground {
            (velocity.y / 2.0 + BASE_KNOCKBACK).min(0.4)
        } else {
            velocity.y
        },
        z: velocity.z / 2.0 + yaw.cos() * BASE_KNOCKBACK,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerInput {
    Move {
        player: PlayerId,
        movement: MovementInput,
    },
    DigBlock {
        player: PlayerId,
        position: BlockPosition,
        break_block: bool,
        action_id: BlockActionId,
    },
    PlaceBlock {
        player: PlayerId,
        clicked: BlockPosition,
        face: Option<BlockFace>,
        state: Option<u16>,
        action_id: BlockActionId,
    },
    Attack {
        player: PlayerId,
        target: PlayerId,
    },
}

impl PlayerInput {
    #[must_use]
    pub const fn player(self) -> PlayerId {
        match self {
            Self::Move { player, .. }
            | Self::DigBlock { player, .. }
            | Self::PlaceBlock { player, .. }
            | Self::Attack { player, .. } => player,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameEffect {
    PlayerMoved {
        player: PlayerId,
        state: PlayerState,
        correction: Option<MovementCorrection>,
        state_changed: bool,
    },
    BlockActionProcessed {
        player: PlayerId,
        action_id: BlockActionId,
        change: Option<BlockChange>,
    },
    AttackProcessed {
        attacker: PlayerId,
        target: PlayerId,
        result: AttackResult,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementCorrection {
    InvalidInput,
    TooFast,
    Collision,
    Void,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttackResult {
    Damaged {
        damage: f32,
        target: CombatState,
        killed: bool,
    },
    Rejected(AttackRejection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttackRejection {
    SelfTarget,
    UnknownTarget,
    DeadTarget,
    OutOfReach,
    Obstructed,
    HurtProtected,
}

struct Combatant {
    state: CombatState,
    last_attack_tick: Option<u64>,
    hurt_protected_until: u64,
    last_damage: f32,
}

impl Default for Combatant {
    fn default() -> Self {
        Self {
            state: CombatState::default(),
            last_attack_tick: None,
            hurt_protected_until: 0,
            last_damage: 0.0,
        }
    }
}

pub struct GameCore {
    tick: u64,
    spawn: PlayerState,
    build_height: BuildHeight,
    players: HashMap<PlayerId, PlayerState>,
    combatants: HashMap<PlayerId, Combatant>,
    inputs: VecDeque<PlayerInput>,
}

impl GameCore {
    #[must_use]
    pub fn new(spawn: PlayerState, build_height: BuildHeight) -> Self {
        Self {
            tick: 0,
            spawn,
            build_height,
            players: HashMap::new(),
            combatants: HashMap::new(),
            inputs: VecDeque::new(),
        }
    }

    #[must_use]
    pub const fn tick_number(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    #[must_use]
    pub fn player(&self, player: PlayerId) -> Option<PlayerState> {
        self.players.get(&player).copied()
    }

    #[must_use]
    pub fn combat_state(&self, player: PlayerId) -> Option<CombatState> {
        self.combatants
            .get(&player)
            .map(|combatant| combatant.state)
    }

    pub fn join(&mut self, player: PlayerId, state: PlayerState) {
        self.players.insert(player, state);
        self.combatants.insert(player, Combatant::default());
    }

    pub fn set_spawn(&mut self, spawn: PlayerState) {
        self.spawn = spawn;
    }

    pub fn leave(&mut self, player: PlayerId) -> Option<PlayerState> {
        self.inputs.retain(|input| input.player() != player);
        self.combatants.remove(&player);
        self.players.remove(&player)
    }

    pub fn enqueue(&mut self, input: PlayerInput) {
        self.inputs.push_back(input);
    }

    pub fn tick(&mut self, tick: u64, world: &mut impl WorldView) -> Vec<GameEffect> {
        self.tick = tick;
        let mut effects = Vec::new();
        let mut pending_blocks = HashMap::new();
        let mut movement_origins = HashMap::new();

        while let Some(input) = self.inputs.pop_front() {
            match input {
                PlayerInput::Move { player, movement } => {
                    let Some(previous) = self.players.get(&player).copied() else {
                        continue;
                    };
                    // The client ground bit is advisory. Authoritative state is
                    // derived from the imported block collision shapes below.
                    let _client_on_ground = movement.on_ground;
                    let input_is_valid = movement.position.is_none_or(valid_position)
                        && movement.rotation.is_none_or(valid_rotation);
                    if !input_is_valid {
                        effects.push(GameEffect::PlayerMoved {
                            player,
                            state: previous,
                            correction: Some(MovementCorrection::InvalidInput),
                            state_changed: false,
                        });
                        continue;
                    }

                    let desired_movement = movement
                        .position
                        .map(|position| subtract_position(position, previous.position));
                    let tick_origin = *movement_origins.entry(player).or_insert(previous.position);
                    if movement.position.is_some_and(|position| {
                        movement_squared(subtract_position(position, tick_origin))
                            > MAX_MOVEMENT_SQUARED
                    }) {
                        effects.push(GameEffect::PlayerMoved {
                            player,
                            state: previous,
                            correction: Some(MovementCorrection::TooFast),
                            state_changed: false,
                        });
                        continue;
                    }

                    let mut state = previous;
                    let mut correction = None;
                    if let Some(desired) = desired_movement {
                        let resolved =
                            resolve_movement(world, &pending_blocks, previous.position, desired);
                        state.position = add_position(previous.position, resolved);
                        if movement_differs(desired, resolved) {
                            correction = Some(MovementCorrection::Collision);
                        }
                        let landed = desired.y < 0.0 && resolved.y > desired.y + COLLISION_EPSILON;
                        state.on_ground =
                            landed || is_on_ground(world, &pending_blocks, state.position);
                    } else {
                        state.on_ground = is_on_ground(world, &pending_blocks, state.position);
                    }
                    if let Some(rotation) = movement.rotation {
                        state.rotation = rotation;
                    }
                    if state.position.y < VOID_FLOOR_Y {
                        state = self.spawn;
                        correction = Some(MovementCorrection::Void);
                    }
                    let state_changed = state != previous;
                    self.players.insert(player, state);
                    if desired_movement.is_some() && state_changed {
                        if let Some(combatant) = self.combatants.get_mut(&player) {
                            // Until velocity integration lands, an accepted client
                            // position acknowledges the last impulse.
                            combatant.state.velocity = Velocity::default();
                        }
                    }
                    effects.push(GameEffect::PlayerMoved {
                        player,
                        state,
                        correction,
                        state_changed,
                    });
                }
                PlayerInput::Attack { player, target } => {
                    let Some(attacker_state) = self.players.get(&player).copied() else {
                        continue;
                    };
                    let rejection = if player == target {
                        Some(AttackRejection::SelfTarget)
                    } else if !self.players.contains_key(&target)
                        || !self.combatants.contains_key(&target)
                    {
                        Some(AttackRejection::UnknownTarget)
                    } else if self
                        .combatants
                        .get(&target)
                        .is_some_and(|combatant| combatant.state.health <= 0.0)
                    {
                        Some(AttackRejection::DeadTarget)
                    } else {
                        None
                    };
                    if let Some(rejection) = rejection {
                        effects.push(GameEffect::AttackProcessed {
                            attacker: player,
                            target,
                            result: AttackResult::Rejected(rejection),
                        });
                        continue;
                    }

                    let target_state = self.players[&target];
                    let eye = Position {
                        x: attacker_state.position.x,
                        y: attacker_state.position.y + PLAYER_EYE_HEIGHT,
                        z: attacker_state.position.z,
                    };
                    let target_point = nearest_point(eye, Aabb::player(target_state.position));
                    if movement_squared(subtract_position(target_point, eye))
                        > PLAYER_ATTACK_REACH * PLAYER_ATTACK_REACH
                    {
                        effects.push(GameEffect::AttackProcessed {
                            attacker: player,
                            target,
                            result: AttackResult::Rejected(AttackRejection::OutOfReach),
                        });
                        continue;
                    }
                    if !has_line_of_sight(world, &pending_blocks, eye, target_point) {
                        effects.push(GameEffect::AttackProcessed {
                            attacker: player,
                            target,
                            result: AttackResult::Rejected(AttackRejection::Obstructed),
                        });
                        continue;
                    }

                    let damage = {
                        let attacker = self
                            .combatants
                            .get_mut(&player)
                            .expect("joined players have combat state");
                        let damage = attack_damage(tick, attacker.last_attack_tick);
                        attacker.last_attack_tick = Some(tick);
                        damage
                    };
                    let victim = self
                        .combatants
                        .get_mut(&target)
                        .expect("target combat state checked above");
                    let protected = tick < victim.hurt_protected_until;
                    if protected && damage <= victim.last_damage {
                        effects.push(GameEffect::AttackProcessed {
                            attacker: player,
                            target,
                            result: AttackResult::Rejected(AttackRejection::HurtProtected),
                        });
                        continue;
                    }

                    let applied_damage = if protected {
                        damage - victim.last_damage
                    } else {
                        victim.hurt_protected_until = tick + HURT_PROTECTION_TICKS;
                        victim.state.velocity =
                            knockback(attacker_state, target_state, victim.state.velocity);
                        damage
                    };
                    victim.last_damage = damage;
                    victim.state.health =
                        (victim.state.health - applied_damage).clamp(0.0, victim.state.max_health);
                    effects.push(GameEffect::AttackProcessed {
                        attacker: player,
                        target,
                        result: AttackResult::Damaged {
                            damage: applied_damage,
                            target: victim.state,
                            killed: victim.state.health <= 0.0,
                        },
                    });
                }
                PlayerInput::DigBlock {
                    player,
                    position,
                    break_block,
                    action_id,
                } => {
                    if !self.players.contains_key(&player) {
                        continue;
                    }
                    let change = (break_block
                        && self.build_height.contains(position.y)
                        && observed_block_state(world, &pending_blocks, position)
                            .and_then(block_data::state)
                            .is_some_and(|state| !state.is_air()))
                    .then_some(BlockChange {
                        position,
                        state: AIR_BLOCK_STATE,
                    });
                    if let Some(change) = change {
                        pending_blocks.insert(change.position, change.state);
                    }
                    effects.push(GameEffect::BlockActionProcessed {
                        player,
                        action_id,
                        change,
                    });
                }
                PlayerInput::PlaceBlock {
                    player,
                    clicked,
                    face,
                    state,
                    action_id,
                } => {
                    if !self.players.contains_key(&player) {
                        continue;
                    }
                    let change = state.and_then(|state| {
                        if block_data::state(state).is_none_or(|metadata| metadata.is_air()) {
                            return None;
                        }
                        let clicked_is_replaceable =
                            observed_block_state(world, &pending_blocks, clicked)
                                .and_then(block_data::state)
                                .is_some_and(block_data::BlockStateMetadata::replaceable);
                        let position = if clicked_is_replaceable {
                            clicked
                        } else {
                            clicked.offset(face?)
                        };
                        if !self.build_height.contains(position.y) {
                            return None;
                        }
                        observed_block_state(world, &pending_blocks, position)
                            .and_then(block_data::state)
                            .is_some_and(block_data::BlockStateMetadata::replaceable)
                            .then_some(BlockChange { position, state })
                    });
                    if let Some(change) = change {
                        pending_blocks.insert(change.position, change.state);
                    }
                    effects.push(GameEffect::BlockActionProcessed {
                        player,
                        action_id,
                        change,
                    });
                }
            }
        }

        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE: PlayerId = PlayerId([1; 16]);
    const BOB: PlayerId = PlayerId([2; 16]);

    #[derive(Default)]
    struct TestWorld {
        blocks: HashMap<BlockPosition, u16>,
    }

    impl TestWorld {
        fn with_block(mut self, position: BlockPosition, state: u16) -> Self {
            self.blocks.insert(position, state);
            self
        }
    }

    impl WorldView for TestWorld {
        fn block_state(&mut self, position: BlockPosition) -> Option<u16> {
            Some(
                self.blocks
                    .get(&position)
                    .copied()
                    .unwrap_or(AIR_BLOCK_STATE),
            )
        }
    }

    const fn build_height() -> BuildHeight {
        BuildHeight {
            min_y: -64,
            max_y_exclusive: 320,
        }
    }

    fn state(x: f64, y: f64, z: f64) -> PlayerState {
        PlayerState {
            position: Position { x, y, z },
            rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
            on_ground: false,
        }
    }

    fn movement(x: f64, y: f64, z: f64) -> PlayerInput {
        PlayerInput::Move {
            player: ALICE,
            movement: MovementInput {
                position: Some(Position { x, y, z }),
                rotation: None,
                on_ground: true,
            },
        }
    }

    fn attack(target: PlayerId) -> PlayerInput {
        PlayerInput::Attack {
            player: ALICE,
            target,
        }
    }

    #[test]
    fn movement_is_queued_until_the_next_tick() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(4.0, -59.0, 7.0));

        assert_eq!(game.player(ALICE), Some(spawn));
        let effects = game.tick(1, &mut TestWorld::default());

        assert_eq!(game.player(ALICE), Some(state(4.0, -59.0, 7.0)));
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn the_same_trace_replays_to_the_same_state_and_effects() {
        fn replay() -> (PlayerState, Vec<GameEffect>) {
            let spawn = state(0.5, -60.0, 0.5);
            let mut game = GameCore::new(spawn, build_height());
            game.join(ALICE, spawn);
            game.enqueue(movement(1.0, -59.0, 1.0));
            game.enqueue(PlayerInput::DigBlock {
                player: ALICE,
                position: BlockPosition { x: 2, y: -61, z: 3 },
                break_block: true,
                action_id: BlockActionId(10),
            });
            game.enqueue(movement(2.0, -59.0, 3.0));
            let mut world =
                TestWorld::default().with_block(BlockPosition { x: 2, y: -61, z: 3 }, 10);
            let effects = game.tick(25, &mut world);
            (game.player(ALICE).expect("player exists"), effects)
        }

        assert_eq!(replay(), replay());
    }

    #[test]
    fn falling_below_the_world_returns_the_player_to_spawn() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, state(0.5, -69.0, 0.5));
        game.enqueue(movement(0.5, -71.0, 0.5));

        assert_eq!(
            game.tick(1, &mut TestWorld::default()),
            vec![GameEffect::PlayerMoved {
                player: ALICE,
                state: spawn,
                correction: Some(MovementCorrection::Void),
                state_changed: true,
            }]
        );
    }

    #[test]
    fn horizontal_movement_stops_at_a_full_block() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(1.5, 0.0, 0.5));
        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 1, y: 0, z: 0 }, block_data::STONE);

        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::PlayerMoved {
                player: ALICE,
                state: state(0.7, 0.0, 0.5),
                correction: Some(MovementCorrection::Collision),
                state_changed: true,
            }]
        );
    }

    #[test]
    fn falling_onto_a_block_derives_grounded_state() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(0.5, -1.0, 0.5));
        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 0, y: -1, z: 0 }, block_data::STONE);

        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::PlayerMoved {
                player: ALICE,
                state: PlayerState {
                    on_ground: true,
                    ..spawn
                },
                correction: Some(MovementCorrection::Collision),
                state_changed: true,
            }]
        );
    }

    #[test]
    fn partial_height_shapes_are_used_for_collision() {
        let spawn = state(0.5, 0.5, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(0.5, -1.0, 0.5));
        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 0, y: -1, z: 0 }, 13_399);

        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::PlayerMoved {
                player: ALICE,
                state: PlayerState {
                    position: Position {
                        x: 0.5,
                        y: -0.5,
                        z: 0.5,
                    },
                    on_ground: true,
                    ..spawn
                },
                correction: Some(MovementCorrection::Collision),
                state_changed: true,
            }]
        );
    }

    #[test]
    fn implausible_and_non_finite_movement_is_rejected() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(100.0, 0.0, 0.5));
        game.enqueue(movement(f64::NAN, 0.0, 0.5));

        assert_eq!(
            game.tick(1, &mut TestWorld::default()),
            vec![
                GameEffect::PlayerMoved {
                    player: ALICE,
                    state: spawn,
                    correction: Some(MovementCorrection::TooFast),
                    state_changed: false,
                },
                GameEffect::PlayerMoved {
                    player: ALICE,
                    state: spawn,
                    correction: Some(MovementCorrection::InvalidInput),
                    state_changed: false,
                },
            ]
        );
        assert_eq!(game.player(ALICE), Some(spawn));
    }

    #[test]
    fn movement_limit_is_cumulative_within_a_tick() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(9.5, 0.0, 0.5));
        game.enqueue(movement(18.5, 0.0, 0.5));

        let effects = game.tick(1, &mut TestWorld::default());
        assert_eq!(game.player(ALICE), Some(state(9.5, 0.0, 0.5)));
        assert_eq!(
            effects[1],
            GameEffect::PlayerMoved {
                player: ALICE,
                state: state(9.5, 0.0, 0.5),
                correction: Some(MovementCorrection::TooFast),
                state_changed: false,
            }
        );
    }

    #[test]
    fn movement_observes_blocks_placed_earlier_in_the_same_tick() {
        let spawn = state(0.5, 0.0, 0.5);
        let placed = BlockPosition { x: 1, y: 0, z: 0 };
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::PlaceBlock {
            player: ALICE,
            clicked: placed,
            face: None,
            state: Some(block_data::STONE),
            action_id: BlockActionId(41),
        });
        game.enqueue(movement(1.5, 0.0, 0.5));

        let effects = game.tick(1, &mut TestWorld::default());
        assert!(matches!(
            effects[0],
            GameEffect::BlockActionProcessed {
                change: Some(BlockChange { position, .. }),
                ..
            } if position == placed
        ));
        assert_eq!(game.player(ALICE), Some(state(0.7, 0.0, 0.5)));
        assert!(matches!(
            effects[1],
            GameEffect::PlayerMoved {
                correction: Some(MovementCorrection::Collision),
                ..
            }
        ));
    }

    #[test]
    fn leaving_discards_queued_inputs() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(movement(4.0, -59.0, 7.0));
        game.enqueue(PlayerInput::DigBlock {
            player: ALICE,
            position: BlockPosition { x: 2, y: -61, z: 3 },
            break_block: true,
            action_id: BlockActionId(10),
        });
        assert_eq!(game.leave(ALICE), Some(spawn));
        assert!(game.tick(1, &mut TestWorld::default()).is_empty());
    }

    #[test]
    fn block_actions_are_queued_and_observe_same_tick_changes_in_order() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::PlaceBlock {
            player: ALICE,
            clicked: BlockPosition { x: 2, y: -62, z: 3 },
            face: Some(BlockFace::Top),
            state: Some(10),
            action_id: BlockActionId(20),
        });
        game.enqueue(PlayerInput::DigBlock {
            player: ALICE,
            position: BlockPosition { x: 2, y: -61, z: 3 },
            break_block: true,
            action_id: BlockActionId(21),
        });

        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 2, y: -62, z: 3 }, block_data::DIRT);
        assert_eq!(
            game.tick(4, &mut world),
            vec![
                GameEffect::BlockActionProcessed {
                    player: ALICE,
                    action_id: BlockActionId(20),
                    change: Some(BlockChange {
                        position: BlockPosition { x: 2, y: -61, z: 3 },
                        state: 10,
                    }),
                },
                GameEffect::BlockActionProcessed {
                    player: ALICE,
                    action_id: BlockActionId(21),
                    change: Some(BlockChange {
                        position: BlockPosition { x: 2, y: -61, z: 3 },
                        state: AIR_BLOCK_STATE,
                    }),
                },
            ]
        );
    }

    #[test]
    fn invalid_block_actions_are_acknowledged_without_a_change() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::PlaceBlock {
            player: ALICE,
            clicked: BlockPosition { x: 0, y: 319, z: 0 },
            face: Some(BlockFace::Top),
            state: Some(10),
            action_id: BlockActionId(30),
        });

        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 0, y: 319, z: 0 }, block_data::DIRT);
        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::BlockActionProcessed {
                player: ALICE,
                action_id: BlockActionId(30),
                change: None,
            }]
        );
    }

    #[test]
    fn world_queries_reject_breaking_air_and_placing_into_an_occupied_block() {
        let spawn = state(0.5, -60.0, 0.5);
        let occupied = BlockPosition { x: 4, y: -61, z: 5 };
        let mut world = TestWorld::default()
            .with_block(occupied, block_data::GRASS_BLOCK)
            .with_block(BlockPosition { x: 4, y: -62, z: 5 }, block_data::DIRT);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::DigBlock {
            player: ALICE,
            position: BlockPosition { x: 3, y: -61, z: 5 },
            break_block: true,
            action_id: BlockActionId(31),
        });
        game.enqueue(PlayerInput::PlaceBlock {
            player: ALICE,
            clicked: BlockPosition { x: 4, y: -62, z: 5 },
            face: Some(BlockFace::Top),
            state: Some(10),
            action_id: BlockActionId(32),
        });

        assert_eq!(
            game.tick(1, &mut world),
            vec![
                GameEffect::BlockActionProcessed {
                    player: ALICE,
                    action_id: BlockActionId(31),
                    change: None,
                },
                GameEffect::BlockActionProcessed {
                    player: ALICE,
                    action_id: BlockActionId(32),
                    change: None,
                },
            ]
        );
    }

    #[test]
    fn placement_replaces_replaceable_blocks_at_the_clicked_position() {
        let spawn = state(0.5, -60.0, 0.5);
        let clicked = BlockPosition { x: 7, y: -60, z: 8 };
        let mut world = TestWorld::default().with_block(clicked, 2_248);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::PlaceBlock {
            player: ALICE,
            clicked,
            face: Some(BlockFace::Top),
            state: Some(block_data::DIRT),
            action_id: BlockActionId(33),
        });

        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::BlockActionProcessed {
                player: ALICE,
                action_id: BlockActionId(33),
                change: Some(BlockChange {
                    position: clicked,
                    state: block_data::DIRT,
                }),
            }]
        );
    }

    #[test]
    fn inputs_from_unknown_players_have_no_effect() {
        let spawn = state(0.5, -60.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(PlayerInput::DigBlock {
            player: BOB,
            position: BlockPosition { x: 2, y: -61, z: 3 },
            break_block: true,
            action_id: BlockActionId(40),
        });

        assert!(game.tick(1, &mut TestWorld::default()).is_empty());
    }

    #[test]
    fn a_recharged_fist_attack_applies_damage_and_pumpkin_knockback() {
        let spawn = state(0.5, 0.0, 0.5);
        let target_state = PlayerState {
            on_ground: true,
            ..state(2.0, 0.0, 0.5)
        };
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.join(BOB, target_state);
        game.enqueue(attack(BOB));

        assert_eq!(
            game.tick(1, &mut TestWorld::default()),
            vec![GameEffect::AttackProcessed {
                attacker: ALICE,
                target: BOB,
                result: AttackResult::Damaged {
                    damage: 1.0,
                    target: CombatState {
                        health: 19.0,
                        max_health: 20.0,
                        velocity: Velocity {
                            x: 0.0,
                            y: 0.4,
                            z: 0.5,
                        },
                    },
                    killed: false,
                },
            }]
        );
    }

    #[test]
    fn fist_damage_uses_pumpkins_attack_speed_recharge_curve() {
        assert_eq!(attack_damage(1, None), 1.0);
        assert!((attack_damage(1, Some(1)) - 0.201_28).abs() < 1.0e-6);
        assert_eq!(attack_damage(14, Some(1)), 1.0);
    }

    #[test]
    fn hurt_protection_rejects_an_immediate_weaker_attack() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.join(BOB, state(2.0, 0.0, 0.5));
        game.enqueue(attack(BOB));
        game.enqueue(attack(BOB));

        let effects = game.tick(1, &mut TestWorld::default());
        assert!(matches!(
            effects[0],
            GameEffect::AttackProcessed {
                result: AttackResult::Damaged { damage: 1.0, .. },
                ..
            }
        ));
        assert_eq!(
            effects[1],
            GameEffect::AttackProcessed {
                attacker: ALICE,
                target: BOB,
                result: AttackResult::Rejected(AttackRejection::HurtProtected),
            }
        );
        assert_eq!(game.combat_state(BOB).unwrap().health, 19.0);
    }

    #[test]
    fn attack_reach_is_measured_to_the_target_hitbox() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.join(BOB, state(4.0, 0.0, 0.5));
        game.enqueue(attack(BOB));

        assert_eq!(
            game.tick(1, &mut TestWorld::default()),
            vec![GameEffect::AttackProcessed {
                attacker: ALICE,
                target: BOB,
                result: AttackResult::Rejected(AttackRejection::OutOfReach),
            }]
        );
    }

    #[test]
    fn solid_block_shapes_obstruct_attacks() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.join(BOB, state(2.5, 0.0, 0.5));
        game.enqueue(attack(BOB));
        let mut world =
            TestWorld::default().with_block(BlockPosition { x: 1, y: 1, z: 0 }, block_data::STONE);

        assert_eq!(
            game.tick(1, &mut world),
            vec![GameEffect::AttackProcessed {
                attacker: ALICE,
                target: BOB,
                result: AttackResult::Rejected(AttackRejection::Obstructed),
            }]
        );
    }

    #[test]
    fn self_and_unknown_targets_are_rejected_deterministically() {
        let spawn = state(0.5, 0.0, 0.5);
        let mut game = GameCore::new(spawn, build_height());
        game.join(ALICE, spawn);
        game.enqueue(attack(ALICE));
        game.enqueue(attack(BOB));

        assert_eq!(
            game.tick(1, &mut TestWorld::default()),
            vec![
                GameEffect::AttackProcessed {
                    attacker: ALICE,
                    target: ALICE,
                    result: AttackResult::Rejected(AttackRejection::SelfTarget),
                },
                GameEffect::AttackProcessed {
                    attacker: ALICE,
                    target: BOB,
                    result: AttackResult::Rejected(AttackRejection::UnknownTarget),
                },
            ]
        );
    }
}
