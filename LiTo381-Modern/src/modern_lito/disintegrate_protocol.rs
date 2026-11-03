//! v27 skill77 receipts. Authority commits damage once; rendering never applies
//! HP/MP or lawful deltas. No native pointer, SQL object-id target, or asset grant.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat, honor_protocol as honor,
    world_map::WorldCell,
    world_protocol::{MatchDescriptor, Phase},
};
pub const STATE_BYTES: usize = 152;
pub const MAX_ACTORS: usize = 100;
pub const SKILL_ID: u32 = 77;
pub const REUSE_MILLIS: u32 = 5000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub target_entity_id: u64,
    pub caster_life: i64,
    pub target_life: i64,
    pub from: WorldCell,
    pub to: WorldCell,
    pub hp_before: i64,
    pub hp_after: i64,
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub damage_loss_after: i64,
    pub critical: bool,
}
impl Hit {
    pub fn effect(&self) -> u32 {
        if self.critical {
            11748
        } else {
            11746
        }
    }
    fn validate(&self) -> Result<(), ApplicationError> {
        let range = (i64::from(self.from.x) - i64::from(self.to.x))
            .abs()
            .max((i64::from(self.from.y) - i64::from(self.to.y)).abs());
        if !positive(self.target_entity_id)
            || self.caster_life < 1
            || self.target_life < 1
            || !(1..=10).contains(&range)
            || self.hp_before < 1
            || self.hp_after < 0
            || self.hp_after > self.hp_before
            || self.raw_damage < 0
            || self.applied_damage != self.raw_damage.min(self.hp_before)
            || self.applied_damage != self.hp_before - self.hp_after
            || self.damage_loss_after < self.applied_damage
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub count: i64,
    pub revision: i64,
    pub mp_spent: i64,
    pub last_sequence: u64,
    pub initial_lawful: i64,
    pub lawful: i64,
    pub mp_cost: u32,
    pub reuse_millis: u32,
    pub hit: Option<Hit>,
}
impl State {
    pub fn initial(entity_id: u64, mp_cost: u32, lawful: i64) -> Self {
        Self {
            entity_id,
            count: 0,
            revision: 0,
            mp_spent: 0,
            last_sequence: 0,
            initial_lawful: lawful,
            lawful,
            mp_cost,
            reuse_millis: if mp_cost == 0 { 0 } else { REUSE_MILLIS },
            hit: None,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !positive(self.entity_id)
            || self.count < 0
            || self.revision < 0
            || self.mp_spent < 0
            || self.last_sequence > i64::MAX as u64
            || self.last_sequence < self.count as u64
            || self.mp_cost != 0 && !(59..=70).contains(&self.mp_cost)
            || self.reuse_millis != if self.mp_cost == 0 { 0 } else { REUSE_MILLIS }
            || self.count.checked_mul(i64::from(self.mp_cost)) != Some(self.mp_spent)
            || self.initial_lawful.checked_add(self.count) != Some(self.lawful)
            || self.mp_cost == 0 && (self.count != 0 || self.initial_lawful != 0)
        {
            return Err(invalid());
        }
        if self.count == 0 {
            if *self != Self::initial(self.entity_id, self.mp_cost, self.initial_lawful) {
                return Err(invalid());
            }
        } else {
            let h = self.hit.as_ref().ok_or_else(invalid)?;
            h.validate()?;
            if self.revision == 0
                || self.mp_cost == 0
                || self.initial_lawful < 500
                || h.target_entity_id == self.entity_id
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid authoritative Disintegrate",
    )
}
pub fn spent(f: &combat::Frame, id: u64) -> i64 {
    f.disintegrate(id).map_or(0, |s| s.mp_spent)
}
pub fn revision(f: &combat::Frame, id: u64) -> i64 {
    f.disintegrate(id).map_or(0, |s| s.revision)
}
fn life(f: &combat::Frame, id: u64) -> Option<i64> {
    f.honor
        .as_ref()
        .map_or(Some(1), |h| h.life(id)?.respawn_count.checked_add(1))
}
fn casts(f: &combat::Frame, id: u64) -> i128 {
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
    if f.disintegrations.is_empty() {
        return Ok(());
    }
    let n = f.world.actors.len();
    if n == 0
        || n > MAX_ACTORS
        || f.disintegrations.len() != n
        || f.vitals.len() != n
        || f.world.casts.len() != n
        || f.regen.len() != n
        || f.weights.len() != n
        || f.damages.len() != n
        || f.world
            .descriptor
            .is_none_or(|d| !d.has(MatchDescriptor::DISINTEGRATE))
        || !f.npc_spells.is_empty()
        || f.world.quest.is_some()
        || f.world.siege.is_some()
        || f.world.time_attack.is_some()
        || !matches!(f.world.room.summary.settings.mode, 1 | 5)
    {
        return Err(invalid());
    }
    let mut revisions = std::collections::BTreeSet::new();
    for ((s, a), v) in f.disintegrations.iter().zip(&f.world.actors).zip(&f.vitals) {
        s.validate()?;
        let c = f.world.cast(a.entity_id).ok_or_else(invalid)?;
        if s.entity_id != a.entity_id
            || s.revision > f.world.revision
            || s.last_sequence > c.request_sequence
            || casts(f, s.entity_id) > i128::from(c.request_sequence)
            || if a.identity().character().is_some() {
                s.mp_cost == 0
            } else {
                *s != State::initial(s.entity_id, 0, 0)
            }
            || matches!(f.world.phase, Phase::Loading | Phase::Countdown) && s.count != 0
        {
            return Err(invalid());
        }
        let Some(h) = s.hit.as_ref() else { continue };
        if !revisions.insert(s.revision)
            || !f.world.manifest.contains(h.from)
            || !f.world.manifest.contains(h.to)
            || [
                c.cast_revision,
                f.damage(s.entity_id).map_or(0, |d| d.cast_revision as i64),
                honor::heal_revision(f, s.entity_id),
                super::fire_storm_protocol::revision(f, s.entity_id),
                f.item(s.entity_id).map_or(0, |i| i.use_revision),
                v.attack_revision,
                v.hurt_revision,
                v.death_revision,
                honor::respawn_revision(f, s.entity_id),
                f.regen(s.entity_id).map_or(0, |r| r.hp_revision),
                f.regen(s.entity_id).map_or(0, |r| r.mp_revision),
            ]
            .contains(&s.revision)
            || v.death_revision > 0
                && s.revision >= v.death_revision
                && (v.hp == 0 || s.revision <= honor::respawn_revision(f, s.entity_id))
            || f.world.completion.as_ref().is_some_and(|c| {
                c.event
                    .engine_revision
                    .checked_add(1)
                    .is_none_or(|r| s.revision > r)
            })
        {
            return Err(invalid());
        }
        let t = f.world.actor(h.target_entity_id).ok_or_else(invalid)?;
        let tv = f.vital(h.target_entity_id).ok_or_else(invalid)?;
        let r = f.regen(h.target_entity_id).ok_or_else(invalid)?;
        let caster_life = life(f, s.entity_id).ok_or_else(invalid)?;
        let target_life = life(f, h.target_entity_id).ok_or_else(invalid)?;
        if honor::target_team(f, s.entity_id) == honor::target_team(f, h.target_entity_id)
            || h.hp_before > tv.max_hp
            || h.damage_loss_after > r.damage_loss
            || h.caster_life > caster_life
            || h.target_life > target_life
            || h.applied_damage > 0 && tv.hurt_revision < s.revision
            || h.hp_after == 0 && tv.death_revision < s.revision
            || h.hp_after == 0 && h.target_life == target_life && tv.hp != 0
        {
            return Err(invalid());
        }
        for (id, at, now) in [
            (s.entity_id, h.caster_life, caster_life),
            (h.target_entity_id, h.target_life, target_life),
        ] {
            let respawn = honor::respawn_revision(f, id);
            if if at == now {
                s.revision <= respawn
            } else {
                s.revision > respawn
            } {
                return Err(invalid());
            }
        }
        if r.hp_revision <= s.revision
            && f.item(h.target_entity_id)
                .is_none_or(|i| i.use_revision <= s.revision)
        {
            let hp = i128::from(tv.max_hp) * i128::from(h.target_life) + i128::from(r.hp_gain)
                - i128::from(r.hp_loss)
                + i128::from(r.item_gain)
                - i128::from(h.damage_loss_after);
            if hp != i128::from(h.hp_after) {
                return Err(invalid());
            }
        }
        if s.revision == f.world.revision
            && (v.hp == 0
                || c.request_sequence != s.last_sequence
                || a.position != h.from
                || t.position != h.to
                || h.caster_life != caster_life
                || h.target_life != target_life
                || tv.hp != h.hp_after
                || r.damage_loss != h.damage_loss_after
                || h.applied_damage > 0 && tv.hurt_revision != s.revision
                || h.applied_damage == 0 && tv.hurt_revision == s.revision
                || h.hp_after == 0 && tv.death_revision != s.revision)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    if a.disintegrations.len() != b.disintegrations.len() {
        return false;
    }
    if a.disintegrations.is_empty() {
        return true;
    }
    if matches!(a.world.phase, Phase::Finished | Phase::Aborted) {
        return a.disintegrations == b.disintegrations;
    }
    // Include new Fire Storm checkpoints in the same damage budget. Otherwise
    // each spell could independently claim the same observed damage-loss delta.
    let mut receipts: std::collections::BTreeMap<u64, Vec<(i64, i64, i64)>> =
        std::collections::BTreeMap::new();
    for (x, y) in a.disintegrations.iter().zip(&b.disintegrations) {
        if x.entity_id != y.entity_id
            || x.initial_lawful != y.initial_lawful
            || x.mp_cost != y.mp_cost
            || x.reuse_millis != y.reuse_millis
            || y.count < x.count
            || y.revision < x.revision
            || y.mp_spent < x.mp_spent
            || y.last_sequence < x.last_sequence
            || y.count == x.count && x != y
        {
            return false;
        }
        let (Some(ac), Some(bc)) = (a.world.cast(x.entity_id), b.world.cast(y.entity_id)) else {
            return false;
        };
        if casts(b, y.entity_id) - casts(a, x.entity_id)
            > i128::from(bc.request_sequence) - i128::from(ac.request_sequence)
        {
            return false;
        }
        if y.count > x.count {
            let Some(h) = y.hit.as_ref() else {
                return false;
            };
            if y.revision <= a.world.revision
                || y.last_sequence <= ac.request_sequence
                || life(a, x.entity_id).is_none_or(|l| h.caster_life < l)
                || life(a, h.target_entity_id).is_none_or(|l| h.target_life < l)
                || a.vital(x.entity_id).is_some_and(|v| v.hp == 0)
                    && life(a, x.entity_id) == Some(h.caster_life)
                || a.vital(h.target_entity_id).is_some_and(|v| v.hp == 0)
                    && life(a, h.target_entity_id) == Some(h.target_life)
            {
                return false;
            }
            receipts.entry(h.target_entity_id).or_default().push((
                y.revision,
                h.applied_damage,
                h.damage_loss_after,
            ));
        }
    }
    for y in &b.fires {
        if a.fire(y.entity_id).is_none_or(|x| x.count >= y.count) {
            continue;
        }
        for t in &y.targets {
            // Skill77 only targets roster actors. Honor objectives retain their
            // existing Fire Storm ledger, not a fabricated human vitals slot.
            if b.vital(t.target_entity_id).is_some() {
                receipts.entry(t.target_entity_id).or_default().push((
                    y.cast_revision,
                    t.applied_damage,
                    t.damage_loss_after,
                ));
            }
        }
    }
    for (id, mut rows) in receipts {
        let (Some(old), Some(new)) = (a.regen(id), b.regen(id)) else {
            return false;
        };
        rows.sort_by_key(|r| r.0);
        let mut loss = i128::from(old.damage_loss);
        let mut revision = a.world.revision;
        for (rev, applied, after) in rows {
            if rev <= revision || i128::from(after) < loss + i128::from(applied) {
                return false;
            }
            revision = rev;
            loss = i128::from(after);
        }
        if loss > i128::from(new.damage_loss) {
            return false;
        }
    }
    true
}
pub(crate) fn decode(bytes: &[u8], actors: usize) -> Result<(Vec<State>, usize), ApplicationError> {
    let head = bytes.get(..8).ok_or_else(invalid)?;
    let count = usize::from(u16::from_be_bytes(head[..2].try_into().unwrap()));
    if count > MAX_ACTORS || count != 0 && count != actors || head[2..].iter().any(|v| *v != 0) {
        return Err(invalid());
    }
    let used = 8 + count * STATE_BYTES;
    let body = bytes.get(8..used).ok_or_else(invalid)?;
    let mut result = Vec::with_capacity(count);
    for (index, s) in body.chunks_exact(STATE_BYTES).enumerate() {
        let i = |n| i64::from_be_bytes(s[n..n + 8].try_into().unwrap());
        let u = |n| u32::from_be_bytes(s[n..n + 4].try_into().unwrap());
        let hit = if i(8) == 0 {
            if s[64..].iter().any(|b| *b != 0) {
                return Err(invalid());
            }
            None
        } else {
            if s[144] > 1 || s[145..].iter().any(|b| *b != 0) {
                return Err(invalid());
            }
            Some(Hit {
                target_entity_id: i(64) as u64,
                caster_life: i(72),
                target_life: i(80),
                from: WorldCell {
                    x: u(88) as i32,
                    y: u(92) as i32,
                },
                to: WorldCell {
                    x: u(96) as i32,
                    y: u(100) as i32,
                },
                hp_before: i(104),
                hp_after: i(112),
                raw_damage: i(120),
                applied_damage: i(128),
                damage_loss_after: i(136),
                critical: s[144] == 1,
            })
        };
        let state = State {
            entity_id: i(0) as u64,
            count: i(8),
            revision: i(16),
            mp_spent: i(24),
            last_sequence: i(32) as u64,
            initial_lawful: i(40),
            lawful: i(48),
            mp_cost: u(56),
            reuse_millis: u(60),
            hit,
        };
        state.validate()?;
        if state.entity_id != index as u64 + 1 {
            return Err(invalid());
        }
        result.push(state);
    }
    Ok((result, used))
}

#[cfg(test)]
#[path = "disintegrate_protocol_tests.rs"]
mod tests;
