//! v40 confirmation of one observed death. Complete canonical state and an
//! exact receipt, never an attack ACK, automatic respawn or legacy revision.
use super::{
    application_protocol as app,
    battle_follow::Observation,
    battle_operation::Lifecycle,
    battle_payload::{self as payload, data as d},
    battle_window::Stamp,
    protocol, world_protocol as world,
};
use std::io::Read;
pub const VERSION: u16 = 40;
pub const REQUEST_KIND: u16 = 68;
pub const RESULT_KIND: u16 = 69;
pub const REQUEST_BYTES: usize = 96;
pub const MIN_RESULT_BYTES: usize = REQUEST_BYTES + 32;
pub const MAX_RESULT_BYTES: usize = MIN_RESULT_BYTES + payload::MAX_WIRE_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    Revived,
    NotActive,
    NotDead,
    StaleLife,
    SpawnBlocked,
    Numeric,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub stamp: Stamp,
    pub death_revision: i64,
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleRevivalRequest[redacted]")
    }
}
impl Request {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let s = self.stamp;
        if !app::valid_context(s.context)
            || s.channel > 3
            || s.world.iter().any(|v| *v < 1)
            || !(1..=100).contains(&s.entity)
            || s.life < 1
            || self.death_revision < 1
            || self.death_revision > s.revision
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(crate) fn from_observation(o: &Observation) -> Option<Self> {
        if !o.legacy_commands_supported() {
            return None;
        }
        let s = Stamp::of(o);
        let life = o
            .data()
            .quest
            .as_ref()?
            .lives
            .0
            .iter()
            .find(|l| l.entity == s.entity)?;
        if o.data().world.phase != d::Phase::Active || life.deaths <= life.respawns {
            return None;
        }
        let q = Self {
            stamp: s,
            death_revision: life.death_revision,
        };
        q.validate().ok()?;
        Some(q)
    }
    fn player(&self) -> d::Player {
        let s = self.stamp;
        d::Player {
            source: s.context.key.source_id as i32,
            object: s.context.key.object_id as i64,
            epoch: s.context.epoch as i64,
            channel: s.channel.into(),
        }
    }
    fn world(&self) -> d::WorldId {
        d::WorldId {
            room: self.stamp.world[0],
            match_id: self.stamp.world[1],
            generation: self.stamp.world[2],
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub reason: Reason,
    pub lifecycle: Lifecycle,
    pub data: payload::UnvalidatedData,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleRevivalReply[redacted]")
    }
}
impl Reply {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = self.echo;
        q.validate()?;
        let s = q.stamp;
        let data = &self.data;
        data.check_scene_values(&q.player(), &q.world())?;
        self.lifecycle.validate(data)?;
        if data.presentation.is_none()
            || data.personal.entity != s.entity
            || data.world.revision < s.revision
            || data.world.descriptor.features as u32 & world::MatchDescriptor::QUEST_LIVES == 0
        {
            return Err(invalid());
        }
        let life = data
            .quest
            .as_ref()
            .ok_or_else(invalid)?
            .lives
            .0
            .iter()
            .find(|l| l.entity == s.entity)
            .ok_or_else(invalid)?;
        let actor = data
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or_else(invalid)?;
        let incarnation = life.respawns.checked_add(1).ok_or_else(invalid)?;
        let dead = life.deaths > life.respawns;
        let same = incarnation == s.life && life.death_revision == q.death_revision;
        if self.reason == Reason::NotActive {
            return if self.lifecycle.phase != world::Phase::Active {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        if !matches!(
            self.lifecycle.phase,
            world::Phase::Active | world::Phase::Aborted
        ) || data.world.phase != d::Phase::Active
        {
            return Err(invalid());
        }
        let valid = match self.reason {
            Reason::Revived => {
                let vital = actor.vital.as_ref().ok_or_else(invalid)?;
                s.life.checked_add(1) == Some(incarnation)
                    && !dead
                    && life.death_revision == q.death_revision
                    && life.respawn_revision == data.world.revision
                    && data.world.revision > s.revision
                    && vital.values.hp == vital.values.maximum_hp
                    && actor.position == life.spawn
            }
            Reason::NotDead => same && !dead,
            Reason::StaleLife => !same,
            Reason::SpawnBlocked | Reason::Numeric => same && dead,
            Reason::NotActive => false,
        };
        if valid {
            Ok(())
        } else {
            Err(invalid())
        }
    }
}
pub fn negotiated(ack: &protocol::Ack) -> bool {
    ack.version >= VERSION && super::battle_world::negotiated(ack)
}
pub(crate) fn reply_negotiated(r: &Reply, ack: &protocol::Ack) -> bool {
    reply_negotiated_selected(r, protocol::extensions::Selected::from(*ack))
}
pub(crate) fn reply_negotiated_selected(r: &Reply, peer: protocol::extensions::Selected) -> bool {
    let ack = &peer.base();
    let needed =
        world::MatchDescriptor::feature_capabilities(r.data.world.descriptor.features as u32);
    negotiated(ack)
        && super::battle_read::command_data_negotiated_selected(&r.data, peer)
        && r.data.presentation.is_some()
        && ack.capabilities & needed == needed
        && (r.data.manifest.sectors.0.len() <= 16
            || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
}
pub fn encode_request(
    id: u32,
    q: &Request,
    version: u16,
) -> Result<Vec<u8>, app::ApplicationError> {
    q.validate()?;
    if !(VERSION..=protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let s = q.stamp;
    let mut b = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for n in [
        s.world[0],
        s.world[1],
        s.world[2],
        s.entity,
        s.life,
        s.revision,
        q.death_revision,
    ] {
        b.extend(n.to_be_bytes());
    }
    Ok(b)
}
fn expected_request(b: &[u8], version: u16) -> Result<Request, app::ApplicationError> {
    if b.len() != REQUEST_BYTES {
        return Err(invalid());
    }
    let q = Request {
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: u32_at(b, 16),
                    object_id: i64_at(b, 20) as u64,
                },
                epoch: i64_at(b, 28) as u64,
            },
            channel: u32_at(b, 36).try_into().map_err(|_| invalid())?,
            world: [i64_at(b, 40), i64_at(b, 48), i64_at(b, 56)],
            entity: i64_at(b, 64),
            life: i64_at(b, 72),
            revision: i64_at(b, 80),
        },
        death_revision: i64_at(b, 88),
    };
    if encode_request(u32_at(b, 8), &q, version)? != b {
        return Err(invalid());
    }
    Ok(q)
}
pub fn read_reply(
    reader: &mut impl Read,
    expected: &[u8],
    version: u16,
) -> Result<Reply, app::ApplicationError> {
    read_reply_format(reader, expected, payload::Format::Legacy(version))
}
pub fn read_reply_selected(
    reader: &mut impl Read,
    expected: &[u8],
    peer: crate::modern_lito::protocol::extensions::Selected,
) -> Result<Reply, app::ApplicationError> {
    read_reply_format(reader, expected, payload::Format::Selected(peer))
}
fn read_reply_format(
    reader: &mut impl Read,
    expected: &[u8],
    format: payload::Format,
) -> Result<Reply, app::ApplicationError> {
    let version = format.version();
    let q = expected_request(expected, version)?;
    let mut h = [0; 16];
    reader.read_exact(&mut h).map_err(|_| invalid())?;
    let size = u32_at(&h, 4) as usize;
    if &h[..4] != b"LTM1"
        || h[8..12] != expected[8..12]
        || h[12..14] != RESULT_KIND.to_be_bytes()
        || h[14..16] != [0; 2]
        || !(MIN_RESULT_BYTES..=format.limit(MAX_RESULT_BYTES)).contains(&size)
    {
        return Err(invalid());
    }
    let mut body = vec![0; size - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    if body[..REQUEST_BYTES - 16] != expected[16..] {
        return Err(invalid());
    }
    let m = &body[REQUEST_BYTES - 16..MIN_RESULT_BYTES - 16];
    let data = &body[MIN_RESULT_BYTES - 16..];
    if m[1..4] != [0; 3] || u32_at(m, 28) as usize != data.len() {
        return Err(invalid());
    }
    let reason = *[
        Reason::Revived,
        Reason::NotActive,
        Reason::NotDead,
        Reason::StaleLife,
        Reason::SpawnBlocked,
        Reason::Numeric,
    ]
    .get(m[0] as usize)
    .ok_or_else(invalid)?;
    let r = Reply {
        echo: q,
        reason,
        lifecycle: Lifecycle::read(&m[4..28])?,
        data: format.decode_command(data)?,
    };
    r.validate()?;
    Ok(r)
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid canonical death confirmation",
    )
}

#[cfg(test)]
#[path = "battle_revival_tests.rs"]
mod tests;
