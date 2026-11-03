//! v31+ NPC Fire Storm sections. Full atomic-frame validation must precede
//! rendering/use; optional v32 chat preserves this section's exact bytes.
//! A retained structure receipt is history, not necessarily this cast's hit.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    npc_protocol::{Id, Scope},
    quest_protocol, time_attack_protocol,
    world_map::WorldCell,
};

pub const VERSION: u16 = 31;
pub const REQUEST_BYTES: usize = 64;
pub const HEADER_BYTES: usize = 72;
pub const TARGET_BYTES: usize = 64;
pub const STRUCTURE_BYTES: usize = 104;
pub const MAX_ACTORS: usize = 100;
pub const MAX_TARGETS: usize = 48;
/// Larger than the existing 256 KiB NPC envelope. Application admission must
/// negotiate a new bounded envelope; it must not silently omit hit history.
pub const MAX_SECTION_BYTES: usize =
    8 + MAX_ACTORS * (HEADER_BYTES + MAX_TARGETS * TARGET_BYTES + STRUCTURE_BYTES);
pub const REUSE_MILLIS: u32 = 2000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub scope: Scope,
    pub sequence: u64,
    pub caster_life: u64,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || !scope_valid(self.scope)
            || !positive(self.sequence)
            || !positive(self.caster_life)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
    pub death: bool,
}
impl Outcome {
    fn validate(&self, before: i64) -> Result<(), ApplicationError> {
        if before < 1
            || self.raw_damage < 0
            || self.applied_damage != before.min(self.raw_damage)
            || self.remaining_hp != before - self.applied_damage
            || self.death != (self.remaining_hp == 0)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: Id,
    pub life: u64,
    pub lane: u8,
    pub cell: WorldCell,
    pub hp_before: i64,
    pub outcome: Outcome,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestKind {
    RoomSymbol = 1,
    FinalSymbol = 2,
    FinalGuardian = 3,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructureId {
    Quest {
        id: quest_protocol::Id,
        kind: QuestKind,
        born_revision: i64,
    },
    Tower(time_attack_protocol::TowerId),
}
impl StructureId {
    pub fn scope(&self) -> Scope {
        match self {
            Self::Quest { id, .. } => id.scope,
            Self::Tower(id) => id.scope,
        }
    }
    pub fn lane(&self) -> u8 {
        match self {
            Self::Quest { id, .. } => id.slot.saturating_sub(1) / 4,
            Self::Tower(_) => 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    pub count: i64,
    pub revision: i64,
    pub sequence: u64,
    pub caster_life: u64,
    pub origin: WorldCell,
    pub id: StructureId,
    pub cell: WorldCell,
    pub maximum_hp: i64,
    pub hp_before: i64,
    pub outcome: Outcome,
}
impl Structure {
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.count < 1
            || self.revision < self.count
            || !positive(self.sequence)
            || self.sequence < self.count as u64
            || !positive(self.caster_life)
            || !scope_valid(self.id.scope())
            || !(1..=3).contains(&distance(self.origin, self.cell))
            || self.maximum_hp < 1
            || self.hp_before > self.maximum_hp
        {
            return Err(invalid());
        }
        if let StructureId::Quest {
            id,
            kind,
            born_revision,
        } = self.id
        {
            if !(1..=8).contains(&id.slot)
                || born_revision < 1
                || self.revision <= born_revision
                || (kind != QuestKind::RoomSymbol) != ((id.slot - 1) % 4 == 3)
            {
                return Err(invalid());
            }
        }
        self.outcome.validate(self.hp_before)
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
    pub caster_life: u64,
    pub scope: Option<Scope>,
    pub origin: Option<WorldCell>,
    pub lane: Option<u8>,
    pub targets: Vec<Target>,
    pub structure: Option<Structure>,
}
impl State {
    pub fn initial(entity_id: u64, mp_cost: u32) -> Self {
        Self {
            entity_id,
            count: 0,
            revision: 0,
            last_sequence: 0,
            mp_spent: 0,
            mp_cost,
            caster_life: 0,
            scope: None,
            origin: None,
            lane: None,
            targets: vec![],
            structure: None,
        }
    }
    /// Only current-cast structure events may trigger an effect. The retained
    /// receipt still participates in authority validation after this returns None.
    pub fn current_structure(&self) -> Option<&Structure> {
        self.structure.as_ref().filter(|s| s.count == self.count)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !(1..=MAX_ACTORS as u64).contains(&self.entity_id)
            || self.count < 0
            || self.revision < self.count
            || self.last_sequence > i64::MAX as u64
            || self.last_sequence < self.count as u64
            || self.mp_spent < 0
            || self.mp_cost != 0 && !(39..=48).contains(&self.mp_cost)
            || self.count.checked_mul(i64::from(self.mp_cost)) != Some(self.mp_spent)
            || self.targets.len() > MAX_TARGETS
        {
            return Err(invalid());
        }
        if self.count == 0 {
            return if *self == Self::initial(self.entity_id, self.mp_cost) {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        let scope = self.scope.ok_or_else(invalid)?;
        let origin = self.origin.ok_or_else(invalid)?;
        let lane = self.lane.ok_or_else(invalid)?;
        if !scope_valid(scope)
            || lane > 1
            || self.revision < 1
            || self.mp_cost == 0
            || !positive(self.caster_life)
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for t in &self.targets {
            if t.id.scope != scope
                || !positive(t.id.serial)
                || t.id.serial <= previous
                || !positive(t.life)
                || t.lane != lane
                || !(1..=3).contains(&distance(origin, t.cell))
            {
                return Err(invalid());
            }
            t.outcome.validate(t.hp_before)?;
            previous = t.id.serial;
        }
        if let Some(s) = &self.structure {
            s.validate()?;
            let latest = s.count == self.count;
            if s.count > self.count
                || s.revision > self.revision
                || s.sequence > self.last_sequence
                || s.caster_life > self.caster_life
                || s.id.scope() != scope
                || s.id.lane() != lane
                || latest
                    && (s.revision != self.revision
                        || s.sequence != self.last_sequence
                        || s.caster_life != self.caster_life
                        || s.origin != origin
                        || self.targets.len() + 1 > MAX_TARGETS
                        || self.targets.iter().any(|t| t.cell == s.cell))
                || !latest
                    && (s.revision >= self.revision
                        || s.sequence >= self.last_sequence
                        || self.count - s.count > self.revision - s.revision
                        || (self.count - s.count) as u64 > self.last_sequence - s.sequence)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
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
        r.scope.room_id,
        r.scope.match_id,
        r.scope.generation,
        r.sequence,
        r.caster_life,
    ] {
        b.extend(n.to_be_bytes());
    }
    Ok(b)
}
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
    let request = Request {
        context,
        channel_id,
        scope,
        sequence: r.u64()?,
        caster_life: r.u64()?,
    };
    request.validate()?;
    Ok(request)
}
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
    if bytes.len() - r.pos < count * HEADER_BYTES {
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
        let mp_cost = r.u32()?;
        let reuse = r.u32()?;
        let caster_life = r.u64()?;
        let origin = r.cell()?;
        let lane = r.u8()?;
        let hits = r.u8()? as usize;
        let present = r.u8()?;
        r.zero(5)?;
        if hits > MAX_TARGETS
            || present > 1
            || reuse != if mp_cost == 0 { 0 } else { REUSE_MILLIS }
            || bytes.len() - r.pos < hits * TARGET_BYTES + usize::from(present) * STRUCTURE_BYTES
            || count == 0 && (lane != 255 || origin != (WorldCell { x: 0, y: 0 }))
        {
            return Err(invalid());
        }
        let mut targets = Vec::with_capacity(hits);
        for _ in 0..hits {
            let id = Id {
                scope,
                serial: r.u64()?,
            };
            let life = r.u64()?;
            let cell = r.cell()?;
            let hp_before = r.i64()?;
            let raw_damage = r.i64()?;
            let applied_damage = r.i64()?;
            let remaining_hp = r.i64()?;
            let flags = r.u8()?;
            r.zero(7)?;
            if flags > 1 {
                return Err(invalid());
            }
            targets.push(Target {
                id,
                life,
                lane,
                cell,
                hp_before,
                outcome: Outcome {
                    raw_damage,
                    applied_damage,
                    remaining_hp,
                    death: flags == 1,
                },
            });
        }
        let structure = if present == 1 {
            let count = r.i64()?;
            let revision = r.i64()?;
            let sequence = r.u64()?;
            let caster_life = r.u64()?;
            let origin = r.cell()?;
            let kind = r.u8()?;
            let slot = r.u8()?;
            let flags = r.u8()?;
            r.zero(5)?;
            if flags > 1 {
                return Err(invalid());
            }
            let cell = r.cell()?;
            let maximum_hp = r.i64()?;
            let hp_before = r.i64()?;
            let born_revision = r.i64()?;
            let outcome = Outcome {
                raw_damage: r.i64()?,
                applied_damage: r.i64()?,
                remaining_hp: r.i64()?,
                death: flags == 1,
            };
            let id = match kind {
                1..=3 => StructureId::Quest {
                    id: quest_protocol::Id { scope, slot },
                    kind: match kind {
                        1 => QuestKind::RoomSymbol,
                        2 => QuestKind::FinalSymbol,
                        _ => QuestKind::FinalGuardian,
                    },
                    born_revision,
                },
                4 if slot == 0 && born_revision == 0 => {
                    StructureId::Tower(time_attack_protocol::TowerId { scope })
                }
                _ => return Err(invalid()),
            };
            Some(Structure {
                count,
                revision,
                sequence,
                caster_life,
                origin,
                id,
                cell,
                maximum_hp,
                hp_before,
                outcome,
            })
        } else {
            None
        };
        let s = State {
            entity_id,
            count,
            revision,
            last_sequence,
            mp_spent,
            mp_cost,
            caster_life,
            scope: (count != 0).then_some(scope),
            origin: (count != 0).then_some(origin),
            lane: (count != 0).then_some(lane),
            targets,
            structure,
        };
        s.validate()?;
        if entity_id <= previous {
            return Err(invalid());
        }
        previous = entity_id;
        states.push(s);
    }
    Ok((states, r.pos))
}
/// Test-only authority encoder. A player never sends these ledgers.
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
    let mut b = Vec::new();
    b.extend((states.len() as u16).to_be_bytes());
    b.extend([0; 6]);
    let mut previous = 0;
    for s in states {
        s.validate()?;
        if s.entity_id <= previous || s.scope.is_some_and(|w| w != scope) {
            return Err(invalid());
        }
        previous = s.entity_id;
        b.extend(s.entity_id.to_be_bytes());
        b.extend(s.count.to_be_bytes());
        b.extend(s.revision.to_be_bytes());
        b.extend(s.last_sequence.to_be_bytes());
        b.extend(s.mp_spent.to_be_bytes());
        b.extend(s.mp_cost.to_be_bytes());
        b.extend((if s.mp_cost == 0 { 0 } else { REUSE_MILLIS }).to_be_bytes());
        b.extend(s.caster_life.to_be_bytes());
        put_cell(&mut b, s.origin.unwrap_or(WorldCell { x: 0, y: 0 }));
        b.extend([
            s.lane.unwrap_or(255),
            s.targets.len() as u8,
            u8::from(s.structure.is_some()),
        ]);
        b.extend([0; 5]);
        for t in &s.targets {
            b.extend(t.id.serial.to_be_bytes());
            b.extend(t.life.to_be_bytes());
            put_cell(&mut b, t.cell);
            b.extend(t.hp_before.to_be_bytes());
            put_outcome(&mut b, &t.outcome);
            b.push(u8::from(t.outcome.death));
            b.extend([0; 7]);
        }
        if let Some(e) = &s.structure {
            b.extend(e.count.to_be_bytes());
            b.extend(e.revision.to_be_bytes());
            b.extend(e.sequence.to_be_bytes());
            b.extend(e.caster_life.to_be_bytes());
            put_cell(&mut b, e.origin);
            let (kind, slot, born) = match e.id {
                StructureId::Quest {
                    id,
                    kind,
                    born_revision,
                } => (kind as u8, id.slot, born_revision),
                StructureId::Tower(_) => (4, 0, 0),
            };
            b.extend([kind, slot, u8::from(e.outcome.death)]);
            b.extend([0; 5]);
            put_cell(&mut b, e.cell);
            b.extend(e.maximum_hp.to_be_bytes());
            b.extend(e.hp_before.to_be_bytes());
            b.extend(born.to_be_bytes());
            put_outcome(&mut b, &e.outcome);
        }
    }
    Ok(b)
}
/// Ledger transition only, not a replacement for atomic-frame validation.
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
            || a.revision > before_revision
            || b.revision > after_revision
            || b.count < a.count
            || a.scope.is_some_and(|s| s != before_scope)
            || b.scope.is_some_and(|s| s != after_scope)
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
            || b.caster_life < a.caster_life
            || (b.count - a.count) as u64 > b.last_sequence - a.last_sequence
            || b.count - a.count > after_revision - before_revision
            || a.count > 0 && a.lane != b.lane
        {
            return false;
        }
        for old in &a.targets {
            for hit in &b.targets {
                if old.id == hit.id
                    && (hit.life < old.life
                        || hit.life == old.life && hit.hp_before > old.outcome.remaining_hp)
                {
                    return false;
                }
            }
        }
        if a.structure == b.structure {
            continue;
        }
        let Some(y) = &b.structure else { return false };
        if y.count <= a.count
            || y.revision <= before_revision
            || y.sequence <= a.last_sequence
            || y.caster_life < a.caster_life
            || y.count - a.count > y.revision - before_revision
            || (y.count - a.count) as u64 > y.sequence - a.last_sequence
        {
            return false;
        }
        if a.structure
            .as_ref()
            .is_some_and(|x| !structure_follows(x, y))
        {
            return false;
        }
    }
    true
}
fn structure_follows(a: &Structure, b: &Structure) -> bool {
    match (&a.id, &b.id) {
        (
            StructureId::Quest {
                id: x,
                kind: k,
                born_revision: br,
            },
            StructureId::Quest {
                id: y,
                kind: l,
                born_revision: cr,
            },
        ) => {
            if a.id.lane() != b.id.lane() || x.slot > y.slot {
                return false;
            }
            if x == y {
                k == l && br == cr && same_target(a, b)
            } else {
                *k == QuestKind::RoomSymbol && *l != QuestKind::FinalGuardian && *cr > a.revision
            }
        }
        (StructureId::Tower(_), StructureId::Tower(_)) => same_target(a, b),
        _ => false,
    }
}
fn same_target(a: &Structure, b: &Structure) -> bool {
    a.cell == b.cell && a.maximum_hp == b.maximum_hp && b.hp_before <= a.outcome.remaining_hp
}
fn distance(a: WorldCell, b: WorldCell) -> i64 {
    (i64::from(a.x) - i64::from(b.x))
        .abs()
        .max((i64::from(a.y) - i64::from(b.y)).abs())
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn scope_valid(s: Scope) -> bool {
    [s.room_id, s.match_id, s.generation]
        .into_iter()
        .all(positive)
}
fn check_count(n: usize, actors: usize) -> Result<(), ApplicationError> {
    if actors == 0 || actors > MAX_ACTORS || n > MAX_ACTORS || n != 0 && n != actors {
        Err(invalid())
    } else {
        Ok(())
    }
}
fn version_check(v: u16) -> Result<(), ApplicationError> {
    if !(VERSION..=super::world_protocol::CODEC_MAX_VERSION).contains(&v) {
        Err(invalid())
    } else {
        Ok(())
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid NPC fire storm wire section",
    )
}
#[cfg(test)]
fn put_cell(b: &mut Vec<u8>, c: WorldCell) {
    b.extend(c.x.to_be_bytes());
    b.extend(c.y.to_be_bytes());
}
#[cfg(test)]
fn put_outcome(b: &mut Vec<u8>, o: &Outcome) {
    b.extend(o.raw_damage.to_be_bytes());
    b.extend(o.applied_damage.to_be_bytes());
    b.extend(o.remaining_hp.to_be_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

#[cfg(test)]
#[path = "npc_fire_storm_wire_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "npc_fire_storm_wire_interop_tests.rs"]
mod interop_tests;
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], ApplicationError> {
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
