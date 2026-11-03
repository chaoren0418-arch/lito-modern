//! Independent v32 private-chat capability. Raw targets round-trip exactly;
//! Unicode address normalization belongs solely to the qualified authority.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
use super::chat_protocol::valid_text;
use std::fmt;
pub const VERSION: u16 = 32;
pub const REQUEST_KIND: u16 = 52;
pub const RESULT_KIND: u16 = 53;
pub const MODERATED_VERSION: u16 = 33;
pub const MODERATED_REQUEST_KIND: u16 = 56;
pub const MODERATED_RESULT_KIND: u16 = 57;
pub const MIN_REQUEST_BYTES: usize = 84;
pub const MAX_REQUEST_BYTES: usize = 84 + 256 + 512;
pub const MIN_RESULT_BYTES: usize = 126;
pub const MAX_FRAME_BYTES: usize = 128 * 1024;
pub const MAX_MESSAGES: usize = 64;
pub const MAX_BLOCKED: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Send,
    Preferences,
    Block,
    Unblock,
    ClearBlocks,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    RateLimited,
    Unreachable,
    BlockListFull,
    Unavailable,
    Silenced,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Address {
    pub source_id: u32,
    pub name: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Peer {
    pub key: CharacterKey,
    pub character_name: String,
    pub source_name: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preferences {
    pub whispers: bool,
    pub world: bool,
    pub trade: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            whispers: true,
            world: true,
            trade: true,
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub room_id: u64,
    pub membership: u64,
    pub sequence: u64,
    pub after: u64,
    pub action: Action,
    pub target: Option<Address>,
    pub preferences: Option<Preferences>,
    pub text: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Message {
    pub id: u64,
    pub sender_sequence: u64,
    pub sender: Peer,
    pub recipient: Peer,
    pub text: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub sequence: u64,
    pub cursor: u64,
    pub available: bool,
    pub truncated: bool,
    pub owner: Peer,
    pub preferences: Preferences,
    pub blocked: Vec<Address>,
    pub messages: Vec<Message>,
}
macro_rules! redacted {
    ($($t:ty),*) => {$(impl fmt::Debug for $t {
        fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($t),"[redacted]"))}
    })*};
}
redacted!(Address, Peer, Request, Message, Reply);
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid modern direct chat frame",
    )
}
fn counter(n: u64) -> bool {
    n <= i64::MAX as u64
}
fn name(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
impl Address {
    fn valid(&self) -> bool {
        app::valid_source(self.source_id) && self.name.len() <= 256 && valid_text(&self.name)
    }
}
impl Peer {
    pub(crate) fn valid(&self) -> bool {
        app::valid_source(self.key.source_id)
            && self.key.object_id > 0
            && counter(self.key.object_id)
            && name(&self.character_name)
            && name(&self.source_name)
    }
    pub fn display_name(&self) -> String {
        format!("{}({})", self.character_name, self.source_name)
    }
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8, room_id: u64, membership: u64) -> Self {
        Self {
            context,
            channel_id,
            room_id,
            membership,
            sequence: 0,
            after: 0,
            action: Action::Read,
            target: None,
            preferences: None,
            text: String::new(),
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.membership == 0
            || ![self.room_id, self.membership, self.sequence, self.after]
                .into_iter()
                .all(counter)
            || (self.action == Action::Read) != (self.sequence == 0)
            || matches!(self.action, Action::Send | Action::Block | Action::Unblock)
                != self.target.is_some()
            || (self.action == Action::Preferences) != self.preferences.is_some()
            || self.target.as_ref().is_some_and(|a| !a.valid())
            || if self.action == Action::Send {
                !valid_text(&self.text)
            } else {
                !self.text.is_empty()
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if !self.owner.valid()
            || self.owner.key != self.echo.context.key
            || ![self.sequence, self.cursor].into_iter().all(counter)
            || self.available
                && (self.sequence == i64::MAX as u64 || self.cursor == i64::MAX as u64)
            || self.cursor < self.echo.after
            || self.echo.action != Action::Read && self.sequence != self.echo.sequence
            || self.status == Status::Unavailable && self.available
            || matches!(
                self.status,
                Status::RateLimited | Status::Unreachable | Status::Silenced
            ) && self.echo.action != Action::Send
            || self.status == Status::BlockListFull && self.echo.action != Action::Block
            || self.blocked.len() > MAX_BLOCKED
            || self.messages.len() > MAX_MESSAGES
        {
            return Err(invalid());
        }
        for (i, a) in self.blocked.iter().enumerate() {
            if !a.valid() || self.blocked[..i].contains(a) {
                return Err(invalid());
            }
        }
        let mut previous = self.echo.after;
        for (index, m) in self.messages.iter().enumerate() {
            if m.id <= previous
                || index > 0 && m.id != previous + 1
                || m.id > self.cursor
                || m.sender_sequence == 0
                || !counter(m.sender_sequence)
                || !m.sender.valid()
                || !m.recipient.valid()
                || !valid_text(&m.text)
                || m.sender.key != self.owner.key && m.recipient.key != self.owner.key
                || m.sender.key == self.owner.key
                    && (m.sender != self.owner || m.sender_sequence > self.sequence)
                || m.recipient.key == self.owner.key && m.recipient != self.owner
            {
                return Err(invalid());
            }
            previous = m.id;
        }
        if previous != self.cursor
            || self.truncated
                != self
                    .messages
                    .first()
                    .is_some_and(|m| m.id - 1 > self.echo.after)
        {
            return Err(invalid());
        }
        if self.status == Status::Ok {
            // Echo confirms the exact requested address; the server owns its
            // canonical name mapping and resolves the actual recipient key.
            if self.echo.action == Action::Send
                && !self.messages.last().is_some_and(|m| {
                    m.sender == self.owner
                        && m.sender_sequence == self.echo.sequence
                        && m.text == self.echo.text
                        && Some(m.recipient.key.source_id)
                            == self.echo.target.as_ref().map(|a| a.source_id)
                })
            {
                return Err(invalid());
            }
            if self.echo.action == Action::Preferences
                && Some(self.preferences) != self.echo.preferences
                || self.echo.action == Action::ClearBlocks && !self.blocked.is_empty()
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    pub fn next_read(&self) -> Request {
        let mut r = Request::read(
            self.echo.context,
            self.echo.channel_id,
            self.echo.room_id,
            self.echo.membership,
        );
        r.after = self.cursor;
        r
    }
    pub fn next_command(
        &self,
        action: Action,
        target: Option<Address>,
        preferences: Option<Preferences>,
        text: String,
    ) -> Result<Request, ApplicationError> {
        self.validate()?;
        if !self.available || action == Action::Read {
            return Err(invalid());
        }
        let mut r = self.next_read();
        r.action = action;
        r.target = target;
        r.preferences = preferences;
        r.text = text;
        r.sequence = self.sequence.checked_add(1).ok_or_else(invalid)?;
        r.validate()?;
        Ok(r)
    }
}
/// Only counters, no message bodies/names. Room changes retain the inbox;
/// successful navigation resets it, but never resets the command sequence.
#[derive(Default)]
pub(crate) struct Confirmation {
    sequence: u64,
    cursor: u64,
}
impl Confirmation {
    pub(crate) fn reset_inbox(&mut self) {
        self.cursor = 0;
    }
    pub(crate) fn permits(&self, r: &Request) -> bool {
        r.after <= self.cursor
            && (r.action == Action::Read || self.sequence.checked_add(1) == Some(r.sequence))
    }
    pub(crate) fn accept(&mut self, r: &Reply) -> bool {
        let expected = if r.echo.action == Action::Read {
            Some(self.sequence)
        } else {
            self.sequence.checked_add(1)
        };
        if r.validate().is_err()
            || !self.permits(&r.echo)
            || expected != Some(r.sequence)
            || r.cursor < self.cursor
        {
            return false;
        }
        self.sequence = r.sequence;
        self.cursor = r.cursor;
        true
    }
}
fn put_text(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}
fn bits(p: Preferences) -> u8 {
    (p.whispers as u8) | ((p.world as u8) << 1) | ((p.trade as u8) << 2)
}
fn preferences(bits: u8) -> Result<Preferences, ApplicationError> {
    if bits > 7 {
        return Err(invalid());
    }
    Ok(Preferences {
        whispers: bits & 1 != 0,
        world: bits & 2 != 0,
        trade: bits & 4 != 0,
    })
}
fn put_request(out: &mut Vec<u8>, r: &Request) {
    out.extend_from_slice(&r.context.key.source_id.to_be_bytes());
    out.extend_from_slice(&r.context.key.object_id.to_be_bytes());
    out.extend_from_slice(&r.context.epoch.to_be_bytes());
    out.extend_from_slice(&[r.channel_id, 0, 0, 0]);
    for n in [r.room_id, r.membership, r.sequence, r.after] {
        out.extend_from_slice(&n.to_be_bytes());
    }
    out.extend_from_slice(&[r.action as u8, r.preferences.map_or(0, bits), 0, 0]);
    out.extend_from_slice(&r.target.as_ref().map_or(0, |a| a.source_id).to_be_bytes());
    put_text(out, r.target.as_ref().map_or("", |a| a.name.as_str()));
    put_text(out, &r.text);
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    encode_profile(id, r, version, false)
}
pub fn encode_moderated_request(
    id: u32,
    r: &Request,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    encode_profile(id, r, version, true)
}
fn encode_profile(
    id: u32,
    r: &Request,
    version: u16,
    moderated: bool,
) -> Result<Vec<u8>, ApplicationError> {
    let minimum = if moderated {
        MODERATED_VERSION
    } else {
        VERSION
    };
    if !(minimum..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    r.validate()?;
    let mut out = app::header(
        id,
        if moderated {
            MODERATED_REQUEST_KIND
        } else {
            REQUEST_KIND
        },
        MIN_REQUEST_BYTES + r.text.len() + r.target.as_ref().map_or(0, |a| a.name.len()),
    )?;
    put_request(&mut out, r);
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.position.checked_add(n).ok_or_else(invalid)?;
        let v = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
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
    fn u64(&mut self) -> Result<u64, ApplicationError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn zero(&mut self, n: usize) -> Result<(), ApplicationError> {
        if self.take(n)?.iter().any(|&b| b != 0) {
            return Err(invalid());
        }
        Ok(())
    }
    fn text(&mut self, max: usize) -> Result<String, ApplicationError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(invalid());
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| invalid())?
            .into())
    }
    fn peer(&mut self) -> Result<Peer, ApplicationError> {
        Ok(Peer {
            key: CharacterKey {
                source_id: self.u32()?,
                object_id: self.u64()?,
            },
            character_name: self.text(256)?,
            source_name: self.text(256)?,
        })
    }
    fn request(&mut self) -> Result<Request, ApplicationError> {
        let context = SelectionContext {
            key: CharacterKey {
                source_id: self.u32()?,
                object_id: self.u64()?,
            },
            epoch: self.u64()?,
        };
        let channel_id = self.u8()?;
        self.zero(3)?;
        let room_id = self.u64()?;
        let membership = self.u64()?;
        let sequence = self.u64()?;
        let after = self.u64()?;
        let action = match self.u8()? {
            0 => Action::Read,
            1 => Action::Send,
            2 => Action::Preferences,
            3 => Action::Block,
            4 => Action::Unblock,
            5 => Action::ClearBlocks,
            _ => return Err(invalid()),
        };
        let flags = self.u8()?;
        self.zero(2)?;
        if flags > 7 || action != Action::Preferences && flags != 0 {
            return Err(invalid());
        }
        let source_id = self.u32()?;
        let name = self.text(256)?;
        let target = if source_id == 0 {
            if !name.is_empty() {
                return Err(invalid());
            }
            None
        } else {
            Some(Address { source_id, name })
        };
        Ok(Request {
            context,
            channel_id,
            room_id,
            membership,
            sequence,
            after,
            action,
            target,
            preferences: if action == Action::Preferences {
                Some(preferences(flags)?)
            } else {
                None
            },
            text: self.text(512)?,
        })
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    decode_profile(bytes, version, false)
}
pub(crate) fn decode_moderated_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    decode_profile(bytes, version, true)
}
fn decode_profile(bytes: &[u8], version: u16, moderated: bool) -> Result<Reply, ApplicationError> {
    let minimum = if moderated {
        MODERATED_VERSION
    } else {
        VERSION
    };
    if !(minimum..=super::protocol::MAX_VERSION).contains(&version)
        || !(MIN_RESULT_BYTES - 16..=MAX_FRAME_BYTES - 16).contains(&bytes.len())
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, position: 0 };
    let echo = r.request()?;
    let status = match r.u8()? {
        0 => Status::Ok,
        1 => Status::RateLimited,
        2 => Status::Unreachable,
        3 => Status::BlockListFull,
        4 => Status::Unavailable,
        5 if moderated => Status::Silenced,
        _ => return Err(invalid()),
    };
    let state = r.u8()?;
    let preferences = preferences(r.u8()?)?;
    let blocked_count = r.u8()? as usize;
    let count = r.u16()? as usize;
    r.zero(2)?;
    if state > 3 || blocked_count > MAX_BLOCKED || count > MAX_MESSAGES {
        return Err(invalid());
    }
    let sequence = r.u64()?;
    let cursor = r.u64()?;
    let owner = r.peer()?;
    let mut blocked = Vec::with_capacity(blocked_count);
    for _ in 0..blocked_count {
        blocked.push(Address {
            source_id: r.u32()?,
            name: r.text(256)?,
        });
    }
    let mut messages = Vec::with_capacity(count);
    for _ in 0..count {
        messages.push(Message {
            id: r.u64()?,
            sender_sequence: r.u64()?,
            sender: r.peer()?,
            recipient: r.peer()?,
            text: r.text(512)?,
        });
    }
    if r.position != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        sequence,
        cursor,
        owner,
        preferences,
        blocked,
        messages,
        available: state & 1 != 0,
        truncated: state & 2 != 0,
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
#[path = "direct_chat_protocol_tests.rs"]
mod tests;
