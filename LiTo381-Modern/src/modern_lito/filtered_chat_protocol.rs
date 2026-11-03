//! Explicit v33 recipient-local public feed. A receipt proves acceptance, not
//! self-delivery. The game controller opts in only through explicit negotiation.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
use super::chat_protocol::{valid_text, Action};
use super::direct_chat_protocol::Peer;
use std::fmt;
pub const VERSION: u16 = 33;
pub const REQUEST_KIND: u16 = 54;
pub const RESULT_KIND: u16 = 55;
pub const MIN_REQUEST_BYTES: usize = 76;
pub const MAX_REQUEST_BYTES: usize = 76 + 512;
pub const MIN_RESULT_BYTES: usize = 126;
pub const MAX_FRAME_BYTES: usize = 72 * 1024;
pub const MAX_MESSAGES: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Normal,
    World,
    Trade,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    RateLimited,
    CommandRequired,
    Silenced,
    Unavailable,
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
    pub kind: Kind,
    pub text: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Message {
    pub id: u64,
    pub sender: Peer,
    pub kind: Kind,
    pub text: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Receipt {
    pub sequence: u64,
    pub sender: Peer,
    pub kind: Kind,
    pub text: String,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub membership: u64,
    pub sequence: u64,
    pub cursor: u64,
    pub available: bool,
    pub truncated: bool,
    pub owner: Peer,
    pub messages: Vec<Message>,
    pub receipt: Option<Receipt>,
}
macro_rules! redacted {($($t:ty),*)=>{$(impl fmt::Debug for $t {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {f.write_str(concat!(stringify!($t),"[redacted]"))}
})*};}
redacted!(Request, Message, Receipt, Reply);
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid filtered chat frame",
    )
}
fn counter(n: u64) -> bool {
    n <= i64::MAX as u64
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8, room_id: u64) -> Self {
        Self {
            context,
            channel_id,
            room_id,
            membership: 0,
            sequence: 0,
            after: 0,
            action: Action::Read,
            kind: Kind::Normal,
            text: String::new(),
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || ![self.room_id, self.membership, self.sequence, self.after]
                .into_iter()
                .all(counter)
            || match self.action {
                Action::Read => {
                    self.sequence != 0
                        || self.kind != Kind::Normal
                        || !self.text.is_empty()
                        || self.membership == 0 && self.after != 0
                }
                Action::Send => {
                    self.sequence == 0 || self.membership == 0 || !valid_text(&self.text)
                }
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
        let send = self.echo.action == Action::Send;
        if self.membership == 0
            || ![self.membership, self.sequence, self.cursor]
                .into_iter()
                .all(counter)
            || !self.owner.valid()
            || self.owner.key != self.echo.context.key
            || self.available
                && (self.sequence == i64::MAX as u64 || self.cursor == i64::MAX as u64)
            || self.echo.membership != 0 && self.membership != self.echo.membership
            || self.cursor < self.echo.after
            || send && self.sequence != self.echo.sequence
            || !send
                && self.status
                    != if self.available {
                        Status::Ok
                    } else {
                        Status::Unavailable
                    }
            || self.messages.len() > 64
        {
            return Err(invalid());
        }
        let mut previous = self.echo.after;
        for (i, m) in self.messages.iter().enumerate() {
            if m.id <= previous
                || m.id > self.cursor
                || i > 0 && m.id != previous + 1
                || !m.sender.valid()
                || !valid_text(&m.text)
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
        let accepted = send && self.status == Status::Ok;
        if accepted != self.receipt.is_some()
            || self.receipt.as_ref().is_some_and(|r| {
                r.sender != self.owner
                    || r.sequence != self.echo.sequence
                    || r.kind != self.echo.kind
                    || r.text != self.echo.text
            })
        {
            return Err(invalid());
        }
        Ok(()) // UNAVAILABLE may leave local counters available: legacy NORMAL sink exhaustion.
    }
    pub fn confirms(&self, request: &Request) -> bool {
        self.echo == *request && self.validate().is_ok()
    }
    pub fn next_read(&self) -> Request {
        let mut r = self.echo.clone();
        r.membership = self.membership;
        r.sequence = 0;
        r.after = self.cursor;
        r.action = Action::Read;
        r.kind = Kind::Normal;
        r.text.clear();
        r
    }
    pub fn next_send(&self, kind: Kind, text: String) -> Result<Request, ApplicationError> {
        self.validate()?;
        if !self.available {
            return Err(invalid());
        }
        let mut r = self.next_read();
        r.action = Action::Send;
        r.kind = kind;
        r.text = text;
        r.sequence = self.sequence.checked_add(1).ok_or_else(invalid)?;
        r.validate()?;
        Ok(r)
    }
}
/// Retains only admission/counters across navigation, never names or history.
#[derive(Default)]
pub(crate) struct Confirmation {
    sequence: u64,
    cursor: u64,
    admission: Option<(SelectionContext, u8, u64, u64)>,
}
impl Confirmation {
    pub(crate) fn permits(&self, request: &Request) -> bool {
        if request.validate().is_err() {
            return false;
        }
        if request.membership == 0 {
            return request.action == Action::Read;
        }
        self.admission
            == Some((
                request.context,
                request.channel_id,
                request.room_id,
                request.membership,
            ))
            && request.after <= self.cursor
            && (request.action == Action::Read
                || self.sequence.checked_add(1) == Some(request.sequence))
    }
    pub(crate) fn accept(&mut self, reply: &Reply) -> bool {
        if reply.validate().is_err() || !self.permits(&reply.echo) {
            return false;
        }
        let expected = if reply.echo.action == Action::Read {
            Some(self.sequence)
        } else {
            self.sequence.checked_add(1)
        };
        if expected != Some(reply.sequence) {
            return false;
        }
        let next = (
            reply.echo.context,
            reply.echo.channel_id,
            reply.echo.room_id,
            reply.membership,
        );
        if let Some(prior) = self.admission {
            if next.3 < prior.3
                || next.3 == prior.3 && next != prior
                || next == prior && reply.cursor < self.cursor
                || next.3 > prior.3 && reply.echo.membership != 0
            {
                return false;
            }
        } else if reply.echo.membership != 0 || reply.echo.action != Action::Read {
            return false;
        }
        self.sequence = reply.sequence;
        self.cursor = reply.cursor;
        self.admission = Some(next);
        true
    }
}
fn put_text(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}
fn put_request(out: &mut Vec<u8>, r: &Request) {
    out.extend_from_slice(&r.context.key.source_id.to_be_bytes());
    out.extend_from_slice(&r.context.key.object_id.to_be_bytes());
    out.extend_from_slice(&r.context.epoch.to_be_bytes());
    out.extend_from_slice(&[r.channel_id, 0, 0, 0]);
    for n in [r.room_id, r.membership, r.sequence, r.after] {
        out.extend_from_slice(&n.to_be_bytes());
    }
    out.extend_from_slice(&[r.action as u8, r.kind as u8]);
    put_text(out, &r.text);
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    r.validate()?;
    let mut out = app::header(id, REQUEST_KIND, MIN_REQUEST_BYTES + r.text.len())?;
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
        let b = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(b)
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
            Err(invalid())
        } else {
            Ok(())
        }
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
    fn kind(&mut self) -> Result<Kind, ApplicationError> {
        match self.u8()? {
            0 => Ok(Kind::Normal),
            1 => Ok(Kind::World),
            2 => Ok(Kind::Trade),
            _ => Err(invalid()),
        }
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
            _ => return Err(invalid()),
        };
        let kind = self.kind()?;
        Ok(Request {
            context,
            channel_id,
            room_id,
            membership,
            sequence,
            after,
            action,
            kind,
            text: self.text(512)?,
        })
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version)
        || !(MIN_RESULT_BYTES - 16..=MAX_FRAME_BYTES - 16).contains(&bytes.len())
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, position: 0 };
    let echo = r.request()?;
    let status = match r.u8()? {
        0 => Status::Ok,
        1 => Status::RateLimited,
        2 => Status::CommandRequired,
        3 => Status::Silenced,
        4 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let flags = r.u8()?;
    let has_receipt = r.u8()?;
    r.zero(1)?;
    let count = r.u16()? as usize;
    r.zero(2)?;
    if flags > 3 || has_receipt > 1 || count > 64 {
        return Err(invalid());
    }
    let membership = r.u64()?;
    let sequence = r.u64()?;
    let cursor = r.u64()?;
    let owner = r.peer()?;
    let mut messages = Vec::with_capacity(count);
    for _ in 0..count {
        let id = r.u64()?;
        let kind = r.kind()?;
        r.zero(3)?;
        messages.push(Message {
            id,
            kind,
            sender: r.peer()?,
            text: r.text(512)?,
        });
    }
    let receipt = if has_receipt == 1 {
        let sequence = r.u64()?;
        let kind = r.kind()?;
        r.zero(3)?;
        Some(Receipt {
            sequence,
            kind,
            sender: r.peer()?,
            text: r.text(512)?,
        })
    } else {
        None
    };
    if r.position != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        membership,
        sequence,
        cursor,
        owner,
        messages,
        receipt,
        available: flags & 1 != 0,
        truncated: flags & 2 != 0,
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
#[path = "filtered_chat_protocol_tests.rs"]
mod tests;
