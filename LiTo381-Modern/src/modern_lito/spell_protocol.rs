//! Negotiated v4 learned-skill catalog and server-chosen, same-map teleport.
//! No client destination, damage, MP prediction, native pointer, or automatic retry.
#[cfg(test)]
#[path = "disintegrate_skills_tests.rs"]
mod disintegrate_skills_tests;
#[cfg(test)]
#[path = "greater_haste_skills_tests.rs"]
mod greater_haste_skills_tests;
#[cfg(test)]
#[path = "slow_skills_tests.rs"]
mod slow_skills_tests;
#[cfg(test)]
#[path = "target_haste_skills_tests.rs"]
mod target_haste_skills_tests;
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat,
    world_map::WorldCell,
    world_protocol as world,
};
use std::sync::Arc;

pub const SKILLS_REQUEST_KIND: u16 = 24;
pub const SKILLS_RESULT_KIND: u16 = 25;
pub const CAST_REQUEST_KIND: u16 = 26;
pub const CAST_RESULT_KIND: u16 = 27;
pub const MAX_SKILLS: usize = 220;
pub const MAX_NAME_BYTES: usize = 256;
pub const MAX_FRAME_BYTES: usize = 128 * 1024;
pub const TELEPORT: u32 = 5;
pub const LIGHT_ARROW: u32 = 4;
pub const BASIC_HEAL: u32 = 1;
pub const FIRE_STORM: u32 = 70;
pub const DISINTEGRATE: u32 = 77;
pub const GREATER_HASTE: u32 = 54;
pub const HASTE: u32 = 43;
pub const SLOW: u32 = 29;
pub const MP_COST: i64 = 10;

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
pub enum Support {
    Teleport,
    Unimplemented,
    UnsupportedTemplate,
    LightArrow,
    BasicHeal,
    FireStorm,
    Disintegrate,
    GreaterHaste,
    Haste,
    Slow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    pub skill_id: u32,
    pub support: Support,
    pub mp_cost: i64,
    pub reuse_millis: i64,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillsReply {
    pub context: SelectionContext,
    pub status: Status,
    pub skills: Vec<Skill>,
}
impl SkillsReply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.skills.len() > MAX_SKILLS
            || (self.status != Status::Ok && !self.skills.is_empty())
        {
            return Err(invalid());
        }
        let mut previous = 0;
        for skill in &self.skills {
            if !(1..=220).contains(&skill.skill_id)
                || skill.skill_id <= previous
                || skill.mp_cost < 0
                || skill.reuse_millis < 0
                || skill.name.is_empty()
                || skill.name.len() > MAX_NAME_BYTES
                || skill.name.trim().is_empty()
                || skill.name.chars().any(char::is_control)
                || skill.support == Support::Teleport
                    && (skill.skill_id != TELEPORT
                        || skill.mp_cost != MP_COST
                        || skill.reuse_millis != 10)
                || skill.support == Support::UnsupportedTemplate
                    && !matches!(
                        skill.skill_id,
                        TELEPORT
                            | LIGHT_ARROW
                            | BASIC_HEAL
                            | FIRE_STORM
                            | DISINTEGRATE
                            | GREATER_HASTE
                            | HASTE
                            | SLOW
                    )
                || skill.support == Support::Unimplemented && skill.skill_id == TELEPORT
                || skill.support == Support::LightArrow
                    && (skill.skill_id != LIGHT_ARROW
                        || skill.mp_cost != 3
                        || skill.reuse_millis != 10)
                || skill.support == Support::FireStorm
                    && (skill.skill_id != FIRE_STORM
                        || skill.mp_cost != 48
                        || skill.reuse_millis != 2000)
                || skill.support == Support::BasicHeal
                    && (skill.skill_id != BASIC_HEAL
                        || skill.mp_cost != 4
                        || skill.reuse_millis != 10)
                || skill.support == Support::Disintegrate
                    && (skill.skill_id != DISINTEGRATE
                        || skill.mp_cost != 70
                        || skill.reuse_millis != 5000)
                || skill.support == Support::Slow
                    && (skill.skill_id != SLOW || skill.mp_cost != 20 || skill.reuse_millis != 500)
                || skill.support == Support::Haste
                    && (skill.skill_id != HASTE || skill.mp_cost != 40 || skill.reuse_millis != 10)
                || skill.support == Support::GreaterHaste
                    && (skill.skill_id != GREATER_HASTE
                        || skill.mp_cost != 60
                        || skill.reuse_millis != 10)
            {
                return Err(invalid());
            }
            previous = skill.skill_id;
        }
        Ok(())
    }
    pub fn teleport(&self) -> Option<&Skill> {
        (self.status == Status::Ok)
            .then(|| {
                self.skills
                    .iter()
                    .find(|s| s.skill_id == TELEPORT && s.support == Support::Teleport)
            })
            .flatten()
    }
    pub fn light_arrow(&self) -> Option<&Skill> {
        (self.status == Status::Ok)
            .then(|| {
                self.skills
                    .iter()
                    .find(|s| s.skill_id == LIGHT_ARROW && s.support == Support::LightArrow)
            })
            .flatten()
    }
    pub fn fire_storm(&self) -> Option<&Skill> {
        self.skills.iter().find(|s| {
            self.status == Status::Ok && s.skill_id == FIRE_STORM && s.support == Support::FireStorm
        })
    }
    pub fn disintegrate(&self) -> Option<&Skill> {
        self.skills.iter().find(|s| {
            self.status == Status::Ok
                && s.skill_id == DISINTEGRATE
                && s.support == Support::Disintegrate
        })
    }
    pub fn greater_haste(&self) -> Option<&Skill> {
        self.skills.iter().find(|s| {
            self.status == Status::Ok
                && s.skill_id == GREATER_HASTE
                && s.support == Support::GreaterHaste
        })
    }
    pub fn slow(&self) -> Option<&Skill> {
        self.skills
            .iter()
            .find(|s| self.status == Status::Ok && s.skill_id == SLOW && s.support == Support::Slow)
    }
    pub fn haste(&self) -> Option<&Skill> {
        self.skills.iter().find(|s| {
            self.status == Status::Ok && s.skill_id == HASTE && s.support == Support::Haste
        })
    }
    pub fn basic_heal(&self) -> Option<&Skill> {
        (self.status == Status::Ok)
            .then(|| {
                self.skills
                    .iter()
                    .find(|s| s.skill_id == BASIC_HEAL && s.support == Support::BasicHeal)
            })
            .flatten()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum SkillsState {
    #[default]
    NotRequested,
    Unsupported,
    Ready(Arc<SkillsReply>),
}
impl SkillsState {
    pub fn disintegrate(&self) -> Option<&Skill> {
        match self {
            Self::Ready(reply) => reply.disintegrate(),
            _ => None,
        }
    }
    pub fn fire_storm(&self) -> Option<&Skill> {
        match self {
            Self::Ready(reply) => reply.fire_storm(),
            _ => None,
        }
    }
    pub fn basic_heal(&self) -> Option<&Skill> {
        match self {
            Self::Ready(reply) => reply.basic_heal(),
            _ => None,
        }
    }
    pub fn teleport(&self) -> Option<&Skill> {
        match self {
            Self::Ready(reply) => reply.teleport(),
            _ => None,
        }
    }
    pub fn light_arrow(&self) -> Option<&Skill> {
        match self {
            Self::Ready(reply) => reply.light_arrow(),
            _ => None,
        }
    }
}

/// v6 atomic combat appendix. Teleport state and its wire remain unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageCastState {
    pub entity_id: u64,
    pub cast_count: u64,
    pub cast_revision: u64,
    pub mp_spent: u64,
    pub skill_id: u32,
    pub target_entity_id: u64,
    pub from: WorldCell,
    pub to: WorldCell,
    pub mp_cost: u32,
}
impl DamageCastState {
    pub fn initial(entity_id: u64, mp_cost: u32) -> Self {
        Self {
            entity_id,
            cast_count: 0,
            cast_revision: 0,
            mp_spent: 0,
            skill_id: 0,
            target_entity_id: 0,
            from: WorldCell { x: 0, y: 0 },
            to: WorldCell { x: 0, y: 0 },
            mp_cost,
        }
    }
    pub fn validate(&self, frame: &world::Frame) -> Result<(), ApplicationError> {
        self.validate_target(frame, false)
    }
    pub fn validate_combat(&self, frame: &combat::Frame) -> Result<(), ApplicationError> {
        self.validate_target(
            &frame.world,
            frame
                .honor
                .as_ref()
                .is_some_and(|h| h.objective(self.target_entity_id).is_some()),
        )
    }
    fn validate_target(
        &self,
        frame: &world::Frame,
        objective: bool,
    ) -> Result<(), ApplicationError> {
        let caster = frame.cast(self.entity_id).ok_or_else(invalid)?;
        if !positive(self.entity_id)
            || !(1..=3).contains(&self.mp_cost)
            || self.cast_revision > frame.revision as u64
            || self.mp_spent > i64::MAX as u64
            || self.cast_count.checked_mul(u64::from(self.mp_cost)) != Some(self.mp_spent)
            || self
                .cast_count
                .checked_add(caster.cast_count)
                .is_none_or(|n| n > caster.request_sequence)
        {
            return Err(invalid());
        }
        if self.cast_count == 0 {
            if *self != Self::initial(self.entity_id, self.mp_cost) {
                return Err(invalid());
            }
        } else if self.cast_revision == 0
            || self.skill_id != LIGHT_ARROW
            || self.target_entity_id == self.entity_id
            || !objective
                && !frame
                    .actors
                    .iter()
                    .any(|a| a.entity_id == self.target_entity_id)
            || self.from == self.to
            || (i64::from(self.from.x) - i64::from(self.to.x))
                .abs()
                .max((i64::from(self.from.y) - i64::from(self.to.y)).abs())
                > 10
            || !frame.manifest.contains(self.from)
            || !frame.manifest.contains(self.to)
        {
            return Err(invalid());
        }
        if frame
            .completion
            .as_ref()
            .is_some_and(|c| self.cast_revision > c.event.engine_revision as u64 + 1)
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn follows(&self, next: &Self) -> bool {
        self.entity_id == next.entity_id
            && self.mp_cost == next.mp_cost
            && next.cast_count >= self.cast_count
            && next.cast_revision >= self.cast_revision
            && next.mp_spent >= self.mp_spent
            && if next.cast_count == self.cast_count {
                self == next
            } else {
                next.cast_revision > self.cast_revision
                    && (next.cast_count - self.cast_count).checked_mul(u64::from(self.mp_cost))
                        == next.mp_spent.checked_sub(self.mp_spent)
            }
    }
}
pub fn damages_follow(old: &combat::Frame, new: &combat::Frame) -> bool {
    old.damages.len() == new.damages.len()
        && old.damages.iter().zip(&new.damages).all(|(a, b)| {
            let Some((before, after)) =
                old.world.cast(a.entity_id).zip(new.world.cast(b.entity_id))
            else {
                return false;
            };
            a.follows(b)
                && (b.cast_count == a.cast_count || b.cast_revision > old.world.revision as u64)
                && b.cast_count
                    .checked_sub(a.cast_count)
                    .and_then(|damage| {
                        after
                            .cast_count
                            .checked_sub(before.cast_count)
                            .and_then(|tp| damage.checked_add(tp))
                    })
                    .zip(after.request_sequence.checked_sub(before.request_sequence))
                    .is_some_and(|(count, seq)| count <= seq)
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastState {
    pub entity_id: u64,
    pub request_sequence: u64,
    pub cast_count: u64,
    pub cast_revision: i64,
    pub mp_spent: i64,
    pub skill_id: u32,
    pub from: WorldCell,
    pub to: WorldCell,
    pub cast_interval_millis: u32,
}
impl CastState {
    pub fn initial(entity_id: u64, cast_interval_millis: u32) -> Self {
        Self {
            entity_id,
            request_sequence: 0,
            cast_count: 0,
            cast_revision: 0,
            mp_spent: 0,
            skill_id: 0,
            from: WorldCell { x: 0, y: 0 },
            to: WorldCell { x: 0, y: 0 },
            cast_interval_millis,
        }
    }
    pub fn validate(&self, frame: &world::Frame) -> Result<(), ApplicationError> {
        if !positive(self.entity_id)
            || !(10..=10000).contains(&self.cast_interval_millis)
            || self.request_sequence > i64::MAX as u64
            || self.cast_count > self.request_sequence
            || self.cast_revision < 0
            || self.cast_revision > frame.revision
            || self.mp_spent < 0
        {
            return Err(invalid());
        }
        if self.cast_count == 0 {
            let mut initial = Self::initial(self.entity_id, self.cast_interval_millis);
            initial.request_sequence = self.request_sequence;
            if *self != initial {
                return Err(invalid());
            }
        } else if self.skill_id != TELEPORT
            || self.cast_revision == 0
            || self.from == self.to
            || !frame.manifest.contains(self.from)
            || !frame.manifest.contains(self.to)
            || i64::try_from(self.cast_count)
                .ok()
                .and_then(|c| c.checked_mul(MP_COST))
                != Some(self.mp_spent)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn follows(&self, next: &Self) -> bool {
        if self.entity_id != next.entity_id
            || self.cast_interval_millis != next.cast_interval_millis
            || next.request_sequence < self.request_sequence
            || next.cast_count < self.cast_count
            || next.cast_revision < self.cast_revision
            || next.mp_spent < self.mp_spent
        {
            return false;
        }
        if self.cast_count == next.cast_count {
            let mut same = *self;
            same.request_sequence = next.request_sequence;
            same == *next
        } else {
            next.cast_revision > self.cast_revision
                && next.cast_count - self.cast_count
                    <= next.request_sequence - self.request_sequence
                && i64::try_from(next.cast_count - self.cast_count)
                    .ok()
                    .and_then(|n| n.checked_mul(MP_COST))
                    == next.mp_spent.checked_sub(self.mp_spent)
        }
    }
}
pub fn casts_follow(old: &world::Frame, new: &world::Frame) -> bool {
    old.casts.len() == new.casts.len()
        && old.casts.iter().zip(&new.casts).all(|(a, b)| a.follows(b))
}
pub fn teleported(old: &world::Frame, new: &world::Frame, entity: u64) -> bool {
    old.cast(entity)
        .zip(new.cast(entity))
        .is_some_and(|(a, b)| b.cast_revision > a.cast_revision)
}
pub fn mp_follows(old: &combat::Frame, new: &combat::Frame, index: usize) -> bool {
    if !old.regen.is_empty() || !new.regen.is_empty() {
        return super::regen_protocol::mp_follows(old, new, index);
    }
    let a = &old.vitals[index];
    let b = &new.vitals[index];
    let spent = old
        .world
        .cast(a.entity_id)
        .zip(new.world.cast(b.entity_id))
        .map_or(Some(0), |(a, b)| b.mp_spent.checked_sub(a.mp_spent));
    let damage_spent =
        old.damage(a.entity_id)
            .zip(new.damage(b.entity_id))
            .map_or(Some(0), |(a, b)| {
                b.mp_spent
                    .checked_sub(a.mp_spent)
                    .and_then(|n| i64::try_from(n).ok())
            });
    spent
        .zip(damage_spent)
        .and_then(|(tp, damage)| tp.checked_add(damage))
        .and_then(|spent| {
            new.npc_spell(b.entity_id)
                .map_or(0, |s| s.mp_spent)
                .checked_sub(old.npc_spell(a.entity_id).map_or(0, |s| s.mp_spent))
                .and_then(|npc| spent.checked_add(npc))
        })
        .is_some_and(|spent| a.mp.checked_sub(spent) == Some(b.mp))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastRequest {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub match_id: u64,
    pub generation: u64,
    pub sequence: u64,
    pub skill_id: u32,
    pub target_entity_id: u64,
}
impl CastRequest {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || !positive(self.match_id)
            || !positive(self.generation)
            || !positive(self.sequence)
            || !(1..=220).contains(&self.skill_id)
            || self.target_entity_id > i64::MAX as u64
            || !matches!(self.skill_id, LIGHT_ARROW | BASIC_HEAL) && self.target_entity_id != 0
        {
            Err(invalid())
        } else {
            Ok(())
        }
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
    NoDestination,
    Numeric,
    TargetDead,
    SameTeam,
    OutOfRange,
    Blocked,
    TooHeavy,
    StaleEffect,
    StaleRegen,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CastReply {
    pub echo: CastRequest,
    pub status: Status,
    pub reason: Reason,
    pub frame: Option<combat::Frame>,
}
impl CastReply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == Status::Ok) != (self.reason == Reason::None) {
            return Err(invalid());
        }
        let Some(frame) = &self.frame else {
            return if self.status == Status::Unavailable && self.reason == Reason::NotActive {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        frame.validate()?;
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        if self.reason == Reason::TooHeavy
            && (self.status != Status::Denied
                || frame.world.phase != world::Phase::Active
                || frame.vital(actor.entity_id).is_none_or(|v| v.hp <= 0)
                || frame
                    .weight(actor.entity_id)
                    .is_none_or(|w| !w.applicable() || w.can_attack()))
        {
            return Err(invalid());
        }
        if frame.world.match_id != self.echo.match_id
            || frame.world.generation != self.echo.generation
        {
            return Err(invalid());
        }
        if self.reason == Reason::StaleEffect
            && (self.status != Status::Denied
                || frame.world.phase != world::Phase::Active
                || frame.timed(actor.entity_id).is_none_or(|s| s.revision <= 0)
                || self.echo.skill_id == FIRE_STORM && frame.fire(actor.entity_id).is_none()
                || frame.vital(actor.entity_id).is_none_or(|v| v.hp <= 0)
                || !matches!(
                    (self.echo.skill_id, self.echo.target_entity_id),
                    (TELEPORT, 0) | (LIGHT_ARROW, 1..) | (BASIC_HEAL, 1..) | (FIRE_STORM, 0)
                ))
        {
            return Err(invalid());
        }
        let state = frame.world.cast(actor.entity_id).ok_or_else(invalid)?;
        if self.reason == Reason::StaleRegen {
            let own_mp = frame
                .regen(actor.entity_id)
                .is_some_and(|r| r.mp_revision > 0);
            let proof = match (self.echo.skill_id, self.echo.target_entity_id) {
                (TELEPORT, 0) => own_mp,
                (FIRE_STORM, 0) => {
                    frame.fire(actor.entity_id).is_some()
                        && (own_mp
                            || frame.world.actors.iter().any(|a| {
                                a.entity_id != actor.entity_id
                                    && frame.vital(a.entity_id).is_some_and(|v| v.hp > 0)
                                    && super::honor_protocol::target_team(frame, a.entity_id)
                                        != super::honor_protocol::target_team(
                                            frame,
                                            actor.entity_id,
                                        )
                                    && super::fire_storm_protocol::in_radius(
                                        actor.position,
                                        a.position,
                                    )
                                    && frame.regen(a.entity_id).is_some_and(|r| r.hp_revision > 0)
                            }))
                }
                (LIGHT_ARROW, 1..) => {
                    self.echo.target_entity_id != actor.entity_id
                        && super::honor_protocol::target_hp(frame, self.echo.target_entity_id)
                            .is_some_and(|hp| hp > 0)
                        && (own_mp
                            || frame
                                .regen(self.echo.target_entity_id)
                                .is_some_and(|r| r.hp_revision > 0))
                }
                (BASIC_HEAL, 1..) => {
                    own_mp
                        && frame
                            .honor
                            .as_ref()
                            .and_then(|h| h.objective(self.echo.target_entity_id))
                            .is_some_and(|o| o.hp > 0)
                        && super::honor_protocol::target_team(frame, self.echo.target_entity_id)
                            == super::honor_protocol::target_team(frame, actor.entity_id)
                }
                _ => false,
            };
            if self.status != Status::Denied
                || frame.world.phase != world::Phase::Active
                || frame.vital(actor.entity_id).is_none_or(|v| v.hp <= 0)
                || state.request_sequence != self.echo.sequence
                || !proof
            {
                return Err(invalid());
            }
        }
        if state.request_sequence > self.echo.sequence
            || !matches!(
                frame.world.phase,
                world::Phase::Finished | world::Phase::Aborted
            ) && state.request_sequence != self.echo.sequence
        {
            return Err(invalid());
        }
        if self.status == Status::Ok {
            if state.request_sequence != self.echo.sequence {
                return Err(invalid());
            }
            match self.echo.skill_id {
                FIRE_STORM
                    if self.echo.target_entity_id == 0
                        && matches!(
                            frame.world.phase,
                            world::Phase::Active | world::Phase::Finished
                        ) =>
                {
                    let fire = frame.fire(actor.entity_id).ok_or_else(invalid)?;
                    if fire.cast_revision != frame.world.revision
                        || fire.last_cast_sequence != self.echo.sequence
                    {
                        return Err(invalid());
                    }
                }
                BASIC_HEAL if frame.world.phase == world::Phase::Active => {
                    let heal = frame
                        .honor
                        .as_ref()
                        .and_then(|h| h.heal(actor.entity_id))
                        .ok_or_else(invalid)?;
                    if heal.revision != frame.world.revision
                        || heal.target_objective_id != self.echo.target_entity_id
                    {
                        return Err(invalid());
                    }
                }
                TELEPORT
                    if self.echo.target_entity_id == 0
                        && frame.world.phase == world::Phase::Active
                        && state.skill_id == TELEPORT
                        && state.cast_revision == frame.world.revision
                        && actor.position == state.to => {}
                LIGHT_ARROW
                    if matches!(
                        frame.world.phase,
                        world::Phase::Active | world::Phase::Finished
                    ) =>
                {
                    let damage = frame.damage(actor.entity_id).ok_or_else(invalid)?;
                    if damage.cast_revision != frame.world.revision as u64
                        || damage.skill_id != LIGHT_ARROW
                        || damage.target_entity_id != self.echo.target_entity_id
                    {
                        return Err(invalid());
                    }
                }
                _ => return Err(invalid()),
            }
        }
        Ok(())
    }
    pub fn confirms(&self, request: &CastRequest) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    pub fn combat_reply(&self) -> Option<combat::Reply> {
        self.frame.as_ref().map(|frame| combat::Reply {
            echo: combat::Request::read(self.echo.context, self.echo.channel_id, &frame.world),
            status: combat::Status::Ok,
            reason: combat::Reason::None,
            frame: Some(frame.clone()),
        })
    }
}
/// A denial may consume a request sequence but cannot fabricate a cast event.
pub fn confirms_transition(old: &combat::Frame, reply: &CastReply) -> bool {
    let Some(new) = &reply.frame else {
        return false;
    };
    let Some(actor) = old
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(reply.echo.context.key))
    else {
        return false;
    };
    let Some(before) = old.world.cast(actor.entity_id) else {
        return false;
    };
    let Some(after) = new.world.cast(actor.entity_id) else {
        return false;
    };
    if before.request_sequence.checked_add(1) != Some(reply.echo.sequence) {
        return false;
    }
    if old.npc_spell(actor.entity_id) != new.npc_spell(actor.entity_id)
        || old.disintegrate(actor.entity_id) != new.disintegrate(actor.entity_id)
        || old.npc_disintegrate(actor.entity_id) != new.npc_disintegrate(actor.entity_id)
        || old.npc_fire(actor.entity_id) != new.npc_fire(actor.entity_id)
    {
        return false;
    }
    if reply.echo.skill_id != FIRE_STORM && old.fire(actor.entity_id) != new.fire(actor.entity_id) {
        return false;
    }
    if reply.status == Status::Ok {
        let unchanged_motion = new
            .world
            .actors
            .iter()
            .find(|a| a.entity_id == actor.entity_id)
            .is_some_and(|a| {
                a.sequence == actor.sequence
                    && a.heading == actor.heading
                    && (reply.echo.skill_id == TELEPORT || a.position == actor.position)
            });
        if reply.echo.skill_id == TELEPORT {
            unchanged_motion
                && before.cast_count.checked_add(1) == Some(after.cast_count)
                && before.mp_spent.checked_add(MP_COST) == Some(after.mp_spent)
                && after.from == actor.position
                && old.damage(actor.entity_id) == new.damage(actor.entity_id)
                && super::honor_protocol::heal_revision(old, actor.entity_id)
                    == super::honor_protocol::heal_revision(new, actor.entity_id)
        } else if reply.echo.skill_id == LIGHT_ARROW {
            let Some((a, b)) = old.damage(actor.entity_id).zip(new.damage(actor.entity_id)) else {
                return false;
            };
            let mut expected = *before;
            expected.request_sequence = after.request_sequence;
            unchanged_motion
                && expected == *after
                && a.cast_count.checked_add(1) == Some(b.cast_count)
                && a.mp_spent.checked_add(u64::from(a.mp_cost)) == Some(b.mp_spent)
                && a.mp_cost == b.mp_cost
                && b.from == actor.position
                && b.target_entity_id == reply.echo.target_entity_id
                && super::honor_protocol::target_position(new, b.target_entity_id) == Some(b.to)
                && super::honor_protocol::heal_revision(old, actor.entity_id)
                    == super::honor_protocol::heal_revision(new, actor.entity_id)
        } else if reply.echo.skill_id == FIRE_STORM {
            let Some((a, b)) = old.fire(actor.entity_id).zip(new.fire(actor.entity_id)) else {
                return false;
            };
            let mut expected = *before;
            expected.request_sequence = after.request_sequence;
            unchanged_motion
                && expected == *after
                && a.count.checked_add(1) == Some(b.count)
                && a.mp_spent.checked_add(i64::from(a.mp_cost)) == Some(b.mp_spent)
                && a.mp_cost == b.mp_cost
                && b.from == actor.position
                && b.last_cast_sequence == reply.echo.sequence
                && old.damage(actor.entity_id) == new.damage(actor.entity_id)
                && super::honor_protocol::heal_revision(old, actor.entity_id)
                    == super::honor_protocol::heal_revision(new, actor.entity_id)
        } else if reply.echo.skill_id == BASIC_HEAL {
            let Some((a, b)) = old
                .honor
                .as_ref()
                .and_then(|h| h.heal(actor.entity_id))
                .zip(new.honor.as_ref().and_then(|h| h.heal(actor.entity_id)))
            else {
                return false;
            };
            let mut expected = *before;
            expected.request_sequence = after.request_sequence;
            unchanged_motion
                && expected == *after
                && a.count.checked_add(1) == Some(b.count)
                && a.mp_spent.checked_add(i64::from(a.mp_cost)) == Some(b.mp_spent)
                && a.mp_cost == b.mp_cost
                && b.from == actor.position
                && b.target_objective_id == reply.echo.target_entity_id
                && old.damage(actor.entity_id) == new.damage(actor.entity_id)
        } else {
            false
        }
    } else {
        let mut expected = *before;
        expected.request_sequence = after.request_sequence;
        expected == *after
            && old.fire(actor.entity_id) == new.fire(actor.entity_id)
            && old.damage(actor.entity_id) == new.damage(actor.entity_id)
            && super::honor_protocol::heal_revision(old, actor.entity_id)
                == super::honor_protocol::heal_revision(new, actor.entity_id)
            && (after.request_sequence == reply.echo.sequence
                || matches!(
                    new.world.phase,
                    world::Phase::Finished | world::Phase::Aborted
                ) && after.request_sequence == before.request_sequence)
    }
}
fn positive(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid spell frame")
}
pub fn encode_skills(id: u32, context: SelectionContext) -> Result<Vec<u8>, ApplicationError> {
    if !app::valid_context(context) {
        return Err(invalid());
    }
    let mut out = app::header(id, SKILLS_REQUEST_KIND, 36)?;
    put_context(&mut out, context);
    Ok(out)
}
pub fn encode_cast(id: u32, request: &CastRequest) -> Result<Vec<u8>, ApplicationError> {
    encode_cast_version(id, request, 4)
}
pub fn encode_cast_version(
    id: u32,
    request: &CastRequest,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    request.validate()?;
    if !(4..=super::protocol::MAX_VERSION).contains(&version)
        || version <= 5 && request.target_entity_id != 0
        || version >= 6 && request.skill_id == LIGHT_ARROW && request.target_entity_id == 0
        || request.skill_id == BASIC_HEAL
            && (if version >= 13 {
                request.target_entity_id == 0
            } else {
                request.target_entity_id != 0
            })
    {
        return Err(invalid());
    }
    let mut out = app::header(id, CAST_REQUEST_KIND, if version >= 6 { 80 } else { 72 })?;
    put_context(&mut out, request.context);
    out.extend([request.channel_id, 0, 0, 0]);
    out.extend(request.match_id.to_be_bytes());
    out.extend(request.generation.to_be_bytes());
    out.extend(request.sequence.to_be_bytes());
    out.extend(request.skill_id.to_be_bytes());
    out.extend([0; 4]);
    if version >= 6 {
        out.extend(request.target_entity_id.to_be_bytes());
    }
    Ok(out)
}
fn put_context(out: &mut Vec<u8>, c: SelectionContext) {
    out.extend(c.key.source_id.to_be_bytes());
    out.extend(c.key.object_id.to_be_bytes());
    out.extend(c.epoch.to_be_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.pos.checked_add(n).ok_or_else(invalid)?;
        let out = self.bytes.get(self.pos..end).ok_or_else(invalid)?;
        self.pos = end;
        Ok(out)
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
    fn context(&mut self) -> Result<SelectionContext, ApplicationError> {
        Ok(SelectionContext {
            key: app::CharacterKey {
                source_id: self.u32()?,
                object_id: self.u64()?,
            },
            epoch: self.u64()?,
        })
    }
    fn status(&mut self) -> Result<Status, ApplicationError> {
        Ok(match self.u8()? {
            0 => Status::Ok,
            1 => Status::Denied,
            2 => Status::Busy,
            3 => Status::Unavailable,
            _ => return Err(invalid()),
        })
    }
}
pub(crate) fn decode_skills(bytes: &[u8], version: u16) -> Result<SkillsReply, ApplicationError> {
    if !(4..=super::protocol::MAX_VERSION).contains(&version) || bytes.len() > 64 * 1024 - 16 {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let context = r.context()?;
    let status = r.status()?;
    r.zero(3)?;
    let mut skills = Vec::new();
    if status == Status::Ok {
        let count = usize::from(r.u16()?);
        r.zero(2)?;
        if count > MAX_SKILLS || bytes.len() - r.pos < count * 27 {
            return Err(invalid());
        }
        for _ in 0..count {
            let skill_id = r.u32()?;
            let support = match r.u8()? {
                0 => Support::Teleport,
                1 => Support::Unimplemented,
                2 => Support::UnsupportedTemplate,
                3 if version >= 6 => Support::LightArrow,
                4 if version >= 13 => Support::BasicHeal,
                5 if version >= 14 => Support::FireStorm,
                6 if version >= 27 => Support::Disintegrate,
                7 if version >= 42 => Support::GreaterHaste,
                8 if version >= 44 => Support::Haste,
                9 if version >= 47 => Support::Slow,
                _ => return Err(invalid()),
            };
            if version <= 5 && skill_id == LIGHT_ARROW && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version <= 13 && skill_id == FIRE_STORM && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version < 27 && skill_id == DISINTEGRATE && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version < 47 && skill_id == SLOW && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version < 44 && skill_id == HASTE && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version < 42 && skill_id == GREATER_HASTE && support != Support::Unimplemented {
                return Err(invalid());
            }
            if version <= 12 && skill_id == BASIC_HEAL && support != Support::Unimplemented {
                return Err(invalid());
            }
            r.zero(3)?;
            let mp_cost = r.i64()?;
            let reuse_millis = r.i64()?;
            let len = usize::from(r.u16()?);
            if len > MAX_NAME_BYTES {
                return Err(invalid());
            }
            let name = std::str::from_utf8(r.take(len)?)
                .map_err(|_| invalid())?
                .to_owned();
            skills.push(Skill {
                skill_id,
                support,
                mp_cost,
                reuse_millis,
                name,
            });
        }
    }
    if r.pos != bytes.len() {
        return Err(invalid());
    }
    let reply = SkillsReply {
        context,
        status,
        skills,
    };
    reply.validate()?;
    Ok(reply)
}
pub(crate) fn decode_cast(bytes: &[u8], version: u16) -> Result<CastReply, ApplicationError> {
    if !(4..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() > combat::frame_limit(version) - 16
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, pos: 0 };
    let context = r.context()?;
    let channel_id = r.u8()?;
    r.zero(3)?;
    let mut echo = CastRequest {
        context,
        channel_id,
        match_id: r.u64()?,
        generation: r.u64()?,
        sequence: r.u64()?,
        skill_id: r.u32()?,
        target_entity_id: 0,
    };
    r.zero(4)?;
    if version >= 6 {
        echo.target_entity_id = r.u64()?;
        if echo.skill_id == LIGHT_ARROW && echo.target_entity_id == 0 {
            return Err(invalid());
        }
    }
    if echo.skill_id == BASIC_HEAL
        && (if version >= 13 {
            echo.target_entity_id == 0
        } else {
            echo.target_entity_id != 0
        })
    {
        return Err(invalid());
    }
    let status = r.status()?;
    let reason = match r.u8()? {
        0 => Reason::None,
        1 => Reason::NotActive,
        2 => Reason::Dead,
        3 => Reason::Unlearned,
        4 => Reason::Unsupported,
        5 => Reason::TooFast,
        6 => Reason::NoMp,
        7 => Reason::NoDestination,
        8 => Reason::Numeric,
        9 if version >= 6 => Reason::TargetDead,
        10 if version >= 6 => Reason::SameTeam,
        11 if version >= 6 => Reason::OutOfRange,
        12 if version >= 6 => Reason::Blocked,
        13 if version >= 9 => Reason::TooHeavy,
        14 if version >= 10 => Reason::StaleEffect,
        15 if version >= 12 => Reason::StaleRegen,
        _ => return Err(invalid()),
    };
    let present = r.u8()?;
    r.zero(1)?;
    let frame = match present {
        0 => None,
        1 => {
            let (frame, used) =
                combat::decode_frame_version(&bytes[r.pos..], context.key, version)?;
            r.take(used)?;
            Some(frame)
        }
        _ => return Err(invalid()),
    };
    if r.pos != bytes.len() {
        return Err(invalid());
    }
    let reply = CastReply {
        echo,
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    const SKILLS_REQUEST: &str =
        "4c544d3100000024fedcba9800180000000000070000000000000d05ffffffffffffffff";
    const SKILLS_RESULT: &str = "4c544d310000004bfedcba9800190000000000070000000000000d05ffffffffffffffff00000000000100000000000500000000000000000000000a000000000000000a00052431343430";
    const CAST_REQUEST: &str = "4c544d3100000048fedcba98001a0000000000070000000000000d05ffffffffffffffff030000000000000000000009000000000000000600000000000000010000000500000000";
    const CAST_NONE: &str = "4c544d310000004cfedcba98001b0000000000070000000000000d05ffffffffffffffff03000000000000000000000900000000000000060000000000000001000000050000000003010000";
    // Independent previous Java/Rust WORLD golden body (versions 1 and 2).
    const WORLD: &str = "0000000000000009000000000000000600000000000000010100000000000000000000050000000000000006000204100000021c0001520001000101000000000000070000000000000d0500010000000000070000000000000d0500000000ee6b2800000301000100000000014e000153000100010000020222222222222222222222222222222222222222222222222222222222222222227fff80000000c012111111111111111111111111111111111111111111111111111111111111111100010000000000000000000100007fd4000080100200000000000000000000000000000002faf080";
    const CAST_STATE: &str = "0000000000000001000000000000000100000000000000010000000000000002000000000000000a0000000500007fd40000801000007fd500008010000004b0";
    fn hex(value: &str) -> Vec<u8> {
        value
            .as_bytes()
            .chunks_exact(2)
            .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
            .collect()
    }
    fn context() -> SelectionContext {
        SelectionContext {
            key: app::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    fn request() -> CastRequest {
        CastRequest {
            context: context(),
            channel_id: 3,
            match_id: 9,
            generation: 6,
            sequence: 1,
            skill_id: 5,
            target_entity_id: 0,
        }
    }
    pub(crate) fn initial() -> combat::Reply {
        let mut reply = combat::tests::active_reply();
        let frame = reply.frame.as_mut().unwrap();
        frame.world.casts = frame
            .world
            .actors
            .iter()
            .map(|a| CastState::initial(a.entity_id, 1000))
            .collect();
        for vital in &mut frame.vitals {
            vital.mp = 100;
            vital.max_mp = 100;
        }
        reply.validate().unwrap();
        reply
    }
    pub(crate) fn success(old: &combat::Reply) -> CastReply {
        let before = old.frame.as_ref().unwrap();
        let mut frame = before.clone();
        frame.world.revision += 1;
        let actor = &mut frame.world.actors[0];
        let from = actor.position;
        actor.position.y += 2;
        let event = &mut frame.world.casts[0];
        event.request_sequence += 1;
        event.cast_count += 1;
        event.cast_revision = frame.world.revision;
        event.mp_spent += 10;
        event.skill_id = 5;
        event.from = from;
        event.to = actor.position;
        frame.vitals[0].mp -= 10;
        CastReply {
            echo: CastRequest {
                context: old.echo.context,
                channel_id: old.echo.channel_id,
                match_id: frame.world.match_id,
                generation: frame.world.generation,
                sequence: event.request_sequence,
                skill_id: 5,
                target_entity_id: 0,
            },
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(frame),
        }
    }
    pub(crate) fn initial_damage() -> combat::Reply {
        let mut reply = initial();
        let frame = reply.frame.as_mut().unwrap();
        frame.damages = frame
            .world
            .actors
            .iter()
            .map(|a| DamageCastState::initial(a.entity_id, 3))
            .collect();
        reply.validate().unwrap();
        reply
    }
    pub(crate) fn damage_success(old: &combat::Reply) -> CastReply {
        let mut frame = old.frame.as_ref().unwrap().clone();
        frame.world.revision += 1;
        frame.world.casts[0].request_sequence += 1;
        let event = &mut frame.damages[0];
        event.cast_count += 1;
        event.cast_revision = frame.world.revision as u64;
        event.mp_spent += u64::from(event.mp_cost);
        event.skill_id = LIGHT_ARROW;
        event.target_entity_id = frame.world.actors[1].entity_id;
        event.from = frame.world.actors[0].position;
        event.to = frame.world.actors[1].position;
        frame.vitals[0].mp -= i64::from(event.mp_cost);
        frame.vitals[1].hp = (frame.vitals[1].hp - 10).max(0);
        frame.vitals[1].hurt_revision = frame.world.revision;
        if frame.vitals[1].hp == 0 {
            frame.vitals[1].death_revision = frame.world.revision;
        }
        CastReply {
            echo: CastRequest {
                context: old.echo.context,
                channel_id: old.echo.channel_id,
                match_id: frame.world.match_id,
                generation: frame.world.generation,
                sequence: frame.world.casts[0].request_sequence,
                skill_id: LIGHT_ARROW,
                target_entity_id: event.target_entity_id,
            },
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(frame),
        }
    }
    #[test]
    fn v6_target_request_literal_and_old_skill4_unsupported_request_are_not_conflated() {
        const TARGET: &str = "4c544d3100000050fedcba98001a0000000000070000000000000d05ffffffffffffffff0300000000000000000000090000000000000006000000000000000100000004000000000000000000000002";
        let mut req = request();
        req.skill_id = 4;
        req.target_entity_id = 2;
        assert_eq!(
            encode_cast_version(0xfedcba98, &req, 6).unwrap(),
            hex(TARGET)
        );
        for version in [4, 5] {
            assert!(encode_cast_version(1, &req, version).is_err());
        }
        req.target_entity_id = 0;
        for version in [4, 5] {
            assert_eq!(encode_cast_version(1, &req, version).unwrap().len(), 72);
        }
        assert!(encode_cast_version(1, &req, 6).is_err());
        req.skill_id = 5;
        req.target_entity_id = 2;
        assert!(req.validate().is_err());
        let mut none = hex(TARGET);
        none[12..14].copy_from_slice(&CAST_RESULT_KIND.to_be_bytes());
        none.extend([3, 1, 0, 0]);
        none[4..8].copy_from_slice(&84u32.to_be_bytes());
        assert_eq!(
            decode_cast(&none[16..], 6).unwrap().echo.target_entity_id,
            2
        );
        for version in [4, 5] {
            assert!(decode_cast(&none[16..], version).is_err());
        }
        for len in 0..none.len() - 16 {
            assert!(decode_cast(&none[16..16 + len], 6).is_err());
        }
        none.push(0);
        assert!(decode_cast(&none[16..], 6).is_err());
    }
    #[test]
    fn learned_light_arrow_support_is_versioned_and_raw_cost_is_not_frozen_cost() {
        let mut bytes = hex(SKILLS_RESULT)[16..].to_vec();
        bytes[28..32].copy_from_slice(&4u32.to_be_bytes());
        bytes[32] = 3;
        bytes[36..44].copy_from_slice(&3i64.to_be_bytes());
        let reply = decode_skills(&bytes, 6).unwrap();
        assert_eq!(reply.light_arrow().unwrap().mp_cost, 3);
        for version in [4, 5] {
            assert!(decode_skills(&bytes, version).is_err());
        }
        bytes[32] = 1;
        assert!(decode_skills(&bytes, 5).unwrap().light_arrow().is_none());
        bytes[32] = 2;
        assert!(decode_skills(&bytes, 6).is_ok());
        assert!(decode_skills(&bytes, 5).is_err());
        bytes[32] = 3;
        bytes[36..44].copy_from_slice(&1i64.to_be_bytes());
        assert!(decode_skills(&bytes, 6).is_err());
    }
    #[test]
    fn damage_cast_is_atomic_with_mp_hp_and_preserves_teleport_history() {
        let initial = initial_damage();
        let tp = success(&initial).combat_reply().unwrap();
        let hit = damage_success(&tp);
        hit.validate().unwrap();
        assert!(confirms_transition(tp.frame.as_ref().unwrap(), &hit));
        let next = hit.combat_reply().unwrap();
        assert!(combat::follows(Some(&initial), &next));
        let f = next.frame.as_ref().unwrap();
        assert_eq!(f.world.casts[0].cast_count, 1);
        assert_eq!(f.world.casts[0].request_sequence, 2);
        assert_eq!(f.damages[0].cast_count, 1);
        assert_eq!(f.vitals[0].mp, 87);
        assert_eq!(f.vitals[1].hp, 40);
        assert_eq!(f.vitals[0].attack_revision, 0);
        assert_eq!(f.world.actors[0].sequence, 0);
        let after_tp = success(&next).combat_reply().unwrap();
        assert!(combat::follows(Some(&next), &after_tp));
        assert_eq!(after_tp.frame.as_ref().unwrap().damages, f.damages);
        for case in 0..12 {
            let mut bad = next.clone();
            let f = bad.frame.as_mut().unwrap();
            match case {
                0 => f.damages.clear(),
                1 => f.damages[0].mp_cost = 2,
                2 => f.damages[0].mp_spent = 2,
                3 => f.vitals[0].mp += 1,
                4 => f.vitals[1].hurt_revision = 0,
                5 => f.damages[0].target_entity_id = 1,
                6 => f.damages[0].from = f.damages[0].to,
                7 => f.world.casts[0].request_sequence = 1,
                8 => f.damages[0].cast_revision = f.world.revision as u64 + 1,
                9 => f.damages[0].entity_id = 2,
                10 => f.damages[0].mp_spent = u64::MAX,
                _ => f.damages[0].skill_id = 5,
            }
            assert!(!combat::follows(Some(&initial), &bad), "case {case}");
        }
        let mut bad = next.clone();
        bad.frame.as_mut().unwrap().damages[0].to.x += 1;
        bad.frame.as_mut().unwrap().world.revision += 1;
        assert!(
            !combat::follows(Some(&next), &bad),
            "same count cannot rewrite historical target"
        );
        let mut too_far = f.clone();
        too_far.damages[0].to = WorldCell {
            x: too_far.damages[0].from.x + 11,
            y: too_far.damages[0].from.y,
        };
        assert!(
            too_far.validate().is_err(),
            "event admission is10 even when later visual paths may be longer"
        );
        let mut wrong_initial = initial.frame.unwrap();
        wrong_initial.damages.clear();
        wrong_initial.vitals[0].mp -= 1;
        assert!(
            wrong_initial.validate().is_err(),
            "v4/v5 teleport-only full-new MP is also exact"
        );
    }
    #[test]
    fn light_arrow_denial_does_not_animate_or_spend_and_prior_denials_are_not_cast_credit() {
        let original = initial_damage();
        let mut denied = damage_success(&original);
        denied.status = Status::Denied;
        denied.reason = Reason::NoMp;
        denied.frame = original.frame.clone();
        let f = denied.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.world.casts[0].request_sequence = 1;
        denied.validate().unwrap();
        assert!(confirms_transition(
            original.frame.as_ref().unwrap(),
            &denied
        ));
        assert!(combat::follows(
            Some(&original),
            &denied.combat_reply().unwrap()
        ));
        let mut before = original.clone();
        before.frame.as_mut().unwrap().world.casts[0].request_sequence = 100;
        let mut impossible = damage_success(&before).combat_reply().unwrap();
        {
            let f = impossible.frame.as_mut().unwrap();
            f.damages[0].cast_count = 2;
            f.damages[0].mp_spent = 6;
            f.vitals[0].mp = 94;
        }
        assert!(!combat::follows(Some(&before), &impossible));
        impossible.frame.as_mut().unwrap().world.casts[0].request_sequence = 102;
        assert!(combat::follows(Some(&before), &impossible));
    }
    #[test]
    fn lethal_light_arrow_keeps_finished_events_frozen_through_saved_receipt() {
        let reference = combat::tests::finished_reply();
        let mut f = reference.frame.unwrap();
        f.world.casts = f
            .world
            .actors
            .iter()
            .map(|a| CastState::initial(a.entity_id, 1000))
            .collect();
        f.damages = f
            .world
            .actors
            .iter()
            .map(|a| DamageCastState::initial(a.entity_id, 3))
            .collect();
        for v in &mut f.vitals {
            v.mp = 100;
            v.max_mp = 100;
        }
        f.vitals[0].attack_sequence = 0;
        f.vitals[0].attack_revision = 0;
        f.vitals[0].last_target_entity_id = 0;
        f.vitals[0].mp = 97;
        f.world.casts[0].request_sequence = 1;
        f.damages[0] = DamageCastState {
            entity_id: 1,
            cast_count: 1,
            cast_revision: 8,
            mp_spent: 3,
            skill_id: 4,
            target_entity_id: 2,
            from: f.world.actors[0].position,
            to: f.world.actors[1].position,
            mp_cost: 3,
        };
        let reply = CastReply {
            echo: CastRequest {
                context: reference.echo.context,
                channel_id: reference.echo.channel_id,
                match_id: f.world.match_id,
                generation: f.world.generation,
                sequence: 1,
                skill_id: 4,
                target_entity_id: 2,
            },
            status: Status::Ok,
            reason: Reason::None,
            frame: Some(f),
        };
        reply.validate().unwrap();
        let pending = reply.combat_reply().unwrap();
        let mut saved = pending.clone();
        let f = saved.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.world.completion.as_mut().unwrap().state = world::SettlementState::Saved;
        assert!(combat::follows(Some(&pending), &saved));
        saved.frame.as_mut().unwrap().damages[0].cast_count += 1;
        assert!(!combat::follows(Some(&pending), &saved));
    }
    #[test]
    fn independent_java_literal_request_reply_and_unsigned_epoch_contract() {
        assert_eq!(
            encode_skills(0xfedcba98, context()).unwrap(),
            hex(SKILLS_REQUEST)
        );
        assert_eq!(
            encode_cast(0xfedcba98, &request()).unwrap(),
            hex(CAST_REQUEST)
        );
        let skills = decode_skills(&hex(SKILLS_RESULT)[16..], 4).unwrap();
        assert_eq!(skills.context, context());
        assert_eq!(skills.teleport().unwrap().name, "$1440");
        let none = decode_cast(&hex(CAST_NONE)[16..], 4).unwrap();
        assert_eq!(none.echo, request());
        assert_eq!(none.status, Status::Unavailable);
        assert!(none.frame.is_none());
        for version in 1..=3 {
            assert!(decode_skills(&hex(SKILLS_RESULT)[16..], version).is_err());
            assert!(decode_cast(&hex(CAST_NONE)[16..], version).is_err());
        }
        for bytes in [hex(SKILLS_RESULT), hex(CAST_NONE)] {
            for end in 16..bytes.len() {
                let body = &bytes[16..end];
                assert!(if bytes[13] == 25 {
                    decode_skills(body, 4).is_err()
                } else {
                    decode_cast(body, 4).is_err()
                });
            }
        }
        let mut invalid = request();
        invalid.context.epoch = 0;
        assert!(invalid.validate().is_err());
        invalid = request();
        invalid.skill_id = 220;
        assert!(invalid.validate().is_ok());
        invalid.skill_id = 221;
        assert!(invalid.validate().is_err());
        for wire in [hex(SKILLS_RESULT), hex(CAST_NONE)] {
            let decoded = app::decode_response_version(&wire, 4).unwrap();
            assert_eq!(decoded.request_id, 0xfedcba98);
            assert_eq!(app::decode_response_version(&wire, 5).unwrap(), decoded);
            assert!(app::decode_response_version(&wire, 3).is_err());
            if wire == hex(SKILLS_RESULT) {
                assert_eq!(app::decode_response_version(&wire, 14).unwrap(), decoded);
                assert_eq!(app::decode_response_version(&wire, 15).unwrap(), decoded);
                assert_eq!(app::decode_response_version(&wire, 16).unwrap(), decoded);
                assert_eq!(app::decode_response_version(&wire, 17).unwrap(), decoded);
            } else {
                // The literal v4 CAST lacks the target-u64 added in v6.
                assert!(app::decode_response_version(&wire, 14).is_err());
                assert!(app::decode_response_version(&wire, 15).is_err());
                assert!(app::decode_response_version(&wire, 16).is_err());
                assert!(app::decode_response_version(&wire, 17).is_err());
            }
            assert!(
                app::decode_response_version(&wire, super::super::protocol::MAX_VERSION + 1)
                    .is_err()
            );
        }
    }
    #[test]
    fn catalog_support_is_explicit_bounded_sorted_and_never_guessed() {
        let mut catalog = decode_skills(&hex(SKILLS_RESULT)[16..], 4).unwrap();
        for case in 0..8 {
            let mut bad = catalog.clone();
            let row = &mut bad.skills[0];
            match case {
                0 => row.skill_id = 6,
                1 => row.mp_cost = 11,
                2 => row.reuse_millis = 11,
                3 => row.name = "\u{a0}\u{2007}".into(),
                4 => row.name = "x\n".into(),
                5 => row.name = "名".repeat(86),
                6 => row.support = Support::Unimplemented,
                7 => {
                    let duplicate = row.clone();
                    bad.skills.push(duplicate);
                }
                _ => unreachable!(),
            };
            assert!(bad.validate().is_err(), "case {case}");
        }
        catalog.skills = (1..=220)
            .map(|skill_id| Skill {
                skill_id,
                support: if skill_id == 5 {
                    Support::UnsupportedTemplate
                } else {
                    Support::Unimplemented
                },
                mp_cost: i64::MAX,
                reuse_millis: i64::MAX,
                name: "名".repeat(85) + "x",
            })
            .collect();
        catalog.validate().unwrap();
        assert!(catalog.teleport().is_none());
        catalog.skills[0].support = Support::UnsupportedTemplate;
        // Basic Heal has a qualified v13 implementation, so a failed source
        // template is now an explicit supported failure state, never usable.
        catalog.validate().unwrap();
        assert!(catalog.basic_heal().is_none());
        catalog.skills[1].support = Support::UnsupportedTemplate;
        assert!(catalog.validate().is_err());
        let mut invalid = hex(SKILLS_RESULT)[16..].to_vec();
        invalid.push(0);
        assert!(decode_skills(&invalid, 4).is_err());
        invalid = hex(SKILLS_RESULT)[16..].to_vec();
        invalid[21] = 1;
        assert!(decode_skills(&invalid, 4).is_err());
        assert!(decode_skills(&vec![0; 65536], 4).is_err());
    }
    #[test]
    fn v4_appendix_is_exact_and_old_world_bytes_remain_unchanged() {
        let old = hex(WORLD);
        for version in [1, 2] {
            let (frame, used) = world::decode_frame_version(&old, context().key, version).unwrap();
            assert_eq!(used, old.len());
            assert!(frame.casts.is_empty());
        }
        let mut v3 = old.clone();
        v3.extend([0; 4]);
        let (_, used) = world::decode_frame_version(&v3, context().key, 3).unwrap();
        assert_eq!(used, v3.len());
        assert!(world::decode_frame_version(&v3, context().key, 4).is_err());
        let mut v4 = v3.clone();
        v4.extend([0; 4]);
        let (frame, used) = world::decode_frame_version(&v4, context().key, 4).unwrap();
        assert_eq!(used, v4.len());
        assert!(frame.casts.is_empty());
        // Same established body plus a literal 64-byte committed teleport row.
        let mut moved = v3.clone();
        moved[16..24].copy_from_slice(&2i64.to_be_bytes());
        moved[24] = 3;
        let actor_start = old.len() - 36;
        moved[actor_start + 12..actor_start + 16].copy_from_slice(&32786i32.to_be_bytes());
        moved[actor_start + 17] = 1;
        moved.extend([0, 1, 0, 0]);
        moved.extend(hex(CAST_STATE));
        // current position may be after a later MOVE, so single-frame decoding
        // doesn't falsely require actor.position == historical event.to.
        let (frame, used) = world::decode_frame_version(&moved, context().key, 4).unwrap();
        assert_eq!(used, moved.len());
        assert_eq!(frame.casts[0].to, WorldCell { x: 32725, y: 32784 });
        assert_eq!(frame.casts[0].cast_interval_millis, 1200);
        for case in 0..4 {
            let mut bad = moved.clone();
            match case {
                0 => bad[v3.len() + 1] = 2,
                1 => bad[v3.len() + 2] = 1,
                2 => {
                    let last = bad.len() - 4;
                    bad[last..].copy_from_slice(&9u32.to_be_bytes());
                }
                3 => bad.truncate(bad.len() - 1),
                _ => unreachable!(),
            };
            assert!(world::decode_frame_version(&bad, context().key, 4).is_err());
        }
    }
    #[test]
    fn authority_teleport_keeps_move_sequence_and_exact_mp_delta() {
        let old = initial();
        let cast = success(&old);
        cast.validate().unwrap();
        assert!(confirms_transition(old.frame.as_ref().unwrap(), &cast));
        let next = cast.combat_reply().unwrap();
        assert!(combat::follows(Some(&old), &next));
        assert_eq!(
            old.frame.as_ref().unwrap().world.actors[0].sequence,
            next.frame.as_ref().unwrap().world.actors[0].sequence
        );
        for case in 0..7 {
            let mut bad = next.clone();
            let frame = bad.frame.as_mut().unwrap();
            match case {
                0 => frame.world.casts[0].cast_revision = 0,
                1 => frame.world.casts[0].mp_spent = 9,
                2 => frame.vitals[0].mp = 91,
                3 => frame.world.actors[0].position.x += 1,
                4 => frame.world.casts.clear(),
                5 => frame.world.casts[0].from = frame.world.casts[0].to,
                6 => frame.world.casts[1].request_sequence = 1,
                _ => unreachable!(),
            };
            if case == 6 {
                assert!(combat::follows(Some(&old), &bad));
            } else {
                assert!(!combat::follows(Some(&old), &bad), "case {case}");
            }
        }
        let mut later = next.clone();
        let frame = later.frame.as_mut().unwrap();
        frame.world.revision += 1;
        frame.world.actors[0].sequence += 1;
        frame.world.actors[0].position.x -= 1;
        assert!(combat::follows(Some(&next), &later));
        assert!(!combat::follows(Some(&later), &next));
    }
    #[test]
    fn denial_consumes_only_request_sequence_and_cannot_reuse_old_denial_credit() {
        let old = initial();
        let mut denied = success(&old);
        denied.status = Status::Denied;
        denied.reason = Reason::Unlearned;
        denied.frame = old.frame.clone();
        let frame = denied.frame.as_mut().unwrap();
        frame.world.revision += 1;
        frame.world.casts[0].request_sequence = 1;
        denied.validate().unwrap();
        assert!(confirms_transition(old.frame.as_ref().unwrap(), &denied));
        assert!(combat::follows(Some(&old), &denied.combat_reply().unwrap()));
        denied.frame.as_mut().unwrap().world.casts[0].request_sequence = 0;
        assert!(denied.validate().is_err());
        assert!(!confirms_transition(old.frame.as_ref().unwrap(), &denied));
        let mut before = CastState::initial(1, 1000);
        before.request_sequence = 100;
        let mut after = success(&old).frame.unwrap().world.casts[0];
        after.request_sequence = 101;
        after.cast_count = 2;
        after.mp_spent = 20;
        assert!(
            !before.follows(&after),
            "old denied requests are not future cast credit"
        );
        after.request_sequence = 102;
        assert!(before.follows(&after));
        after.cast_interval_millis += 1;
        assert!(!before.follows(&after));
        let reference = initial().frame.unwrap().world;
        for value in [0, 9, 10001, u32::MAX] {
            assert!(CastState::initial(1, value).validate(&reference).is_err());
        }
        for value in [10, 1000, 10000] {
            assert!(CastState::initial(1, value).validate(&reference).is_ok());
        }
    }
    #[test]
    fn skipped_casts_and_later_move_are_monotonic_without_replaying_history() {
        let old = initial();
        let first = success(&old).combat_reply().unwrap();
        let second = success(&first).combat_reply().unwrap();
        assert!(combat::follows(Some(&old), &second));
        assert_eq!(second.frame.as_ref().unwrap().vitals[0].mp, 80);
        let mut moved = second.clone();
        let frame = moved.frame.as_mut().unwrap();
        frame.world.revision += 1;
        frame.world.actors[0].sequence = 1;
        frame.world.actors[0].position.x -= 2;
        assert!(combat::follows(Some(&old), &moved));
        let mut bad = second.clone();
        bad.frame.as_mut().unwrap().world.casts[0].from.x += 1;
        assert!(!combat::follows(Some(&second), &bad));
    }

    #[test]
    fn kinds_have_distinct_allocation_budgets_and_reject_headers_before_body_read() {
        for (kind, limit) in [
            (SKILLS_RESULT_KIND, 65536usize),
            (CAST_RESULT_KIND, 131072usize),
        ] {
            let mut header = hex(SKILLS_RESULT)[..16].to_vec();
            header[4..8].copy_from_slice(&((limit + 1) as u32).to_be_bytes());
            header[12..14].copy_from_slice(&kind.to_be_bytes());
            let mut reader = std::io::Cursor::new(header);
            assert!(app::read_response_version(&mut reader, 4).is_err());
            assert_eq!(reader.position(), 16);
        }
    }

    #[test]
    fn final_result_freezes_casts_and_rejects_events_newer_than_the_result() {
        let mut frame = super::super::world_protocol::tests::finished();
        frame.casts = frame
            .actors
            .iter()
            .map(|a| CastState::initial(a.entity_id, 1000))
            .collect();
        let a = &frame.actors[0];
        frame.casts[0] = CastState {
            entity_id: 1,
            request_sequence: 1,
            cast_count: 1,
            cast_revision: 7,
            cast_interval_millis: 1000,
            mp_spent: 10,
            skill_id: 5,
            from: WorldCell {
                x: a.position.x,
                y: a.position.y - 2,
            },
            to: a.position,
        };
        frame.validate(a.key).unwrap();
        let mut saved = frame.clone();
        saved.revision += 1;
        saved.completion.as_mut().unwrap().state = world::SettlementState::Saved;
        assert!(world::terminal_follows(&frame, &saved));
        saved.casts[0].request_sequence += 1;
        assert!(!world::terminal_follows(&frame, &saved));
        let mut wrong = frame;
        wrong.revision += 3;
        wrong.casts[0].cast_revision = wrong.revision;
        assert!(wrong.validate(wrong.actors[0].key).is_err());
    }
}
