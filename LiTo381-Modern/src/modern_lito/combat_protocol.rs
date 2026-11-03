//! Exact atomic server world/vitals. No client dice, damage, optimistic health,
//! or native packet casts. Event revisions denote authority events, not timing.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    world_protocol as world,
};
pub const REQUEST_KIND: u16 = 22;
pub const RESULT_KIND: u16 = 23;
pub const REQUEST_BYTES: usize = 56;
pub const MAX_FRAME_BYTES: usize = 128 * 1024;
pub const MAX_TIMED_FRAME_BYTES: usize = 256 * 1024;
pub fn frame_limit(version: u16) -> usize {
    if version >= 14 {
        1024 * 1024
    } else if version >= 10 {
        MAX_TIMED_FRAME_BYTES
    } else {
        MAX_FRAME_BYTES
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Attack,
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub match_id: u64,
    pub generation: u64,
    pub sequence: u64,
    pub target_entity_id: u64,
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8, world: &world::Frame) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            match_id: world.match_id,
            generation: world.generation,
            sequence: 0,
            target_entity_id: 0,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_source(self.context.key.source_id)
            || !positive(self.context.key.object_id)
            || self.context.epoch == 0
            || self.channel_id > 3
            || !positive(self.match_id)
            || !positive(self.generation)
            || match self.action {
                Action::Read => self.sequence != 0 || self.target_entity_id != 0,
                Action::Attack => !positive(self.sequence) || !positive(self.target_entity_id),
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vital {
    pub entity_id: u64,
    pub hp: i64,
    pub max_hp: i64,
    pub mp: i64,
    pub max_mp: i64,
    pub ac: i64,
    pub weapon_type: u8,
    pub attack_sequence: u64,
    pub attack_revision: i64,
    pub hurt_revision: i64,
    pub death_revision: i64,
    pub last_target_entity_id: u64,
    pub attack_interval_nanos: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub world: world::Frame,
    pub vitals: Vec<Vital>,
    pub damages: Vec<super::spell_protocol::DamageCastState>,
    pub items: Vec<super::item_protocol::ItemState>,
    pub weights: Vec<super::weight_protocol::ActorWeight>,
    pub timed: Vec<super::timed_protocol::State>,
    pub ammo: Vec<super::ammo_protocol::State>,
    pub regen: Vec<super::regen_protocol::State>,
    pub honor: Option<super::honor_protocol::State>,
    pub fires: Vec<super::fire_storm_protocol::State>,
    pub npc_spells: Vec<super::npc_spell_protocol::State>,
    pub disintegrations: Vec<super::disintegrate_protocol::State>,
    pub npc_disintegrations: Vec<super::npc_disintegrate_wire::State>,
    pub npc_fires: Vec<super::npc_fire_storm_wire::State>,
}
impl Frame {
    pub fn npc_fire(&self, entity: u64) -> Option<&super::npc_fire_storm_wire::State> {
        self.npc_fires.iter().find(|s| s.entity_id == entity)
    }
    pub fn npc_disintegrate(&self, entity: u64) -> Option<&super::npc_disintegrate_wire::State> {
        self.npc_disintegrations
            .iter()
            .find(|s| s.entity_id == entity)
    }
    pub fn disintegrate(&self, entity: u64) -> Option<&super::disintegrate_protocol::State> {
        self.disintegrations.iter().find(|s| s.entity_id == entity)
    }
    pub fn npc_spell(&self, entity: u64) -> Option<&super::npc_spell_protocol::State> {
        self.npc_spells.iter().find(|s| s.entity_id == entity)
    }
    pub fn fire(&self, entity: u64) -> Option<&super::fire_storm_protocol::State> {
        self.fires.iter().find(|s| s.entity_id == entity)
    }
    pub fn regen(&self, entity: u64) -> Option<&super::regen_protocol::State> {
        self.regen.iter().find(|s| s.entity_id == entity)
    }
    pub fn ammo(&self, entity: u64) -> Option<&super::ammo_protocol::State> {
        self.ammo.iter().find(|s| s.entity_id == entity)
    }
    pub fn timed(&self, entity: u64) -> Option<&super::timed_protocol::State> {
        self.timed.iter().find(|s| s.entity_id == entity)
    }
    pub fn weight(&self, entity: u64) -> Option<&super::weight_protocol::Snapshot> {
        self.weights
            .iter()
            .find(|w| w.entity_id == entity)
            .map(|w| &w.weight)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        let Some(first) = self.world.actors.first() else {
            return Err(invalid());
        };
        self.world
            .validate(first.identity().character().ok_or_else(invalid)?)?;
        if let Some(d) = &self.world.descriptor {
            use world::MatchDescriptor as M;
            if !d.combat()
                || [
                    (M::LIGHT_ARROW, !self.damages.is_empty()),
                    (M::ITEMS, !self.items.is_empty()),
                    (M::WEIGHT, !self.weights.is_empty()),
                    (M::TIMED, !self.timed.is_empty()),
                    (M::AMMO, !self.ammo.is_empty()),
                    (M::REGEN, !self.regen.is_empty()),
                    (M::HONOR, self.honor.is_some()),
                    (M::FIRE_STORM, !self.fires.is_empty()),
                    (M::NPC_LIGHT_ARROW, !self.npc_spells.is_empty()),
                    (M::DISINTEGRATE, !self.disintegrations.is_empty()),
                    (M::NPC_DISINTEGRATE, !self.npc_disintegrations.is_empty()),
                    (M::NPC_FIRE_STORM, !self.npc_fires.is_empty()),
                ]
                .into_iter()
                .any(|(feature, present)| d.has(feature) != present)
            {
                return Err(invalid());
            }
        }
        if self.world.phase == world::Phase::Preparing
            || self.vitals.len() != self.world.actors.len()
            || !self.damages.is_empty() && self.damages.len() != self.world.actors.len()
        {
            return Err(invalid());
        }
        super::weight_protocol::validate_world(self)?;
        for (damage, actor) in self.damages.iter().zip(&self.world.actors) {
            if damage.entity_id != actor.entity_id {
                return Err(invalid());
            }
            damage.validate_combat(self)?;
        }
        for (v, a) in self.vitals.iter().zip(&self.world.actors) {
            if v.entity_id != a.entity_id
                || v.max_hp < 1
                || v.hp < 0
                || v.hp > v.max_hp
                || v.max_mp < 0
                || v.mp < 0
                || v.mp > v.max_mp
                || v.weapon_type > 18
                || v.attack_sequence > i64::MAX as u64
                || v.attack_revision < 0
                || v.hurt_revision < 0
                || v.death_revision < 0
                || v.attack_revision > self.world.revision
                || v.hurt_revision > self.world.revision
                || v.death_revision > self.world.revision
                || self.honor.is_none()
                    && super::quest_life::life(&self.world, v.entity_id).is_none()
                    && super::siege_protocol::life(&self.world, v.entity_id).is_none()
                    && (v.hp == 0) != (v.death_revision > 0)
                || (v.attack_revision == 0) != (v.last_target_entity_id == 0)
                || v.attack_revision > 0
                    && (v.attack_sequence == 0
                        || v.last_target_entity_id == v.entity_id
                        || super::honor_protocol::target(self, v.last_target_entity_id).is_none())
                || !(1_000_000..=10_000_000_000).contains(&v.attack_interval_nanos)
            {
                return Err(invalid());
            }
            if self.regen.is_empty()
                && self
                    .world
                    .cast(a.entity_id)
                    .is_some_and(|cast| cast.mp_spent > v.max_mp)
            {
                return Err(invalid());
            }
            if let Some(teleport) = self
                .world
                .cast(a.entity_id)
                .filter(|_| self.regen.is_empty())
            {
                let remaining = i64::try_from(self.damage(a.entity_id).map_or(0, |d| d.mp_spent))
                    .ok()
                    .and_then(|spent| teleport.mp_spent.checked_add(spent))
                    .and_then(|spent| {
                        spent.checked_add(self.npc_spell(a.entity_id).map_or(0, |s| s.mp_spent))
                    })
                    .and_then(|spent| v.max_mp.checked_sub(spent));
                if remaining != Some(v.mp) {
                    return Err(invalid());
                }
            }
        }
        if let Some(completion) = &self.world.completion {
            if completion.event.mode != 1 && self.honor.is_none() && self.world.quest.is_none() {
                return Err(invalid());
            }
            let event = &completion.event;
            let final_revision = event.engine_revision.checked_add(1).ok_or_else(invalid)?;
            let mut living_winner = false;
            let mut final_death = false;
            for v in &self.vitals {
                let p = event
                    .participants
                    .iter()
                    .find(|p| p.entity_id == v.entity_id)
                    .ok_or_else(invalid)?;
                if v.attack_revision > final_revision
                    || v.hurt_revision > final_revision
                    || v.death_revision > final_revision
                {
                    return Err(invalid());
                }
                if event.mode != 1 {
                    continue;
                }
                if p.verdict == world::Verdict::Loss {
                    if v.hp != 0 {
                        return Err(invalid());
                    }
                    final_death |= v.death_revision == final_revision;
                } else {
                    living_winner |= v.hp > 0;
                }
            }
            if event.mode == 1 && (!living_winner || !final_death) {
                return Err(invalid());
            }
        }
        super::honor_protocol::validate_frame(self)?;
        super::item_protocol::validate_states(self)?;
        super::timed_protocol::validate_frame(self)?;
        super::ammo_protocol::validate_frame(self)?;
        super::regen_protocol::validate_frame(self)?;
        super::npc_spell_protocol::validate_frame(self)?;
        super::fire_storm_protocol::validate_frame(self)?;
        super::disintegrate_protocol::validate_frame(self)?;
        super::npc_disintegrate_wire::validate_frame(self)?;
        super::npc_fire_storm_frame::validate_frame(self)?;
        super::quest_protocol::validate_combat(self)?;
        super::time_attack_protocol::validate_combat(self)?;
        super::siege_protocol::validate_combat(self)?;
        Ok(())
    }
    pub fn vital(&self, entity_id: u64) -> Option<&Vital> {
        self.vitals.iter().find(|v| v.entity_id == entity_id)
    }
    pub fn damage(&self, entity_id: u64) -> Option<&super::spell_protocol::DamageCastState> {
        self.damages.iter().find(|d| d.entity_id == entity_id)
    }
    pub fn item(&self, entity_id: u64) -> Option<&super::item_protocol::ItemState> {
        self.items.iter().find(|s| s.entity_id == entity_id)
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
        if f.world.match_id != self.echo.match_id || f.world.generation != self.echo.generation {
            return Err(invalid());
        }
        let a = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        if self.reason == Reason::TooHeavy
            && (self.status != Status::Denied
                || self.echo.action != Action::Attack
                || f.world.phase != world::Phase::Active
                || f.vital(a.entity_id)
                    .is_none_or(|v| v.hp <= 0 || v.attack_sequence != self.echo.sequence)
                || f.weight(a.entity_id)
                    .is_none_or(|w| !w.applicable() || w.can_attack()))
        {
            return Err(invalid());
        }
        if self.reason == Reason::NoAmmo
            && (self.status != Status::Denied
                || self.echo.action != Action::Attack
                || f.world.phase != world::Phase::Active
                || f.vital(a.entity_id).is_none_or(|v| {
                    v.hp <= 0
                        || !super::ammo_protocol::bow(v.weapon_type)
                        || v.attack_sequence != self.echo.sequence
                })
                || f.ammo(a.entity_id).is_none_or(|s| s.available()))
        {
            return Err(invalid());
        }
        if self.reason == Reason::StaleEffect
            && (self.status != Status::Denied
                || self.echo.action != Action::Attack
                || f.world.phase != world::Phase::Active
                || f.timed(a.entity_id).is_none_or(|s| s.revision <= 0)
                || f.vital(a.entity_id)
                    .is_none_or(|v| v.hp <= 0 || v.attack_sequence != self.echo.sequence))
        {
            return Err(invalid());
        }
        if self.reason == Reason::StaleRegen
            && (self.status != Status::Denied
                || self.echo.action != Action::Attack
                || f.world.phase != world::Phase::Active
                || f.vital(a.entity_id)
                    .is_none_or(|v| v.hp <= 0 || v.attack_sequence != self.echo.sequence)
                || self.echo.target_entity_id == a.entity_id
                || f.vital(self.echo.target_entity_id)
                    .is_none_or(|v| v.hp <= 0)
                || f.regen(self.echo.target_entity_id)
                    .is_none_or(|r| r.hp_revision <= 0))
        {
            return Err(invalid());
        }
        if self.echo.action == Action::Attack {
            if self.echo.target_entity_id == a.entity_id
                || super::honor_protocol::target(f, self.echo.target_entity_id).is_none()
            {
                return Err(invalid());
            }
            let v = f.vital(a.entity_id).ok_or_else(invalid)?;
            if self.status == Status::Ok
                && (!matches!(f.world.phase, world::Phase::Active | world::Phase::Finished)
                    || v.attack_sequence != self.echo.sequence
                    || v.attack_revision != f.world.revision
                    || v.last_target_entity_id != self.echo.target_entity_id)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    /// This represents exactly the embedded world, not a made-up movement ACK.
    pub fn world_reply(&self) -> Option<world::Reply> {
        self.frame.as_ref().map(|f| world::Reply {
            echo: world::Request {
                match_id: f.world.match_id,
                generation: f.world.generation,
                ..world::Request::read(self.echo.context, self.echo.channel_id)
            },
            status: world::Status::Ok,
            reason: world::Reason::None,
            frame: Some(f.world.clone()),
        })
    }
}
/// Maintain this history across interim WORLD-only frames: clearing UI vitals
/// is not permission to accept a lower attack/death sequence on the next read.
pub fn follows(prior: Option<&Reply>, next: &Reply) -> bool {
    if next.validate().is_err() {
        return false;
    }
    let Some(old) = prior else { return true };
    if old.echo.context != next.echo.context || old.echo.channel_id != next.echo.channel_id {
        return false;
    }
    let (Some(a), Some(b)) = (&old.frame, &next.frame) else {
        return old.frame.is_none();
    };
    if !world::follows(old.world_reply().as_ref(), &next.world_reply().unwrap()) {
        return false;
    }
    if a.world.revision == b.world.revision {
        return a == b;
    }
    vitals_follow(a, b)
}
pub(crate) fn vitals_follow(a: &Frame, b: &Frame) -> bool {
    if !super::spell_protocol::damages_follow(a, b)
        || !super::item_protocol::states_follow(a, b)
        || !super::weight_protocol::follows(a, b)
        || !super::timed_protocol::follows(a, b)
        || !super::ammo_protocol::follows(a, b)
        || !super::regen_protocol::follows(a, b)
        || !super::honor_protocol::follows(a, b)
        || !super::fire_storm_protocol::follows(a, b)
        || !super::npc_spell_protocol::follows(a, b)
        || !super::disintegrate_protocol::follows(a, b)
        || !super::npc_disintegrate_wire::frames_follow(a, b)
        || !super::npc_fire_storm_frame::frames_follow(a, b)
        || a.world.phase == world::Phase::Finished
            && (a.vitals != b.vitals
                || a.damages != b.damages
                || a.items != b.items
                || a.weights != b.weights
                || a.timed != b.timed
                || a.ammo != b.ammo
                || a.regen != b.regen
                || a.honor != b.honor
                || a.fires != b.fires
                || a.npc_spells != b.npc_spells
                || a.disintegrations != b.disintegrations
                || a.npc_disintegrations != b.npc_disintegrations
                || a.npc_fires != b.npc_fires)
    {
        return false;
    }
    a.vitals
        .iter()
        .zip(&b.vitals)
        .enumerate()
        .all(|(index, (old, new))| {
            old.entity_id == new.entity_id
                && new.attack_sequence >= old.attack_sequence
                && new.attack_revision >= old.attack_revision
                && new.hurt_revision >= old.hurt_revision
                && new.death_revision >= old.death_revision
                && old.max_hp == new.max_hp
                && old.max_mp == new.max_mp
                && old.ac == new.ac
                && old.weapon_type == new.weapon_type
                && old.attack_interval_nanos == new.attack_interval_nanos
                && (a.honor.is_some()
                    || super::quest_life::life(&a.world, old.entity_id).is_some()
                    || super::siege_protocol::life(&a.world, old.entity_id).is_some()
                    || old.death_revision == 0
                    || old.hp == new.hp && old.death_revision == new.death_revision)
                && (old.attack_revision != new.attack_revision
                    || old.last_target_entity_id == new.last_target_entity_id)
                && super::item_protocol::hp_follows(a, b, index)
                && super::spell_protocol::mp_follows(a, b, index)
        })
}
fn positive(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid combat frame")
}
pub fn encode_request(id: u32, request: &Request) -> Result<Vec<u8>, ApplicationError> {
    request.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    out.extend(request.context.key.source_id.to_be_bytes());
    out.extend(request.context.key.object_id.to_be_bytes());
    out.extend(request.context.epoch.to_be_bytes());
    out.extend([request.channel_id, request.action as u8, 0, 0]);
    out.extend(request.match_id.to_be_bytes());
    out.extend(request.generation.to_be_bytes());
    out.extend(request.sequence.to_be_bytes());
    out.extend(request.target_entity_id.to_be_bytes());
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let result = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(result)
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
    fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|b| *b != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
pub(crate) fn decode_body(bytes: &[u8]) -> Result<Reply, ApplicationError> {
    decode_body_version(bytes, 1)
}
pub(crate) fn decode_body_version(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if bytes.len() > frame_limit(version) - 16
        || !(1..=super::protocol::IMPLEMENTED_WORLD_VERSION).contains(&version)
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
        _ => return Err(invalid()),
    };
    r.zero(2)?;
    let echo = Request {
        context,
        channel_id,
        action,
        match_id: r.u64()?,
        generation: r.u64()?,
        sequence: r.u64()?,
        target_entity_id: r.u64()?,
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
        4 => Reason::SameTeam,
        5 => Reason::OutOfRange,
        6 => Reason::Blocked,
        7 => Reason::TooFast,
        8 => Reason::Unsupported,
        9 => Reason::Numeric,
        10 if version >= 9 => Reason::TooHeavy,
        11 if version >= 10 => Reason::StaleEffect,
        12 if version >= 11 => Reason::NoAmmo,
        13 if version >= 12 => Reason::StaleRegen,
        _ => return Err(invalid()),
    };
    let present = r.u8()?;
    r.zero(1)?;
    if present > 1 {
        return Err(invalid());
    }
    let frame = if present == 1 {
        let (frame, used) = decode_frame_version(&bytes[r.pos..], context.key, version)?;
        r.take(used)?;
        Some(frame)
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
pub(crate) fn decode_frame_version(
    bytes: &[u8],
    key: app::CharacterKey,
    version: u16,
) -> Result<(Frame, usize), ApplicationError> {
    let (world, used) = world::decode_frame_version(bytes, key, version)?;
    let mut r = Reader { bytes, pos: used };
    let count = usize::from(r.u16()?);
    r.zero(2)?;
    if count == 0 || count > 100 || count != world.actors.len() || bytes.len() - r.pos < count * 104
    {
        return Err(invalid());
    }
    let mut vitals = Vec::with_capacity(count);
    for _ in 0..count {
        let entity_id = r.u64()?;
        let hp = r.i64()?;
        let max_hp = r.i64()?;
        let mp = r.i64()?;
        let max_mp = r.i64()?;
        let ac = r.i64()?;
        let weapon_type = r.u8()?;
        r.zero(7)?;
        vitals.push(Vital {
            entity_id,
            hp,
            max_hp,
            mp,
            max_mp,
            ac,
            weapon_type,
            attack_sequence: r.u64()?,
            attack_revision: r.i64()?,
            hurt_revision: r.i64()?,
            death_revision: r.i64()?,
            last_target_entity_id: r.u64()?,
            attack_interval_nanos: r.u64()?,
        });
    }
    let mut damages = Vec::new();
    if version >= 6 {
        let count = usize::from(r.u16()?);
        r.zero(2)?;
        if count > 100
            || count != 0 && count != world.actors.len()
            || bytes.len() - r.pos < count * 64
        {
            return Err(invalid());
        }
        damages.reserve(count);
        for _ in 0..count {
            damages.push(super::spell_protocol::DamageCastState {
                entity_id: r.u64()?,
                cast_count: r.u64()?,
                cast_revision: r.u64()?,
                mp_spent: r.u64()?,
                skill_id: r.u32()?,
                target_entity_id: r.u64()?,
                from: super::world_map::WorldCell {
                    x: r.u32()? as i32,
                    y: r.u32()? as i32,
                },
                to: super::world_map::WorldCell {
                    x: r.u32()? as i32,
                    y: r.u32()? as i32,
                },
                mp_cost: r.u32()?,
            });
        }
    }
    let items = if version >= 8 {
        let (items, used) = super::item_protocol::decode_states_version(
            &bytes[r.pos..],
            world.actors.len(),
            version,
        )?;
        r.take(used)?;
        items
    } else {
        Vec::new()
    };
    let weights = if version >= 9 {
        let (weights, used) = super::weight_protocol::decode_actors(
            &bytes[r.pos..],
            world.actors.len(),
            world.audience_lane.is_some(),
        )?;
        r.take(used)?;
        weights
    } else {
        Vec::new()
    };
    let timed = if version >= 10 {
        let (states, used) =
            super::timed_protocol::decode_states(&bytes[r.pos..], world.actors.len())?;
        r.take(used)?;
        states
    } else {
        Vec::new()
    };
    let ammo = if version >= 11 {
        let (states, used) =
            super::ammo_protocol::decode_states(&bytes[r.pos..], world.actors.len())?;
        r.take(used)?;
        states
    } else {
        Vec::new()
    };
    let regen = if version >= 12 {
        let (states, used) = super::regen_protocol::decode_states(
            &bytes[r.pos..],
            world.actors.len(),
            world.audience_lane.is_some(),
        )?;
        r.take(used)?;
        states
    } else {
        Vec::new()
    };
    let honor = if version >= 13 {
        let (state, used) = super::honor_protocol::decode(&bytes[r.pos..], world.actors.len())?;
        r.take(used)?;
        state
    } else {
        None
    };
    let fires = if version >= 14 {
        let (states, used) =
            super::fire_storm_protocol::decode(&bytes[r.pos..], world.actors.len())?;
        r.pos += used;
        states
    } else {
        Vec::new()
    };
    let npc_spells = if version >= 24 {
        let (states, used) = super::npc_spell_protocol::decode_states(&bytes[r.pos..], &world)?;
        r.pos += used;
        states
    } else {
        Vec::new()
    };
    let disintegrations = if version >= 27 {
        let (states, used) =
            super::disintegrate_protocol::decode(&bytes[r.pos..], world.actors.len())?;
        r.pos += used;
        states
    } else {
        Vec::new()
    };
    let npc_disintegrations = if version >= 29 {
        let (states, used) = super::npc_disintegrate_wire::decode_states(
            &bytes[r.pos..],
            super::quest_protocol::scope(&world),
            world.actors.len(),
            version,
        )?;
        r.pos += used;
        states
    } else {
        Vec::new()
    };
    let npc_fires = if version >= 31 {
        let (states, used) = super::npc_fire_storm_wire::decode_states(
            &bytes[r.pos..],
            super::quest_protocol::scope(&world),
            world.actors.len(),
            version,
        )?;
        r.take(used)?;
        states
    } else {
        Vec::new()
    };
    if version <= 13
        && honor.as_ref().is_some_and(|h| {
            h.objectives
                .iter()
                .any(|o| o.last_kind == super::honor_protocol::Interaction::FireStorm)
        })
    {
        return Err(invalid());
    }
    let frame = Frame {
        world,
        vitals,
        damages,
        items,
        weights,
        timed,
        ammo,
        regen,
        honor,
        fires,
        npc_spells,
        disintegrations,
        npc_disintegrations,
        npc_fires,
    };
    frame.validate()?;
    Ok((frame, r.pos))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    const REQUEST: &str = "4c544d3100000048fedcba9800160000000000070000000000000d05ffffffffffffffff030000000000000000000009000000000000000600000000000000000000000000000000";
    const NONE: &str = "4c544d310000004cfedcba9800170000000000070000000000000d05ffffffffffffffff03000000000000000000000900000000000000060000000000000000000000000000000003010000";
    const PREFIX: &str = "4c544d31000001a1fedcba9800170000000000070000000000000d05ffffffffffffffff03000000000000000000000900000000000000060000000000000000000000000000000000000100";
    // Previous independent WORLD golden, without its header/echo/status.
    const WORLD: &str = "0000000000000009000000000000000600000000000000010100000000000000000000050000000000000006000204100000021c0001520001000101000000000000070000000000000d0500010000000000070000000000000d0500000000ee6b2800000301000100000000014e000153000100010000020222222222222222222222222222222222222222222222222222222222222222227fff80000000c012111111111111111111111111111111111111111111111111111111111111111100010000000000000000000100007fd4000080100200000000000000000000000000000002faf080";
    const VITAL: &str = "000000000000000100000000ee6b280000000001dcd650000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002faf080";
    fn hex(s: &str) -> Vec<u8> {
        s.as_bytes()
            .chunks_exact(2)
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    fn wire() -> Vec<u8> {
        hex(&format!("{PREFIX}{WORLD}00010000{VITAL}"))
    }
    #[test]
    fn v6_damage_appendix_has_exact_64_byte_rows_and_cannot_leak_into_older_frames() {
        let mut world = hex(WORLD);
        world.extend([0; 4]); // unchanged v3 completion
        world.extend([0, 1, 0, 0]); // unchanged v4 teleport state
        world.extend(1u64.to_be_bytes());
        world.extend([0; 52]);
        world.extend(1000u32.to_be_bytes());
        let mut data = hex(PREFIX);
        data.extend(&world);
        data.extend([0, 1, 0, 0]);
        data.extend(hex(VITAL));
        let damage_offset = data.len();
        data.extend([0, 1, 0, 0]);
        // Independent wire literal: entity1, all event/counter fields0, frozen cost3.
        data.extend(hex(concat!(
            "0000000000000001", // entity u64
            "0000000000000000", // count u64
            "0000000000000000", // revision u64
            "0000000000000000", // spent u64
            "00000000",         // skill u32
            "0000000000000000", // target u64
            "00000000",         // from x i32
            "00000000",         // from y i32
            "00000000",         // to x i32
            "00000000",         // to y i32
            "00000003",         // prepared cost u32
        )));
        let len = data.len() as u32;
        data[4..8].copy_from_slice(&len.to_be_bytes());
        assert_eq!(data.len() - damage_offset, 68);
        let decoded = app::decode_response_version(&data, 6).unwrap();
        let app::Response::Combat(reply) = decoded.response else {
            panic!("wrong kind")
        };
        assert_eq!(
            reply.frame.unwrap().damages,
            [super::super::spell_protocol::DamageCastState::initial(1, 3)]
        );
        for version in [4, 5] {
            assert!(app::decode_response_version(&data, version).is_err());
        }
        for case in 0..8 {
            let mut invalid = data.clone();
            match case {
                0 => invalid[damage_offset + 1] = 2,
                1 => invalid[damage_offset + 2] = 1,
                2 => invalid[damage_offset + 11] = 2,
                3 => invalid[damage_offset + 19] = 1,
                4 => {
                    let last = invalid.len() - 1;
                    invalid[last] = 0;
                }
                5 => {
                    let last = invalid.len() - 1;
                    invalid[last] = 4;
                }
                6 => invalid.push(0),
                _ => {
                    invalid.pop();
                }
            }
            let len = invalid.len() as u32;
            invalid[4..8].copy_from_slice(&len.to_be_bytes());
            assert!(
                app::decode_response_version(&invalid, 6).is_err(),
                "case {case}"
            );
        }
        for n in 0..data.len() {
            assert!(
                app::decode_response_version(&data[..n], 6).is_err(),
                "truncated {n}"
            );
        }
        let mut empty = data[..damage_offset].to_vec();
        empty.extend([0; 4]);
        let len = empty.len() as u32;
        empty[4..8].copy_from_slice(&len.to_be_bytes());
        assert!(app::decode_response_version(&empty, 6).is_ok());
    }
    fn read() -> Request {
        Request {
            context: SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel_id: 3,
            action: Action::Read,
            match_id: 9,
            generation: 6,
            sequence: 0,
            target_entity_id: 0,
        }
    }
    pub(crate) fn active_reply() -> Reply {
        let mut world = crate::modern_lito::world_runtime::tests::frame(1);
        world.phase = world::Phase::Active;
        for actor in &mut world.actors {
            actor.loaded = true
        }
        let vitals = world
            .actors
            .iter()
            .map(|a| Vital {
                entity_id: a.entity_id,
                hp: 50,
                max_hp: 50,
                mp: 0,
                max_mp: 0,
                ac: i64::MIN,
                weapon_type: 0,
                attack_sequence: 0,
                attack_revision: 0,
                hurt_revision: 0,
                death_revision: 0,
                last_target_entity_id: 0,
                attack_interval_nanos: 1_000_000_000,
            })
            .collect();
        let echo = Request::read(
            SelectionContext {
                key: world.actors[0].key,
                epoch: 7,
            },
            0,
            &world,
        );
        let reply = Reply {
            echo,
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(Frame {
                world,
                vitals,
                damages: Vec::new(),
                items: Vec::new(),
                weights: Vec::new(),
                timed: Vec::new(),
                ammo: Vec::new(),
                regen: Vec::new(),
                honor: None,
                fires: Vec::new(),
                npc_spells: Vec::new(),
                disintegrations: Vec::new(),
                npc_disintegrations: Vec::new(),
                npc_fires: Vec::new(),
            }),
        };
        reply.validate().unwrap();
        reply
    }
    pub(crate) fn finished_reply() -> Reply {
        let mut reply = active_reply();
        let f = reply.frame.as_mut().unwrap();
        f.world = world::tests::finished();
        reply.echo.match_id = f.world.match_id;
        reply.echo.generation = f.world.generation;
        reply.echo.action = Action::Attack;
        reply.echo.sequence = 1;
        reply.echo.target_entity_id = 2;
        f.vitals[0].attack_sequence = 1;
        f.vitals[0].attack_revision = 8;
        f.vitals[0].last_target_entity_id = 2;
        f.vitals[1].hp = 0;
        f.vitals[1].hurt_revision = 8;
        f.vitals[1].death_revision = 8;
        reply.validate().unwrap();
        reply
    }
    #[test]
    fn last_attack_finished_ack_and_later_read_receipt_keep_atomic_vitals() {
        let pending = finished_reply();
        let mut active = pending.clone();
        {
            let f = active.frame.as_mut().unwrap();
            f.world.phase = world::Phase::Active;
            f.world.completion = None;
            f.world.revision = 7;
            f.vitals[1].hp = 50;
            f.vitals[1].hurt_revision = 0;
            f.vitals[1].death_revision = 0;
            f.vitals[0].attack_sequence = 0;
            f.vitals[0].attack_revision = 0;
            f.vitals[0].last_target_entity_id = 0;
        }
        active.echo.action = Action::Read;
        active.echo.sequence = 0;
        active.echo.target_entity_id = 0;
        assert!(follows(Some(&active), &pending));
        let mut receipt = pending.clone();
        let f = receipt.frame.as_mut().unwrap();
        f.world.revision = 9;
        f.world.completion.as_mut().unwrap().state = world::SettlementState::Saved;
        assert!(
            receipt.validate().is_err(),
            "a later receipt is not the original attack ACK"
        );
        receipt.echo.action = Action::Read;
        receipt.echo.sequence = 0;
        receipt.echo.target_entity_id = 0;
        assert!(receipt.validate().is_ok());
        assert!(follows(Some(&pending), &receipt));
        assert!(follows(Some(&receipt), &receipt));
        assert!(!follows(Some(&receipt), &pending));
        let mut changed = receipt.clone();
        changed.frame.as_mut().unwrap().vitals[0].attack_sequence += 1;
        assert!(
            !follows(Some(&pending), &changed),
            "persistence must not alter combat events"
        );
    }
    #[test]
    fn finished_vitals_prove_last_kill_without_recomputing_or_excluding_dead_winners() {
        let good = finished_reply().frame.unwrap();
        for case in 0..4 {
            let mut f = good.clone();
            match case {
                0 => {
                    f.vitals[1].hp = 1;
                    f.vitals[1].death_revision = 0;
                }
                1 => {
                    f.vitals[0].hp = 0;
                    f.vitals[0].hurt_revision = 7;
                    f.vitals[0].death_revision = 7;
                }
                2 => {
                    f.vitals[1].hurt_revision = 7;
                    f.vitals[1].death_revision = 7;
                }
                _ => {
                    f.world.revision = 9;
                    f.world.completion.as_mut().unwrap().state = world::SettlementState::Saved;
                    f.vitals[0].attack_revision = 9;
                }
            }
            assert!(f.validate().is_err(), "case {case}");
        }
        // The server verdict belongs to the entire team, not just its survivors.
        let mut f = good.clone();
        let mut member = f.world.room.members[0].clone();
        member.key.source_id = 43;
        f.world.room.members.push(member.clone());
        f.world.room.summary.settings.capacity = 3;
        f.world.room.summary.settings.attributes = 0;
        f.world.room.summary.users = 3;
        f.world.room.summary.ready = 3;
        let mut actor = f.world.actors[0].clone();
        actor.entity_id = 3;
        actor.key = member.key;
        actor.position.x += 2;
        f.world.actors.push(actor);
        f.world
            .completion
            .as_mut()
            .unwrap()
            .event
            .participants
            .push(world::Participant {
                kind: member.kind,
                entity_id: 3,
                key: member.key,
                team: 0,
                verdict: world::Verdict::Win,
            });
        let mut dead = f.vitals[0].clone();
        dead.entity_id = 3;
        dead.hp = 0;
        dead.attack_sequence = 0;
        dead.attack_revision = 0;
        dead.last_target_entity_id = 0;
        dead.hurt_revision = 7;
        dead.death_revision = 7;
        f.vitals.push(dead);
        assert!(f.validate().is_ok());
    }
    #[test]
    fn independent_java_rust_full_i64_and_no_frame_goldens() {
        assert_eq!(encode_request(0xfedcba98, &read()).unwrap(), hex(REQUEST));
        let reply = decode_body(&wire()[16..]).unwrap();
        assert_eq!(reply.echo, read());
        assert_eq!(reply.frame.as_ref().unwrap().vitals[0].hp, 4_000_000_000);
        assert_eq!(
            reply.frame.as_ref().unwrap().vitals[0].max_hp,
            8_000_000_000
        );
        assert_eq!(reply.frame.as_ref().unwrap().vitals[0].ac, i64::MIN);
        assert!(matches!(
            app::decode_response(&wire()).unwrap().response,
            app::Response::Combat(_)
        ));
        let none = decode_body(&hex(NONE)[16..]).unwrap();
        assert!(none.frame.is_none());
        assert_eq!(none.reason, Reason::NotActive);
    }
    #[test]
    fn every_short_read_reserved_trailing_and_frame_budget_rejects() {
        for wire in [wire(), hex(NONE)] {
            for n in 0..wire.len() - 16 {
                assert!(decode_body(&wire[16..16 + n]).is_err(), "short body {n}");
            }
            let mut trailing = wire[16..].to_vec();
            trailing.push(0);
            assert!(decode_body(&trailing).is_err());
            for offset in [38, 39, 75] {
                let mut bad = wire.clone();
                bad[offset] = 1;
                assert!(app::decode_response(&bad).is_err());
            }
        }
        assert!(decode_body(&vec![0; MAX_FRAME_BYTES]).is_err());
        let mut oversized = wire();
        oversized.resize(MAX_FRAME_BYTES + 1, 0);
        oversized[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(app::read_response(&mut std::io::Cursor::new(oversized)).is_err());
    }
    #[test]
    fn numeric_coverage_and_event_invariants_reject_peer_coercion() {
        for (offset, value) in [
            (8, -1i64),
            (8, 8_000_000_001),
            (16, 0),
            (24, -1),
            (24, 1),
            (32, -1),
            (56, -1),
            (64, -1),
            (64, 2),
            (72, 2),
            (80, 1),
            (88, 2),
            (96, 999_999),
            (96, 10_000_000_001),
        ] {
            let mut bad = wire();
            let start = bad.len() - 104 + offset;
            bad[start..start + 8].copy_from_slice(&value.to_be_bytes());
            assert!(app::decode_response(&bad).is_err(), "vital offset {offset}");
        }
        for (offset, value) in [(48, 19), (49, 1)] {
            let mut bad = wire();
            let start = bad.len() - 104 + offset;
            bad[start] = value;
            assert!(app::decode_response(&bad).is_err());
        }
        let mut frame = active_reply().frame.unwrap();
        frame.vitals.pop();
        assert!(frame.validate().is_err());
    }
    #[test]
    fn full_echo_including_same_object_other_source_epoch_and_target_is_required() {
        let reply = decode_body(&wire()[16..]).unwrap();
        assert!(reply.confirms(&read()));
        for change in 0..4 {
            let mut request = read();
            match change {
                0 => request.context.key.source_id = 42,
                1 => request.context.epoch -= 1,
                2 => request.match_id += 1,
                3 => request.channel_id = 2,
                _ => unreachable!(),
            };
            assert!(!reply.confirms(&request));
        }
        let mut request = read();
        request.action = Action::Attack;
        assert!(request.validate().is_err());
        request.sequence = 1;
        request.target_entity_id = 2;
        assert!(request.validate().is_ok());
        request.sequence = u64::MAX;
        assert!(request.validate().is_err());
    }
    #[test]
    fn attack_and_hurt_revisions_are_distinct_from_consumed_sequence_and_never_regress() {
        let old = active_reply();
        let mut denied = old.clone();
        denied.echo.action = Action::Attack;
        denied.echo.sequence = 1;
        denied.echo.target_entity_id = 2;
        denied.status = Status::Denied;
        denied.reason = Reason::TooFast;
        let frame = denied.frame.as_mut().unwrap();
        frame.world.revision += 1;
        frame.vitals[0].attack_sequence = 1;
        assert!(follows(Some(&old), &denied));
        assert_eq!(frame_unused_attack_revision(&denied), 0);
        let mut hit = denied.clone();
        hit.status = Status::Ok;
        hit.reason = Reason::None;
        hit.echo.sequence = 2;
        let f = hit.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[0].attack_sequence = 2;
        f.vitals[0].attack_revision = f.world.revision;
        f.vitals[0].last_target_entity_id = 2;
        f.vitals[1].hp = 40;
        f.vitals[1].hurt_revision = f.world.revision;
        assert!(follows(Some(&denied), &hit));
        let mut regressed = hit.clone();
        regressed.frame.as_mut().unwrap().vitals[0].attack_sequence = 0;
        assert!(!follows(Some(&hit), &regressed));
        let mut silent = hit.clone();
        silent.frame.as_mut().unwrap().vitals[1].hp = 39;
        assert!(!follows(Some(&hit), &silent));
        let mut none = hit.clone();
        none.status = Status::Unavailable;
        none.reason = Reason::NotActive;
        none.frame = None;
        assert!(!follows(Some(&hit), &none));
    }
    fn frame_unused_attack_revision(reply: &Reply) -> i64 {
        reply.frame.as_ref().unwrap().vitals[0].attack_revision
    }
    #[test]
    fn death_is_terminal_and_frozen_maxima_and_resources_do_not_change() {
        let old = active_reply();
        let mut dead = old.clone();
        let f = dead.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[1].hp = 0;
        f.vitals[1].hurt_revision = f.world.revision;
        f.vitals[1].death_revision = f.world.revision;
        assert!(follows(Some(&old), &dead));
        let mut revived = dead.clone();
        let f = revived.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[1].hp = 1;
        f.vitals[1].death_revision = 0;
        f.vitals[1].hurt_revision = f.world.revision;
        assert!(!follows(Some(&dead), &revived));
        for change in 0..4 {
            let mut altered = old.clone();
            let f = altered.frame.as_mut().unwrap();
            f.world.revision += 1;
            match change {
                0 => f.vitals[0].max_hp += 1,
                1 => f.vitals[0].ac += 1,
                2 => f.vitals[0].weapon_type = 1,
                3 => f.vitals[0].attack_interval_nanos += 1,
                _ => unreachable!(),
            };
            assert!(!follows(Some(&old), &altered));
        }
    }
    #[test]
    fn none_is_only_preparation_unavailable_and_new_world_requires_new_history() {
        let base = active_reply();
        for status in [
            Status::Ok,
            Status::Denied,
            Status::Busy,
            Status::Unavailable,
        ] {
            for reason in [Reason::None, Reason::NotActive, Reason::Unsupported] {
                let reply = Reply {
                    status,
                    reason,
                    frame: None,
                    ..base.clone()
                };
                assert_eq!(
                    reply.validate().is_ok(),
                    status == Status::Unavailable && reason == Reason::NotActive
                );
            }
        }
        let mut next = base.clone();
        next.echo.match_id += 1;
        next.frame.as_mut().unwrap().world.match_id += 1;
        assert!(!follows(Some(&base), &next));
        assert!(follows(None, &next));
    }
}
