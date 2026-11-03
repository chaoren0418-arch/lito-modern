//! Independent v34 automatic grouping. Queue acknowledgement is not membership.
//! Scores are never client input; nested room authority includes the exact self.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
use super::rooms;
use std::{fmt, sync::Arc};
pub const VERSION: u16 = 34;
pub const REQUEST_KIND: u16 = 58;
pub const RESULT_KIND: u16 = 59;
pub const REQUEST_BYTES: usize = 56;
pub const MIN_RESULT_BYTES: usize = 60;
pub const MAX_FRAME_BYTES: usize = rooms::MAX_FRAME_BYTES + 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Criteria {
    pub mode: u8,
    pub map_id: u32,
    pub players: u16,
}
impl Criteria {
    pub fn bounds(mode: u8) -> Option<(u16, u16)> {
        match mode {
            0 => Some((2, 10)),
            1 | 2 | 5 => Some((2, 100)),
            3 => Some((30, 100)),
            4 => Some((1, 60)),
            _ => None,
        }
    }
    pub fn validate(self) -> Result<(), ApplicationError> {
        let (min, max) = Self::bounds(self.mode).ok_or_else(invalid)?;
        if !(min..=max).contains(&self.players)
            || self.map_id != 0 && !rooms::valid_pair(self.mode, self.map_id)
            || matches!(self.mode, 1 | 2 | 3 | 5) && self.players % 2 != 0
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn accepts(self, option: &rooms::ModeOption) -> bool {
        self.validate().is_ok()
            && option.world_available
            && option.mode == self.mode
            && (self.map_id == 0 || self.map_id == option.map_id)
            && self.players >= option.minimum_players
            && self.players.max(2) <= option.maximum_players
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read,
    Start,
    Cancel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok,
    Busy,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Idle,
    Waiting,
    Matched,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub ticket_id: u64,
    pub criteria: Option<Criteria>,
}
impl Request {
    pub fn read(context: SelectionContext, channel_id: u8, ticket_id: u64) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            ticket_id,
            criteria: None,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.ticket_id > i64::MAX as u64
            || (self.action == Action::Start) != (self.criteria.is_some())
            || self.action == Action::Start && self.ticket_id != 0
            || self.action == Action::Cancel && self.ticket_id == 0
        {
            return Err(invalid());
        }
        if let Some(c) = self.criteria {
            c.validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub phase: Phase,
    pub ticket_id: u64,
    pub criteria: Option<Criteria>,
    pub rooms: rooms::Reply,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub status: Status,
    pub snapshot: Option<Snapshot>,
}
macro_rules! redacted {($($t:ty),*)=>{$(impl fmt::Debug for $t {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str(concat!(stringify!($t),"[redacted]"))}
})*};}
redacted!(Request, Snapshot, Reply);
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if (self.status == Status::Ok) != self.snapshot.is_some() {
            return Err(invalid());
        }
        let Some(s) = &self.snapshot else {
            return if self.echo.action == Action::Start {
                Ok(())
            } else {
                Err(invalid())
            };
        };
        s.rooms.validate()?;
        if s.rooms.reason == rooms::Reason::GmNotAuthorized
            || s.rooms.common != rooms::Common::read(self.echo.context, self.echo.channel_id)
            || s.ticket_id > i64::MAX as u64
            || if s.phase == Phase::Idle {
                s.ticket_id != 0 || s.criteria.is_some()
            } else {
                s.ticket_id == 0 || s.criteria.is_none()
            }
            || s.phase == Phase::Waiting && s.rooms.joined.is_some()
            || s.phase == Phase::Matched && s.rooms.joined.is_none()
        {
            return Err(invalid());
        }
        if let Some(c) = s.criteria {
            c.validate()?;
        }
        if s.rooms.status == rooms::Status::Conflict {
            if self.echo.action == Action::Start
                || self.echo.ticket_id == 0
                || self.echo.ticket_id == s.ticket_id
            {
                return Err(invalid());
            }
        } else if self.echo.ticket_id != 0
            && s.phase != Phase::Idle
            && self.echo.ticket_id != s.ticket_id
        {
            return Err(invalid());
        }
        if s.rooms.status == rooms::Status::Ok
            && match self.echo.action {
                Action::Start => s.phase != Phase::Waiting || s.criteria != self.echo.criteria,
                Action::Read => self.echo.ticket_id != 0 && s.phase == Phase::Idle,
                Action::Cancel => s.phase == Phase::Waiting,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn confirms(&self, r: &Request) -> bool {
        self.echo == *r && self.validate().is_ok()
    }
}

/// UI retains only confirmed server state; pending operations are owned by the
/// existing room/navigation mailbox barrier, not a guessed queue membership.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Unsupported,
    NotRequested,
    Ready(Arc<Reply>),
}
impl State {
    pub fn supported(&self) -> bool {
        !matches!(self, Self::Unsupported)
    }
    pub fn snapshot(&self) -> Option<&Snapshot> {
        match self {
            Self::Ready(r) => r.snapshot.as_ref(),
            _ => None,
        }
    }
    pub fn waiting(&self) -> Option<&Snapshot> {
        self.snapshot().filter(|s| s.phase == Phase::Waiting)
    }
}
#[derive(Default)]
pub(crate) struct Confirmation {
    high_ticket: u64,
    active: Option<(SelectionContext, u8, u64, Criteria, Phase)>,
}
impl Confirmation {
    pub(crate) fn retire(&mut self) {
        self.active = None;
    }
    pub(crate) fn permits(&self, r: &Request) -> bool {
        r.validate().is_ok()
            && match r.action {
                Action::Start => self.active.is_none(),
                Action::Read if r.ticket_id == 0 => true,
                _ => self
                    .active
                    .is_some_and(|a| (a.0, a.1, a.2) == (r.context, r.channel_id, r.ticket_id)),
            }
    }
    pub(crate) fn accept(&mut self, r: &Reply) -> bool {
        if !self.permits(&r.echo) || r.validate().is_err() {
            return false;
        }
        let Some(s) = &r.snapshot else {
            return true;
        };
        if s.phase == Phase::Idle {
            self.retire();
            return true;
        }
        let next = (
            r.echo.context,
            r.echo.channel_id,
            s.ticket_id,
            s.criteria.unwrap(),
            s.phase,
        );
        if let Some(old) = self.active {
            if next.2 == old.2 {
                if (next.0, next.1, next.3) != (old.0, old.1, old.3)
                    || old.4 == Phase::Matched && next.4 != Phase::Matched
                {
                    return false;
                }
            } else if s.rooms.status != rooms::Status::Conflict || next.2 <= self.high_ticket {
                return false;
            }
        } else if next.2 <= self.high_ticket {
            return false;
        }
        self.high_ticket = self.high_ticket.max(next.2);
        self.active = Some(next);
        true
    }
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid matchmaking frame")
}
fn put_criteria(out: &mut Vec<u8>, criteria: Option<Criteria>) {
    if let Some(c) = criteria {
        out.extend_from_slice(&[c.mode, 0]);
        out.extend_from_slice(&c.players.to_be_bytes());
        out.extend_from_slice(&c.map_id.to_be_bytes());
    } else {
        out.extend_from_slice(&[0; 8]);
    }
}
fn put_request(out: &mut Vec<u8>, r: &Request) {
    out.extend_from_slice(&r.context.key.source_id.to_be_bytes());
    out.extend_from_slice(&r.context.key.object_id.to_be_bytes());
    out.extend_from_slice(&r.context.epoch.to_be_bytes());
    out.extend_from_slice(&[r.channel_id, r.action as u8, 0, 0]);
    out.extend_from_slice(&r.ticket_id.to_be_bytes());
    put_criteria(out, r.criteria);
}
pub fn encode_request(id: u32, r: &Request, version: u16) -> Result<Vec<u8>, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    r.validate()?;
    let mut out = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
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
    fn criteria(&mut self, present: bool) -> Result<Option<Criteria>, ApplicationError> {
        if !present {
            self.zero(8)?;
            return Ok(None);
        }
        let mode = self.u8()?;
        self.zero(1)?;
        let c = Criteria {
            mode,
            players: self.u16()?,
            map_id: self.u32()?,
        };
        c.validate()?;
        Ok(Some(c))
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
        let action = match self.u8()? {
            0 => Action::Read,
            1 => Action::Start,
            2 => Action::Cancel,
            _ => return Err(invalid()),
        };
        self.zero(2)?;
        let ticket_id = self.u64()?;
        let criteria = self.criteria(action == Action::Start)?;
        let r = Request {
            context,
            channel_id,
            action,
            ticket_id,
            criteria,
        };
        r.validate()?;
        Ok(r)
    }
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(VERSION..=super::protocol::MAX_VERSION).contains(&version)
        || !(MIN_RESULT_BYTES - 16..=MAX_FRAME_BYTES - 16).contains(&bytes.len())
    {
        return Err(invalid());
    }
    let mut input = Reader { bytes, position: 0 };
    let echo = input.request()?;
    let status = match input.u8()? {
        0 => Status::Ok,
        1 => Status::Busy,
        2 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    input.zero(3)?;
    let snapshot = if status == Status::Ok {
        let phase = match input.u8()? {
            0 => Phase::Idle,
            1 => Phase::Waiting,
            2 => Phase::Matched,
            _ => return Err(invalid()),
        };
        input.zero(3)?;
        let ticket_id = input.u64()?;
        let criteria = input.criteria(phase != Phase::Idle)?;
        let rooms = rooms::decode_body_version(&bytes[input.position..], version)?;
        input.position = bytes.len();
        Some(Snapshot {
            phase,
            ticket_id,
            criteria,
            rooms,
        })
    } else {
        None
    };
    if input.position != bytes.len() {
        return Err(invalid());
    }
    let r = Reply {
        echo,
        status,
        snapshot,
    };
    r.validate()?;
    Ok(r)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn matchmaking_is_explicit_v34_with_all_source_room_world_dependencies() {
        use super::super::protocol as p;
        assert_eq!(p::CAP_MATCHMAKING, 33554432);
        assert_eq!(
            p::Hello::current(1).supported_capabilities & p::CAP_MATCHMAKING,
            0
        );
        let mut h = p::Hello::current(1);
        h.supported_capabilities |= p::CAP_MATCHMAKING;
        assert!(p::encode(p::Message::Hello(h)).is_ok());
        for v in 1..34 {
            let mut old = h;
            old.max_version = v;
            assert!(p::encode(p::Message::Hello(old)).is_err());
            assert!(p::encode(p::Message::Ack(p::Ack {
                request_id: 1,
                version: v,
                numeric_schema: 1,
                capabilities: h.supported_capabilities
            }))
            .is_err());
        }
        for bit in [
            p::CAP_EXPLICIT_I64_STATS,
            p::CAP_SOURCE_SCOPED_IDENTITIES,
            p::CAP_TOURNAMENT_NAVIGATION,
            p::CAP_CHARACTER_DATA,
            p::CAP_EQUIPMENT,
            p::CAP_ROOMS,
            p::CAP_WORLD,
            p::CAP_MATCH_DESCRIPTOR,
        ] {
            let mut bad = h;
            bad.supported_capabilities &= !bit;
            bad.required_capabilities &= !bit;
            assert!(
                p::encode(p::Message::Hello(bad)).is_err(),
                "dependency {bit}"
            );
        }
    }
    pub(crate) fn context() -> SelectionContext {
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    pub(crate) fn start() -> Request {
        Request {
            context: context(),
            channel_id: 0,
            action: Action::Start,
            ticket_id: 0,
            criteria: Some(Criteria {
                mode: 1,
                map_id: 514,
                players: 2,
            }),
        }
    }
    pub(crate) fn queued() -> Reply {
        Reply {
            echo: start(),
            status: Status::Ok,
            snapshot: Some(Snapshot {
                phase: Phase::Waiting,
                ticket_id: 1,
                criteria: start().criteria,
                rooms: rooms::tests::sample(context(), false),
            }),
        }
    }
    pub(crate) fn frame(r: &Reply) -> Vec<u8> {
        let mut out = app::header(1, RESULT_KIND, MIN_RESULT_BYTES).unwrap();
        put_request(&mut out, &r.echo);
        out.extend_from_slice(&[r.status as u8, 0, 0, 0]);
        if let Some(s) = &r.snapshot {
            out.extend_from_slice(&[s.phase as u8, 0, 0, 0]);
            out.extend_from_slice(&s.ticket_id.to_be_bytes());
            put_criteria(&mut out, s.criteria);
            out.extend_from_slice(&rooms::tests::frame_version(&s.rooms, 34)[16..]);
        }
        let length = out.len() as u32;
        out[4..8].copy_from_slice(&length.to_be_bytes());
        out
    }
    #[test]
    fn java_golden_request_and_legacy_header_rejection() {
        let mut r = start();
        r.channel_id = 3;
        let bytes = encode_request(u32::MAX, &r, 34).unwrap();
        assert_eq!(bytes.iter().map(|b|format!("{b:02x}")).collect::<String>(),"4c544d3100000038ffffffff003a0000000000070000000000000d05ffffffffffffffff0301000000000000000000000100000200000202");
        let reply = frame(&queued());
        for v in 1..34 {
            assert!(encode_request(1, &r, v).is_err());
            let mut reader = std::io::Cursor::new(&reply);
            assert!(app::read_response_version(&mut reader, v).is_err());
            assert_eq!(reader.position(), 16);
        }
    }
    #[test]
    fn all_truncations_reserved_bytes_and_tail_are_rejected() {
        let r = queued();
        let bytes = frame(&r);
        assert_eq!(decode_body(&bytes[16..], 34).unwrap(), r);
        for n in 0..bytes.len() {
            assert!(app::decode_response_version(&bytes[..n], 34).is_err());
        }
        for (at, value) in [
            (37, 3),
            (38, 1),
            (39, 1),
            (40, 128),
            (48, 6),
            (49, 1),
            (50, 255),
            (51, 0),
            (56, 3),
            (57, 1),
            (60, 3),
            (61, 1),
            (64, 128),
            (73, 1),
            (80, 1),
            (100, 1),
        ] {
            let mut b = bytes.clone();
            b[at] = value;
            assert!(app::decode_response_version(&b, 34).is_err(), "offset {at}");
        }
        let mut extra = bytes.clone();
        extra.push(0);
        let n = extra.len() as u32;
        extra[4..8].copy_from_slice(&n.to_be_bytes());
        assert!(app::decode_response_version(&extra, 34).is_err());
        let mut large = bytes[..16].to_vec();
        large[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(app::read_response_version(&mut large.as_slice(), 34).is_err());
    }
    #[test]
    fn backend_failures_do_not_invent_a_queue_and_only_start_can_fail_backend() {
        for status in [Status::Busy, Status::Unavailable] {
            let r = Reply {
                echo: start(),
                status,
                snapshot: None,
            };
            let b = frame(&r);
            assert_eq!(b.len(), 60);
            assert_eq!(decode_body(&b[16..], 34).unwrap(), r);
            let mut wrong = r.clone();
            wrong.echo = Request::read(context(), 0, 0);
            assert!(wrong.validate().is_err());
        }
    }
    #[test]
    fn confirmed_admission_is_not_an_optimistic_room_or_implicit_leave() {
        let r = queued();
        let mut c = Confirmation::default();
        assert!(c.accept(&r));
        assert!(!c.permits(&start()));
        let mut matched = r.clone();
        matched.echo = Request::read(context(), 0, 1);
        matched.snapshot.as_mut().unwrap().phase = Phase::Matched;
        assert!(matched.validate().is_err());
        matched.snapshot.as_mut().unwrap().rooms = rooms::tests::sample(context(), true);
        assert!(c.accept(&matched));
        assert_eq!(decode_body(&frame(&matched)[16..], 34).unwrap(), matched);
        let mut back = r.clone();
        back.echo = matched.echo;
        assert!(!c.accept(&back));
        matched.echo.action = Action::Cancel;
        assert!(c.accept(&matched)); // racing cancel does not leave
        c.retire();
        assert!(!c.accept(&matched));
        assert!(!c.accept(&r)); // retired id cannot resurrect
        let mut new = r;
        new.snapshot.as_mut().unwrap().ticket_id = 2;
        assert!(c.accept(&new));
        let mut old = Request::read(context(), 0, 1);
        old.action = Action::Cancel;
        assert!(!c.permits(&old));
    }
    #[test]
    fn criteria_match_all_six_server_modes_and_reject_unqualified_capacity() {
        for mode in 0..=5 {
            let (min, max) = Criteria::bounds(mode).unwrap();
            for map_id in std::iter::once(0).chain(rooms::MODE_MAPS[mode as usize].iter().copied())
            {
                for players in 0..=101 {
                    let c = Criteria {
                        mode,
                        map_id,
                        players,
                    };
                    assert_eq!(
                        c.validate().is_ok(),
                        (min..=max).contains(&players)
                            && (!matches!(mode, 1 | 2 | 3 | 5) || players % 2 == 0)
                    );
                }
            }
        }
        let c = start().criteria.unwrap();
        let mut option = rooms::tests::sample(context(), false)
            .mode_option(1, 514)
            .unwrap()
            .clone();
        assert!(!c.accepts(&option));
        option.world_available = true;
        assert!(c.accepts(&option));
        option.maximum_players = 1;
        assert!(!c.accepts(&option));
    }
    #[test]
    fn nested_room_scope_self_reason_and_request_counter_are_strict() {
        let mut r = queued();
        r.snapshot.as_mut().unwrap().rooms.reason = rooms::Reason::GmNotAuthorized;
        assert!(r.validate().is_err());
        for id in [0, i64::MAX as u64 + 1, u64::MAX] {
            let mut cancel = Request::read(context(), 0, id);
            cancel.action = Action::Cancel;
            assert!(cancel.validate().is_err());
        }
        let mut r = queued();
        r.echo.action = Action::Cancel;
        r.echo.ticket_id = 1;
        r.echo.criteria = None;
        assert!(r.validate().is_err());
    }
}
