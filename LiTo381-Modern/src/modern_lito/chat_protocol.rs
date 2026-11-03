//! Optional v32 chat. Qualified identity/audience are server-owned; messages
//! are plain Unicode text, never markup, SQL or locally executed GM commands.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
use std::fmt;
pub const VERSION: u16 = 32;
pub const REQUEST_KIND: u16 = 50;
pub const RESULT_KIND: u16 = 51;
pub const MAX_REQUEST_BYTES: usize = 16 + 60 + 512;
pub const MAX_FRAME_BYTES: usize = 72 * 1024;
pub const MAX_MESSAGES: usize = 64;
/// Connection-wide counters retained across source/channel/room navigation.
/// No message history or display names are retained by this guard.
#[derive(Default)]
pub(crate) struct Confirmation {
    sequence: u64,
    membership: u64,
}
impl Confirmation {
    pub(crate) fn accept(&mut self, reply: &Reply) -> bool {
        let expected = match reply.echo.action {
            Action::Read => Some(self.sequence),
            Action::Send => self.sequence.checked_add(1),
        };
        if reply.validate().is_err()
            || expected != Some(reply.sequence)
            || reply.membership != 0 && reply.membership < self.membership
            || reply.membership == 0 && reply.status != Status::Unavailable
        {
            return false;
        }
        self.sequence = reply.sequence;
        self.membership = self.membership.max(reply.membership);
        true
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Send,
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
    pub text: String,
}
impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChatRequest[redacted]")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    RateLimited,
    CommandRequired,
    Unavailable,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Message {
    pub id: u64,
    pub sender: CharacterKey,
    pub character_name: String,
    pub source_name: String,
    pub text: String,
}
impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChatMessage[redacted]")
    }
}
impl Message {
    pub fn display_name(&self) -> String {
        format!("{}({})", self.character_name, self.source_name)
    }
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
    pub messages: Vec<Message>,
}
impl fmt::Debug for Reply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChatReply[redacted]")
    }
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid modern chat frame")
}
fn counter(n: u64) -> bool {
    n <= i64::MAX as u64
}
// Format controls from the server JDK 17 Unicode 13 policy. ZWJ/ZWNJ are
// retained for ordinary composed text; invisible/bidi control injection is not.
fn format_control(c: char) -> bool {
    matches!(c as u32,0xad|0x600..=0x605|0x61c|0x6dd|0x70f|0x8e2|0x180e|0x200b|0x200e..=0x200f
        |0x202a..=0x202e|0x2060..=0x2064|0x2066..=0x206f|0xfeff|0xfff9..=0xfffb|0x110bd|0x110cd
        |0x13430..=0x13438|0x1bca0..=0x1bca3|0x1d173..=0x1d17a|0xe0001|0xe0020..=0xe007f)
}
pub fn valid_text(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 512
        && text.chars().count() <= 256
        && text
            .chars()
            .any(|c| !c.is_whitespace() && !matches!(c, '\u{200c}' | '\u{200d}'))
        && !text
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') || format_control(c))
}
fn name(text: &str) -> bool {
    !text.trim().is_empty() && text.len() <= 256 && !text.chars().any(char::is_control)
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
                        || !self.text.is_empty()
                        || self.membership == 0 && self.after != 0
                }
                Action::Send => {
                    self.membership == 0 || self.sequence == 0 || !valid_text(&self.text)
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
        if ![self.membership, self.sequence, self.cursor]
            .into_iter()
            .all(counter)
            || self.available && self.membership == 0
            || self.cursor < self.echo.after
            || self.echo.membership != 0 && self.membership != self.echo.membership
            || self.echo.action == Action::Send && self.sequence != self.echo.sequence
            || self.echo.action == Action::Read
                && !matches!(self.status, Status::Ok | Status::Unavailable)
            || self.status == Status::Unavailable && self.available
            || self.messages.len() > MAX_MESSAGES
        {
            return Err(invalid());
        }
        let mut previous = self.echo.after;
        for m in &self.messages {
            if m.id <= previous
                || m.id > self.cursor
                || !app::valid_source(m.sender.source_id)
                || m.sender.object_id == 0
                || !counter(m.sender.object_id)
                || !name(&m.character_name)
                || !name(&m.source_name)
                || !valid_text(&m.text)
            {
                return Err(invalid());
            }
            previous = m.id;
        }
        if !self.messages.is_empty() && previous != self.cursor {
            return Err(invalid());
        }
        if self.echo.action == Action::Send && self.status == Status::Ok {
            if !self
                .messages
                .last()
                .is_some_and(|m| m.sender == self.echo.context.key && m.text == self.echo.text)
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
        let mut r = self.echo.clone();
        r.membership = self.membership;
        r.after = self.cursor;
        r.sequence = 0;
        r.action = Action::Read;
        r.text.clear();
        r
    }
    pub fn next_send(&self, text: String) -> Result<Request, ApplicationError> {
        self.validate()?;
        if !self.available {
            return Err(invalid());
        }
        let mut r = self.next_read();
        r.action = Action::Send;
        r.sequence = self.sequence.checked_add(1).ok_or_else(invalid)?;
        r.text = text;
        r.validate()?;
        Ok(r)
    }
}
fn put_text(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(&(text.len() as u16).to_be_bytes());
    out.extend_from_slice(text.as_bytes());
}
fn put_request(out: &mut Vec<u8>, r: &Request) {
    out.extend_from_slice(&r.context.key.source_id.to_be_bytes());
    out.extend_from_slice(&r.context.key.object_id.to_be_bytes());
    out.extend_from_slice(&r.context.epoch.to_be_bytes());
    out.extend_from_slice(&[r.channel_id, 0, 0, 0]);
    for n in [r.room_id, r.membership, r.sequence, r.after] {
        out.extend_from_slice(&n.to_be_bytes());
    }
    out.extend_from_slice(&[r.action as u8, 0]);
    put_text(out, &r.text);
}
pub fn encode_request(
    id: u32,
    request: &Request,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    request.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 76 + request.text.len())?;
    put_request(&mut out, request);
    Ok(out)
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.position.checked_add(n).ok_or_else(invalid)?;
        let value = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(value)
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
    fn text(&mut self, max: usize) -> Result<String, ApplicationError> {
        let n = self.u16()? as usize;
        if n > max {
            return Err(invalid());
        }
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| invalid())?
            .into())
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version)
        || !(88..=MAX_FRAME_BYTES - 16).contains(&bytes.len())
    {
        return Err(invalid());
    }
    let mut r = Reader { bytes, position: 0 };
    let context = SelectionContext {
        key: CharacterKey {
            source_id: r.u32()?,
            object_id: r.u64()?,
        },
        epoch: r.u64()?,
    };
    let channel_id = r.u8()?;
    if r.take(3)? != [0; 3] {
        return Err(invalid());
    }
    let room_id = r.u64()?;
    let membership = r.u64()?;
    let sequence = r.u64()?;
    let after = r.u64()?;
    let action = match r.u8()? {
        0 => Action::Read,
        1 => Action::Send,
        _ => return Err(invalid()),
    };
    if r.u8()? != 0 {
        return Err(invalid());
    }
    let echo = Request {
        context,
        channel_id,
        room_id,
        membership,
        sequence,
        after,
        action,
        text: r.text(512)?,
    };
    let status = match r.u8()? {
        0 => Status::Ok,
        1 => Status::RateLimited,
        2 => Status::CommandRequired,
        3 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let flags = r.u8()?;
    let count = r.u16()? as usize;
    if flags > 3 || count > MAX_MESSAGES {
        return Err(invalid());
    }
    let membership = r.u64()?;
    let sequence = r.u64()?;
    let cursor = r.u64()?;
    let mut messages = Vec::with_capacity(count);
    for _ in 0..count {
        messages.push(Message {
            id: r.u64()?,
            sender: CharacterKey {
                source_id: r.u32()?,
                object_id: r.u64()?,
            },
            character_name: r.text(256)?,
            source_name: r.text(256)?,
            text: r.text(512)?,
        });
    }
    if r.position != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        echo,
        status,
        membership,
        sequence,
        cursor,
        available: flags & 1 != 0,
        truncated: flags & 2 != 0,
        messages,
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> Request {
        Request::read(
            SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            3,
            0,
        )
    }
    fn reply() -> Reply {
        Reply {
            echo: request(),
            status: Status::Ok,
            membership: 1,
            sequence: 0,
            cursor: 0,
            available: true,
            truncated: false,
            messages: vec![],
        }
    }
    fn frame(r: &Reply) -> Vec<u8> {
        let mut b = Vec::new();
        put_request(&mut b, &r.echo);
        b.extend_from_slice(&[
            r.status as u8,
            (r.available as u8) | ((r.truncated as u8) << 1),
        ]);
        b.extend_from_slice(&(r.messages.len() as u16).to_be_bytes());
        for n in [r.membership, r.sequence, r.cursor] {
            b.extend_from_slice(&n.to_be_bytes());
        }
        for m in &r.messages {
            b.extend_from_slice(&m.id.to_be_bytes());
            b.extend_from_slice(&m.sender.source_id.to_be_bytes());
            b.extend_from_slice(&m.sender.object_id.to_be_bytes());
            put_text(&mut b, &m.character_name);
            put_text(&mut b, &m.source_name);
            put_text(&mut b, &m.text);
        }
        let mut out = app::header(1, RESULT_KIND, 16 + b.len()).unwrap();
        out.extend(b);
        out
    }
    #[test]
    fn request_matches_java_golden_and_does_not_narrow_epoch() {
        let b = encode_request(u32::MAX, &request(), 32).unwrap();
        assert_eq!(
            b.iter().map(|x| format!("{x:02x}")).collect::<String>(),
            concat!(
                "4c544d310000004cffffffff00320000000000070000000000000d05ffffffffffffffff03000000",
                "000000000000000000000000000000000000000000000000000000000000000000000000"
            )
        );
        for v in 1..32 {
            assert!(encode_request(1, &request(), v).is_err());
        }
        let mut r = request();
        r.membership = i64::MAX as u64 + 1;
        assert!(r.validate().is_err());
    }
    #[test]
    fn unicode_bounds_and_controls_match_server_policy() {
        for s in [
            "你好 👋",
            "👩\u{200d}💻",
            "e\u{301}",
            "  空白保留  ",
            &"😀".repeat(128),
            &"a".repeat(256),
        ] {
            assert!(valid_text(s));
        }
        for s in [
            "",
            "  ",
            "\u{a0}",
            "\u{200d}",
            "a\n",
            "a\u{2028}",
            "a\u{2029}",
            "a\u{202e}",
            "a\u{feff}",
            &"😀".repeat(129),
            &"a".repeat(257),
        ] {
            assert!(!valid_text(s));
        }
    }
    #[test]
    fn connection_guard_rejects_sequence_or_membership_rollback_across_navigation() {
        let mut guard = Confirmation::default();
        let mut r = reply();
        assert!(guard.accept(&r));
        let mut wrong = r.clone();
        wrong.sequence = 1;
        assert!(!guard.accept(&wrong));
        r.echo = r.next_send("test".into()).unwrap();
        r.status = Status::RateLimited;
        r.sequence = 1;
        assert!(guard.accept(&r));
        assert!(
            !guard.accept(&r),
            "a response cannot confirm a replayed send"
        );
        r.echo = r.next_read();
        r.echo.context.key.source_id = 42;
        r.echo.context.epoch = 2;
        r.echo.membership = 0;
        r.membership = 2;
        r.status = Status::Ok;
        assert!(guard.accept(&r));
        wrong = r.clone();
        wrong.membership = 1;
        assert!(!guard.accept(&wrong));
        wrong = r.clone();
        wrong.sequence = 0;
        assert!(!guard.accept(&wrong));
        r.membership = 0;
        r.available = false;
        r.status = Status::Unavailable;
        assert!(guard.accept(&r));
    }
    #[test]
    fn handshake_chat_is_optional_dependency_checked_and_requires_v32() {
        use super::super::protocol as p;
        assert_eq!(p::CAP_CHAT, 4194304);
        assert_eq!(p::RUNTIME_CAPABILITIES & p::CAP_CHAT, 0);
        let caps = 39 | p::CAP_CHAT;
        let mut h = p::Hello::current(1);
        h.supported_capabilities = caps;
        assert!(p::encode(p::Message::Hello(h)).is_ok());
        for bit in [1, 2, 4, 32] {
            assert!(p::encode(p::Message::Hello(p::Hello {
                supported_capabilities: caps & !bit,
                ..h
            }))
            .is_err());
            assert!(p::encode(p::Message::Hello(p::Hello {
                required_capabilities: caps & !bit,
                ..h
            }))
            .is_err());
        }
        for v in 1..32 {
            assert!(p::encode(p::Message::Hello(p::Hello {
                max_version: v,
                ..h
            }))
            .is_err());
            assert!(p::encode(p::Message::Ack(p::Ack {
                request_id: 1,
                version: v,
                numeric_schema: 1,
                capabilities: caps
            }))
            .is_err());
        }
    }
    #[test]
    fn bounded_full_batch_crosses_64k_but_not_chat_limit_and_stream_retains_next_frame() {
        let mut r = reply();
        r.cursor = 64;
        for i in 1..=64 {
            r.messages.push(Message {
                id: i,
                sender: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                character_name: "a".repeat(256),
                source_name: "b".repeat(256),
                text: "😀".repeat(128),
            });
        }
        let b = frame(&r);
        assert!(b.len() > 65536 && b.len() < MAX_FRAME_BYTES);
        assert_eq!(
            app::decode_response_version(&b, 32).unwrap().response,
            app::Response::Chat(r.clone())
        );
        let mut joined = b.clone();
        joined.extend_from_slice(&b);
        let mut stream = std::io::Cursor::new(joined);
        for _ in 0..2 {
            assert_eq!(
                app::read_response_version(&mut stream, 32)
                    .unwrap()
                    .response,
                app::Response::Chat(r.clone())
            );
        }
        assert_eq!(stream.position(), (b.len() * 2) as u64);
        assert!(!format!("{r:?}").contains("😀"));
        for v in 1..32 {
            let mut stream = std::io::Cursor::new(b.clone());
            assert!(app::read_response_version(&mut stream, v).is_err());
            assert_eq!(stream.position(), 16);
        }
    }
    #[test]
    fn malformed_batch_reserved_bits_utf8_count_and_truncation_fail_closed() {
        let mut r = reply();
        r.cursor = 1;
        r.messages.push(Message {
            id: 1,
            sender: r.echo.context.key,
            character_name: "角色".into(),
            source_name: "太陽神".into(),
            text: "👋".into(),
        });
        let b = frame(&r);
        for i in 0..b.len() {
            assert!(app::decode_response_version(&b[..i], 32).is_err());
        }
        for (at, value) in [
            (37, 1),
            (38, 1),
            (39, 1),
            (73, 1),
            (76, 4),
            (77, 4),
            (79, 65),
            (b.len() - 1, 255),
        ] {
            let mut bad = b.clone();
            bad[at] = value;
            assert!(
                app::decode_response_version(&bad, 32).is_err(),
                "offset {at}"
            );
        }
        let mut bad = b[..16].to_vec();
        bad[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        let mut stream = std::io::Cursor::new(bad);
        assert!(app::read_response_version(&mut stream, 32).is_err());
        assert_eq!(stream.position(), 16);
    }
    #[test]
    fn replies_confirm_exact_request_and_send_owner_and_never_auto_retry_denial() {
        let initial = reply();
        let request = initial.next_send("測試".into()).unwrap();
        let mut r = initial;
        r.echo = request.clone();
        r.sequence = 1;
        r.cursor = 1;
        r.messages.push(Message {
            id: 1,
            sender: request.context.key,
            character_name: "3333".into(),
            source_name: "太陽神".into(),
            text: request.text.clone(),
        });
        assert!(r.confirms(&request));
        assert_eq!(r.messages[0].display_name(), "3333(太陽神)");
        assert_eq!(r.next_send("再一次".into()).unwrap().sequence, 2);
        let mut wrong = r.clone();
        wrong.messages[0].sender.source_id = 42;
        assert!(!wrong.confirms(&request));
        wrong = r.clone();
        wrong.echo.context.epoch = 1;
        assert!(!wrong.confirms(&request));
        wrong = r.clone();
        wrong.sequence = 0;
        assert!(!wrong.confirms(&request));
        wrong = r.clone();
        wrong.membership = 2;
        assert!(!wrong.confirms(&request));
        r.status = Status::RateLimited;
        r.messages.clear();
        assert!(r.confirms(&request));
        assert_eq!(r.next_send("自行重試".into()).unwrap().sequence, 2);
        r.available = false;
        assert!(r.next_send("不可發送".into()).is_err());
    }
}
