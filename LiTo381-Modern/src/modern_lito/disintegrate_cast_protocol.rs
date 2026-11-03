//! v27 typed skill77 command: authority owns damage, resources and randomness.
use super::{
    application_protocol::{self as app, ApplicationError, SelectionContext},
    combat_protocol as combat, protocol,
    spell_protocol::Status,
    world_protocol as world,
};
pub const VERSION: u16 = 27;
pub const REQUEST_KIND: u16 = 44;
pub const RESULT_KIND: u16 = 45;
pub const REQUEST_BYTES: usize = 72;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub match_id: u64,
    pub generation: u64,
    pub sequence: u64,
    pub caster_life: u64,
    pub target_entity_id: u64,
    pub target_life: u64,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || ![
                self.match_id,
                self.generation,
                self.sequence,
                self.caster_life,
                self.target_entity_id,
                self.target_life,
            ]
            .into_iter()
            .all(positive)
        {
            return Err(invalid());
        }
        Ok(())
    }
    /// No optimistic MP/damage/cooldown. Life tokens come from the last atomic view.
    pub fn ready_in(&self, f: &combat::Frame) -> bool {
        let w = &f.world;
        self.validate().is_ok()
            && f.validate().is_ok()
            && w.phase == world::Phase::Active
            && w.match_id == self.match_id
            && w.generation == self.generation
            && w.descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::DISINTEGRATE))
            && w.actors
                .iter()
                .find(|a| a.matches_character(self.context.key))
                .is_some_and(|a| {
                    a.entity_id != self.target_entity_id
                        && f.vital(a.entity_id).is_some_and(|v| v.hp > 0)
                        && f.vital(self.target_entity_id).is_some_and(|v| v.hp > 0)
                        && life(f, a.entity_id) == Some(self.caster_life)
                        && life(f, self.target_entity_id) == Some(self.target_life)
                        && w.cast(a.entity_id).is_some_and(|c| {
                            c.request_sequence.checked_add(1) == Some(self.sequence)
                        })
                })
    }
}
pub(crate) fn life(f: &combat::Frame, entity: u64) -> Option<u64> {
    if !f.world.actors.iter().any(|a| a.entity_id == entity) {
        return None;
    }
    match f.honor.as_ref().and_then(|h| h.life(entity)) {
        Some(l) => u64::try_from(l.respawn_count).ok()?.checked_add(1),
        None => Some(1),
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
    Numeric,
    TargetDead,
    SameTeam,
    OutOfRange,
    Blocked,
    TooHeavy,
    StaleEffect,
    StaleRegen,
    Lawful,
    StaleLife,
    StaleState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub reason: Reason,
    pub frame: Option<combat::Frame>,
}
impl Reply {
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    /// Retained authority snapshot is a read; it never relabels a denied cast OK.
    pub fn combat_reply(&self) -> combat::Reply {
        combat::Reply {
            echo: combat::Request {
                context: self.echo.context,
                channel_id: self.echo.channel_id,
                action: combat::Action::Read,
                match_id: self.echo.match_id,
                generation: self.echo.generation,
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
        let w = &f.world;
        if !matches!(self.status, Status::Ok | Status::Denied)
            || self.reason == Reason::NotActive && w.phase == world::Phase::Active
        {
            return Err(invalid());
        }
        if w.match_id != self.echo.match_id
            || w.generation != self.echo.generation
            || w.descriptor
                .is_none_or(|d| !d.has(world::MatchDescriptor::DISINTEGRATE))
        {
            return Err(invalid());
        }
        let a = w
            .actors
            .iter()
            .find(|a| a.matches_character(self.echo.context.key))
            .ok_or_else(invalid)?;
        if a.entity_id == self.echo.target_entity_id {
            return Err(invalid());
        }
        let v = f.vital(a.entity_id).ok_or_else(invalid)?;
        let target = f.vital(self.echo.target_entity_id).ok_or_else(invalid)?;
        let s = f.disintegrate(a.entity_id).ok_or_else(invalid)?;
        let clock = w.cast(a.entity_id).ok_or_else(invalid)?;
        if self.status == Status::Ok {
            if !matches!(w.phase, world::Phase::Active | world::Phase::Finished)
                || v.hp == 0
                || s.revision != w.revision
                || s.last_sequence != self.echo.sequence
                || clock.request_sequence != self.echo.sequence
                || s.hit.as_ref().is_none_or(|h| {
                    h.target_entity_id != self.echo.target_entity_id
                        || h.caster_life as u64 != self.echo.caster_life
                        || h.target_life as u64 != self.echo.target_life
                })
            {
                return Err(invalid());
            }
        } else if self.status == Status::Denied && self.reason != Reason::NotActive {
            if w.phase != world::Phase::Active
                || (if self.reason == Reason::StaleState {
                    clock.request_sequence < self.echo.sequence
                } else {
                    clock.request_sequence != self.echo.sequence
                })
                || self.reason == Reason::Dead && v.hp != 0
                || self.reason == Reason::TargetDead && target.hp != 0
                || self.reason == Reason::NoMp && v.mp >= i64::from(s.mp_cost)
                || self.reason == Reason::Lawful && s.lawful >= 500
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// A serial owner may advance the request clock, never another action history.
/// Full combat::follows is also required by transport (HP, regen, other actors).
pub fn confirms_transition(old: &combat::Frame, reply: &Reply) -> bool {
    if old.validate().is_err() || reply.validate().is_err() {
        return false;
    }
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
    let id = actor.entity_id;
    let Some((before, after)) = old.world.cast(id).zip(new.world.cast(id)) else {
        return false;
    };
    let Some((a, b)) = old.disintegrate(id).zip(new.disintegrate(id)) else {
        return false;
    };
    let mut clock = *before;
    clock.request_sequence = after.request_sequence;
    if before.request_sequence.checked_add(1) != Some(reply.echo.sequence)
        || clock != *after
        || old.damage(id) != new.damage(id)
        || old.fire(id) != new.fire(id)
        || old.npc_spell(id) != new.npc_spell(id)
        || old.npc_fire(id) != new.npc_fire(id)
        || old.item(id) != new.item(id)
        || old.ammo(id) != new.ammo(id)
        || old.honor.as_ref().and_then(|h| h.heal(id))
            != new.honor.as_ref().and_then(|h| h.heal(id))
    {
        return false;
    }
    if reply.status == Status::Ok {
        a.count.checked_add(1) == Some(b.count)
            && a.mp_spent.checked_add(i64::from(a.mp_cost)) == Some(b.mp_spent)
            && a.lawful.checked_add(1) == Some(b.lawful)
            && a.mp_cost == b.mp_cost
            && a.initial_lawful == b.initial_lawful
            && a.reuse_millis == b.reuse_millis
            && b.hit.as_ref().is_some_and(|h| h.from == actor.position)
            && new
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
pub fn negotiated(ack: &protocol::Ack) -> bool {
    ack.version >= VERSION
        && ack.capabilities & (protocol::CAP_DISINTEGRATE | protocol::CAP_MATCH_DESCRIPTOR)
            == protocol::CAP_DISINTEGRATE | protocol::CAP_MATCH_DESCRIPTOR
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    r.validate()?;
    if !(VERSION..=protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let mut b = app::header(id, REQUEST_KIND, 16 + REQUEST_BYTES)?;
    b.extend(r.context.key.source_id.to_be_bytes());
    b.extend(r.context.key.object_id.to_be_bytes());
    b.extend(r.context.epoch.to_be_bytes());
    b.extend([r.channel_id, 0, 0, 0]);
    for v in [
        r.match_id,
        r.generation,
        r.sequence,
        r.caster_life,
        r.target_entity_id,
        r.target_life,
    ] {
        b.extend(v.to_be_bytes());
    }
    Ok(b)
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=protocol::MAX_VERSION).contains(&version)
        || bytes.len() < REQUEST_BYTES + 4
        || bytes.len() > combat::frame_limit(version) - 16
    {
        return Err(invalid());
    }
    let u = |at| u64::from_be_bytes(bytes[at..at + 8].try_into().unwrap());
    if bytes[21..24] != [0; 3] || bytes[75] != 0 {
        return Err(invalid());
    }
    let echo = Request {
        context: SelectionContext {
            key: app::CharacterKey {
                source_id: u32::from_be_bytes(bytes[..4].try_into().unwrap()),
                object_id: u(4),
            },
            epoch: u(12),
        },
        channel_id: bytes[20],
        match_id: u(24),
        generation: u(32),
        sequence: u(40),
        caster_life: u(48),
        target_entity_id: u(56),
        target_life: u(64),
    };
    echo.validate()?;
    let status = *[
        Status::Ok,
        Status::Denied,
        Status::Busy,
        Status::Unavailable,
    ]
    .get(bytes[72] as usize)
    .ok_or_else(invalid)?;
    let reason = *[
        Reason::None,
        Reason::NotActive,
        Reason::Dead,
        Reason::Unlearned,
        Reason::Unsupported,
        Reason::TooFast,
        Reason::NoMp,
        Reason::Numeric,
        Reason::TargetDead,
        Reason::SameTeam,
        Reason::OutOfRange,
        Reason::Blocked,
        Reason::TooHeavy,
        Reason::StaleEffect,
        Reason::StaleRegen,
        Reason::Lawful,
        Reason::StaleLife,
        Reason::StaleState,
    ]
    .get(bytes[73] as usize)
    .ok_or_else(invalid)?;
    let frame = match bytes[74] {
        0 if bytes.len() == 76 => None,
        1 => {
            let (f, used) = combat::decode_frame_version(&bytes[76..], echo.context.key, version)?;
            if used != bytes.len() - 76 {
                return Err(invalid());
            }
            Some(f)
        }
        _ => return Err(invalid()),
    };
    let reply = Reply {
        echo,
        status,
        reason,
        frame,
    };
    reply.validate()?;
    Ok(reply)
}
fn positive(v: u64) -> bool {
    v > 0 && v <= i64::MAX as u64
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid Disintegrate command or result",
    )
}

#[cfg(test)]
#[path = "disintegrate_cast_protocol_tests.rs"]
mod tests;
