//! v14 self-centred Fire Storm. Receipts describe already committed damage;
//! neither the render Hit cue nor a local target scan may apply it again.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat, honor_protocol as honor,
    world_map::WorldCell,
    world_protocol::Phase,
};

pub const SKILL_ID: u32 = 70;
pub const STATE_BYTES: usize = 64;
pub const TARGET_BYTES: usize = 56;
pub const MAX_TARGETS: usize = 101;
pub const REUSE_MILLIS: u32 = 2000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub target_entity_id: u64,
    pub hp_before: i64,
    pub hp_after: i64,
    pub applied_damage: i64,
    pub damage_loss_after: i64,
    pub respawn_count: i64,
    pub position: WorldCell,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub count: i64,
    pub cast_revision: i64,
    pub mp_spent: i64,
    pub last_cast_sequence: u64,
    pub from: WorldCell,
    pub mp_cost: u32,
    pub reuse_millis: u32,
    pub targets: Vec<Target>,
}
impl State {
    pub fn initial(entity_id: u64, mp_cost: u32) -> Self {
        Self {
            entity_id,
            count: 0,
            cast_revision: 0,
            mp_spent: 0,
            last_cast_sequence: 0,
            from: WorldCell { x: 0, y: 0 },
            mp_cost,
            reuse_millis: if mp_cost == 0 { 0 } else { REUSE_MILLIS },
            targets: Vec::new(),
        }
    }
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.entity_id == 0
            || self.entity_id > i64::MAX as u64
            || self.count < 0
            || self.cast_revision < 0
            || self.mp_spent < 0
            || self.last_cast_sequence > i64::MAX as u64
            || self.count.checked_mul(i64::from(self.mp_cost)) != Some(self.mp_spent)
            || self.targets.len() > MAX_TARGETS
            || self.mp_cost != 0 && !(39..=48).contains(&self.mp_cost)
            || self.reuse_millis != if self.mp_cost == 0 { 0 } else { REUSE_MILLIS }
            || if self.count == 0 {
                *self != Self::initial(self.entity_id, self.mp_cost)
            } else {
                self.mp_cost == 0
                    || self.cast_revision == 0
                    || self.last_cast_sequence < self.count as u64
            }
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for t in &self.targets {
            if t.target_entity_id <= previous
                || t.target_entity_id == self.entity_id
                || t.target_entity_id > i64::MAX as u64
                || t.hp_before <= 0
                || t.hp_after < 0
                || t.hp_after > t.hp_before
                || t.applied_damage != t.hp_before - t.hp_after
                || t.damage_loss_after < t.applied_damage
                || t.respawn_count < 0
                || !in_radius(self.from, t.position)
            {
                return Err(invalid());
            }
            previous = t.target_entity_id;
        }
        Ok(())
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative Fire Storm",
    )
}
pub fn in_radius(a: WorldCell, b: WorldCell) -> bool {
    (i64::from(a.x) - i64::from(b.x))
        .abs()
        .max((i64::from(a.y) - i64::from(b.y)).abs())
        <= 3
}
pub fn revision(f: &combat::Frame, id: u64) -> i64 {
    f.fire(id).map_or(0, |s| s.cast_revision)
}
pub fn spent(f: &combat::Frame, id: u64) -> i64 {
    f.fire(id).map_or(0, |s| s.mp_spent)
}
pub fn damage_loss(f: &combat::Frame, id: u64) -> Option<i64> {
    f.regen(id)
        .map(|s| s.damage_loss)
        .or_else(|| f.honor.as_ref()?.objective(id).map(|o| o.damage_loss))
}
pub fn life_count(f: &combat::Frame, id: u64) -> i64 {
    f.honor
        .as_ref()
        .and_then(|h| h.life(id))
        .map_or(0, |l| l.respawn_count)
}
fn max_hp(f: &combat::Frame, id: u64) -> Option<i64> {
    f.vital(id)
        .map(|v| v.max_hp)
        .or_else(|| f.honor.as_ref()?.objective(id).map(|o| o.max_hp))
}
fn total_casts(f: &combat::Frame, id: u64) -> i128 {
    i128::from(f.world.cast(id).map_or(0, |s| s.cast_count))
        + i128::from(f.damage(id).map_or(0, |s| s.cast_count))
        + i128::from(
            f.honor
                .as_ref()
                .and_then(|h| h.heal(id))
                .map_or(0, |s| s.count),
        )
        + i128::from(f.fire(id).map_or(0, |s| s.count))
        + i128::from(f.disintegrate(id).map_or(0, |s| s.count))
}
pub fn validate_frame(f: &combat::Frame) -> Result<(), ApplicationError> {
    if f.fires.is_empty() {
        return Ok(());
    }
    let n = f.world.actors.len();
    if f.fires.len() != n || f.regen.len() != n || f.world.casts.len() != n || f.weights.len() != n
    {
        return Err(invalid());
    }
    let mut revisions = std::collections::BTreeSet::new();
    for ((s, a), v) in f.fires.iter().zip(&f.world.actors).zip(&f.vitals) {
        s.validate()?;
        let c = f.world.cast(a.entity_id).ok_or_else(invalid)?;
        if s.entity_id != a.entity_id
            || s.cast_revision > f.world.revision
            || s.cast_revision > 0 && !revisions.insert(s.cast_revision)
            || s.last_cast_sequence > c.request_sequence
            || total_casts(f, a.entity_id) > i128::from(c.request_sequence)
            || (a.identity().character().is_none() && *s != State::initial(a.entity_id, 0))
            || (a.identity().character().is_some() && !(39..=48).contains(&s.mp_cost))
            || matches!(f.world.phase, Phase::Loading | Phase::Countdown) && s.count != 0
            || s.count > 0 && !f.world.manifest.contains(s.from)
            || s.cast_revision > 0
                && [
                    c.cast_revision,
                    f.damage(a.entity_id).map_or(0, |d| d.cast_revision as i64),
                    honor::heal_revision(f, a.entity_id),
                    f.item(a.entity_id).map_or(0, |i| i.use_revision),
                    v.attack_revision,
                    v.hurt_revision,
                    v.death_revision,
                    honor::respawn_revision(f, a.entity_id),
                ]
                .contains(&s.cast_revision)
            || v.death_revision > 0
                && s.cast_revision >= v.death_revision
                && (v.hp == 0 || s.cast_revision <= honor::respawn_revision(f, a.entity_id))
            || s.cast_revision == f.world.revision
                && (a.position != s.from || v.hp <= 0 || s.last_cast_sequence != c.request_sequence)
            || f.world.completion.as_ref().is_some_and(|c| {
                c.event
                    .engine_revision
                    .checked_add(1)
                    .is_none_or(|r| s.cast_revision > r)
            })
        {
            return Err(invalid());
        }
        for t in &s.targets {
            let loss = damage_loss(f, t.target_entity_id).ok_or_else(invalid)?;
            let hurt = honor::target_hurt_revision(f, t.target_entity_id).ok_or_else(invalid)?;
            let life = life_count(f, t.target_entity_id);
            let objective = f
                .honor
                .as_ref()
                .and_then(|h| h.objective(t.target_entity_id));
            let anchored_hp = if let Some(r) = f.regen(t.target_entity_id) {
                if r.hp_revision <= s.cast_revision
                    && f.item(t.target_entity_id)
                        .is_none_or(|i| i.use_revision <= s.cast_revision)
                {
                    let maximum = i128::from(max_hp(f, t.target_entity_id).ok_or_else(invalid)?);
                    Some(
                        maximum + i128::from(r.hp_gain) - i128::from(r.hp_loss)
                            + i128::from(r.item_gain)
                            + i128::from(t.respawn_count) * maximum
                            - i128::from(t.damage_loss_after),
                    )
                } else {
                    None
                }
            } else {
                objective
                    .filter(|o| {
                        o.heal_gain == 0
                            || f.honor.as_ref().is_some_and(|h| {
                                h.heals.iter().all(|h| h.revision <= s.cast_revision)
                            })
                    })
                    .map(|o| {
                        i128::from(o.max_hp) + i128::from(o.heal_gain)
                            - i128::from(t.damage_loss_after)
                    })
            };
            if max_hp(f, t.target_entity_id).is_none_or(|hp| t.hp_before > hp)
                || anchored_hp.is_some_and(|hp| hp != i128::from(t.hp_after))
                || !f.world.manifest.contains(t.position)
                || loss < t.damage_loss_after
                || t.respawn_count > life
                || objective.is_some() && t.respawn_count != 0
                || honor::target_team(f, s.entity_id) == honor::target_team(f, t.target_entity_id)
                || t.applied_damage > 0 && hurt < s.cast_revision
                || t.applied_damage == 0 && hurt == s.cast_revision
                || objective.is_some_and(|o| {
                    o.position != t.position
                        || t.applied_damage > 0 && o.state_revision < s.cast_revision
                })
                || t.hp_after == 0
                    && life == t.respawn_count
                    && honor::target_hp(f, t.target_entity_id) != Some(0)
                || s.cast_revision == f.world.revision
                    && (honor::target_hp(f, t.target_entity_id) != Some(t.hp_after)
                        || honor::target_position(f, t.target_entity_id) != Some(t.position)
                        || loss != t.damage_loss_after
                        || life != t.respawn_count
                        || t.applied_damage > 0 && hurt != s.cast_revision
                        || objective.is_some_and(|o| {
                            t.applied_damage > 0
                                && (o.last_kind != honor::Interaction::FireStorm
                                    || o.last_actor_entity_id != s.entity_id)
                        }))
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.fires.len() != b.fires.len() {
        return false;
    }
    if matches!(a.world.phase, Phase::Finished | Phase::Aborted) {
        return a.fires == b.fires;
    }
    for (x, y) in a.fires.iter().zip(&b.fires) {
        if x.entity_id != y.entity_id
            || x.mp_cost != y.mp_cost
            || x.reuse_millis != y.reuse_millis
            || y.count < x.count
            || y.cast_revision < x.cast_revision
            || y.mp_spent < x.mp_spent
            || y.last_cast_sequence < x.last_cast_sequence
            || y.count == x.count && x != y
            || y.count > x.count
                && (y.cast_revision <= a.world.revision
                    || y.last_cast_sequence <= x.last_cast_sequence
                    || a.vital(x.entity_id).is_some_and(|v| v.hp == 0)
                        && !honor::respawned(a, b, x.entity_id))
        {
            return false;
        }
        let (Some(ac), Some(bc)) = (a.world.cast(x.entity_id), b.world.cast(y.entity_id)) else {
            return false;
        };
        if y.count > x.count && y.last_cast_sequence <= ac.request_sequence {
            return false;
        }
        if total_casts(b, y.entity_id) - total_casts(a, x.entity_id)
            > i128::from(bc.request_sequence) - i128::from(ac.request_sequence)
        {
            return false;
        }
        if y.count > x.count {
            for t in &y.targets {
                let Some(old_loss) = damage_loss(a, t.target_entity_id) else {
                    return false;
                };
                if i128::from(t.damage_loss_after)
                    < i128::from(old_loss) + i128::from(t.applied_damage)
                    || t.respawn_count < life_count(a, t.target_entity_id)
                    || honor::target_hp(a, t.target_entity_id) == Some(0)
                        && t.respawn_count == life_count(a, t.target_entity_id)
                {
                    return false;
                }
            }
        }
    }
    // Coalesced casts can contain several latest receipts for the same victim.
    // Check their ordered cumulative checkpoints and sum all distinct damage.
    let mut receipts: std::collections::BTreeMap<u64, Vec<(i64, &Target)>> =
        std::collections::BTreeMap::new();
    for (x, y) in a.fires.iter().zip(&b.fires) {
        if y.count > x.count {
            for t in &y.targets {
                receipts
                    .entry(t.target_entity_id)
                    .or_default()
                    .push((y.cast_revision, t));
            }
        }
    }
    for (id, mut rows) in receipts {
        rows.sort_by_key(|r| r.0);
        let (Some(old_loss), Some(new_loss)) = (damage_loss(a, id), damage_loss(b, id)) else {
            return false;
        };
        let mut previous_loss = i128::from(old_loss);
        let mut previous_revision = 0;
        let mut minimum = 0i128;
        for (rev, t) in rows {
            if rev <= previous_revision
                || i128::from(t.damage_loss_after) < previous_loss + i128::from(t.applied_damage)
            {
                return false;
            }
            previous_revision = rev;
            previous_loss = i128::from(t.damage_loss_after);
            minimum += i128::from(t.applied_damage);
        }
        if i128::from(new_loss) - i128::from(old_loss) < minimum {
            return false;
        }
    }
    true
}
pub(crate) fn negotiated(response: &app::Response, enabled: bool) -> bool {
    let f = match response {
        app::Response::Combat(r) => r.frame.as_ref(),
        app::Response::Siege(r) => r.frame.as_ref(),
        app::Response::Cast(r) => r.frame.as_ref(),
        app::Response::Item(r) => r.frame.as_ref(),
        app::Response::Ammo(r) => r.frame.as_ref(),
        _ => return true,
    };
    f.is_none_or(|f| {
        if f.world.descriptor.is_some() {
            f.world.feature_presence_matches(
                super::world_protocol::MatchDescriptor::FIRE_STORM,
                !f.fires.is_empty(),
                enabled,
            )
        } else {
            f.fires.is_empty() || enabled
        }
    })
}
pub(crate) fn decode(bytes: &[u8], actors: usize) -> Result<(Vec<State>, usize), ApplicationError> {
    struct R<'a> {
        b: &'a [u8],
        p: usize,
    }
    impl<'a> R<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
            let e = self.p.checked_add(n).ok_or_else(invalid)?;
            let s = self.b.get(self.p..e).ok_or_else(invalid)?;
            self.p = e;
            Ok(s)
        }
        fn u16(&mut self) -> Result<u16, ApplicationError> {
            Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
        }
        fn u32(&mut self) -> Result<u32, ApplicationError> {
            Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
        }
        fn u64(&mut self) -> Result<u64, ApplicationError> {
            Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
        }
        fn i64(&mut self) -> Result<i64, ApplicationError> {
            Ok(self.u64()? as i64)
        }
        fn cell(&mut self) -> Result<WorldCell, ApplicationError> {
            Ok(WorldCell {
                x: self.u32()? as i32,
                y: self.u32()? as i32,
            })
        }
        fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
            if self.take(n)?.iter().any(|b| *b != 0) {
                Err(invalid())
            } else {
                Ok(())
            }
        }
    }
    let mut r = R { b: bytes, p: 0 };
    let count = usize::from(r.u16()?);
    r.zero(6)?;
    if count > 100 || count != 0 && count != actors || bytes.len() - r.p < count * STATE_BYTES {
        return Err(invalid());
    }
    let mut states = Vec::with_capacity(count);
    for _ in 0..count {
        let mut s = State {
            entity_id: r.u64()?,
            count: r.i64()?,
            cast_revision: r.i64()?,
            mp_spent: r.i64()?,
            last_cast_sequence: r.u64()?,
            from: r.cell()?,
            mp_cost: r.u32()?,
            reuse_millis: r.u32()?,
            targets: Vec::new(),
        };
        let targets = usize::from(r.u16()?);
        r.zero(6)?;
        if targets > MAX_TARGETS || bytes.len() - r.p < targets * TARGET_BYTES {
            return Err(invalid());
        }
        s.targets.reserve(targets);
        for _ in 0..targets {
            s.targets.push(Target {
                target_entity_id: r.u64()?,
                hp_before: r.i64()?,
                hp_after: r.i64()?,
                applied_damage: r.i64()?,
                damage_loss_after: r.i64()?,
                respawn_count: r.i64()?,
                position: r.cell()?,
            });
        }
        s.validate()?;
        states.push(s);
    }
    Ok((states, r.p))
}

#[cfg(test)]
pub(crate) fn encode(states: &[State], out: &mut Vec<u8>) {
    out.extend_from_slice(&(states.len() as u16).to_be_bytes());
    out.extend_from_slice(&[0; 6]);
    for s in states {
        for v in [
            s.entity_id,
            s.count as u64,
            s.cast_revision as u64,
            s.mp_spent as u64,
            s.last_cast_sequence,
        ] {
            out.extend_from_slice(&v.to_be_bytes());
        }
        out.extend_from_slice(&s.from.x.to_be_bytes());
        out.extend_from_slice(&s.from.y.to_be_bytes());
        out.extend_from_slice(&s.mp_cost.to_be_bytes());
        out.extend_from_slice(&s.reuse_millis.to_be_bytes());
        out.extend_from_slice(&(s.targets.len() as u16).to_be_bytes());
        out.extend_from_slice(&[0; 6]);
        for t in &s.targets {
            for v in [
                t.target_entity_id,
                t.hp_before as u64,
                t.hp_after as u64,
                t.applied_damage as u64,
                t.damage_loss_after as u64,
                t.respawn_count as u64,
            ] {
                out.extend_from_slice(&v.to_be_bytes());
            }
            out.extend_from_slice(&t.position.x.to_be_bytes());
            out.extend_from_slice(&t.position.y.to_be_bytes());
        }
    }
}

#[cfg(test)]
#[path = "fire_storm_protocol_tests.rs"]
pub(crate) mod tests;
