//! v17 bounded NPC snapshots. NPC IDs are never participant entity IDs, SQL
//! characters or source credentials. Decoding does not authorize a playable mode.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat,
    participant::{MemberKind, ParticipantId},
    world_map::WorldCell,
    world_protocol as world,
};
use std::collections::HashSet;

pub const VERSION: u16 = 17;
pub const RESOURCE_VERSION: u16 = 18;
pub const NAMES_VERSION: u16 = 28;
pub const REQUEST_KIND: u16 = 38;
pub const RESULT_KIND: u16 = 39;
pub const REQUEST_BYTES: usize = 72;
pub const MAX_FRAME_BYTES: usize = 256 * 1024;
pub(crate) fn frame_limit(version: u16) -> usize {
    if version >= 31 {
        1024 * 1024
    } else {
        MAX_FRAME_BYTES
    }
}
const MAX_ELAPSED: i64 = 1_860_000;
const ACTOR_BYTES: usize = 112;
const EXCHANGE_BYTES: usize = 104;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Scope {
    pub room_id: u64,
    pub match_id: u64,
    pub generation: u64,
}
impl Scope {
    fn valid(self) -> bool {
        positive(self.room_id) && positive(self.match_id) && positive(self.generation)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id {
    pub scope: Scope,
    pub serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Attack,
    AttackObjective,
    ConfirmDeath,
    AttackTower,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub world: Scope,
    pub sequence: u64,
    pub serial: u64,
    pub life: u64,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_source(self.context.key.source_id)
            || !positive(self.context.key.object_id)
            || self.context.epoch == 0
            || self.channel_id > 3
            || !self.world.valid()
            || match self.action {
                Action::Read => self.sequence != 0 || self.serial != 0 || self.life != 0,
                Action::AttackTower => {
                    !positive(self.sequence) || self.serial != 0 || self.life != 0
                }
                Action::Attack => {
                    !positive(self.sequence) || !positive(self.serial) || !positive(self.life)
                }
                Action::AttackObjective => {
                    !positive(self.sequence) || !(1..=8).contains(&self.serial) || self.life != 0
                }
                Action::ConfirmDeath => {
                    !positive(self.sequence)
                        || self.sequence < 2
                        || !(1..=100).contains(&self.serial)
                        || !positive(self.life)
                }
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn target(&self) -> Option<Id> {
        (self.action == Action::Attack).then_some(Id {
            scope: self.world,
            serial: self.serial,
        })
    }
    pub fn tower(&self) -> Option<super::time_attack_protocol::TowerId> {
        (self.action == Action::AttackTower)
            .then_some(super::time_attack_protocol::TowerId { scope: self.world })
    }
    pub fn objective(&self) -> Option<super::quest_protocol::Id> {
        (self.action == Action::AttackObjective).then_some(super::quest_protocol::Id {
            scope: self.world,
            slot: self.serial as u8,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    Denied,
    Busy,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    Dead,
    TargetDead,
    WrongLane,
    OutOfRange,
    Blocked,
    TooFast,
    TooHeavy,
    Unsupported,
    Numeric,
    StaleState,
    NotDead,
    StaleLife,
    SpawnBlocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NpcAction {
    Idle,
    Walk,
    Hurt,
    Death,
    Attack,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub serial: u64,
    pub catalog_source_id: u32,
    pub npc_id: u32,
    pub graphic: u32,
    pub lane: u8,
    pub stage: u8,
    pub heading: u8,
    pub action: NpcAction,
    pub cell: WorldCell,
    pub hp: i64,
    pub max_hp: i64,
    pub mp: i64,
    pub max_mp: i64,
    pub life: u64,
    pub born_revision: i64,
    pub changed_revision: i64,
    pub corpse_until_millis: i64,
    pub next_move_millis: i64,
    pub next_attack_millis: i64,
}
impl Actor {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }
    fn validate(&self) -> Result<(), ApplicationError> {
        if !positive(self.serial)
            || !app::valid_source(self.catalog_source_id)
            || self.npc_id == 0
            || self.npc_id > i32::MAX as u32
            || super::npc_assets::NpcAppearance::new(self.graphic).is_err()
            || self.lane > 1
            || !(1..=4).contains(&self.stage)
            || self.heading > 7
            || self.max_hp < 1
            || self.hp < 0
            || self.hp > self.max_hp
            || self.max_mp < 0
            || self.mp < 0
            || self.mp > self.max_mp
            || !positive(self.life)
            || self.born_revision < 1
            || self.changed_revision < self.born_revision
            || [
                self.corpse_until_millis,
                self.next_move_millis,
                self.next_attack_millis,
            ]
            .iter()
            .any(|v| !(0..=MAX_ELAPSED + 60_000).contains(v))
            || (self.hp == 0) != (self.action == NpcAction::Death)
            || (self.hp == 0) != (self.corpse_until_millis > 0)
            || self.hp == 0 && (self.next_move_millis != 0 || self.next_attack_millis != 0)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    PlayerToNpc,
    NpcToParticipant,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub hit: bool,
    pub death: bool,
    pub maximum: bool,
    pub critical: bool,
    pub raw_damage: i64,
    pub damage: i64,
    pub applied_damage: i64,
    pub remaining_hp: i64,
    pub material_bonus: i64,
    pub blessing_bonus: i64,
}
impl Outcome {
    fn validate(&self) -> Result<(), ApplicationError> {
        if self.damage < 0
            || self.applied_damage < 0
            || self.applied_damage > self.damage
            || self.remaining_hp < 0
            || !(0..=20).contains(&self.material_bonus)
            || !(0..=4).contains(&self.blessing_bonus)
            || !self.hit
                && (self.raw_damage != 0
                    || self.damage != 0
                    || self.applied_damage != 0
                    || self.death
                    || self.maximum
                    || self.critical
                    || self.material_bonus != 0
                    || self.blessing_bonus != 0)
            || self.death != (self.remaining_hp == 0 && self.applied_damage > 0)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub participant: ParticipantId,
    pub direction: Direction,
    pub participant_life: u64,
    pub npc: Id,
    pub npc_life: u64,
    pub request_sequence: u64,
    pub revision: i64,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub source_id: u32,
    pub npc_id: u32,
    pub graphic: u32,
    pub name: String,
    pub name_id: String,
}
impl Name {
    fn key(&self) -> (u32, u32) {
        (self.source_id, self.npc_id)
    }
    fn validate(&self) -> bool {
        app::valid_source(self.source_id)
            && app::valid_source(self.npc_id)
            && super::npc_assets::NpcAppearance::new(self.graphic).is_ok()
            && label_valid(&self.name, false)
            && label_valid(&self.name_id, true)
    }
}
fn label_valid(value: &str, empty: bool) -> bool {
    value.len() <= 256
        && (empty && value.is_empty() || !value.trim().is_empty())
        && !value.chars().any(char::is_control)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub combat: combat::Frame,
    pub scope: Scope,
    pub revision: i64,
    pub elapsed_millis: i64,
    pub spawned: u64,
    pub closed: bool,
    pub actors: Vec<Actor>,
    pub exchanges: Vec<Exchange>,
    /// v18: full immutable generation roster, not just currently spawned forms.
    /// None is only the preserved v17 codec contract, never production preflight.
    pub resources: Option<Vec<u32>>,
    /// v28: immutable generation-wide selected-catalog labels, including forms
    /// not spawned yet. Never keyed only by graphic or by player object ID.
    pub names: Option<Vec<Name>>,
}
impl Frame {
    pub fn name(&self, id: Id) -> Option<&Name> {
        let actor = self.actor(id)?;
        self.names.as_ref()?.iter().find(|n| {
            n.source_id == actor.catalog_source_id
                && n.npc_id == actor.npc_id
                && n.graphic == actor.graphic
        })
    }
    pub fn actor(&self, id: Id) -> Option<&Actor> {
        (id.scope == self.scope)
            .then(|| self.actors.iter().find(|a| a.serial == id.serial))
            .flatten()
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if let Some(names) = &self.names {
            let forms = self.resources.as_ref().ok_or_else(invalid)?;
            if names.is_empty()
                || names.len() > 128
                || names.iter().any(|n| !n.validate())
                || names.windows(2).any(|w| w[0].key() >= w[1].key())
                || names
                    .iter()
                    .map(|n| n.graphic)
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    != *forms
                || self.actors.iter().any(|a| {
                    self.name(Id {
                        scope: self.scope,
                        serial: a.serial,
                    })
                    .is_none()
                })
            {
                return Err(invalid());
            }
        }
        if let Some(forms) = &self.resources {
            if forms.is_empty()
                || forms.len() > 10
                || forms.windows(2).any(|w| w[0] >= w[1])
                || forms
                    .iter()
                    .any(|g| super::npc_assets::NpcAppearance::new(*g).is_err())
                || self
                    .actors
                    .iter()
                    .any(|n| forms.binary_search(&n.graphic).is_err())
            {
                return Err(invalid());
            }
        }
        self.combat.validate()?;
        super::npc_spell_protocol::validate_npcs(self)?;
        super::npc_disintegrate_wire::validate_npcs(self)?;
        super::npc_fire_storm_frame::validate_npcs(self)?;
        let w = &self.combat.world;
        if !self.scope.valid()
            || !w
                .descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::NPC))
            || self.scope.room_id != w.room.summary.room_id
            || self.scope.match_id != w.match_id
            || self.scope.generation != w.generation
            || self.revision < 1
            || self.revision >= w.revision
            || !(0..=MAX_ELAPSED).contains(&self.elapsed_millis)
            || self.spawned > i64::MAX as u64
            || self.actors.len() > 128
            || self.exchanges.len() > 228
            || self.closed != (w.phase == world::Phase::Aborted)
            || self.closed && !self.actors.is_empty()
            || w.phase == world::Phase::Preparing
            || w.phase == world::Phase::Finished && w.quest.is_none() && w.time_attack.is_none()
            || !matches!(
                w.phase,
                world::Phase::Active | world::Phase::Aborted | world::Phase::Finished
            ) && (self.spawned != 0 || self.elapsed_millis != 0 || !self.exchanges.is_empty())
        {
            return Err(invalid());
        }
        super::quest_protocol::validate_npcs(self)?;
        super::time_attack_protocol::validate_npcs(self)?;
        let mut cells: HashSet<_> = w
            .actors
            .iter()
            .filter(|a| w.actor_occupies(a.entity_id))
            .map(|a| (a.position.x, a.position.y))
            .collect();
        if let Some(d) = &w.time_attack {
            if d.tower_hp > 0 {
                cells.insert((d.tower_cell.x, d.tower_cell.y));
            }
        }
        let mut previous = 0;
        for a in &self.actors {
            a.validate()?;
            if a.serial <= previous
                || a.serial > self.spawned
                || a.changed_revision > self.revision
                || !w.manifest.contains(a.cell)
                || w.audience_lane.is_some_and(|lane| a.lane != lane)
                || a.alive() && !cells.insert((a.cell.x, a.cell.y))
                || !a.alive() && a.corpse_until_millis <= self.elapsed_millis
            {
                return Err(invalid());
            }
            previous = a.serial;
        }
        let mut humans = HashSet::new();
        let mut npcs = HashSet::new();
        let mut previous = 0;
        for e in &self.exchanges {
            e.outcome.validate()?;
            if !e.participant.valid()
                || !positive(e.participant_life)
                || e.npc.scope != self.scope
                || !positive(e.npc.serial)
                || e.npc.serial > self.spawned
                || !positive(e.npc_life)
                || e.revision <= previous
                || e.revision > w.revision
            {
                return Err(invalid());
            }
            let p = w
                .actors
                .iter()
                .find(|a| a.identity() == e.participant)
                .ok_or_else(invalid)?;
            let v = self.combat.vital(p.entity_id).ok_or_else(invalid)?;
            if let Some(l) = super::quest_life::life(w, p.entity_id) {
                if e.participant_life > l.incarnation()
                    || if e.participant_life == l.incarnation() {
                        e.revision <= l.respawn_revision
                    } else {
                        e.revision > l.respawn_revision
                    }
                {
                    return Err(invalid());
                }
            } else if e.participant_life != 1 {
                return Err(invalid());
            }
            let n = self.actor(e.npc);
            if n.is_some_and(|n| e.npc_life > n.life) {
                return Err(invalid());
            }
            match e.direction {
                Direction::PlayerToNpc => {
                    if e.participant.kind != MemberKind::Human
                        || !positive(e.request_sequence)
                        || e.request_sequence > v.attack_sequence
                        || !humans.insert(e.participant)
                    {
                        return Err(invalid());
                    }
                    if e.revision == w.revision
                        && (n
                            .is_none_or(|n| n.life != e.npc_life || n.hp != e.outcome.remaining_hp)
                            || v.hp == 0
                            || e.request_sequence != v.attack_sequence)
                    {
                        return Err(invalid());
                    }
                }
                Direction::NpcToParticipant => {
                    if e.request_sequence != 0 || !npcs.insert(e.npc) || n.is_none() {
                        return Err(invalid());
                    }
                    if e.revision == w.revision
                        && (v.hp != e.outcome.remaining_hp
                            || n.is_none_or(|n| {
                                n.life != e.npc_life || n.action != NpcAction::Attack
                            })
                            || e.outcome.applied_damage > 0 && v.hurt_revision != e.revision)
                    {
                        return Err(invalid());
                    }
                }
            }
            previous = e.revision;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub reason: Reason,
    pub frame: Option<Frame>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == Status::Ok) != (self.reason == Reason::None) {
            return Err(invalid());
        }
        let Some(f) = &self.frame else {
            return if self.status == Status::Unavailable && self.reason == Reason::NotActive {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        f.validate()?;
        if f.scope != self.echo.world {
            return Err(invalid());
        }
        let w = &f.combat.world;
        let p = w
            .actors
            .iter()
            .find(|p| p.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        let v = f.combat.vital(p.entity_id).ok_or_else(invalid)?;
        if self.echo.action == Action::ConfirmDeath {
            let l = super::quest_life::life(w, p.entity_id).ok_or_else(invalid)?;
            if self.echo.serial != p.entity_id
                || self.status == Status::Ok
                    && (w.phase != world::Phase::Active
                        || l.dead()
                        || v.hp != v.max_hp
                        || l.death_revision as u64 != self.echo.sequence
                        || self.echo.life.checked_add(1) != Some(l.incarnation())
                        || l.respawn_revision != w.revision
                        || p.position != l.spawn)
                || self.status == Status::Denied
                    && !matches!(
                        self.reason,
                        Reason::NotActive
                            | Reason::NotDead
                            | Reason::StaleLife
                            | Reason::SpawnBlocked
                            | Reason::Numeric
                    )
                || self.status == Status::Denied
                    && self.reason != Reason::NotActive
                    && w.phase != world::Phase::Active
                || self.reason == Reason::NotDead && l.dead()
                || self.reason == Reason::SpawnBlocked && !l.dead()
            {
                return Err(invalid());
            }
            return Ok(());
        }
        if matches!(
            self.reason,
            Reason::NotDead | Reason::StaleLife | Reason::SpawnBlocked
        ) {
            return Err(invalid());
        }
        if self.echo.action == Action::AttackObjective {
            let q = w.quest.as_ref().ok_or_else(invalid)?;
            let final_revision = w
                .completion
                .as_ref()
                .map_or(w.revision, |c| c.event.engine_revision.saturating_add(1));
            if self.status == Status::Ok
                && (!matches!(w.phase, world::Phase::Active | world::Phase::Finished)
                    || v.hp == 0
                    || v.attack_sequence != self.echo.sequence
                    || !q.exchanges.iter().any(|e| {
                        e.entity_id == p.entity_id
                            && u64::from(e.slot) == self.echo.serial
                            && e.sequence == self.echo.sequence
                            && e.revision == final_revision
                    }))
            {
                return Err(invalid());
            }
        }
        // Settlement publication can advance the enclosing frame before the
        // final NPC attack is acknowledged. Its receipt pins the original
        // engine event; do not accept arbitrary older exchanges.
        let attack_revision = w
            .time_attack
            .as_ref()
            .and_then(|d| d.receipt.as_ref())
            .map_or(Ok(w.revision), |r| {
                r.revision.checked_add(1).ok_or_else(invalid)
            })?;
        if self.echo.action == Action::Attack
            && self.status == Status::Ok
            && (w.phase != world::Phase::Active
                && !(w.phase == world::Phase::Finished && w.time_attack.is_some())
                || v.hp == 0
                || v.attack_sequence != self.echo.sequence
                || !f.exchanges.iter().any(|e| {
                    e.direction == Direction::PlayerToNpc
                        && e.participant == p.identity()
                        && Some(e.npc) == self.echo.target()
                        && e.npc_life == self.echo.life
                        && e.request_sequence == self.echo.sequence
                        && e.revision == attack_revision
                }))
        {
            return Err(invalid());
        }
        if self.status == Status::Denied && self.reason != Reason::NotActive {
            if self.echo.action == Action::Read
                || w.phase != world::Phase::Active
                || if self.reason == Reason::StaleState {
                    v.attack_sequence < self.echo.sequence
                } else {
                    v.attack_sequence != self.echo.sequence
                }
                || self.reason == Reason::Dead && v.hp != 0
            {
                return Err(invalid());
            }
        }
        if self.echo.action == Action::AttackTower {
            let d = w.time_attack.as_ref().ok_or_else(invalid)?;
            if self.status == Status::Ok
                && (w.phase != world::Phase::Active
                    || v.hp == 0
                    || d.departed(p.entity_id)
                    || v.attack_sequence != self.echo.sequence
                    || !d.exchanges.iter().any(|e| {
                        e.entity_id == p.entity_id
                            && e.sequence == self.echo.sequence
                            && e.revision.checked_add(1) == Some(w.revision)
                    }))
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn confirms(&self, r: &Request) -> bool {
        self.echo == *r && self.validate().is_ok()
    }
    pub(crate) fn combat_reply(&self) -> combat::Reply {
        combat::Reply {
            echo: combat::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: combat::Action::Read,
                match_id: self.echo.world.match_id,
                generation: self.echo.world.generation,
                sequence: 0,
                target_entity_id: 0,
            },
            status: combat::Status::Ok,
            reason: combat::Reason::None,
            frame: self.frame.as_ref().map(|f| f.combat.clone()),
        }
    }
}
/// Authoritative full snapshots may skip updates, but cannot rewind or resurrect
/// a retired serial. Receipts survive polling; no optimistic damage is applied.
pub fn follows(prior: Option<&Reply>, next: &Reply) -> bool {
    if next.validate().is_err() {
        return false;
    }
    let Some(old) = prior else {
        return true;
    };
    if old.echo.context != next.echo.context
        || old.echo.channel_id != next.echo.channel_id
        || old.echo.world != next.echo.world
    {
        return false;
    }
    let (Some(a), Some(b)) = (&old.frame, &next.frame) else {
        return old.frame.is_none();
    };
    if !combat::follows(Some(&old.combat_reply()), &next.combat_reply())
        || a.resources != b.resources
        || a.names != b.names
        || b.revision < a.revision
        || b.elapsed_millis < a.elapsed_millis
        || b.spawned < a.spawned
    {
        return false;
    }
    if a.combat.world.revision == b.combat.world.revision {
        return a == b;
    }
    if a.revision == b.revision
        && (a.actors != b.actors
            || a.elapsed_millis != b.elapsed_millis
            || a.spawned != b.spawned
            || a.closed != b.closed)
    {
        return false;
    }
    for old in &a.actors {
        let Some(new) = b.actors.iter().find(|n| n.serial == old.serial) else {
            if !b.closed && (old.alive() || b.elapsed_millis < old.corpse_until_millis) {
                return false;
            }
            continue;
        };
        if new.catalog_source_id != old.catalog_source_id
            || new.npc_id != old.npc_id
            || new.graphic != old.graphic
            || new.max_hp != old.max_hp
            || new.max_mp != old.max_mp
            || new.lane != old.lane
            || new.stage != old.stage
            || new.born_revision != old.born_revision
            || new.life < old.life
            || new.changed_revision < old.changed_revision
            || new.changed_revision == old.changed_revision && new != old
            || new.life == old.life && !old.alive() && new.alive()
        {
            return false;
        }
    }
    for new in &b.actors {
        if !a.actors.iter().any(|old| old.serial == new.serial)
            && (new.serial <= a.spawned || new.born_revision <= a.revision)
        {
            return false;
        }
    }
    for old in &a.exchanges {
        let same_attacker = |e: &&Exchange| {
            e.direction == old.direction
                && match e.direction {
                    Direction::PlayerToNpc => e.participant == old.participant,
                    Direction::NpcToParticipant => e.npc == old.npc,
                }
        };
        let Some(new) = b.exchanges.iter().find(same_attacker) else {
            if old.direction == Direction::PlayerToNpc || b.actor(old.npc).is_some() {
                return false;
            }
            continue;
        };
        if new.revision < old.revision
            || new.revision == old.revision && new != old
            || new.revision > old.revision && new.revision <= a.combat.world.revision
            || new.revision > old.revision
                && match new.direction {
                    Direction::PlayerToNpc => new.request_sequence <= old.request_sequence,
                    Direction::NpcToParticipant => new.npc_life < old.npc_life,
                }
        {
            return false;
        }
    }
    for new in &b.exchanges {
        if new.revision <= a.combat.world.revision && !a.exchanges.iter().any(|old| old == new) {
            return false;
        }
    }
    true
}
pub fn encode_request(id: u32, r: &Request) -> Result<Vec<u8>, ApplicationError> {
    r.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    out.extend(r.context.key.source_id.to_be_bytes());
    out.extend(r.context.key.object_id.to_be_bytes());
    out.extend(r.context.epoch.to_be_bytes());
    out.extend([r.channel_id, r.action as u8, 0, 0]);
    for value in [
        r.world.room_id,
        r.world.match_id,
        r.world.generation,
        r.sequence,
        r.serial,
        r.life,
    ] {
        out.extend(value.to_be_bytes());
    }
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn names(&mut self) -> Result<Vec<Name>, ApplicationError> {
        if self.u16()? != 1 {
            return Err(invalid());
        }
        let count = usize::from(self.u16()?);
        self.zero(4)?;
        if !(1..=128).contains(&count) {
            return Err(invalid());
        }
        let mut rows = Vec::with_capacity(count);
        for _ in 0..count {
            let row = Name {
                source_id: self.u32()?,
                npc_id: self.u32()?,
                graphic: self.u32()?,
                name: self.text(false)?,
                name_id: self.text(true)?,
            };
            if !row.validate() || rows.last().is_some_and(|p: &Name| p.key() >= row.key()) {
                return Err(invalid());
            }
            rows.push(row);
        }
        Ok(rows)
    }
    fn text(&mut self, empty: bool) -> Result<String, ApplicationError> {
        let count = usize::from(self.u16()?);
        if count > 256 {
            return Err(invalid());
        }
        let value = std::str::from_utf8(self.take(count)?).map_err(|_| invalid())?;
        if !label_valid(value, empty) {
            return Err(invalid());
        }
        Ok(value.to_owned())
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let v = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
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
        if self.take(n)?.iter().any(|v| *v != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    fn boolean(&mut self) -> Result<bool, ApplicationError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
    }
    fn scope(&mut self) -> Result<Scope, ApplicationError> {
        Ok(Scope {
            room_id: self.u64()?,
            match_id: self.u64()?,
            generation: self.u64()?,
        })
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if version < VERSION
        || version > super::protocol::MAX_VERSION
        || bytes.len() > frame_limit(version) - 16
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
    let action = match r.u8()? {
        0 => Action::Read,
        1 => Action::Attack,
        2 if version >= 21 => Action::AttackObjective,
        3 if version >= 22 => Action::ConfirmDeath,
        4 if version >= 25 => Action::AttackTower,
        _ => return Err(invalid()),
    };
    r.zero(2)?;
    let echo = Request {
        context,
        channel_id,
        action,
        world: r.scope()?,
        sequence: r.u64()?,
        serial: r.u64()?,
        life: r.u64()?,
    };
    echo.validate()?;
    let status = match r.u8()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Busy,
        3 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match r.u8()? {
        0 => Reason::None,
        1 => Reason::NotActive,
        2 => Reason::Dead,
        3 => Reason::TargetDead,
        4 => Reason::WrongLane,
        5 => Reason::OutOfRange,
        6 => Reason::Blocked,
        7 => Reason::TooFast,
        8 => Reason::TooHeavy,
        9 => Reason::Unsupported,
        10 => Reason::Numeric,
        11 => Reason::StaleState,
        12 if version >= 22 => Reason::NotDead,
        13 if version >= 22 => Reason::StaleLife,
        14 if version >= 22 => Reason::SpawnBlocked,
        _ => return Err(invalid()),
    };
    let present = r.boolean()?;
    r.zero(1)?;
    let frame = if present {
        let (frame, used) = decode_frame(&bytes[r.pos..], context.key, version)?;
        r.take(used)?;
        Some(frame)
    } else {
        None
    };
    if r.pos != bytes.len() {
        return Err(invalid());
    }
    let result = Reply {
        echo,
        status,
        reason,
        frame,
    };
    result.validate()?;
    Ok(result)
}
pub(crate) fn decode_frame(
    bytes: &[u8],
    key: app::CharacterKey,
    version: u16,
) -> Result<(Frame, usize), ApplicationError> {
    if !(VERSION..=world::CODEC_MAX_VERSION).contains(&version)
        || bytes.len() > frame_limit(version)
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let (combat, used) = combat::decode_frame_version(&bytes[r.pos..], key, version)?;
    r.take(used)?;
    let scope = r.scope()?;
    let revision = r.i64()?;
    let elapsed_millis = r.i64()?;
    let spawned = r.u64()?;
    let closed = r.boolean()?;
    r.zero(3)?;
    let count = usize::from(r.u16()?);
    let events = usize::from(r.u16()?);
    if count > 128
        || events > 228
        || bytes.len() - r.pos < count * ACTOR_BYTES + events * EXCHANGE_BYTES
    {
        return Err(invalid());
    }
    let mut actors = Vec::with_capacity(count);
    for _ in 0..count {
        let serial = r.u64()?;
        let catalog_source_id = r.u32()?;
        let npc_id = r.u32()?;
        let graphic = r.u32()?;
        let lane = r.u8()?;
        let stage = r.u8()?;
        let heading = r.u8()?;
        let action = match r.u8()? {
            0 => NpcAction::Idle,
            1 => NpcAction::Walk,
            2 => NpcAction::Hurt,
            3 => NpcAction::Death,
            4 => NpcAction::Attack,
            _ => return Err(invalid()),
        };
        actors.push(Actor {
            serial,
            catalog_source_id,
            npc_id,
            graphic,
            lane,
            stage,
            heading,
            action,
            cell: WorldCell {
                x: r.i32()?,
                y: r.i32()?,
            },
            hp: r.i64()?,
            max_hp: r.i64()?,
            mp: r.i64()?,
            max_mp: r.i64()?,
            life: r.u64()?,
            born_revision: r.i64()?,
            changed_revision: r.i64()?,
            corpse_until_millis: r.i64()?,
            next_move_millis: r.i64()?,
            next_attack_millis: r.i64()?,
        });
    }
    let mut exchanges = Vec::with_capacity(events);
    for _ in 0..events {
        let kind = match r.u8()? {
            0 => MemberKind::Human,
            1 => MemberKind::ServerBot,
            _ => return Err(invalid()),
        };
        let direction = match r.u8()? {
            0 => Direction::PlayerToNpc,
            1 => Direction::NpcToParticipant,
            _ => return Err(invalid()),
        };
        let flags = r.u8()?;
        if flags > 15 {
            return Err(invalid());
        }
        r.zero(1)?;
        let participant = ParticipantId {
            kind,
            source_id: r.u32()?,
            id: r.u64()?,
        };
        let participant_life = r.u64()?;
        let npc = Id {
            scope,
            serial: r.u64()?,
        };
        let npc_life = r.u64()?;
        let request_sequence = r.u64()?;
        let revision = r.i64()?;
        let outcome = Outcome {
            hit: flags & 1 != 0,
            death: flags & 2 != 0,
            maximum: flags & 4 != 0,
            critical: flags & 8 != 0,
            raw_damage: r.i64()?,
            damage: r.i64()?,
            applied_damage: r.i64()?,
            remaining_hp: r.i64()?,
            material_bonus: r.i64()?,
            blessing_bonus: r.i64()?,
        };
        exchanges.push(Exchange {
            participant,
            direction,
            participant_life,
            npc,
            npc_life,
            request_sequence,
            revision,
            outcome,
        });
    }
    let resources = if version >= RESOURCE_VERSION {
        if r.u16()? != 1 {
            return Err(invalid());
        }
        let count = usize::from(r.u16()?);
        r.zero(4)?;
        if !(1..=10).contains(&count) {
            return Err(invalid());
        }
        let mut forms = Vec::with_capacity(count);
        for _ in 0..count {
            forms.push(r.u32()?);
        }
        Some(forms)
    } else {
        None
    };
    let names = if version >= NAMES_VERSION {
        Some(r.names()?)
    } else {
        None
    };
    let frame = Frame {
        combat,
        scope,
        revision,
        elapsed_millis,
        spawned,
        closed,
        actors,
        exchanges,
        resources,
        names,
    };
    if r.pos > frame_limit(version) - 16 {
        return Err(invalid());
    }
    frame.validate()?;
    Ok((frame, r.pos))
}
fn positive(v: u64) -> bool {
    v > 0 && v <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid NPC frame")
}

#[cfg(test)]
#[path = "npc_protocol_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "npc_names_tests.rs"]
mod names_tests;
