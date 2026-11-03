//! v26 siege authority. Engine revisions are NOT world/vital revisions.
//! Towers have typed slots, never synthetic PC/NPC ids. Neither a tower hit
//! nor a portal request contains client damage or a client-selected destination.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat,
    world_map::WorldCell,
    world_protocol::{self as world, SettlementState},
};
use std::collections::BTreeSet;

pub const VERSION: u16 = 26;
pub const REQUEST_KIND: u16 = 42;
pub const RESULT_KIND: u16 = 43;
pub const REQUEST_BYTES: usize = 56;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub room_id: u64,
    pub match_id: u64,
    pub generation: u64,
}
impl Scope {
    pub fn of(w: &world::Frame) -> Self {
        Self {
            room_id: w.room.summary.room_id,
            match_id: w.match_id,
            generation: w.generation,
        }
    }
    fn valid(self) -> bool {
        [self.room_id, self.match_id, self.generation]
            .into_iter()
            .all(positive)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Slot {
    OuterLeft,
    OuterRight,
    Inner,
}
impl Slot {
    pub const ALL: [Self; 3] = [Self::OuterLeft, Self::OuterRight, Self::Inner];
    fn decode(n: u8) -> Result<Self, ApplicationError> {
        Self::ALL.get(n as usize).copied().ok_or_else(bad)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub scope: Scope,
    pub slot: Slot,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Waiting,
    Outer,
    Inner,
    Finished,
    Aborted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Objective {
    pub cell: WorldCell,
    pub graphic: u32,
    pub max_hp: i64,
    pub hp: i64,
    pub damage_loss: i64,
    pub hit_count: i64,
    pub revision: i64,
    pub death_revision: i64,
    pub last_actor: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Life {
    pub entity_id: u64,
    pub deaths: i64,
    pub death_revision: i64,
    pub respawns: i64,
    pub respawn_revision: i64,
    pub hp_gain: i64,
    pub due_millis: i64,
}
impl Life {
    pub fn dead(&self) -> bool {
        self.deaths > self.respawns
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub id: [u8; 16],
    pub revision: i64,
    pub recorded_at_millis: i64,
    pub state: SettlementState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data {
    pub phase: Phase,
    pub attacker: u8,
    pub winner: Option<u8>,
    pub revision: i64,
    pub gate_revision: i64,
    pub finish_revision: i64,
    pub finished_elapsed_millis: i64,
    pub duration_millis: i64,
    pub respawn_millis: i64,
    pub portal_outside: WorldCell,
    pub portal_inside: WorldCell,
    pub objectives: [Objective; 3],
    /// Canonical x/y ordering; camps keep their configured allocation order.
    pub gate: Vec<WorldCell>,
    pub attack_camp: Vec<WorldCell>,
    pub defense_camp: Vec<WorldCell>,
    pub lives: Vec<Life>,
    pub receipt: Option<Receipt>,
}
impl Data {
    pub fn open(&self) -> bool {
        self.gate_revision > 0
    }
    pub fn camp(&self, team: u8) -> &[WorldCell] {
        if team == self.attacker {
            &self.attack_camp
        } else {
            &self.defense_camp
        }
    }
    pub fn portal_destination(&self, cell: WorldCell) -> Option<WorldCell> {
        if cell == self.portal_outside {
            Some(self.portal_inside)
        } else if cell == self.portal_inside {
            Some(self.portal_outside)
        } else {
            None
        }
    }
    pub fn attackable(&self, team: u8, slot: Slot) -> bool {
        team == self.attacker
            && self.objectives[slot as usize].hp > 0
            && (self.phase == Phase::Outer && slot != Slot::Inner
                || self.phase == Phase::Inner && slot == Slot::Inner)
    }
    pub fn blocked(&self, cell: WorldCell) -> bool {
        self.objectives.iter().any(|t| t.cell == cell) || !self.open() && self.gate.contains(&cell)
    }
    pub(crate) fn same_policy(&self, other: &Self) -> bool {
        self.attacker == other.attacker
            && self.duration_millis == other.duration_millis
            && self.respawn_millis == other.respawn_millis
            && self.portal_outside == other.portal_outside
            && self.portal_inside == other.portal_inside
            && self.gate == other.gate
            && self.attack_camp == other.attack_camp
            && self.defense_camp == other.defense_camp
            && self
                .objectives
                .iter()
                .zip(&other.objectives)
                .all(|(a, b)| a.cell == b.cell && a.graphic == b.graphic && a.max_hp == b.max_hp)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if self.attacker > 1
            || self.winner.is_some_and(|w| w > 1)
            || self.revision < 1
            || !(0..=self.revision).contains(&self.gate_revision)
            || !(0..=self.revision).contains(&self.finish_revision)
            || !(50..=1_800_000).contains(&self.duration_millis)
            || !(1..=60_000).contains(&self.respawn_millis)
            || !(0..=self.duration_millis).contains(&self.finished_elapsed_millis)
            || !(1..=64).contains(&self.gate.len())
            || !(1..=50).contains(&self.attack_camp.len())
            || !(1..=50).contains(&self.defense_camp.len())
            || !(30..=100).contains(&self.lives.len())
            || self.gate.windows(2).any(|p| p[0] >= p[1])
        {
            return Err(bad());
        }
        let mut reserved = BTreeSet::new();
        for cell in self.geometry() {
            if !reserved.insert(cell) {
                return Err(bad());
            }
        }
        for t in &self.objectives {
            if !matches!(t.graphic, 2665 | 2667 | 2669 | 2671)
                || t.max_hp < 1
                || !(0..=t.max_hp).contains(&t.hp)
                || t.damage_loss != t.max_hp - t.hp
                || t.hit_count < 0
                || !(0..=self.revision).contains(&t.revision)
                || !(0..=t.revision).contains(&t.death_revision)
                || t.last_actor > 100
                || (t.hit_count == 0) != (t.revision == 0)
                || (t.hit_count == 0) != (t.last_actor == 0)
                || (t.hp == 0) != (t.death_revision > 0)
                || t.hit_count == 0 && t.damage_loss != 0
                || t.hp == 0 && t.death_revision != t.revision
                || t.hit_count > t.revision
            {
                return Err(bad());
            }
        }
        let outer_dead = self.objectives[..2].iter().all(|t| t.hp == 0);
        let inner = &self.objectives[2];
        if self.open() != outer_dead
            || inner.hp == 0 && !outer_dead
            || outer_dead
                && self.gate_revision
                    != self.objectives[0]
                        .death_revision
                        .max(self.objectives[1].death_revision)
            || inner.hit_count > 0 && (!outer_dead || inner.revision <= self.gate_revision)
            || self.phase == Phase::Waiting && self.objectives.iter().any(|t| t.hit_count != 0)
            || self.phase == Phase::Outer && outer_dead
            || self.phase == Phase::Inner && (!outer_dead || inner.hp == 0)
            || (self.phase == Phase::Finished) != self.receipt.is_some()
        {
            return Err(bad());
        }
        if self.phase == Phase::Finished {
            let r = self.receipt.as_ref().ok_or_else(bad)?;
            if self.finish_revision != self.revision
                || self.winner.is_none()
                || (self.winner == Some(self.attacker)) != (inner.hp == 0)
                || inner.hp == 0
                    && (self.finish_revision != inner.death_revision
                        || self.finished_elapsed_millis >= self.duration_millis)
                || inner.hp != 0 && self.finished_elapsed_millis != self.duration_millis
                || r.revision != self.finish_revision
                || r.recorded_at_millis < 0
                || r.id == [0; 16]
            {
                return Err(bad());
            }
        } else if self.winner.is_some()
            || self.finish_revision != 0
            || self.finished_elapsed_millis != 0
            || inner.hp == 0
        {
            return Err(bad());
        }
        for (i, l) in self.lives.iter().enumerate() {
            if l.entity_id != i as u64 + 1
                || l.deaths < 0
                || l.respawns < 0
                || l.respawns > l.deaths
                || l.deaths - l.respawns > 1
                || l.hp_gain < 0
                || !(0..=1_860_000).contains(&l.due_millis)
                || l.death_revision < 0
                || l.respawn_revision < 0
                || (l.deaths == 0) != (l.death_revision == 0)
                || (l.respawns == 0) != (l.respawn_revision == 0)
                || (l.respawns == 0) != (l.hp_gain == 0)
                || l.deaths > l.death_revision
                || l.respawns > l.respawn_revision
                || l.dead() && l.death_revision <= l.respawn_revision
                || !l.dead()
                    && (l.due_millis != 0 || l.deaths > 0 && l.respawn_revision <= l.death_revision)
            {
                return Err(bad());
            }
        }
        Ok(())
    }
    pub(crate) fn geometry(&self) -> impl Iterator<Item = WorldCell> + '_ {
        self.gate
            .iter()
            .copied()
            .chain(self.objectives.iter().map(|t| t.cell))
            .chain(self.attack_camp.iter().copied())
            .chain(self.defense_camp.iter().copied())
            .chain([self.portal_outside, self.portal_inside])
    }
}
pub fn life(w: &world::Frame, entity: u64) -> Option<&Life> {
    w.siege
        .as_ref()?
        .lives
        .iter()
        .find(|l| l.entity_id == entity)
}
pub(crate) fn validate_world(w: &world::Frame) -> Result<(), ApplicationError> {
    if w.siege.is_some()
        != w.descriptor
            .is_some_and(|d| d.has(world::MatchDescriptor::SIEGE))
    {
        return Err(bad());
    }
    let Some(d) = &w.siege else {
        return Ok(());
    };
    d.validate()?;
    let waiting = matches!(
        w.phase,
        world::Phase::Preparing | world::Phase::Loading | world::Phase::Countdown
    );
    let pre_engine = w.revision == 1 && w.phase == world::Phase::Preparing
        || w.revision == 2
            && w.phase == world::Phase::Aborted
            && w.actors.iter().all(|a| !a.loaded)
            && d.objectives.iter().all(|t| t.hit_count == 0);
    if w.room.summary.settings.mode != 3
        || !matches!(w.room.summary.settings.map_id, 547 | 563 | 573)
        || w.audience_lane.is_some()
        || w.completion.is_some()
        || w.race.is_some()
        || w.quest.is_some()
        || w.time_attack.is_some()
        || d.lives.len() != w.actors.len()
        || d.revision > w.revision
        || d.revision == w.revision && !pre_engine
        || w.descriptor
            .is_none_or(|p| i64::from(p.active_timeout_millis) != d.duration_millis)
        || (w.phase == world::Phase::Finished) != (d.phase == Phase::Finished)
        || (w.phase == world::Phase::Aborted) != (d.phase == Phase::Aborted)
        || waiting != (d.phase == Phase::Waiting)
        || d.geometry().any(|p| !w.manifest.contains(p))
    {
        return Err(bad());
    }
    let mut teams = [0usize; 2];
    for (a, l) in w.actors.iter().zip(&d.lives) {
        let team = w.member(a.entity_id).ok_or_else(bad)?.team;
        if team > 1 || a.entity_id != l.entity_id || d.blocked(a.position)
            || l.death_revision >= w.revision || l.respawn_revision >= w.revision
            || waiting && (l.deaths != 0 || !d.camp(team).contains(&a.position))
            || w.phase != world::Phase::Active && l.due_millis != 0
            // A respawn at this engine tick must use a configured role camp.
            || l.respawn_revision > 0 && l.respawn_revision == w.revision - 1 && !d.camp(team).contains(&a.position)
        {
            return Err(bad());
        }
        teams[team as usize] += 1;
    }
    if teams.iter().any(|n| !(15..=50).contains(n)) {
        return Err(bad());
    }
    for t in &d.objectives {
        if t.hit_count > 0 && w.member(t.last_actor).is_none_or(|m| m.team != d.attacker) {
            return Err(bad());
        }
    }
    if d.receipt.as_ref().is_some_and(|r| r.revision >= w.revision) {
        return Err(bad());
    }
    Ok(())
}
pub(crate) fn validate_combat(f: &combat::Frame) -> Result<(), ApplicationError> {
    let Some(d) = &f.world.siege else {
        return Ok(());
    };
    if d.lives.len() != f.vitals.len() {
        return Err(bad());
    }
    for (l, v) in d.lives.iter().zip(&f.vitals) {
        if l.entity_id != v.entity_id
            || v.death_revision
                != if l.death_revision == 0 {
                    0
                } else {
                    l.death_revision.checked_add(1).ok_or_else(bad)?
                }
            || l.respawns.checked_mul(v.max_hp) != Some(l.hp_gain)
            || l.dead() != (v.hp == 0)
            || f.world.phase == world::Phase::Active && v.hp == 0 && l.due_millis == 0
            || l.respawn_revision > 0
                && l.respawn_revision == f.world.revision - 1
                && v.hp != v.max_hp
        {
            return Err(bad());
        }
    }
    Ok(())
}
pub(crate) fn respawned(a: &world::Frame, b: &world::Frame, entity: u64) -> bool {
    life(a, entity)
        .zip(life(b, entity))
        .is_some_and(|(x, y)| y.respawns > x.respawns)
}
pub(crate) fn respawn_is_latest_relocation(
    a: &world::Frame,
    b: &world::Frame,
    entity: u64,
) -> bool {
    respawned(a, b, entity)
        && life(b, entity).is_some_and(|l| {
            !super::spell_protocol::teleported(a, b, entity)
                || b.cast(entity)
                    .is_some_and(|c| c.cast_revision <= l.respawn_revision)
        })
}
pub(crate) fn follows(a: &world::Frame, b: &world::Frame) -> bool {
    let (Some(x), Some(y)) = (&a.siege, &b.siege) else {
        return a.siege.is_none() && b.siege.is_none();
    };
    if !x.same_policy(y)
        || y.revision < x.revision
        || y.revision > x.revision && y.revision < a.revision
        || y.lives.len() != x.lives.len()
        || x.gate_revision > 0 && x.gate_revision != y.gate_revision
    {
        return false;
    }
    if matches!(x.phase, Phase::Finished | Phase::Aborted) {
        if x.phase == Phase::Aborted {
            return x == y;
        }
        let (Some(old), Some(new)) = (&x.receipt, &y.receipt) else {
            return false;
        };
        let mut copy = x.clone();
        copy.receipt = Some(new.clone());
        return &copy == y
            && old.id == new.id
            && old.revision == new.revision
            && old.recorded_at_millis == new.recorded_at_millis
            && (old.state == new.state
                || old.state == SettlementState::Pending && new.state != SettlementState::Pending);
    }
    if (y.phase as u8) < (x.phase as u8) {
        return false;
    }
    for (old, new) in x.objectives.iter().zip(&y.objectives) {
        if new.hit_count < old.hit_count
            || new.hp > old.hp
            || new.revision < old.revision
            || old.death_revision > 0 && old != new
            || new.hit_count == old.hit_count && old != new
            || new.hit_count > old.hit_count
                && (new.revision < a.revision || new.revision <= old.revision)
        {
            return false;
        }
    }
    // The objective state can stay unchanged while actors move, die or respawn.
    if y.revision == x.revision
        && (y.phase != x.phase
            || y.objectives != x.objectives
            || y.gate_revision != x.gate_revision
            || y.finish_revision != x.finish_revision
            || y.finished_elapsed_millis != x.finished_elapsed_millis
            || y.winner != x.winner
            || y.receipt != x.receipt)
    {
        return false;
    }
    for (old, new) in x.lives.iter().zip(&y.lives) {
        if new.entity_id != old.entity_id
            || new.deaths < old.deaths
            || new.respawns < old.respawns
            || new.hp_gain < old.hp_gain
            || (new.deaths == old.deaths && new.death_revision != old.death_revision)
            || new.deaths > old.deaths && new.death_revision < a.revision
            || new.respawns == old.respawns
                && (new.respawn_revision != old.respawn_revision || new.hp_gain != old.hp_gain)
            || new.respawns > old.respawns
                && (new.respawn_revision < a.revision || new.hp_gain <= old.hp_gain)
            || old.dead()
                && new.deaths == old.deaths
                && new.respawns == old.respawns
                && if matches!(b.phase, world::Phase::Finished | world::Phase::Aborted) {
                    new.due_millis != 0
                } else {
                    new.due_millis != old.due_millis
                }
        {
            return false;
        }
        if old.dead()
            && new.respawns == old.respawns
            && a.actor(old.entity_id).map(|v| v.position)
                != b.actor(new.entity_id).map(|v| v.position)
        {
            return false;
        }
        if new.respawns > old.respawns {
            let (Some(before), Some(after), Some(member)) = (
                a.actor(old.entity_id),
                b.actor(new.entity_id),
                b.member(new.entity_id),
            ) else {
                return false;
            };
            // No client move occurred after the respawn: only the qualified camp
            // or a separately witnessed server teleport may explain the position.
            if before.sequence == after.sequence
                && respawn_is_latest_relocation(a, b, new.entity_id)
                && !y.camp(member.team).contains(&after.position)
            {
                return false;
            }
        }
    }
    true
}
pub(crate) fn terminal_receipt_follows(a: &world::Frame, b: &world::Frame) -> bool {
    b.phase == world::Phase::Finished
        && a.actors == b.actors
        && a.casts == b.casts
        && follows(a, b)
        && a.siege
            .as_ref()
            .and_then(|d| d.receipt.as_ref())
            .is_some_and(|r| r.state == SettlementState::Pending)
        && b.siege
            .as_ref()
            .and_then(|d| d.receipt.as_ref())
            .is_some_and(|r| r.state != SettlementState::Pending)
        && a.revision.checked_add(1) == Some(b.revision)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    AttackTower(Slot),
    Portal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub scope: Scope,
    pub sequence: u64,
    pub action: Action,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || !self.scope.valid()
            || !positive(self.sequence)
        {
            Err(bad())
        } else {
            Ok(())
        }
    }
    pub fn ready_in(&self, f: &combat::Frame) -> bool {
        let Some(d) = &f.world.siege else {
            return false;
        };
        let Some(a) = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.context.key))
        else {
            return false;
        };
        let Some(v) = f.vital(a.entity_id) else {
            return false;
        };
        let Some(m) = f.world.member(a.entity_id) else {
            return false;
        };
        self.validate().is_ok()
            && Scope::of(&f.world) == self.scope
            && f.world.phase == world::Phase::Active
            && v.hp > 0
            && match self.action {
                Action::AttackTower(slot) => {
                    d.attackable(m.team, slot)
                        && v.attack_sequence.checked_add(1) == Some(self.sequence)
                }
                Action::Portal => {
                    m.team != d.attacker
                        && d.portal_destination(a.position).is_some()
                        && a.sequence.checked_add(1) == Some(self.sequence)
                }
            }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    Dead,
    TargetDead,
    SameTeam,
    OutOfRange,
    Blocked,
    TooFast,
    Unsupported,
    Numeric,
    TooHeavy,
    StaleEffect,
    NoAmmo,
    StaleRegen,
    Attacker,
    NotAtPortal,
    Occupied,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: combat::Status,
    pub reason: Reason,
    pub frame: Option<combat::Frame>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == combat::Status::Ok) != (self.reason == Reason::None) {
            return Err(bad());
        }
        let Some(f) = &self.frame else {
            return if self.status == combat::Status::Unavailable && self.reason == Reason::NotActive
            {
                Ok(())
            } else {
                Err(bad())
            };
        };
        f.world.validate(self.echo.context.key)?;
        f.validate()?;
        let d = f.world.siege.as_ref().ok_or_else(bad)?;
        let a = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(bad)?;
        if Scope::of(&f.world) != self.echo.scope {
            return Err(bad());
        }
        if self.status == combat::Status::Ok {
            let valid = match self.echo.action {
                Action::AttackTower(slot) => {
                    let t = &d.objectives[slot as usize];
                    t.last_actor == a.entity_id
                        && t.revision == f.world.revision - 1
                        && f.vital(a.entity_id)
                            .is_some_and(|v| v.attack_sequence == self.echo.sequence)
                }
                Action::Portal => {
                    f.world
                        .member(a.entity_id)
                        .is_some_and(|m| m.team != d.attacker)
                        && a.sequence == self.echo.sequence
                        && d.portal_destination(a.position).is_some()
                        && f.world.phase == world::Phase::Active
                }
            };
            if !valid {
                return Err(bad());
            }
        }
        Ok(())
    }
    pub fn combat_reply(&self) -> combat::Reply {
        combat::Reply {
            echo: combat::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: combat::Action::Read,
                match_id: self.echo.scope.match_id,
                generation: self.echo.scope.generation,
                sequence: 0,
                target_entity_id: 0,
            },
            status: if self.frame.is_some() {
                combat::Status::Ok
            } else {
                combat::Status::Unavailable
            },
            reason: if self.frame.is_some() {
                combat::Reason::None
            } else {
                combat::Reason::NotActive
            },
            frame: self.frame.clone(),
        }
    }
    pub fn confirms(&self, q: &Request, prior: &combat::Reply) -> bool {
        if self.echo != *q || self.validate().is_err() {
            return false;
        }
        if self.frame.is_none() {
            return self.status == combat::Status::Unavailable;
        }
        if !combat::follows(Some(prior), &self.combat_reply()) {
            return false;
        }
        let (Some(a), Some(b)) = (&prior.frame, &self.frame) else {
            return false;
        };
        if !q.ready_in(a) {
            return false;
        }
        if self.status != combat::Status::Ok {
            return true;
        }
        let Some(actor) = a
            .world
            .actors
            .iter()
            .find(|v| v.matches_character(q.context.key))
        else {
            return false;
        };
        match q.action {
            Action::AttackTower(slot) => {
                let (Some(x), Some(y)) = (&a.world.siege, &b.world.siege) else {
                    return false;
                };
                x.objectives[slot as usize].hit_count.checked_add(1)
                    == Some(y.objectives[slot as usize].hit_count)
            }
            Action::Portal => {
                a.world
                    .siege
                    .as_ref()
                    .and_then(|d| d.portal_destination(actor.position))
                    == b.world.actor(actor.entity_id).map(|v| v.position)
            }
        }
    }
}
pub fn encode_request(id: u32, q: &Request) -> Result<Vec<u8>, ApplicationError> {
    q.validate()?;
    let mut b = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    b.extend(q.context.key.source_id.to_be_bytes());
    b.extend(q.context.key.object_id.to_be_bytes());
    b.extend(q.context.epoch.to_be_bytes());
    let (action, slot) = match q.action {
        Action::AttackTower(slot) => (0, slot as u8),
        Action::Portal => (1, 255),
    };
    b.extend([q.channel_id, action, slot, 0]);
    for n in [
        q.scope.room_id,
        q.scope.match_id,
        q.scope.generation,
        q.sequence,
    ] {
        b.extend(n.to_be_bytes());
    }
    Ok(b)
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
    fn cell(&mut self) -> Result<WorldCell, ApplicationError> {
        Ok(WorldCell {
            x: self.u32()? as i32,
            y: self.u32()? as i32,
        })
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
        0 => Phase::Waiting,
        1 => Phase::Outer,
        2 => Phase::Inner,
        3 => Phase::Finished,
        4 => Phase::Aborted,
        _ => return Err(bad()),
    };
    let attacker = r.u8()?;
    let winner = match r.u8()? {
        0 => None,
        1 => Some(0),
        2 => Some(1),
        _ => return Err(bad()),
    };
    let receipt = r.u8()?;
    let gates = usize::from(r.u8()?);
    let attack = usize::from(r.u8()?);
    let defense = usize::from(r.u8()?);
    let lives = usize::from(r.u8()?);
    r.zero(2)?;
    if receipt > 1
        || !(1..=64).contains(&gates)
        || !(1..=50).contains(&attack)
        || !(1..=50).contains(&defense)
        || !(30..=100).contains(&lives)
        || bytes.len() - r.pos
            < 268 + (gates + attack + defense) * 8 + lives * 56 + usize::from(receipt) * 40
    {
        return Err(bad());
    }
    let revision = r.i64()?;
    let gate_revision = r.i64()?;
    let finish_revision = r.i64()?;
    let finished_elapsed_millis = r.i64()?;
    let duration_millis = r.i64()?;
    let respawn_millis = r.i64()?;
    let portal_outside = r.cell()?;
    let portal_inside = r.cell()?;
    let mut objectives = Vec::with_capacity(3);
    for _ in 0..3 {
        objectives.push(Objective {
            cell: r.cell()?,
            graphic: r.u32()?,
            max_hp: r.i64()?,
            hp: r.i64()?,
            damage_loss: r.i64()?,
            hit_count: r.i64()?,
            revision: r.i64()?,
            death_revision: r.i64()?,
            last_actor: r.u64()?,
        });
    }
    let mut gate = Vec::with_capacity(gates);
    for _ in 0..gates {
        gate.push(r.cell()?);
    }
    let mut attack_camp = Vec::with_capacity(attack);
    for _ in 0..attack {
        attack_camp.push(r.cell()?);
    }
    let mut defense_camp = Vec::with_capacity(defense);
    for _ in 0..defense {
        defense_camp.push(r.cell()?);
    }
    let mut rows = Vec::with_capacity(lives);
    for _ in 0..lives {
        rows.push(Life {
            entity_id: r.u64()?,
            deaths: r.i64()?,
            death_revision: r.i64()?,
            respawns: r.i64()?,
            respawn_revision: r.i64()?,
            hp_gain: r.i64()?,
            due_millis: r.i64()?,
        });
    }
    let receipt = if receipt == 1 {
        let id = r.take(16)?.try_into().unwrap();
        let revision = r.i64()?;
        let recorded_at_millis = r.i64()?;
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
            recorded_at_millis,
            state,
        })
    } else {
        None
    };
    let d = Data {
        phase,
        attacker,
        winner,
        revision,
        gate_revision,
        finish_revision,
        finished_elapsed_millis,
        duration_millis,
        respawn_millis,
        portal_outside,
        portal_inside,
        objectives: objectives.try_into().map_err(|_| bad())?,
        gate,
        attack_camp,
        defense_camp,
        lives: rows,
        receipt,
    };
    d.validate()?;
    Ok((Some(d), r.pos))
}
pub fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=super::protocol::IMPLEMENTED_WORLD_VERSION).contains(&version)
        || bytes.len() < REQUEST_BYTES + 4
        || bytes.len() > combat::frame_limit(version) - 16
    {
        return Err(bad());
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
    let action = r.u8()?;
    let slot = r.u8()?;
    r.zero(1)?;
    let action = match (action, slot) {
        (0, n) => Action::AttackTower(Slot::decode(n)?),
        (1, 255) => Action::Portal,
        _ => return Err(bad()),
    };
    let scope = Scope {
        room_id: r.u64()?,
        match_id: r.u64()?,
        generation: r.u64()?,
    };
    let sequence = r.u64()?;
    let status = match r.u8()? {
        0 => combat::Status::Ok,
        1 => combat::Status::Denied,
        2 => combat::Status::Busy,
        3 => combat::Status::Unavailable,
        _ => return Err(bad()),
    };
    let reason = match r.u8()? {
        0 => Reason::None,
        1 => Reason::NotActive,
        2 => Reason::Dead,
        3 => Reason::TargetDead,
        4 => Reason::SameTeam,
        5 => Reason::OutOfRange,
        6 => Reason::Blocked,
        7 => Reason::TooFast,
        8 => Reason::Unsupported,
        9 => Reason::Numeric,
        10 => Reason::TooHeavy,
        11 => Reason::StaleEffect,
        12 => Reason::NoAmmo,
        13 => Reason::StaleRegen,
        14 => Reason::Attacker,
        15 => Reason::NotAtPortal,
        16 => Reason::Occupied,
        _ => return Err(bad()),
    };
    let present = r.u8()?;
    r.zero(1)?;
    let frame = match present {
        0 => None,
        1 => {
            let (f, used) = combat::decode_frame_version(&bytes[r.pos..], context.key, version)?;
            r.take(used)?;
            Some(f)
        }
        _ => return Err(bad()),
    };
    if r.pos != bytes.len() {
        return Err(bad());
    }
    let reply = Reply {
        echo: Request {
            context,
            channel_id,
            scope,
            sequence,
            action,
        },
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok(reply)
}
fn positive(n: u64) -> bool {
    n > 0 && n <= i64::MAX as u64
}
fn bad() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid dedicated siege authority",
    )
}

#[cfg(test)]
#[path = "siege_protocol_tests.rs"]
pub(crate) mod tests;
