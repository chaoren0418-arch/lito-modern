//! v31 typed NPC/structure AoE application contract. No client damage or target
//! list. Room/skill/runtime admission is separate from this bounded codec.
use super::{
    application_protocol::{self as app, ApplicationError},
    npc_fire_storm_wire as wire, npc_protocol as npc, world_protocol as world,
};
pub use wire::Request;
pub const VERSION: u16 = 31;
pub const REQUEST_KIND: u16 = 48;
pub const RESULT_KIND: u16 = 49;

pub fn negotiated(ack: &super::protocol::Ack) -> bool {
    let required = super::protocol::CAP_NPC_FIRE_STORM
        | super::protocol::CAP_NPC
        | super::protocol::CAP_MATCH_DESCRIPTOR
        | super::protocol::CAP_REGEN;
    ack.version >= VERSION && ack.capabilities & required == required
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
    TooHeavy,
    Numeric,
    StaleState,
    StaleRegen,
}
impl Request {
    pub fn ready_in(&self, f: &npc::Frame) -> bool {
        let w = &f.combat.world;
        self.validate().is_ok()
            && f.validate().is_ok()
            && self.scope == f.scope
            && w.phase == world::Phase::Active
            && f.resources.is_some()
            && f.names.is_some()
            && w.descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::NPC_FIRE_STORM))
            && w.actors
                .iter()
                .find(|a| a.matches_character(self.context.key))
                .is_some_and(|a| {
                    f.combat.vital(a.entity_id).is_some_and(|v| v.hp > 0)
                        && f.combat.weight(a.entity_id).is_none_or(|v| v.can_attack())
                        && f.combat.npc_fire(a.entity_id).is_some()
                        && w.cast(a.entity_id).is_some_and(|s| {
                            s.request_sequence.checked_add(1) == Some(self.sequence)
                        })
                        && super::quest_life::life(w, a.entity_id).map_or(1, |l| l.incarnation())
                            == self.caster_life
                        && w.time_attack
                            .as_ref()
                            .is_none_or(|t| !t.departed(a.entity_id))
                })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: npc::Status,
    pub reason: Reason,
    pub frame: Option<npc::Frame>,
}
impl Reply {
    pub fn confirms(&self, r: &Request) -> bool {
        self.echo == *r && self.validate().is_ok()
    }
    pub fn npc_reply(&self) -> npc::Reply {
        npc::Reply {
            echo: npc::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: npc::Action::Read,
                world: self.echo.scope,
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
        if !matches!(self.status, npc::Status::Ok | npc::Status::Denied) {
            return Err(invalid());
        }
        f.validate()?;
        let w = &f.combat.world;
        if f.scope != self.echo.scope
            || w.descriptor
                .is_none_or(|d| !d.has(world::MatchDescriptor::NPC_FIRE_STORM))
            || self.reason == Reason::NotActive && w.phase == world::Phase::Active
        {
            return Err(invalid());
        }
        let a = w
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        let v = f.combat.vital(a.entity_id).ok_or_else(invalid)?;
        let s = f.combat.npc_fire(a.entity_id).ok_or_else(invalid)?;
        let clock = w.cast(a.entity_id).ok_or_else(invalid)?;
        let incarnation = super::quest_life::life(w, a.entity_id).map_or(1, |l| l.incarnation());
        if self.status == npc::Status::Ok {
            if !matches!(w.phase, world::Phase::Active | world::Phase::Finished)
                || v.hp == 0
                || incarnation != self.echo.caster_life
                || s.revision > w.revision
                || s.last_sequence != self.echo.sequence
                || clock.request_sequence != self.echo.sequence
                || s.caster_life != self.echo.caster_life
                || s.count == 0
                || s.origin != Some(a.position)
            {
                return Err(invalid());
            }
        } else if self.reason != Reason::NotActive {
            if w.phase != world::Phase::Active
                || (if self.reason == Reason::StaleState {
                    clock.request_sequence < self.echo.sequence
                } else {
                    clock.request_sequence != self.echo.sequence
                })
                || self.reason == Reason::Dead && v.hp != 0
                || self.reason == Reason::NoMp && v.mp >= i64::from(s.mp_cost)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// Serialized owner confirmation; the authoritative full-frame history must
/// follow too. A denial cannot spend MP or silently cast another owner skill.
pub fn confirms_transition(old: &npc::Frame, reply: &Reply) -> bool {
    if old.validate().is_err() || reply.validate().is_err() {
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
    let Some((a, b)) = old.combat.npc_fire(id).zip(new.combat.npc_fire(id)) else {
        return false;
    };
    let mut clock = *before;
    clock.request_sequence = after.request_sequence;
    let prior = npc::Reply {
        echo: reply.npc_reply().echo,
        status: npc::Status::Ok,
        reason: npc::Reason::None,
        frame: Some(old.clone()),
    };
    if before.request_sequence.checked_add(1) != Some(reply.echo.sequence)
        || clock != *after
        || !npc::follows(Some(&prior), &reply.npc_reply())
        || old.combat.npc_spell(id) != new.combat.npc_spell(id)
        || old.combat.npc_disintegrate(id) != new.combat.npc_disintegrate(id)
        || old.combat.disintegrate(id) != new.combat.disintegrate(id)
        || old.combat.damage(id) != new.combat.damage(id)
        || old.combat.fire(id) != new.combat.fire(id)
    {
        return false;
    }
    if reply.status == npc::Status::Ok {
        a.count.checked_add(1) == Some(b.count)
            && a.mp_cost == b.mp_cost
            && a.mp_spent.checked_add(i64::from(a.mp_cost)) == Some(b.mp_spent)
            && b.origin == Some(actor.position)
            && new
                .combat
                .world
                .actors
                .iter()
                .find(|n| n.entity_id == id)
                .is_some_and(|n| {
                    n.position == actor.position
                        && n.sequence == actor.sequence
                        && n.heading == actor.heading
                })
    } else {
        a == b
            && (after.request_sequence == reply.echo.sequence
                || reply.reason == Reason::NotActive
                    && after.request_sequence == before.request_sequence)
    }
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    let body = wire::encode_request(r, version)?;
    let mut out = app::header(id, REQUEST_KIND, 16 + body.len())?;
    out.extend(body);
    Ok(out)
}
pub(crate) fn decode_reply(bytes: &[u8], version: u16) -> Result<(Reply, usize), ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() < 68
        || bytes.len() > npc::frame_limit(version) - 16
    {
        return Err(invalid());
    }
    let echo = wire::decode_request(&bytes[..64], version)?;
    let status = match bytes[64] {
        0 => npc::Status::Ok,
        1 => npc::Status::Denied,
        2 => npc::Status::Busy,
        3 => npc::Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match bytes[65] {
        0 => Reason::None,
        1 => Reason::NotActive,
        2 => Reason::Dead,
        3 => Reason::Unlearned,
        4 => Reason::Unsupported,
        5 => Reason::TooFast,
        6 => Reason::NoMp,
        7 => Reason::TooHeavy,
        8 => Reason::Numeric,
        9 => Reason::StaleState,
        10 => Reason::StaleRegen,
        _ => return Err(invalid()),
    };
    if bytes[66] > 1 || bytes[67] != 0 {
        return Err(invalid());
    }
    let (frame, used) = if bytes[66] == 1 {
        let (f, n) = npc::decode_frame(&bytes[68..], echo.context.key, version)?;
        (Some(f), 68 + n)
    } else {
        (None, 68)
    };
    let reply = Reply {
        echo,
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok((reply, used))
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid NPC70 application contract",
    )
}

#[cfg(test)]
#[path = "npc_fire_storm_protocol_tests.rs"]
pub(crate) mod tests;
