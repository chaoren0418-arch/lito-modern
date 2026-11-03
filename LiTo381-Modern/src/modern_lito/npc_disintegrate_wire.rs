//! v29 NPC skill77 sections and atomic-frame validation. Application/transport
//! dispatch is separate; player controller and effects still require admission.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    npc_protocol::{Id, Scope},
    world_map::WorldCell,
};

pub const VERSION: u16 = 29;
pub const REQUEST_BYTES: usize = 80;
pub const STATE_BYTES: usize = 144;
pub const MAX_ACTORS: usize = 100;
pub const REUSE_MILLIS: u32 = 5000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub target: Id,
    pub sequence: u64,
    pub caster_life: u64,
    pub target_life: u64,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || !scope_valid(self.target.scope)
            || ![
                self.target.serial,
                self.sequence,
                self.caster_life,
                self.target_life,
            ]
            .into_iter()
            .all(positive)
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hit {
    pub target: Id,
    pub target_life: u64,
    pub lane: u8,
    pub caster_life: u64,
    pub from: WorldCell,
    pub to: WorldCell,
    pub hp_before: i64,
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
    pub death: bool,
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
        if !scope_valid(self.target.scope)
            || ![self.target.serial, self.target_life, self.caster_life]
                .into_iter()
                .all(positive)
            || self.lane > 1
            || !(1..=10).contains(&range)
            || self.hp_before < 1
            || self.raw_damage < 0
            || self.applied_damage != self.hp_before.min(self.raw_damage)
            || self.remaining_hp != self.hp_before - self.applied_damage
            || self.death != (self.remaining_hp == 0)
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
    pub last_sequence: u64,
    pub mp_spent: i64,
    pub mp_cost: u32,
    pub reuse_millis: u32,
    pub initial_lawful: i64,
    pub lawful: i64,
    pub hit: Option<Hit>,
}
impl State {
    pub fn initial(entity_id: u64, mp_cost: u32, lawful: i64) -> Self {
        Self {
            entity_id,
            count: 0,
            revision: 0,
            last_sequence: 0,
            mp_spent: 0,
            mp_cost,
            reuse_millis: if mp_cost == 0 { 0 } else { REUSE_MILLIS },
            initial_lawful: lawful,
            lawful,
            hit: None,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !(1..=MAX_ACTORS as u64).contains(&self.entity_id)
            || self.count < 0
            || self.revision < 0
            || self.last_sequence > i64::MAX as u64
            || self.last_sequence < self.count as u64
            || self.mp_spent < 0
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
            self.hit.as_ref().ok_or_else(invalid)?.validate()?;
            if self.revision == 0 || self.mp_cost == 0 || self.initial_lawful < 500 {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

/// Decode an exact request payload, not an LTM1 frame. Never dispatch based on
/// this function alone; the eventual application route must authenticate scope.
pub(crate) fn decode_request(bytes: &[u8], version: u16) -> Result<Request, ApplicationError> {
    version_check(version)?;
    if bytes.len() != REQUEST_BYTES {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let context = SelectionContext {
        key: app::CharacterKey {
            source_id: r.u32()?,
            object_id: r.u64()?,
        },
        epoch: r.u64()?,
    };
    let channel_id = r.u8()?;
    r.zero(3)?;
    let scope = Scope {
        room_id: r.u64()?,
        match_id: r.u64()?,
        generation: r.u64()?,
    };
    let sequence = r.u64()?;
    let caster_life = r.u64()?;
    let serial = r.u64()?;
    let target_life = r.u64()?;
    let request = Request {
        context,
        channel_id,
        target: Id { scope, serial },
        sequence,
        caster_life,
        target_life,
    };
    request.validate()?;
    Ok(request)
}
pub(crate) fn encode_request(r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    version_check(version)?;
    r.validate()?;
    let mut b = Vec::with_capacity(REQUEST_BYTES);
    b.extend(r.context.key.source_id.to_be_bytes());
    b.extend(r.context.key.object_id.to_be_bytes());
    b.extend(r.context.epoch.to_be_bytes());
    b.extend([r.channel_id, 0, 0, 0]);
    for n in [
        r.target.scope.room_id,
        r.target.scope.match_id,
        r.target.scope.generation,
        r.sequence,
        r.caster_life,
        r.target.serial,
        r.target_life,
    ] {
        b.extend(n.to_be_bytes());
    }
    Ok(b)
}

/// Return consumed bytes so the containing atomic NPC frame can follow. Scope
/// comes only from that enclosing frame; it is never a player entity namespace.
pub(crate) fn decode_states(
    bytes: &[u8],
    scope: Scope,
    actors: usize,
    version: u16,
) -> Result<(Vec<State>, usize), ApplicationError> {
    version_check(version)?;
    if !scope_valid(scope) {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let count = r.u16()? as usize;
    r.zero(6)?;
    check_count(count, actors)?;
    if bytes.len() - r.pos < count * STATE_BYTES {
        return Err(invalid());
    }
    let mut states = Vec::with_capacity(count);
    let mut previous = 0;
    for _ in 0..count {
        let entity_id = r.u64()?;
        let count = r.i64()?;
        let revision = r.i64()?;
        let last_sequence = r.u64()?;
        let mp_spent = r.i64()?;
        let initial_lawful = r.i64()?;
        let lawful = r.i64()?;
        let mp_cost = r.u32()?;
        let reuse_millis = r.u32()?;
        let present = r.u8()?;
        let lane = r.u8()?;
        let flags = r.u8()?;
        r.zero(5)?;
        let caster_life = r.u64()?;
        let serial = r.u64()?;
        let target_life = r.u64()?;
        let from = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let to = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let hp_before = r.i64()?;
        let raw_damage = r.i64()?;
        let applied_damage = r.i64()?;
        let remaining_hp = r.i64()?;
        if present > 1
            || flags > 3
            || present == 0
                && (lane != 255
                    || flags != 0
                    || caster_life != 0
                    || serial != 0
                    || target_life != 0
                    || from != (WorldCell { x: 0, y: 0 })
                    || to != (WorldCell { x: 0, y: 0 })
                    || hp_before != 0
                    || raw_damage != 0
                    || applied_damage != 0
                    || remaining_hp != 0)
        {
            return Err(invalid());
        }
        let hit = (present == 1).then_some(Hit {
            target: Id { scope, serial },
            target_life,
            lane,
            caster_life,
            from,
            to,
            hp_before,
            raw_damage,
            applied_damage,
            remaining_hp,
            death: flags & 1 != 0,
            critical: flags & 2 != 0,
        });
        let state = State {
            entity_id,
            count,
            revision,
            last_sequence,
            mp_spent,
            mp_cost,
            reuse_millis,
            initial_lawful,
            lawful,
            hit,
        };
        state.validate()?;
        if entity_id <= previous {
            return Err(invalid());
        }
        previous = entity_id;
        states.push(state);
    }
    Ok((states, r.pos))
}

/// Audit encoder; the player never sends an authority ledger to a server.
#[cfg(test)]
fn encode_states(
    states: &[State],
    scope: Scope,
    actors: usize,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    version_check(version)?;
    check_count(states.len(), actors)?;
    if !scope_valid(scope) {
        return Err(invalid());
    }
    let mut b = Vec::with_capacity(8 + states.len() * STATE_BYTES);
    b.extend((states.len() as u16).to_be_bytes());
    b.extend([0; 6]);
    let mut previous = 0;
    for s in states {
        s.validate()?;
        if s.entity_id <= previous || s.hit.as_ref().is_some_and(|h| h.target.scope != scope) {
            return Err(invalid());
        }
        previous = s.entity_id;
        b.extend(s.entity_id.to_be_bytes());
        b.extend(s.count.to_be_bytes());
        b.extend(s.revision.to_be_bytes());
        b.extend(s.last_sequence.to_be_bytes());
        b.extend(s.mp_spent.to_be_bytes());
        b.extend(s.initial_lawful.to_be_bytes());
        b.extend(s.lawful.to_be_bytes());
        b.extend(s.mp_cost.to_be_bytes());
        b.extend(s.reuse_millis.to_be_bytes());
        if let Some(h) = &s.hit {
            b.extend([
                1,
                h.lane,
                u8::from(h.death) | (u8::from(h.critical) << 1),
                0,
                0,
                0,
                0,
                0,
            ]);
            for n in [h.caster_life, h.target.serial, h.target_life] {
                b.extend(n.to_be_bytes());
            }
            for n in [h.from.x, h.from.y, h.to.x, h.to.y] {
                b.extend(n.to_be_bytes());
            }
            for n in [h.hp_before, h.raw_damage, h.applied_damage, h.remaining_hp] {
                b.extend(n.to_be_bytes());
            }
        } else {
            b.extend([0, 255, 0, 0, 0, 0, 0, 0]);
            b.extend([0; 72]);
        }
    }
    Ok(b)
}

/// Only the monotonic receipt ledger. Callers must additionally validate the
/// complete old/new atomic frame (life, MP totals, action sequences, mode,
/// scope, target and unrelated-event revisions) before accepting a transition.
pub fn follows(
    before_scope: Scope,
    before: &[State],
    before_revision: i64,
    after_scope: Scope,
    after: &[State],
    after_revision: i64,
) -> bool {
    if !scope_valid(before_scope)
        || before_scope != after_scope
        || before_revision < 1
        || after_revision < before_revision
        || before.len() != after.len()
        || before.len() > MAX_ACTORS
    {
        return false;
    }
    let mut previous = 0;
    for (a, b) in before.iter().zip(after) {
        if a.validate().is_err()
            || b.validate().is_err()
            || a.entity_id <= previous
            || a.entity_id != b.entity_id
            || a.mp_cost != b.mp_cost
            || a.initial_lawful != b.initial_lawful
            || a.revision > before_revision
            || b.revision > after_revision
            || b.count < a.count
            || a.hit
                .as_ref()
                .is_some_and(|h| h.target.scope != before_scope)
            || b.hit
                .as_ref()
                .is_some_and(|h| h.target.scope != after_scope)
        {
            return false;
        }
        previous = a.entity_id;
        if a.count == b.count {
            if a != b {
                return false;
            }
            continue;
        }
        if b.revision <= before_revision
            || b.last_sequence <= a.last_sequence
            || (b.count - a.count) as u64 > b.last_sequence - a.last_sequence
            || b.count - a.count > after_revision - before_revision
            || a.hit.as_ref().zip(b.hit.as_ref()).is_some_and(|(x, y)| {
                y.caster_life < x.caster_life
                    || x.target == y.target && y.target_life < x.target_life
            })
        {
            return false;
        }
    }
    true
}

fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}

fn frame_scope(f: &super::combat_protocol::Frame) -> Scope {
    super::quest_protocol::scope(&f.world)
}
fn event_revision(f: &super::combat_protocol::Frame) -> Result<i64, ApplicationError> {
    if let Some(r) = f
        .world
        .time_attack
        .as_ref()
        .and_then(|t| t.receipt.as_ref())
    {
        r.revision.checked_add(1).ok_or_else(invalid)
    } else if let Some(c) = &f.world.completion {
        c.event.engine_revision.checked_add(1).ok_or_else(invalid)
    } else {
        Ok(f.world.revision)
    }
}
fn casts(f: &super::combat_protocol::Frame, id: u64) -> i128 {
    i128::from(f.world.cast(id).map_or(0, |c| c.cast_count))
        + i128::from(f.npc_spell(id).map_or(0, |c| c.cast_count))
        + i128::from(f.npc_disintegrate(id).map_or(0, |c| c.count))
        + i128::from(f.npc_fire(id).map_or(0, |c| c.count))
}
pub(crate) fn validate_frame(f: &super::combat_protocol::Frame) -> Result<(), ApplicationError> {
    use super::world_protocol::{MatchDescriptor as D, Phase};
    if f.npc_disintegrations.is_empty() {
        return Ok(());
    }
    let w = &f.world;
    let n = w.actors.len();
    if n == 0
        || n > MAX_ACTORS
        || f.npc_disintegrations.len() != n
        || f.vitals.len() != n
        || w.casts.len() != n
        || f.regen.len() != n
        || f.weights.len() != n
        || f.items.len() != n
        || !f.npc_spells.is_empty() && f.npc_spells.len() != n
        || w.descriptor
            .is_none_or(|d| !d.has(D::NPC_DISINTEGRATE | D::NPC | D::SPELLS | D::REGEN))
        || !f.disintegrations.is_empty()
        || !f.damages.is_empty()
        || !f.timed.is_empty()
        || !f.ammo.is_empty()
        || !f.fires.is_empty()
        || f.honor.is_some()
        || w.siege.is_some()
        || w.room.summary.settings.mode != if w.time_attack.is_some() { 4 } else { 2 }
    {
        return Err(invalid());
    }
    let mut occupied = std::collections::BTreeSet::new();
    for c in &w.casts {
        occupied.insert(c.cast_revision);
    }
    for a in &f.npc_spells {
        occupied.insert(a.revision);
    }
    for i in &f.items {
        occupied.insert(i.use_revision);
    }
    for v in &f.vitals {
        occupied.extend([v.attack_revision, v.hurt_revision, v.death_revision]);
    }
    for r in &f.regen {
        occupied.extend([r.hp_revision, r.mp_revision]);
    }
    if let Some(q) = &w.quest {
        for e in &q.exchanges {
            occupied.insert(e.revision);
        }
        for l in &q.lives {
            occupied.extend([l.respawn_revision, l.death_revision]);
        }
        for a in &q.actors {
            occupied.extend([a.hurt_revision, a.death_revision]);
        }
    }
    if let Some(t) = &w.time_attack {
        for e in &t.exchanges {
            occupied.insert(e.revision.checked_add(1).ok_or_else(invalid)?);
        }
    }
    let end = event_revision(f)?;
    let scope = frame_scope(f);
    let mut seen = std::collections::BTreeSet::new();
    for ((s, a), v) in f.npc_disintegrations.iter().zip(&w.actors).zip(&f.vitals) {
        s.validate()?;
        let c = w.cast(a.entity_id).ok_or_else(invalid)?;
        if s.entity_id != a.entity_id
            || s.revision > end
            || s.last_sequence > c.request_sequence
            || casts(f, s.entity_id) > i128::from(c.request_sequence)
            || if a.identity().character().is_some() {
                s.mp_cost == 0
            } else {
                *s != State::initial(s.entity_id, 0, 0)
            }
            || matches!(w.phase, Phase::Loading | Phase::Countdown) && s.count != 0
        {
            return Err(invalid());
        }
        let Some(h) = &s.hit else { continue };
        let life = super::quest_life::life(w, s.entity_id);
        let incarnation = life.map_or(1, |l| l.incarnation());
        if !seen.insert(s.revision)
            || occupied.contains(&s.revision)
            || h.target.scope != scope
            || w.member(s.entity_id).is_none_or(|m| m.team != h.lane)
            || !w.manifest.contains(h.from)
            || !w.manifest.contains(h.to)
            || h.caster_life > incarnation
            || life.is_some_and(|l| {
                if h.caster_life == incarnation {
                    s.revision <= l.respawn_revision
                } else {
                    s.revision > l.respawn_revision
                }
            })
            || h.caster_life == incarnation && v.hp == 0 && s.revision >= v.death_revision
            || s.revision == end
                && (v.hp == 0
                    || c.request_sequence != s.last_sequence
                    || h.caster_life != incarnation
                    || a.position != h.from
                    || w.time_attack
                        .as_ref()
                        .is_some_and(|t| t.departed(s.entity_id)))
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn validate_npcs(f: &super::npc_protocol::Frame) -> Result<(), ApplicationError> {
    use super::npc_protocol::NpcAction;
    let end = event_revision(&f.combat)?;
    for s in &f.combat.npc_disintegrations {
        let Some(h) = &s.hit else { continue };
        if h.target.scope != f.scope
            || h.target.serial > f.spawned
            || f.exchanges.iter().any(|e| e.revision == s.revision)
        {
            return Err(invalid());
        }
        let target = f.actor(h.target);
        if target.is_some_and(|n| {
            n.lane != h.lane
                || n.life < h.target_life
                || h.hp_before > n.max_hp
                || h.death && h.target_life == n.life && n.hp > 0
        }) {
            return Err(invalid());
        }
        if s.revision == end
            && (target.is_none_or(|n| {
                n.life != h.target_life
                    || n.cell != h.to
                    || n.hp != h.remaining_hp
                    || h.applied_damage > 0
                        && n.action
                            != if h.death {
                                NpcAction::Death
                            } else {
                                NpcAction::Hurt
                            }
            }) || f.combat.vital(s.entity_id).is_none_or(|v| v.hp == 0))
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn frames_follow(
    a: &super::combat_protocol::Frame,
    b: &super::combat_protocol::Frame,
) -> bool {
    if !follows(
        frame_scope(a),
        &a.npc_disintegrations,
        a.world.revision,
        frame_scope(b),
        &b.npc_disintegrations,
        b.world.revision,
    ) {
        return false;
    }
    a.npc_disintegrations.iter().all(|s| {
        a.world
            .cast(s.entity_id)
            .zip(b.world.cast(s.entity_id))
            .is_some_and(|(x, y)| {
                y.request_sequence
                    .checked_sub(x.request_sequence)
                    .is_some_and(|budget| {
                        casts(b, s.entity_id) - casts(a, s.entity_id) <= i128::from(budget)
                    })
            })
    })
}
fn scope_valid(s: Scope) -> bool {
    [s.room_id, s.match_id, s.generation]
        .into_iter()
        .all(positive)
}
fn version_check(v: u16) -> Result<(), ApplicationError> {
    if !(VERSION..=super::world_protocol::CODEC_MAX_VERSION).contains(&v) {
        Err(invalid())
    } else {
        Ok(())
    }
}
fn check_count(n: usize, actors: usize) -> Result<(), ApplicationError> {
    if actors == 0 || actors > MAX_ACTORS || n > MAX_ACTORS || n != 0 && n != actors {
        Err(invalid())
    } else {
        Ok(())
    }
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid NPC77 wire section")
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let b = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(b)
    }
    fn u8(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, ApplicationError> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, ApplicationError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, ApplicationError> {
        Ok(self.u32()? as i32)
    }
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i64(&mut self) -> Result<i64, ApplicationError> {
        Ok(self.u64()? as i64)
    }
    fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|b| *b != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
#[path = "npc_disintegrate_wire_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "npc_disintegrate_wire_interop_tests.rs"]
mod interop_tests;

#[cfg(test)]
#[path = "npc_disintegrate_frame_tests.rs"]
mod frame_tests;
