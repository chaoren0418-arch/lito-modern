//! v21 own-force objective authority. No NPC aliases, client damage, or enemy
//! coordinates. A public terminal proof is separate from the visible objective.
use super::{
    application_protocol::{self as app, ApplicationError},
    combat_protocol as combat, npc_protocol as npc,
    world_map::WorldCell,
    world_protocol as world,
};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    pub scope: npc::Scope,
    pub slot: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Flow {
    FinalGuardian,
    RoomSymbols,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    SpawnPending,
    Fighting,
    Objective,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lane {
    pub lane: u8,
    pub stage: u8,
    pub phase: Phase,
    pub living: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub slot: u8,
    pub graphic: u32,
    pub cell: WorldCell,
    pub max_hp: i64,
}
impl Target {
    pub fn lane(&self) -> u8 {
        (self.slot - 1) / 4
    }
    pub fn stage(&self) -> u8 {
        (self.slot - 1) % 4 + 1
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub slot: u8,
    pub hp: i64,
    pub born_revision: i64,
    pub revision: i64,
    pub hurt_revision: i64,
    pub death_revision: i64,
}
impl Actor {
    pub fn lane(&self) -> u8 {
        (self.slot - 1) / 4
    }
    pub fn stage(&self) -> u8 {
        (self.slot - 1) % 4 + 1
    }
    pub fn action(&self, target: &Target) -> u8 {
        super::honor_protocol::action_for_hp(self.hp, target.max_hp)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub hit: bool,
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
    pub death: bool,
    pub maximum: bool,
    pub critical: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub entity_id: u64,
    pub slot: u8,
    pub sequence: u64,
    pub revision: i64,
    pub outcome: Outcome,
}
impl Exchange {
    pub fn lane(&self) -> u8 {
        (self.slot - 1) / 4
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub flow: Flow,
    pub progress_revision: i64,
    pub lanes: Vec<Lane>,
    pub targets: Vec<Target>,
    pub actors: Vec<Actor>,
    pub exchanges: Vec<Exchange>,
    pub winning_lane: Option<u8>,
    pub death_revision: i64,
    pub lives: Vec<super::quest_life::State>,
}
impl Data {
    pub fn target(&self, slot: u8) -> Option<&Target> {
        self.targets.iter().find(|t| t.slot == slot)
    }
    pub fn actor(&self, slot: u8) -> Option<&Actor> {
        self.actors.iter().find(|a| a.slot == slot)
    }
    pub fn player_attack(&self, entity: u64) -> Option<&Exchange> {
        self.exchanges.iter().find(|e| e.entity_id == entity)
    }
}
pub fn scope(world: &world::Frame) -> npc::Scope {
    npc::Scope {
        room_id: world.room.summary.room_id,
        match_id: world.match_id,
        generation: world.generation,
    }
}
pub fn target(world: &world::Frame, id: Id) -> Option<(&Target, &Actor)> {
    if scope(world) != id.scope {
        return None;
    }
    let q = world.quest.as_ref()?;
    Some((q.target(id.slot)?, q.actor(id.slot)?))
}
pub(crate) fn validate(world: &world::Frame) -> Result<(), ApplicationError> {
    super::quest_life::validate(world)?;
    if world.quest.is_some()
        != world
            .descriptor
            .is_some_and(|d| d.has(world::MatchDescriptor::QUEST_OBJECTIVES))
    {
        return Err(invalid());
    }
    let Some(q) = &world.quest else {
        return if world.completion.as_ref().is_some_and(|c| c.event.mode == 2) {
            Err(invalid())
        } else {
            Ok(())
        };
    };
    let lane = world.audience_lane.ok_or_else(invalid)?;
    let flow = if matches!(world.room.summary.settings.map_id, 513 | 527) {
        Flow::RoomSymbols
    } else {
        Flow::FinalGuardian
    };
    if lane > 1
        || world.room.summary.settings.mode != 2
        || q.flow != flow
        || q.progress_revision < 1
        || q.progress_revision > world.revision
        || q.lanes.len() != 1
        || q.actors.len() > 1
        || q.exchanges.len() > 100
        || q.winning_lane.is_some_and(|w| w > 1)
        || q.death_revision < 0
        || (q.winning_lane.is_some()) != (q.death_revision > 0)
        || (world.phase == world::Phase::Finished) != q.winning_lane.is_some()
    {
        return Err(invalid());
    }
    let l = &q.lanes[0];
    let count = if flow == Flow::RoomSymbols { 4 } else { 1 };
    if l.lane != lane
        || !(1..=4).contains(&l.stage)
        || l.living > 128
        || (l.phase == Phase::Fighting) != (l.living > 0)
        || q.targets.len() != count
    {
        return Err(invalid());
    }
    for (i, t) in q.targets.iter().enumerate() {
        let stage = if count == 1 { 4 } else { i as u8 + 1 };
        if t.slot != lane * 4 + stage
            || !matches!(t.graphic, 2665 | 2667 | 2669 | 2671)
            || t.max_hp < 1
            || !world.manifest.contains(t.cell)
        {
            return Err(invalid());
        }
    }
    for a in &q.actors {
        let t = q.target(a.slot).ok_or_else(invalid)?;
        if a.hp < 0
            || a.hp > t.max_hp
            || a.born_revision < 1
            || a.revision < a.born_revision
            || a.revision > world.revision
            || a.hurt_revision < 0
            || a.hurt_revision > a.revision
            || a.hurt_revision > 0 && a.hurt_revision <= a.born_revision
            || a.death_revision < 0
            || a.death_revision > a.revision
            || (a.hp == 0) != (a.death_revision > 0)
            || a.death_revision > 0 && a.death_revision != a.hurt_revision
            || (a.hp < t.max_hp) != (a.hurt_revision > 0)
            || a.revision == a.born_revision && (a.hp != t.max_hp || a.hurt_revision != 0)
            || a.hp > 0
                && (a.stage() != l.stage
                    || l.phase != Phase::Objective
                    || world.actors.iter().any(|p| p.position == t.cell))
            || a.hp == 0
                && if a.stage() == l.stage {
                    q.winning_lane != Some(lane) || a.stage() != 4
                } else {
                    a.stage() + 1 != l.stage || a.stage() == 4
                }
        {
            return Err(invalid());
        }
    }
    let mut last = 0;
    let mut humans = HashSet::new();
    for e in &q.exchanges {
        let t = q.target(e.slot).ok_or_else(invalid)?;
        let p = world.actor(e.entity_id).ok_or_else(invalid)?;
        let member = world.member(e.entity_id).ok_or_else(invalid)?;
        let o = &e.outcome;
        let hp = o
            .applied_damage
            .checked_add(o.remaining_hp)
            .ok_or_else(invalid)?;
        if p.identity().kind != world::MemberKind::Human
            || member.team != e.lane()
            || !humans.insert(e.entity_id)
            || e.revision <= last
            || e.revision > world.revision
            || e.sequence == 0
            || e.sequence > i64::MAX as u64
            || t.stage() > l.stage
            || hp < 1
            || hp > t.max_hp
            || o.raw_damage < 0
            || o.applied_damage < 0
            || o.applied_damage > o.raw_damage
            || o.remaining_hp < 0
            || (o.remaining_hp == 0) != o.death
            || o.death && o.applied_damage == 0
            || !o.hit
                && (o.raw_damage != 0
                    || o.applied_damage != 0
                    || o.death
                    || o.maximum
                    || o.critical)
        {
            return Err(invalid());
        }
        if let Some(a) = q.actor(e.slot) {
            if e.revision <= a.born_revision
                || e.revision > a.revision
                || o.remaining_hp < a.hp
                || e.revision == a.revision
                    && (o.remaining_hp != a.hp
                        || o.applied_damage > 0 && a.hurt_revision != e.revision)
            {
                return Err(invalid());
            }
        }
        last = e.revision;
    }
    if matches!(
        world.phase,
        world::Phase::Preparing | world::Phase::Loading | world::Phase::Countdown
    ) && (q.progress_revision != 1
        || !q.actors.is_empty()
        || !q.exchanges.is_empty()
        || l.stage != 1
        || l.phase != Phase::SpawnPending)
    {
        return Err(invalid());
    }
    if let Some(winner) = q.winning_lane {
        let event = &world.completion.as_ref().ok_or_else(invalid)?.event;
        if event.mode != 2
            || event.engine_revision.checked_add(1) != Some(q.death_revision)
            || q.death_revision > world.revision
            || event
                .participants
                .iter()
                .any(|p| (p.verdict == world::Verdict::Win) != (p.team == winner))
        {
            return Err(invalid());
        }
        if lane == winner {
            let a = q.actor(lane * 4 + 4).ok_or_else(invalid)?;
            if a.hp != 0
                || a.death_revision != q.death_revision
                || l.stage != 4
                || l.phase != Phase::Objective
                || !world
                    .descriptor
                    .is_some_and(|d| d.has(world::MatchDescriptor::NPC_FIRE_STORM))
                    && !q.exchanges.iter().any(|e| {
                        e.slot == a.slot && e.revision == q.death_revision && e.outcome.death
                    })
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_combat(frame: &combat::Frame) -> Result<(), ApplicationError> {
    super::quest_life::validate_combat(frame)?;
    if let Some(q) = &frame.world.quest {
        for e in &q.exchanges {
            let v = frame.vital(e.entity_id).ok_or_else(invalid)?;
            if e.sequence > v.attack_sequence
                || e.revision == frame.world.revision
                    && (v.hp == 0 || e.sequence != v.attack_sequence)
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}
pub(crate) fn validate_npcs(frame: &npc::Frame) -> Result<(), ApplicationError> {
    let Some(q) = &frame.combat.world.quest else {
        return Ok(());
    };
    if frame.closed {
        return Ok(());
    }
    let l = &q.lanes[0];
    let living: Vec<_> = frame.actors.iter().filter(|a| a.alive()).collect();
    if living.len() != usize::from(l.living)
        || living
            .iter()
            .any(|a| a.lane != l.lane || a.stage != l.stage)
    {
        return Err(invalid());
    }
    for a in &q.actors {
        if a.hp > 0
            && living
                .iter()
                .any(|n| Some(n.cell) == q.target(a.slot).map(|t| t.cell))
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub(crate) fn follows(a: &world::Frame, b: &world::Frame) -> bool {
    if !super::quest_life::follows(a, b) {
        return false;
    }
    let (Some(x), Some(y)) = (&a.quest, &b.quest) else {
        return a.quest.is_none() && b.quest.is_none();
    };
    if x.lanes.len() != 1
        || y.lanes.len() != 1
        || x.flow != y.flow
        || x.targets != y.targets
        || y.progress_revision < x.progress_revision
        || x.progress_revision == y.progress_revision && x.lanes != y.lanes
        || matches!(a.phase, world::Phase::Aborted | world::Phase::Finished) && x != y
    {
        return false;
    }
    if y.lanes[0].lane != x.lanes[0].lane || y.lanes[0].stage < x.lanes[0].stage {
        return false;
    }
    for old in &x.actors {
        let Some(new) = y.actors.first() else {
            return false;
        };
        if new.slot < old.slot {
            return false;
        }
        if new.slot == old.slot
            && (new.born_revision != old.born_revision
                || new.revision < old.revision
                || new.hp > old.hp
                || new.revision == old.revision && new != old
                || new.revision > old.revision && new.revision <= a.revision)
        {
            return false;
        }
    }
    for new in &y.actors {
        if x.actor(new.slot).is_none() && new.born_revision <= a.revision {
            return false;
        }
    }
    for old in &x.exchanges {
        let Some(new) = y.player_attack(old.entity_id) else {
            return false;
        };
        if new.revision < old.revision
            || new.revision == old.revision && new != old
            || new.revision > old.revision
                && (new.revision <= a.revision || new.sequence <= old.sequence)
        {
            return false;
        }
    }
    for new in &y.exchanges {
        if new.revision <= a.revision && !x.exchanges.contains(new) {
            return false;
        }
    }
    true
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let r = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(r)
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
pub(crate) fn decode(bytes: &[u8]) -> Result<(Option<Data>, usize), ApplicationError> {
    decode_version(bytes, 21)
}
pub(crate) fn decode_version(
    bytes: &[u8],
    version: u16,
) -> Result<(Option<Data>, usize), ApplicationError> {
    let mut r = Reader { bytes, pos: 0 };
    let present = r.u8()?;
    if present == 0 {
        r.zero(7)?;
        return Ok((None, r.pos));
    }
    if present != 1 {
        return Err(invalid());
    }
    let schema = r.u8()?;
    if schema != 1 && (schema != 2 || version < 22) {
        return Err(invalid());
    }
    let flow = match r.u8()? {
        0 => Flow::FinalGuardian,
        1 => Flow::RoomSymbols,
        _ => return Err(invalid()),
    };
    let targets = usize::from(r.u8()?);
    let lanes = usize::from(r.u8()?);
    let actors = usize::from(r.u8()?);
    let events = usize::from(r.u16()?);
    let progress_revision = r.i64()?;
    let death_revision = r.i64()?;
    let winning_lane = match r.u8()? {
        0 => None,
        1 => Some(0),
        2 => Some(1),
        _ => return Err(invalid()),
    };
    let lives = usize::from(r.u8()?);
    r.zero(6)?;
    if targets > 8
        || lanes > 2
        || actors > 2
        || events > 100
        || (schema == 2) != (lives > 0)
        || lives > 100
        || bytes.len() - r.pos < targets * 24 + lanes * 8 + actors * 48 + events * 64 + lives * 72
    {
        return Err(invalid());
    }
    let mut ls = Vec::with_capacity(lanes);
    let mut ts = Vec::with_capacity(targets);
    let mut ac = Vec::with_capacity(actors);
    let mut es = Vec::with_capacity(events);
    for _ in 0..lanes {
        let lane = r.u8()?;
        let stage = r.u8()?;
        let phase = match r.u8()? {
            0 => Phase::SpawnPending,
            1 => Phase::Fighting,
            2 => Phase::Objective,
            _ => return Err(invalid()),
        };
        r.zero(1)?;
        let living = r.u16()?;
        r.zero(2)?;
        ls.push(Lane {
            lane,
            stage,
            phase,
            living,
        });
    }
    for _ in 0..targets {
        let slot = r.u8()?;
        r.zero(3)?;
        let graphic = r.u32()?;
        let cell = WorldCell {
            x: r.i32()?,
            y: r.i32()?,
        };
        let max_hp = r.i64()?;
        ts.push(Target {
            slot,
            graphic,
            cell,
            max_hp,
        });
    }
    for _ in 0..actors {
        let slot = r.u8()?;
        r.zero(7)?;
        ac.push(Actor {
            slot,
            hp: r.i64()?,
            born_revision: r.i64()?,
            revision: r.i64()?,
            hurt_revision: r.i64()?,
            death_revision: r.i64()?,
        });
    }
    for _ in 0..events {
        let entity_id = r.u64()?;
        let slot = r.u8()?;
        let flags = r.u8()?;
        if flags > 15 {
            return Err(invalid());
        }
        r.zero(6)?;
        let sequence = r.u64()?;
        let revision = r.i64()?;
        let outcome = Outcome {
            hit: flags & 1 != 0,
            death: flags & 2 != 0,
            maximum: flags & 4 != 0,
            critical: flags & 8 != 0,
            raw_damage: r.i64()?,
            applied_damage: r.i64()?,
            remaining_hp: r.i64()?,
        };
        r.zero(8)?;
        es.push(Exchange {
            entity_id,
            slot,
            sequence,
            revision,
            outcome,
        });
    }
    let mut life_rows = Vec::with_capacity(lives);
    for _ in 0..lives {
        let entity_id = r.u64()?;
        let lane = r.u8()?;
        r.zero(7)?;
        let deaths = r.i64()?;
        let death_revision = r.i64()?;
        let respawns = r.i64()?;
        let respawn_revision = r.i64()?;
        let hp_gain = r.i64()?;
        life_rows.push(super::quest_life::State {
            entity_id,
            lane,
            deaths,
            death_revision,
            respawns,
            respawn_revision,
            hp_gain,
            spawn: WorldCell {
                x: r.i32()?,
                y: r.i32()?,
            },
            death_cell: WorldCell {
                x: r.i32()?,
                y: r.i32()?,
            },
        });
    }
    Ok((
        Some(Data {
            flow,
            progress_revision,
            death_revision,
            winning_lane,
            lanes: ls,
            targets: ts,
            actors: ac,
            exchanges: es,
            lives: life_rows,
        }),
        r.pos,
    ))
}
#[cfg(test)]
pub(crate) fn encode(out: &mut Vec<u8>, q: Option<&Data>) {
    let Some(q) = q else {
        out.extend([0; 8]);
        return;
    };
    out.extend([
        1,
        if q.lives.is_empty() { 1 } else { 2 },
        q.flow as u8,
        q.targets.len() as u8,
        q.lanes.len() as u8,
        q.actors.len() as u8,
    ]);
    out.extend((q.exchanges.len() as u16).to_be_bytes());
    out.extend(q.progress_revision.to_be_bytes());
    out.extend(q.death_revision.to_be_bytes());
    out.push(q.winning_lane.map_or(0, |w| w + 1));
    out.push(q.lives.len() as u8);
    out.extend([0; 6]);
    for l in &q.lanes {
        out.extend([l.lane, l.stage, l.phase as u8, 0]);
        out.extend(l.living.to_be_bytes());
        out.extend([0; 2]);
    }
    for t in &q.targets {
        out.extend([t.slot, 0, 0, 0]);
        out.extend(t.graphic.to_be_bytes());
        out.extend(t.cell.x.to_be_bytes());
        out.extend(t.cell.y.to_be_bytes());
        out.extend(t.max_hp.to_be_bytes());
    }
    for a in &q.actors {
        out.push(a.slot);
        out.extend([0; 7]);
        for v in [
            a.hp,
            a.born_revision,
            a.revision,
            a.hurt_revision,
            a.death_revision,
        ] {
            out.extend(v.to_be_bytes());
        }
    }
    for e in &q.exchanges {
        let o = &e.outcome;
        out.extend(e.entity_id.to_be_bytes());
        out.extend([
            e.slot,
            u8::from(o.hit)
                | u8::from(o.death) << 1
                | u8::from(o.maximum) << 2
                | u8::from(o.critical) << 3,
        ]);
        out.extend([0; 6]);
        out.extend(e.sequence.to_be_bytes());
        for v in [
            e.revision,
            o.raw_damage,
            o.applied_damage,
            o.remaining_hp,
            0,
        ] {
            out.extend(v.to_be_bytes());
        }
    }
    for l in &q.lives {
        out.extend(l.entity_id.to_be_bytes());
        out.push(l.lane);
        out.extend([0; 7]);
        for v in [
            l.deaths,
            l.death_revision,
            l.respawns,
            l.respawn_revision,
            l.hp_gain,
        ] {
            out.extend(v.to_be_bytes());
        }
        for v in [l.spawn.x, l.spawn.y, l.death_cell.x, l.death_cell.y] {
            out.extend(v.to_be_bytes());
        }
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid Quest world appendix",
    )
}
#[cfg(test)]
#[path = "quest_protocol_tests.rs"]
pub(crate) mod tests;
