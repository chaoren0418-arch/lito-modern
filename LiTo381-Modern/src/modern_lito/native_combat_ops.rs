//! Non-skill combat operation rules (T029 2/2): native empty swing and its
//! refusal conditions, attack-only pacing, auto-chase steps and target ring.
//!
//! Sources (read-only, lito-launcher-server):
//! - `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/EmptyMelee.java`
//!   (0.6.49, `docs/maintenance/empty-melee-0.6.49.json` L34–42)
//! - `BattleSpellCatalog.isInNativeScreen`, `BattleInventoryWeight.BUFFED_REGEN_LIMIT`,
//!   `BattleAnimationCatalog.BOW_WALK / THROWING_KNIFE_WALK`.
//! The server stays authoritative: these gates only avoid sending requests
//! the formal server is known to drop silently, and explain why.
use super::native_world::World;

/// weight240 at or above this value: formal attack paths send S_MESSAGE_CODE 110.
pub const OVERWEIGHT_GAUGE: u8 = 197;
/// Bounded chase: same budget as the isolated combat probe route (64 steps).
pub const CHASE_STEP_LIMIT: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Refusal {
    BattleEnded,
    NotReady,
    Dead,
    Observer,
    Invisible,
    Paralyzed,
    Overweight,
    RangedStance,
    InvalidCoordinate,
    Pacing,
}
impl Refusal {
    pub fn text(self) -> &'static str {
        match self {
            Self::BattleEnded => "戰鬥已結束，無法攻擊。",
            Self::NotReady => "角色尚未就緒，無法攻擊。",
            Self::Dead => "角色已死亡，無法攻擊。",
            Self::Observer => "觀戰中無法攻擊。",
            Self::Invisible => "隱身中無法空揮。",
            Self::Paralyzed => "行動受限中，無法攻擊。",
            Self::Overweight => "負重過重，無法攻擊。",
            Self::RangedStance => "持弓／投擲武器時不能空揮。",
            Self::InvalidCoordinate => "攻擊方向的座標不合法。",
            Self::Pacing => "攻擊動作尚未結束。",
        }
    }
}

/// Mirror of formal `BattleSpellCatalog.isInNativeScreen`.
pub fn in_native_screen(sx: u16, sy: u16, tx: u16, ty: u16) -> bool {
    let (sx, sy, tx, ty) = (i32::from(sx), i32::from(sy), i32::from(tx), i32::from(ty));
    let distance = (tx - sx).abs() + (ty - sy).abs();
    if distance > 19 {
        return false;
    }
    if distance <= 18 {
        return true;
    }
    let edge = (tx - (sx - 18)).abs() + (ty - (sy - 18)).abs();
    (19..=52).contains(&edge)
}

/// Local preflight for the native target-zero swing, in the formal
/// EmptyMelee order. `observer` is the controller's observer flag.
/// Returns the coordinate one cell ahead in `heading`.
pub fn empty_melee_target(world: &World, observer: bool, heading: u8) -> Result<[u16; 2], Refusal> {
    if world.battle_result.is_some() {
        return Err(Refusal::BattleEnded);
    }
    if observer || world.observer_mode {
        return Err(Refusal::Observer);
    }
    if !world.ready() {
        return Err(Refusal::NotReady);
    }
    let own = world.own_actor().ok_or(Refusal::NotReady)?;
    if own.dead || !world.hp.is_some_and(|hp| hp[0] > 0) {
        return Err(Refusal::Dead);
    }
    if own.draw_state.invisible {
        return Err(Refusal::Invisible);
    }
    if !world.movement_blocks.is_empty() {
        return Err(Refusal::Paralyzed);
    }
    if matches!(own.walk_action, 20 | 62) {
        return Err(Refusal::RangedStance);
    }
    let (x, y) = super::native_world::destination(own.x, own.y, heading)
        .map_err(|_| Refusal::InvalidCoordinate)?;
    if x == 0 || y == 0 || (x == own.x && y == own.y) || !in_native_screen(own.x, own.y, x, y) {
        return Err(Refusal::InvalidCoordinate);
    }
    if world.status.is_some_and(|s| s.weight_gauge >= OVERWEIGHT_GAUGE) {
        return Err(Refusal::Overweight);
    }
    Ok([x, y])
}

pub fn empty_melee_request(world: &World, observer: bool, heading: u8) -> Result<Vec<u8>, Refusal> {
    let [x, y] = empty_melee_target(world, observer, heading)?;
    Ok(super::native_combat::empty_request(x, y))
}

/// Attack-only interval (formal EmptyMelee.interval, which cites
/// ActionIntervalCalculator's ATTACK multipliers). Movement-only brave modes
/// (e.g. 4) do not shorten attacks. `base_ms` is the attack action duration.
pub fn attack_interval_ms(base_ms: u32, speed: u8, brave: u8) -> u32 {
    let base = i64::from(base_ms);
    let mut ms: i64 = match speed {
        2 => base * 2,
        1 => (base as f64 * 0.755) as i64,
        _ => base,
    };
    if brave == 6 {
        ms = (ms as f64 * 0.775) as i64;
    }
    if brave == 1 || brave == 5 {
        ms = (ms as f64 * 0.755) as i64;
    }
    if brave == 3 {
        ms = (((ms as f64 * 0.855) as i64) as f64 * 0.9) as i64;
    }
    ms.clamp(1, i64::from(u32::MAX)) as u32
}

/// Client-side copy of the session-only empty-swing deadline. A swing that
/// the server would drop inside its interval is refused locally instead.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pacer {
    ready_at: Option<std::time::Instant>,
}
impl Pacer {
    pub fn admit(&self, now: std::time::Instant) -> Result<(), Refusal> {
        match self.ready_at {
            Some(at) if now < at => Err(Refusal::Pacing),
            _ => Ok(()),
        }
    }
    pub fn commit(&mut self, now: std::time::Instant, interval_ms: u32) {
        self.ready_at = Some(now + std::time::Duration::from_millis(u64::from(interval_ms)));
    }
    pub fn clear(&mut self) {
        self.ready_at = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChaseStep {
    InReach,
    Step(u8),
    Blocked,
}
const DIR: [(i32, i32); 8] = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
fn chebyshev(a: [i32; 2], b: [i32; 2]) -> i32 {
    (a[0] - b[0]).abs().max((a[1] - b[1]).abs())
}
/// One greedy step that strictly reduces Chebyshev distance. Tries the direct
/// heading, then its two neighbours. `can_step(x, y, heading)` is the caller's
/// terrain/obstacle gate from the current cell. No pathfinding, no prediction.
pub fn chase_step(own: [u16; 2], target: [u16; 2], reach: u16, can_step: impl Fn(i32, i32, u8) -> bool) -> ChaseStep {
    let own = [i32::from(own[0]), i32::from(own[1])];
    let target = [i32::from(target[0]), i32::from(target[1])];
    let distance = chebyshev(own, target);
    if distance <= i32::from(reach) {
        return ChaseStep::InReach;
    }
    let direct = ((target[0] - own[0]).signum(), (target[1] - own[1]).signum());
    let Some(h) = DIR.iter().position(|d| *d == direct) else { return ChaseStep::InReach; };
    let h = h as u8;
    for heading in [h, (h + 7) % 8, (h + 1) % 8] {
        let (dx, dy) = DIR[usize::from(heading)];
        let next = [own[0] + dx, own[1] + dy];
        if chebyshev(next, target) < distance && can_step(own[0], own[1], heading) {
            return ChaseStep::Step(heading);
        }
    }
    ChaseStep::Blocked
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RingTone {
    /// All local gates pass and the target is within reach.
    InReach,
    /// All local gates pass; the target is out of reach (chase possible).
    OutOfReach,
    /// Target is dead or destroyed.
    Dead,
    /// Selected but not attackable by local gates (self, door, invisible…).
    Neutral,
}
impl RingTone {
    /// Display colour (R, G, B). Presentation choice of this client, not a
    /// recovered original palette.
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Self::InReach => [230, 60, 50],
            Self::OutOfReach => [240, 190, 60],
            Self::Dead => [120, 120, 120],
            Self::Neutral => [90, 170, 230],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct TargetRing {
    pub id: u32,
    pub cell: [u16; 2],
    pub tone: RingTone,
}
pub fn target_ring(world: &World, target: Option<u32>) -> Option<TargetRing> {
    let id = target?;
    let actor = world.actors.get(&id)?;
    if !world.actor_visible(id) {
        return None;
    }
    let tone = if actor.dead || super::native_obstacles::destroyed_guardian(actor) {
        RingTone::Dead
    } else {
        match world.attack_reach(id) {
            Some((distance, reach)) if distance <= reach => RingTone::InReach,
            Some(_) => RingTone::OutOfReach,
            None => RingTone::Neutral,
        }
    };
    Some(TargetRing { id, cell: [actor.x, actor.y], tone })
}
/// Ellipse bounds (left, top, right, bottom) in 800×600 frame pixels around
/// the projected cell centre: one 48×24 tile diamond.
pub fn ring_bounds(projection: super::world_map::ViewProjection, cell: [u16; 2]) -> Option<[i32; 4]> {
    let p = projection.cell_to_pixel(super::world_map::WorldCell { x: i32::from(cell[0]), y: i32::from(cell[1]) })?;
    let bounds = [p.x - 24, p.y - 12, p.x + 24, p.y + 12];
    (bounds[2] > 0 && bounds[3] > 0 && bounds[0] < 800 && bounds[1] < 600).then_some(bounds)
}

#[cfg(test)]
#[path = "native_combat_ops_tests.rs"]
mod tests;
