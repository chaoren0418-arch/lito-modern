//! v24 typed NPC light-arrow. No target alias, optimistic mana or damage.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat, npc_protocol as npc,
    world_map::WorldCell,
    world_protocol as world,
};
pub const VERSION: u16 = 24;
pub const REQUEST_KIND: u16 = 40;
pub const RESULT_KIND: u16 = 41;
pub const REQUEST_BYTES: usize = 80;
pub const STATE_BYTES: usize = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub target: npc::Id,
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
    /// Validate the player's last atomic view. Authority can still deny a
    /// raced target/cooldown; this never predicts damage or spends local MP.
    pub fn ready_in(&self, f: &npc::Frame) -> bool {
        let w = &f.combat.world;
        self.validate().is_ok()
            && self.target.scope == f.scope
            && w.phase == world::Phase::Active
            && f.resources.is_some()
            && w.descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::NPC_LIGHT_ARROW))
            && f.actor(self.target)
                .is_some_and(|n| n.alive() && n.life == self.target_life)
            && w.actors
                .iter()
                .find(|a| a.matches_character(self.context.key))
                .is_some_and(|a| {
                    f.combat.vital(a.entity_id).is_some_and(|v| v.hp > 0)
                        && w.time_attack
                            .as_ref()
                            .is_none_or(|t| !t.departed(a.entity_id))
                        && f.combat.weight(a.entity_id).is_none_or(|v| v.can_attack())
                        && f.combat.npc_spell(a.entity_id).is_some()
                        && w.cast(a.entity_id).is_some_and(|s| {
                            s.request_sequence.checked_add(1) == Some(self.sequence)
                        })
                        && super::quest_life::life(w, a.entity_id).map_or(1, |l| l.incarnation())
                            == self.caster_life
                })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    Dead,
    Unlearned,
    Unsupported,
    TooFast,
    NoMp,
    TargetDead,
    WrongLane,
    OutOfRange,
    Blocked,
    TooHeavy,
    Numeric,
    StaleState,
    StaleRegen,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub target: npc::Id,
    pub target_life: u64,
    pub lane: u8,
    pub from: WorldCell,
    pub to: WorldCell,
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
    pub death: bool,
    pub critical: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub entity_id: u64,
    pub request_sequence: u64,
    pub cast_count: u64,
    pub revision: i64,
    pub mp_spent: i64,
    pub mp_cost: i64,
    pub caster_life: u64,
    pub receipt: Option<Receipt>,
}
impl State {
    pub fn initial(entity_id: u64, mp_cost: i64) -> Self {
        Self {
            entity_id,
            mp_cost,
            request_sequence: 0,
            cast_count: 0,
            revision: 0,
            mp_spent: 0,
            caster_life: 0,
            receipt: None,
        }
    }
    fn validate(&self) -> Result<(), ApplicationError> {
        if !positive(self.entity_id)
            || self.request_sequence > i64::MAX as u64
            || self.request_sequence < self.cast_count
            || self.revision < 0
            || self.mp_spent < 0
            || !(0..=3).contains(&self.mp_cost)
        {
            return Err(invalid());
        }
        if self.cast_count == 0 {
            if *self != Self::initial(self.entity_id, self.mp_cost) {
                return Err(invalid());
            }
            return Ok(());
        }
        let Some(r) = &self.receipt else {
            return Err(invalid());
        };
        if self.mp_cost == 0
            || self.revision == 0
            || !positive(self.caster_life)
            || i128::from(self.cast_count) * i128::from(self.mp_cost) != i128::from(self.mp_spent)
            || !scope_valid(r.target.scope)
            || !positive(r.target.serial)
            || !positive(r.target_life)
            || r.lane > 1
            || r.from == r.to
            || r.raw_damage < 0
            || r.applied_damage < 0
            || r.applied_damage > r.raw_damage
            || r.remaining_hp < 0
            || r.death != (r.remaining_hp == 0)
            || r.death && r.applied_damage == 0
        {
            return Err(invalid());
        }
        Ok(())
    }
}
pub fn validate_frame(f: &combat::Frame) -> Result<(), ApplicationError> {
    let enabled = f
        .world
        .descriptor
        .is_some_and(|d| d.has(world::MatchDescriptor::NPC_LIGHT_ARROW));
    if enabled == f.npc_spells.is_empty() {
        return Err(invalid());
    }
    if !enabled {
        return Ok(());
    }
    if f.npc_spells.len() != f.world.actors.len()
        || f.npc_spells.len() != f.vitals.len()
        || f.npc_spells.len() != f.world.casts.len()
    {
        return Err(invalid());
    }
    let scope = npc::Scope {
        room_id: f.world.room.summary.room_id,
        match_id: f.world.match_id,
        generation: f.world.generation,
    };
    let end = event_revision(&f.world)?;
    for (((s, a), v), clock) in f
        .npc_spells
        .iter()
        .zip(&f.world.actors)
        .zip(&f.vitals)
        .zip(&f.world.casts)
    {
        s.validate()?;
        if s.entity_id != a.entity_id
            || s.revision > end
            || s.request_sequence > clock.request_sequence
            || s.cast_count
                .checked_add(clock.cast_count)
                .is_none_or(|n| n > clock.request_sequence)
            || if a.identity().character().is_some() {
                s.mp_cost < 1
            } else {
                *s != State::initial(s.entity_id, 0)
            }
        {
            return Err(invalid());
        }
        let Some(r) = &s.receipt else {
            continue;
        };
        let life = f
            .world
            .quest
            .as_ref()
            .and_then(|q| q.lives.iter().find(|l| l.entity_id == s.entity_id));
        if r.target.scope != scope
            || f.world
                .room
                .members
                .iter()
                .find(|m| m.identity() == a.identity())
                .is_none_or(|m| m.team != r.lane)
            || !f.world.manifest.contains(r.from)
            || !f.world.manifest.contains(r.to)
            || matches!(
                f.world.phase,
                world::Phase::Loading | world::Phase::Countdown
            )
            || life.map_or(s.caster_life != 1, |l| {
                s.caster_life > l.incarnation()
                    || if s.caster_life == l.incarnation() {
                        s.revision <= l.respawn_revision
                    } else {
                        s.revision > l.respawn_revision
                    }
            })
            || s.revision == end
                && (v.hp == 0
                    || a.position != r.from
                    || clock.request_sequence != s.request_sequence
                    || f.world
                        .time_attack
                        .as_ref()
                        .is_some_and(|t| t.departed(s.entity_id)))
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn validate_npcs(f: &npc::Frame) -> Result<(), ApplicationError> {
    let end = event_revision(&f.combat.world)?;
    for s in &f.combat.npc_spells {
        let Some(r) = &s.receipt else {
            continue;
        };
        if r.target.scope != f.scope || r.target.serial > f.spawned {
            return Err(invalid());
        }
        let target = f.actors.iter().find(|a| a.serial == r.target.serial);
        if target.is_some_and(|a| {
            a.lane != r.lane
                || a.life < r.target_life
                || r.remaining_hp > a.max_hp
                || r.applied_damage > a.max_hp - r.remaining_hp
        }) || s.revision == end
            && target.is_none_or(|a| a.life != r.target_life || a.hp != r.remaining_hp)
        {
            return Err(invalid());
        }
    }
    Ok(())
}
fn event_revision(w: &world::Frame) -> Result<i64, ApplicationError> {
    w.time_attack
        .as_ref()
        .and_then(|t| t.receipt.as_ref())
        .map_or(Ok(w.revision), |r| {
            r.revision.checked_add(1).ok_or_else(invalid)
        })
}
pub(crate) fn follows(a: &combat::Frame, b: &combat::Frame) -> bool {
    a.npc_spells.len() == b.npc_spells.len()
        && a.npc_spells.iter().zip(&b.npc_spells).all(|(x, y)| {
            if x.entity_id != y.entity_id || x.mp_cost != y.mp_cost || y.cast_count < x.cast_count {
                return false;
            }
            if x.cast_count == y.cast_count {
                return x == y;
            }
            let Some(old) = a.world.cast(x.entity_id) else {
                return false;
            };
            let Some(new) = b.world.cast(x.entity_id) else {
                return false;
            };
            y.revision > a.world.revision
                && y.revision <= b.world.revision
                && y.request_sequence > old.request_sequence
                && new
                    .cast_count
                    .checked_sub(old.cast_count)
                    .and_then(|tp| tp.checked_add(y.cast_count - x.cast_count))
                    .zip(new.request_sequence.checked_sub(old.request_sequence))
                    .is_some_and(|(count, seq)| count <= seq)
                && y.caster_life >= x.caster_life
        })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: npc::Status,
    pub reason: Reason,
    pub frame: Option<npc::Frame>,
}
impl Reply {
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    /// The snapshot is a read, not a claim that a denied spell succeeded.
    /// Its status is kept separate from the typed spell result above.
    pub fn npc_reply(&self) -> npc::Reply {
        npc::Reply {
            echo: npc::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: npc::Action::Read,
                world: self.echo.target.scope,
                sequence: 0,
                serial: 0,
                life: 0,
            },
            status: if self.frame.is_some() {
                npc::Status::Ok
            } else {
                npc::Status::Unavailable
            },
            reason: if self.frame.is_some() {
                npc::Reason::None
            } else {
                npc::Reason::NotActive
            },
            frame: self.frame.clone(),
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == npc::Status::Ok) != (self.reason == Reason::None) {
            return Err(invalid());
        }
        let Some(f) = &self.frame else {
            return if self.status == npc::Status::Unavailable && self.reason == Reason::NotActive {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        f.validate()?;
        let w = &f.combat.world;
        if f.scope != self.echo.target.scope
            || w.descriptor
                .is_none_or(|d| !d.has(world::MatchDescriptor::NPC_LIGHT_ARROW))
        {
            return Err(invalid());
        }
        let a = w
            .actors
            .iter()
            .find(|a| a.identity().character() == Some(self.echo.context.key))
            .ok_or_else(invalid)?;
        let v = f
            .combat
            .vitals
            .iter()
            .find(|v| v.entity_id == a.entity_id)
            .ok_or_else(invalid)?;
        let s = f
            .combat
            .npc_spells
            .iter()
            .find(|s| s.entity_id == a.entity_id)
            .ok_or_else(invalid)?;
        let clock = w.cast(a.entity_id).ok_or_else(invalid)?;
        let life = w
            .quest
            .as_ref()
            .and_then(|q| q.lives.iter().find(|l| l.entity_id == a.entity_id))
            .map_or(1, |l| l.incarnation());
        if self.status == npc::Status::Ok {
            if !(w.phase == world::Phase::Active
                || w.phase == world::Phase::Finished && w.time_attack.is_some())
                || v.hp == 0
                || life != self.echo.caster_life
                || clock.request_sequence != self.echo.sequence
                || s.request_sequence != self.echo.sequence
                || s.revision != event_revision(w)?
                || s.caster_life != self.echo.caster_life
                || s.receipt.as_ref().is_none_or(|r| {
                    r.target != self.echo.target || r.target_life != self.echo.target_life
                })
            {
                return Err(invalid());
            }
        } else if self.status == npc::Status::Denied && self.reason != Reason::NotActive {
            if w.phase != world::Phase::Active
                || (if self.reason == Reason::StaleState {
                    clock.request_sequence < self.echo.sequence
                } else {
                    clock.request_sequence != self.echo.sequence
                })
                || self.reason == Reason::Dead && v.hp != 0
                || self.reason == Reason::NoMp && v.mp >= s.mp_cost
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// One owner submits one spell at a time. A denial can consume its shared
/// request sequence but must preserve every successful spell receipt.
pub fn confirms_transition(old: &npc::Frame, reply: &Reply) -> bool {
    if reply.validate().is_err() {
        return false;
    }
    let Some(new) = &reply.frame else {
        return false;
    };
    let Some(actor) = old
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(reply.echo.context.key))
    else {
        return false;
    };
    let id = actor.entity_id;
    let Some((before, after)) = old.combat.world.cast(id).zip(new.combat.world.cast(id)) else {
        return false;
    };
    let Some((a, b)) = old.combat.npc_spell(id).zip(new.combat.npc_spell(id)) else {
        return false;
    };
    let mut clock = *before;
    clock.request_sequence = after.request_sequence;
    if before.request_sequence.checked_add(1) != Some(reply.echo.sequence)
        || clock != *after
        || old.combat.damage(id) != new.combat.damage(id)
        || old.combat.fire(id) != new.combat.fire(id)
        || old.combat.npc_disintegrate(id) != new.combat.npc_disintegrate(id)
        || old.combat.npc_fire(id) != new.combat.npc_fire(id)
    {
        return false;
    }
    if reply.status == npc::Status::Ok {
        a.cast_count.checked_add(1) == Some(b.cast_count)
            && a.mp_spent.checked_add(a.mp_cost) == Some(b.mp_spent)
            && a.mp_cost == b.mp_cost
            && b.receipt.as_ref().is_some_and(|r| r.from == actor.position)
            && new
                .combat
                .world
                .actors
                .iter()
                .find(|a| a.entity_id == id)
                .is_some_and(|next| {
                    next.position == actor.position
                        && next.sequence == actor.sequence
                        && next.heading == actor.heading
                })
    } else {
        a == b
            && (after.request_sequence == reply.echo.sequence
                || reply.reason == Reason::NotActive
                    && after.request_sequence == before.request_sequence)
    }
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    r.validate()?;
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let mut b = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    b.extend(r.context.key.source_id.to_be_bytes());
    b.extend(r.context.key.object_id.to_be_bytes());
    b.extend(r.context.epoch.to_be_bytes());
    b.extend([r.channel_id, 0, 0, 0]);
    for v in [
        r.target.scope.room_id,
        r.target.scope.match_id,
        r.target.scope.generation,
        r.sequence,
        r.caster_life,
        r.target.serial,
        r.target_life,
    ] {
        b.extend(v.to_be_bytes());
    }
    Ok(b)
}
pub(crate) fn decode_states(
    bytes: &[u8],
    w: &world::Frame,
) -> Result<(Vec<State>, usize), ApplicationError> {
    let mut r = Reader { bytes, pos: 0 };
    let count = r.u16()? as usize;
    r.zero(6)?;
    if count > 100
        || count != 0 && count != w.actors.len()
        || bytes.len() - r.pos < count * STATE_BYTES
    {
        return Err(invalid());
    }
    let scope = npc::Scope {
        room_id: w.room.summary.room_id,
        match_id: w.match_id,
        generation: w.generation,
    };
    let mut states = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = r.u64()?;
        let request_sequence = r.u64()?;
        let cast_count = r.u64()?;
        let revision = r.i64()?;
        let mp_spent = r.i64()?;
        let mp_cost = r.i64()?;
        let caster_life = r.u64()?;
        let serial = r.u64()?;
        let target_life = r.u64()?;
        let lane = r.u8()?;
        let flags = r.u8()?;
        r.zero(6)?;
        let from = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let to = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let raw_damage = r.i64()?;
        let applied_damage = r.i64()?;
        let remaining_hp = r.i64()?;
        let present = flags & 1 != 0;
        if flags > 7
            || present != (cast_count > 0)
            || !present
                && (flags != 0
                    || serial != 0
                    || target_life != 0
                    || lane != 255
                    || from != WorldCell { x: 0, y: 0 }
                    || to != WorldCell { x: 0, y: 0 }
                    || raw_damage != 0
                    || applied_damage != 0
                    || remaining_hp != 0)
        {
            return Err(invalid());
        }
        let receipt = present.then_some(Receipt {
            target: npc::Id { scope, serial },
            target_life,
            lane,
            from,
            to,
            raw_damage,
            applied_damage,
            remaining_hp,
            death: flags & 2 != 0,
            critical: flags & 4 != 0,
        });
        let s = State {
            entity_id,
            request_sequence,
            cast_count,
            revision,
            mp_spent,
            mp_cost,
            caster_life,
            receipt,
        };
        s.validate()?;
        states.push(s);
    }
    Ok((states, r.pos))
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() > npc::frame_limit(version) - 16
    {
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
    let scope = npc::Scope {
        room_id: r.u64()?,
        match_id: r.u64()?,
        generation: r.u64()?,
    };
    let sequence = r.u64()?;
    let caster_life = r.u64()?;
    let serial = r.u64()?;
    let target_life = r.u64()?;
    let echo = Request {
        context,
        channel_id,
        target: npc::Id { scope, serial },
        sequence,
        caster_life,
        target_life,
    };
    echo.validate()?;
    let status = match r.u8()? {
        0 => npc::Status::Ok,
        1 => npc::Status::Denied,
        2 => npc::Status::Busy,
        3 => npc::Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reasons = [
        Reason::None,
        Reason::NotActive,
        Reason::Dead,
        Reason::Unlearned,
        Reason::Unsupported,
        Reason::TooFast,
        Reason::NoMp,
        Reason::TargetDead,
        Reason::WrongLane,
        Reason::OutOfRange,
        Reason::Blocked,
        Reason::TooHeavy,
        Reason::Numeric,
        Reason::StaleState,
        Reason::StaleRegen,
    ];
    let reason = *reasons.get(r.u8()? as usize).ok_or_else(invalid)?;
    let present = r.u8()?;
    if present > 1 {
        return Err(invalid());
    }
    r.zero(1)?;
    let frame = if present == 1 {
        let (f, used) = npc::decode_frame(&bytes[r.pos..], context.key, version)?;
        r.take(used)?;
        Some(f)
    } else {
        None
    };
    if r.pos != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok(reply)
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
fn positive(v: u64) -> bool {
    v > 0 && v <= i64::MAX as u64
}
fn scope_valid(s: npc::Scope) -> bool {
    [s.room_id, s.match_id, s.generation]
        .into_iter()
        .all(positive)
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid NPC light-arrow frame",
    )
}

#[cfg(test)]
#[path = "npc_spell_protocol_tests.rs"]
pub(crate) mod tests;
