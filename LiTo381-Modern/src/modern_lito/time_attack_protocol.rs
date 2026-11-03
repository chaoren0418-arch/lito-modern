//! v25 cooperative state, inside the existing world/combat/NPC envelope.
//! Engine revisions stay exact here; the enclosing wire uses engine + 1.
use super::{
    application_protocol::ApplicationError,
    combat_protocol, npc_protocol,
    participant::MemberKind,
    world_map::WorldCell,
    world_protocol::{self as world, MatchDescriptor, SettlementState},
};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    WaitingForTower,
    SpawnPending,
    Fighting,
    Finished,
    Aborted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum End {
    None,
    Cleared,
    Timeout,
    LeaderDefeated,
    PartyDefeated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Ordinary,
    FinalBosses,
    Dragon,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TowerId {
    pub scope: npc_protocol::Scope,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub id: [u8; 16],
    pub revision: i64,
    pub finished_at_millis: i64,
    pub state: SettlementState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub entity_id: u64,
    pub sequence: u64,
    pub revision: i64,
    pub flags: u8,
    pub raw_damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub revision: i64,
    pub phase: Phase,
    pub end: End,
    pub waves: Vec<Kind>,
    pub wave: u8,
    pub leader: u8,
    pub duration_millis: i64,
    pub started_millis: i64,
    pub observed_millis: i64,
    pub alive_mask: u64,
    pub departed_mask: u64,
    pub tower_cell: WorldCell,
    pub tower_graphic: u32,
    pub tower_max_hp: i64,
    pub tower_hp: i64,
    pub tower_hurt_revision: i64,
    pub tower_death_revision: i64,
    pub plan_sha256: [u8; 32],
    pub living: Vec<u64>,
    pub exchanges: Vec<Exchange>,
    pub receipt: Option<Receipt>,
}
impl Data {
    pub fn departed(&self, entity: u64) -> bool {
        (1..=60).contains(&entity) && self.departed_mask & (1 << (entity - 1)) != 0
    }
    pub fn won(&self, entity: u64) -> bool {
        self.end == End::Cleared
            && (1..=60).contains(&entity)
            && !self.departed(entity)
            && self.alive_mask & (1 << (entity - 1)) != 0
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.validate_with_fire(false)
    }
    // Magic proof is checked by the complete atomic frame, never by this appendix alone.
    fn validate_with_fire(&self, fire: bool) -> Result<(), ApplicationError> {
        if self.revision < 0
            || self.revision == 0
                && (self.phase != Phase::WaitingForTower
                    || self.observed_millis != 0
                    || self.departed_mask != 0)
            || !(2..=32).contains(&self.waves.len())
            || self.wave as usize >= self.waves.len()
            || !(1..=60).contains(&self.leader)
            || !(50..=1_800_000).contains(&self.duration_millis)
            || !(-1..60_000).contains(&self.started_millis)
            || !(0..=1_860_000).contains(&self.observed_millis)
            || self.started_millis > self.observed_millis
            || self.alive_mask >> 60 != 0
            || self.departed_mask >> 60 != 0
            || !matches!(self.tower_graphic, 2665 | 2667 | 2669 | 2671)
            || self.tower_max_hp < 1
            || self.tower_hp < 0
            || self.tower_hp > self.tower_max_hp
            || self.tower_hurt_revision < 0
            || self.tower_hurt_revision > self.revision
            || self.tower_death_revision < 0
            || self.tower_death_revision > self.revision
            || (self.tower_hp == 0) != (self.tower_death_revision > 0)
            || (self.tower_hp < self.tower_max_hp) != (self.tower_hurt_revision > 0)
            || (self.started_millis >= 0) != (self.tower_hp == 0)
            || self.tower_death_revision > 0
                && self.tower_death_revision != self.tower_hurt_revision
            || self.plan_sha256 == [0; 32]
            || self.living.len() > 128
            || self.exchanges.len() > 60
            || (self.phase == Phase::Finished) != self.receipt.is_some()
            || self.receipt.is_some() != (self.end != End::None)
        {
            return Err(bad());
        }
        let (mut finals, mut dragons) = (0, 0);
        for (i, k) in self.waves.iter().enumerate() {
            match k {
                Kind::Ordinary if finals != 0 || dragons != 0 => return Err(bad()),
                Kind::FinalBosses => {
                    finals += 1;
                    if finals != 1 || dragons != 0 {
                        return Err(bad());
                    }
                }
                Kind::Dragon => {
                    dragons += 1;
                    if finals != 1 || dragons != 1 || i + 1 != self.waves.len() {
                        return Err(bad());
                    }
                }
                _ => {}
            }
        }
        if self.waves[0] != Kind::Ordinary
            || finals != 1
            || self.phase == Phase::WaitingForTower
                && (self.started_millis != -1 || self.wave != 0 || !self.living.is_empty())
            || self.phase == Phase::Fighting && (self.started_millis < 0 || self.living.is_empty())
            || self.phase == Phase::SpawnPending
                && (self.started_millis < 0 || !self.living.is_empty())
            || self.started_millis < 0 && (self.wave != 0 || !self.living.is_empty())
            || self.end == End::Cleared && !self.living.is_empty()
            || self
                .living
                .iter()
                .any(|id| *id == 0 || *id > i64::MAX as u64)
            || self.living.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(bad());
        }
        let mut seen = HashSet::new();
        let mut previous = 0;
        if !matches!(self.phase, Phase::Finished | Phase::Aborted)
            && (self.alive_mask & !self.departed_mask) & (1_u64 << (self.leader - 1)) == 0
        {
            return Err(bad());
        }
        for e in &self.exchanges {
            if !(1..=60).contains(&e.entity_id)
                || e.sequence == 0
                || e.sequence > i64::MAX as u64
                || e.revision <= previous
                || e.revision > self.revision
                || !seen.insert(e.entity_id)
                || e.flags > 15
                || e.raw_damage < 0
                || e.applied_damage < 0
                || e.applied_damage > e.raw_damage
                || e.remaining_hp < self.tower_hp
                || e.remaining_hp > self.tower_max_hp
                || e.applied_damage > self.tower_max_hp - e.remaining_hp
                || (e.flags & 2 != 0) != (e.remaining_hp == 0)
                || e.flags & 1 == 0 && (e.flags != 0 || e.raw_damage != 0 || e.applied_damage != 0)
                || e.flags & 2 != 0 && e.applied_damage == 0
            {
                return Err(bad());
            }
            previous = e.revision;
        }
        if !fire
            && self.tower_death_revision > 0
            && !self
                .exchanges
                .iter()
                .any(|e| e.revision == self.tower_death_revision && e.flags & 2 != 0)
        {
            return Err(bad());
        }
        if let Some(r) = &self.receipt {
            if r.id == [0; 16]
                || r.revision < self.revision
                || r.finished_at_millis < 0
                || self.tower_death_revision > 0 && self.tower_death_revision >= r.revision
                || self.started_millis < 0 && (self.wave != 0 || self.end != End::PartyDefeated)
                || self.end == End::Timeout
                    && self.observed_millis - self.started_millis != self.duration_millis
                || self.started_millis >= 0
                    && self.end != End::Timeout
                    && self.observed_millis - self.started_millis >= self.duration_millis
                || self.end == End::Cleared && self.wave as usize + 1 != self.waves.len()
            {
                return Err(bad());
            }
            let bit = 1_u64 << (self.leader - 1);
            let eligible = self.alive_mask & !self.departed_mask;
            if self.end == End::LeaderDefeated && (self.alive_mask | self.departed_mask) & bit != 0
                || self.end == End::PartyDefeated
                    && (eligible != 0 || self.departed_mask & bit == 0)
                || matches!(self.end, End::Cleared | End::Timeout) && eligible & bit == 0
            {
                return Err(bad());
            }
        } else if self.started_millis >= 0
            && self.phase != Phase::Aborted
            && self.observed_millis - self.started_millis >= self.duration_millis
        {
            return Err(bad());
        }
        Ok(())
    }
}
pub(crate) fn validate_world(w: &world::Frame) -> Result<(), ApplicationError> {
    if w.time_attack.is_some()
        != w.descriptor
            .is_some_and(|d| d.has(MatchDescriptor::TIME_ATTACK))
    {
        return Err(bad());
    }
    let Some(d) = &w.time_attack else {
        return Ok(());
    };
    d.validate_with_fire(
        w.descriptor
            .is_some_and(|d| d.has(MatchDescriptor::NPC_FIRE_STORM)),
    )?;
    let count = w.actors.len();
    if w.room.summary.settings.mode != 4
        || w.room.summary.settings.map_id != 540
        || count == 0
        || count > 60
        || w.audience_lane.is_some()
        || w.completion.is_some()
        || w.race.is_some()
        || w.quest.is_some()
        || d.revision >= w.revision
        || (w.phase == world::Phase::Preparing) != (d.revision == 0)
        || d.leader as usize > count
        || d.alive_mask >> count != 0
        || d.departed_mask >> count != 0
        || !w.manifest.contains(d.tower_cell)
        || (w.phase == world::Phase::Finished) != (d.phase == Phase::Finished)
        || (w.phase == world::Phase::Aborted) != (d.phase == Phase::Aborted)
        || matches!(
            w.phase,
            world::Phase::Preparing | world::Phase::Loading | world::Phase::Countdown
        ) && (d.phase != Phase::WaitingForTower
            || d.observed_millis != 0
            || !d.exchanges.is_empty()
            || d.departed_mask != 0
            || d.alive_mask != (1_u64 << count) - 1)
        || w.actors.iter().any(|a| {
            a.kind != MemberKind::Human
                || w.member(a.entity_id).is_none_or(|m| m.team != 0)
                || !d.departed(a.entity_id) && d.tower_hp > 0 && a.position == d.tower_cell
        })
        || d.exchanges.iter().any(|e| e.entity_id > count as u64)
        || d.receipt.as_ref().is_some_and(|r| r.revision >= w.revision)
    {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn validate_combat(c: &combat_protocol::Frame) -> Result<(), ApplicationError> {
    let Some(d) = &c.world.time_attack else {
        return Ok(());
    };
    if c.vitals.iter().any(|v| !(1..=60).contains(&v.entity_id)) {
        return Err(bad());
    }
    let alive = c
        .vitals
        .iter()
        .filter(|v| v.hp > 0)
        .fold(0_u64, |m, v| m | (1 << (v.entity_id - 1)));
    if alive != d.alive_mask
        || d.exchanges.iter().any(|e| {
            c.vital(e.entity_id)
                .is_none_or(|v| e.sequence > v.attack_sequence)
        })
    {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn validate_npcs(f: &npc_protocol::Frame) -> Result<(), ApplicationError> {
    let Some(d) = &f.combat.world.time_attack else {
        return Ok(());
    };
    let living: Vec<_> = f
        .actors
        .iter()
        .filter(|a| a.hp > 0)
        .map(|a| a.serial)
        .collect();
    if !f.closed && living != d.living
        || d.living.iter().any(|id| *id > f.spawned)
        || f.actors
            .iter()
            .any(|a| a.lane != 0 || a.hp > 0 && a.stage != d.waves[d.wave as usize] as u8 + 1)
    {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn follows(a: &world::Frame, b: &world::Frame) -> bool {
    let (Some(x), Some(y)) = (&a.time_attack, &b.time_attack) else {
        return a.time_attack.is_none() && b.time_attack.is_none();
    };
    if x.waves != y.waves
        || x.duration_millis != y.duration_millis
        || x.plan_sha256 != y.plan_sha256
        || x.tower_cell != y.tower_cell
        || x.tower_graphic != y.tower_graphic
        || x.tower_max_hp != y.tower_max_hp
        || y.revision < x.revision
        || y.observed_millis < x.observed_millis
        || y.wave < x.wave
        || y.tower_hp > x.tower_hp
        || x.alive_mask & y.alive_mask != y.alive_mask
        || x.departed_mask & y.departed_mask != x.departed_mask
        || x.started_millis >= 0 && x.started_millis != y.started_millis
        || x.tower_death_revision > 0 && x.tower_death_revision != y.tower_death_revision
        || y.tower_hurt_revision < x.tower_hurt_revision
        || x.phase == Phase::Aborted && x != y
    {
        return false;
    }
    if x.phase == Phase::Finished {
        let (Some(old), Some(new)) = (&x.receipt, &y.receipt) else {
            return false;
        };
        let mut copy = x.clone();
        copy.receipt = Some(new.clone());
        return &copy == y
            && old.id == new.id
            && old.revision == new.revision
            && old.finished_at_millis == new.finished_at_millis
            && (old.state == new.state
                || old.state == SettlementState::Pending && new.state != SettlementState::Pending);
    }
    for old in &x.exchanges {
        let Some(new) = y.exchanges.iter().find(|e| e.entity_id == old.entity_id) else {
            return false;
        };
        if new.revision < old.revision
            || new.revision == old.revision && new != old
            || new.revision > old.revision
                && (new.sequence <= old.sequence || new.revision < a.revision)
        {
            return false;
        }
    }
    if y.exchanges
        .iter()
        .any(|e| e.revision < a.revision && !x.exchanges.contains(e))
    {
        return false;
    }
    // Observed time can advance without a state revision; all other values cannot.
    if y.revision == x.revision {
        let mut copy = x.clone();
        copy.observed_millis = y.observed_millis;
        copy.alive_mask = y.alive_mask;
        if &copy != y {
            return false;
        }
    }
    true
}
pub(crate) fn terminal_receipt_follows(a: &world::Frame, b: &world::Frame) -> bool {
    let (Some(x), Some(y)) = (&a.time_attack, &b.time_attack) else {
        return false;
    };
    b.phase == world::Phase::Finished
        && a.actors == b.actors
        && a.casts == b.casts
        && follows(a, b)
        && x.receipt
            .as_ref()
            .is_some_and(|r| r.state == SettlementState::Pending)
        && y.receipt
            .as_ref()
            .is_some_and(|r| r.state != SettlementState::Pending)
        && a.revision.checked_add(1) == Some(b.revision)
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(bad)?;
        let v = self.bytes.get(self.pos..end).ok_or_else(bad)?;
        self.pos = end;
        Ok(v)
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
    fn i64(&mut self) -> Result<i64, ApplicationError> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(self.i64()? as u64)
    }
    fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|b| *b != 0) {
            Err(bad())
        } else {
            Ok(())
        }
    }
}
pub(crate) fn decode(bytes: &[u8]) -> Result<(Option<Data>, usize), ApplicationError> {
    decode_with_fire(bytes, false)
}
pub(crate) fn decode_with_fire(
    bytes: &[u8],
    fire: bool,
) -> Result<(Option<Data>, usize), ApplicationError> {
    let mut r = Reader { bytes, pos: 0 };
    let present = r.u8()?;
    r.zero(7)?;
    if present == 0 {
        return Ok((None, r.pos));
    }
    if present != 1 || r.u16()? != 1 {
        return Err(bad());
    }
    let phase = match r.u8()? {
        0 => Phase::WaitingForTower,
        1 => Phase::SpawnPending,
        2 => Phase::Fighting,
        3 => Phase::Finished,
        4 => Phase::Aborted,
        _ => return Err(bad()),
    };
    let end = match r.u8()? {
        0 => End::None,
        1 => End::Cleared,
        2 => End::Timeout,
        3 => End::LeaderDefeated,
        4 => End::PartyDefeated,
        _ => return Err(bad()),
    };
    let wave = r.u8()?;
    let leader = r.u8()?;
    let count = r.u8()?;
    let receipt = r.u8()?;
    if !(2..=32).contains(&count) || receipt > 1 {
        return Err(bad());
    }
    let revision = r.i64()?;
    let started_millis = r.i64()?;
    let observed_millis = r.i64()?;
    let alive_mask = r.u64()?;
    let departed_mask = r.u64()?;
    let tower_cell = WorldCell {
        x: r.u32()? as i32,
        y: r.u32()? as i32,
    };
    let tower_graphic = r.u32()?;
    let duration_millis = i64::from(r.u32()?);
    let tower_max_hp = r.i64()?;
    let tower_hp = r.i64()?;
    let tower_hurt_revision = r.i64()?;
    let tower_death_revision = r.i64()?;
    let plan_sha256 = r.take(32)?.try_into().unwrap();
    let living_count = usize::from(r.u16()?);
    let hits = usize::from(r.u16()?);
    r.zero(4)?;
    if living_count > 128
        || hits > 60
        || bytes.len() - r.pos
            < usize::from(count) + living_count * 8 + hits * 56 + usize::from(receipt) * 40
    {
        return Err(bad());
    }
    let mut waves = Vec::with_capacity(count as usize);
    for _ in 0..count {
        waves.push(match r.u8()? {
            0 => Kind::Ordinary,
            1 => Kind::FinalBosses,
            2 => Kind::Dragon,
            _ => return Err(bad()),
        });
    }
    let mut living = Vec::with_capacity(living_count);
    for _ in 0..living_count {
        living.push(r.u64()?);
    }
    let mut exchanges = Vec::with_capacity(hits);
    for _ in 0..hits {
        let entity_id = r.u64()?;
        let sequence = r.u64()?;
        let revision = r.i64()?;
        let flags = r.u8()?;
        r.zero(7)?;
        exchanges.push(Exchange {
            entity_id,
            sequence,
            revision,
            flags,
            raw_damage: r.i64()?,
            applied_damage: r.i64()?,
            remaining_hp: r.i64()?,
        });
    }
    let receipt = if receipt == 1 {
        let id = r.take(16)?.try_into().unwrap();
        let revision = r.i64()?;
        let finished_at_millis = r.i64()?;
        let state = match r.u8()? {
            0 => SettlementState::Pending,
            1 => SettlementState::Saved,
            2 => SettlementState::Failed,
            _ => return Err(bad()),
        };
        r.zero(7)?;
        Some(Receipt {
            id,
            revision,
            finished_at_millis,
            state,
        })
    } else {
        None
    };
    let d = Data {
        revision,
        phase,
        end,
        waves,
        wave,
        leader,
        duration_millis,
        started_millis,
        observed_millis,
        alive_mask,
        departed_mask,
        tower_cell,
        tower_graphic,
        tower_max_hp,
        tower_hp,
        tower_hurt_revision,
        tower_death_revision,
        plan_sha256,
        living,
        exchanges,
        receipt,
    };
    d.validate_with_fire(fire)?;
    Ok((Some(d), r.pos))
}
fn bad() -> ApplicationError {
    super::application_protocol::error(
        super::application_protocol::ErrorKind::MalformedFrame,
        "invalid cooperative world appendix",
    )
}

#[cfg(test)]
#[path = "time_attack_protocol_tests.rs"]
pub(crate) mod tests;
