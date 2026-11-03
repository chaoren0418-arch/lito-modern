//! Explicit v39 world lifecycle, without synthesizing a legacy World/Combat
//! frame. A preparation attachment never means renderer or LOAD permission.
use super::{
    application_protocol as app,
    battle_operation::Lifecycle,
    battle_payload::{self as payload, data as d},
    protocol, world_protocol as world,
};
use std::io::Read;
pub const VERSION: u16 = 39;
pub const REQUEST_KIND: u16 = 66;
pub const RESULT_KIND: u16 = 67;
pub const REQUEST_BYTES: usize = 112;
pub const METADATA_BYTES: usize = 56;
pub const MIN_RESULT_BYTES: usize = REQUEST_BYTES + METADATA_BYTES;
pub const MAX_RESULT_BYTES: usize = MIN_RESULT_BYTES + payload::MAX_WIRE_BYTES;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Prepare,
    Loaded,
    Leave,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Detached,
    BeforeEngine,
    Data,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Request {
    pub context: app::SelectionContext,
    pub channel: u8,
    pub action: Action,
    pub room: i64,
    pub room_revision: i64,
    pub match_id: i64,
    pub generation: i64,
    pub digest: [u8; 32],
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleWorldRequest[redacted]")
    }
}
impl Request {
    pub fn read(context: app::SelectionContext, channel: u8, world: Option<&d::WorldId>) -> Self {
        Self {
            context,
            channel,
            action: Action::Read,
            room: world.map_or(0, |w| w.room),
            room_revision: 0,
            match_id: world.map_or(0, |w| w.match_id),
            generation: world.map_or(0, |w| w.generation),
            digest: [0; 32],
        }
    }
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        let has = self.room > 0 && self.match_id > 0 && self.generation > 0;
        let empty = self.room == 0 && self.match_id == 0 && self.generation == 0;
        let zero = self.digest == [0; 32];
        if !app::valid_context(self.context)
            || self.channel > 3
            || !match self.action {
                Action::Read => self.room_revision == 0 && zero && (has || empty),
                Action::Prepare => {
                    self.room > 0
                        && (1..i64::MAX).contains(&self.room_revision)
                        && self.match_id == 0
                        && self.generation == 0
                        && zero
                }
                Action::Loaded => self.room_revision == 0 && has && !zero,
                Action::Leave => self.room_revision == 0 && has && zero,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn player(&self) -> d::Player {
        d::Player {
            source: self.context.key.source_id as i32,
            object: self.context.key.object_id as i64,
            epoch: self.context.epoch as i64,
            channel: self.channel.into(),
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Attachment {
    pub world: d::WorldId,
    pub lifecycle: Lifecycle,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: world::Status,
    pub reason: world::Reason,
    pub state: State,
    pub attachment: Option<Attachment>,
    pub data: Option<payload::UnvalidatedData>,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleWorldReply[redacted]")
    }
}
impl Reply {
    /// Also used at atomic stream admission: public DTOs are not trusted just
    /// because the byte decoder normally constructs them.
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = &self.echo;
        q.validate()?;
        if (self.status == world::Status::Ok) != (self.reason == world::Reason::None) {
            return Err(invalid());
        }
        match (&self.state, &self.attachment, &self.data) {
            (State::Detached, None, None) => {}
            (State::BeforeEngine | State::Data, Some(a), data) => {
                let key = &a.world;
                if key.room < 1
                    || key.match_id < 1
                    || key.generation < 1
                    || q.match_id != 0
                        && (q.room != key.room
                            || q.match_id != key.match_id
                            || q.generation != key.generation)
                {
                    return Err(invalid());
                }
                if self.state == State::Data {
                    let data = data.as_ref().ok_or_else(invalid)?;
                    data.check_scene_values(&q.player(), key)?;
                    if data.presentation.is_none() {
                        return Err(invalid());
                    }
                    a.lifecycle.validate(data)?;
                } else {
                    let l = a.lifecycle;
                    let preparing = l.phase == world::Phase::Preparing;
                    if data.is_some()
                        || !(preparing || l.phase == world::Phase::Aborted)
                        || l.revision < 1
                        || preparing && (l.revision != 1 || l.abort != world::AbortReason::None)
                        || !preparing && (l.revision < 2 || l.abort == world::AbortReason::None)
                        || l.settlement.is_some()
                        || l.race_settlement.is_some()
                        || l.race_terminal_revision != 0
                            && (preparing || l.race_terminal_revision != l.revision)
                    {
                        return Err(invalid());
                    }
                }
            }
            _ => return Err(invalid()),
        }
        if self.status == world::Status::Ok {
            match q.action {
                Action::Read => {
                    if q.match_id != 0 && self.attachment.is_none() {
                        return Err(invalid());
                    }
                }
                Action::Prepare => {
                    let a = self.attachment.as_ref().ok_or_else(invalid)?;
                    if self.state != State::BeforeEngine
                        || a.lifecycle.phase != world::Phase::Preparing
                        || a.lifecycle.revision != 1
                        || a.world.room != q.room
                        || a.world.generation != q.room_revision + 1
                    {
                        return Err(invalid());
                    }
                }
                Action::Loaded => {
                    let d = self.data.as_ref().ok_or_else(invalid)?;
                    if d.manifest.digest != q.digest
                        || !d
                            .actors
                            .0
                            .iter()
                            .any(|a| a.entity == d.personal.entity && a.loaded)
                    {
                        return Err(invalid());
                    }
                }
                Action::Leave => {
                    if self.attachment.is_some() {
                        return Err(invalid());
                    }
                }
            }
        }
        Ok(())
    }
}
pub fn negotiated(ack: &protocol::Ack) -> bool {
    ack.version >= VERSION && super::battle_operation::negotiated(ack)
}
pub(crate) fn reply_negotiated(r: &Reply, ack: &protocol::Ack) -> bool {
    reply_negotiated_selected(r, protocol::extensions::Selected::from(*ack))
}
pub(crate) fn reply_negotiated_selected(r: &Reply, peer: protocol::extensions::Selected) -> bool {
    let ack = &peer.base();
    negotiated(ack)
        && r.data.as_ref().is_none_or(|d| {
            let needed =
                world::MatchDescriptor::feature_capabilities(d.world.descriptor.features as u32);
            super::battle_read::command_data_negotiated_selected(d, peer)
                && d.presentation.is_some()
                && ack.capabilities & needed == needed
                && (d.manifest.sectors.0.len() <= 16
                    || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
        })
}
pub fn encode_request(
    id: u32,
    r: &Request,
    version: u16,
) -> Result<Vec<u8>, app::ApplicationError> {
    r.validate()?;
    if !(VERSION..=protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let mut b = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(r.context.key.source_id.to_be_bytes());
    b.extend(r.context.key.object_id.to_be_bytes());
    b.extend(r.context.epoch.to_be_bytes());
    b.extend(u32::from(r.channel).to_be_bytes());
    b.push(r.action as u8);
    b.extend([0; 7]);
    for n in [r.room, r.room_revision, r.match_id, r.generation] {
        b.extend(n.to_be_bytes());
    }
    b.extend(r.digest);
    Ok(b)
}
pub(crate) fn expected_request(
    expected: &[u8],
    version: u16,
) -> Result<Request, app::ApplicationError> {
    if expected.len() != REQUEST_BYTES {
        return Err(invalid());
    }
    let r = Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: u32_at(expected, 16),
                object_id: i64_at(expected, 20) as u64,
            },
            epoch: i64_at(expected, 28) as u64,
        },
        channel: u32_at(expected, 36).try_into().map_err(|_| invalid())?,
        action: match expected[40] {
            0 => Action::Read,
            1 => Action::Prepare,
            2 => Action::Loaded,
            3 => Action::Leave,
            _ => return Err(invalid()),
        },
        room: i64_at(expected, 48),
        room_revision: i64_at(expected, 56),
        match_id: i64_at(expected, 64),
        generation: i64_at(expected, 72),
        digest: expected[80..112].try_into().unwrap(),
    };
    if encode_request(u32_at(expected, 8), &r, version)? != expected {
        return Err(invalid());
    }
    Ok(r)
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
    read_reply_contract(
        reader,
        expected,
        q,
        RESULT_KIND,
        format.limit(MAX_RESULT_BYTES),
        |bytes| format.decode_command(bytes),
    )
}
/// Shared bounded envelope only. Each caller selects and validates its own
/// request opcode, capability and payload contract before using this parser.
pub(crate) fn read_reply_contract(
    reader: &mut impl Read,
    expected: &[u8],
    q: Request,
    result_kind: u16,
    maximum: usize,
    decode: impl FnOnce(&[u8]) -> Result<payload::UnvalidatedData, app::ApplicationError>,
) -> Result<Reply, app::ApplicationError> {
    if expected.len() != REQUEST_BYTES || maximum < MIN_RESULT_BYTES {
        return Err(invalid());
    }
    let mut header = [0; 16];
    reader.read_exact(&mut header).map_err(|_| invalid())?;
    let length = u32_at(&header, 4) as usize;
    if &header[..4] != b"LTM1"
        || header[8..12] != expected[8..12]
        || header[12..14] != result_kind.to_be_bytes()
        || header[14..16] != [0; 2]
        || !(MIN_RESULT_BYTES..=maximum).contains(&length)
    {
        return Err(invalid());
    }
    let mut body = vec![0; length - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    if body[..REQUEST_BYTES - 16] != expected[16..] {
        return Err(invalid());
    }
    let meta = &body[REQUEST_BYTES - 16..MIN_RESULT_BYTES - 16];
    let status = match meta[0] {
        0 => world::Status::Ok,
        1 => world::Status::Denied,
        2 => world::Status::Busy,
        3 => world::Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match meta[1] {
        0 => world::Reason::None,
        1 => world::Reason::InvalidState,
        2 => world::Reason::NotOwner,
        3 => world::Reason::RoomRevision,
        4 => world::Reason::NotReady,
        5 => world::Reason::NoBinding,
        6 => world::Reason::Capacity,
        7 => world::Reason::PreparationFailed,
        8 => world::Reason::NotActive,
        _ => return Err(invalid()),
    };
    let state = match meta[2] {
        0 => State::Detached,
        1 => State::BeforeEngine,
        2 => State::Data,
        _ => return Err(invalid()),
    };
    let data_bytes = &body[MIN_RESULT_BYTES - 16..];
    if meta[3] != 0
        || (status == world::Status::Ok) != (reason == world::Reason::None)
        || u32_at(meta, 52) as usize != data_bytes.len()
        || (state == State::Data) != !data_bytes.is_empty()
    {
        return Err(invalid());
    }
    let (attachment, data) = if state == State::Detached {
        if meta[4..52].iter().any(|b| *b != 0) {
            return Err(invalid());
        }
        (None, None)
    } else {
        let key = d::WorldId {
            room: i64_at(meta, 4),
            match_id: i64_at(meta, 12),
            generation: i64_at(meta, 20),
        };
        let lifecycle = Lifecycle::read(&meta[28..52])?;
        let data = if state == State::Data {
            Some(decode(data_bytes)?)
        } else {
            None
        };
        (
            Some(Attachment {
                world: key,
                lifecycle,
            }),
            data,
        )
    };
    let reply = Reply {
        echo: q,
        status,
        reason,
        state,
        attachment,
        data,
    };
    reply.validate()?;
    Ok(reply)
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
        "invalid canonical world lifecycle",
    )
}

#[cfg(test)]
#[path = "battle_world_tests.rs"]
mod tests;
