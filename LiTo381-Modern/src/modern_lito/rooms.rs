//! Versioned room directory/membership, not a playable world. Passwords only
//! inhabit owned zeroizing requests; no room response carries a password.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
pub use super::participant::{MemberKind, ParticipantId};
use std::sync::Arc;
use zeroize::Zeroizing;

pub const REQUEST_KIND: u16 = 18;
pub const RESULT_KIND: u16 = 19;
pub const MAX_FRAME_BYTES: usize = 128 * 1024;
pub const MAX_ROOMS: usize = 100;
pub const MAX_MEMBERS: usize = 100;
pub const OPTION_COUNT: usize = 40;
/// Exact six room-setting categories in server BattleRules; these identifiers
/// do not assert that matching shared terrain or a playable world exists.
pub const MODE_MAPS: [&[u32]; 6] = [
    &[565, 566, 569, 570],
    &[
        504, 505, 506, 508, 509, 514, 516, 525, 526, 529, 531, 534, 551, 552, 564,
    ],
    &[513, 518, 519, 520, 521, 522, 527, 553, 554],
    &[547, 563, 573],
    &[540],
    &[510, 512, 517, 524, 528, 530, 532, 535],
];
pub fn valid_pair(mode: u8, map: u32) -> bool {
    MODE_MAPS
        .get(mode as usize)
        .is_some_and(|maps| maps.contains(&map))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Action {
    Read = 0,
    Create = 1,
    Join = 2,
    Leave = 3,
    Team = 4,
    Ready = 5,
    Configure = 6,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Common {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub action: Action,
    pub value: u8,
    pub room_id: u64,
    pub expected_revision: i64,
}
impl Common {
    pub fn read(context: SelectionContext, channel_id: u8) -> Self {
        Self {
            context,
            channel_id,
            action: Action::Read,
            value: 0,
            room_id: 0,
            expected_revision: 0,
        }
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || (if matches!(self.action, Action::Team | Action::Ready) {
                self.value > 1
            } else {
                self.value != 0
            })
            || (if matches!(self.action, Action::Read | Action::Create) {
                self.room_id != 0 || self.expected_revision != 0
            } else {
                !positive(self.room_id) || self.expected_revision < 1
            })
        {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicDraft {
    pub capacity: u16,
    pub mode: u8,
    /// Reply/public flags: 4 private, 16 allowUpper, 32 equalSides.
    pub attributes: u8,
    pub map_id: u32,
    pub name: String,
}
impl PublicDraft {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !(2..=100).contains(&self.capacity)
            || self.mode > 5
            || self.attributes & !52 != 0
            || !valid_pair(self.mode, self.map_id)
            || !text(&self.name, 64, false)
        {
            return Err(invalid());
        }
        Ok(())
    }
}
/// No Debug/Clone: a room password must not become a diagnostic or UI snapshot.
pub struct Draft {
    public: PublicDraft,
    password: Zeroizing<String>,
}
impl Draft {
    pub fn new(public: PublicDraft, password: String) -> Result<Self, ApplicationError> {
        let value = Self {
            public,
            password: Zeroizing::new(password),
        };
        value.public.validate()?;
        if !text(&value.password, 64, true)
            || (value.public.attributes & 4 != 0) != !value.password.is_empty()
        {
            return Err(invalid());
        }
        Ok(value)
    }
    pub fn public(&self) -> &PublicDraft {
        &self.public
    }
}
/// Ephemeral owner-only input. Echo/expectation deliberately exclude password.
pub struct Request {
    pub common: Common,
    draft: Option<Draft>,
    password: Option<Zeroizing<String>>,
}
impl Request {
    pub fn simple(common: Common) -> Result<Self, ApplicationError> {
        if matches!(
            common.action,
            Action::Create | Action::Configure | Action::Join
        ) {
            return Err(invalid());
        }
        let request = Self {
            common,
            draft: None,
            password: None,
        };
        request.validate()?;
        Ok(request)
    }
    pub fn read(context: SelectionContext, channel_id: u8) -> Result<Self, ApplicationError> {
        Self::simple(Common::read(context, channel_id))
    }
    pub fn draft(common: Common, draft: Draft) -> Result<Self, ApplicationError> {
        let request = Self {
            common,
            draft: Some(draft),
            password: None,
        };
        request.validate()?;
        Ok(request)
    }
    pub fn join(common: Common, password: String) -> Result<Self, ApplicationError> {
        let request = Self {
            common,
            draft: None,
            password: Some(Zeroizing::new(password)),
        };
        request.validate()?;
        Ok(request)
    }
    pub fn expectation(&self) -> Expectation {
        Expectation {
            common: self.common,
            draft: self.draft.as_ref().map(|value| value.public.clone()),
        }
    }
    pub fn capacity_allowed(&self, directory: &Reply) -> bool {
        self.draft.as_ref().is_none_or(|draft| {
            directory
                .mode_option(draft.public.mode, draft.public.map_id)
                .is_some_and(|option| draft.public.capacity <= option.maximum_players)
        })
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.common.validate()?;
        if matches!(self.common.action, Action::Create | Action::Configure) != self.draft.is_some()
            || (self.common.action == Action::Join) != self.password.is_some()
        {
            return Err(invalid());
        }
        if let Some(password) = &self.password {
            if !text(password, 64, true) {
                return Err(invalid());
            }
        }
        if let Some(draft) = &self.draft {
            draft.public.validate()?;
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expectation {
    pub common: Common,
    pub draft: Option<PublicDraft>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Ok = 0,
    Denied = 1,
    Conflict = 2,
    Full = 3,
    Unavailable = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None = 0,
    InvalidState = 1,
    LevelRestricted = 2,
    RoomNotFound = 3,
    BadPassword = 4,
    NotMember = 5,
    NotOwner = 6,
    RevisionMismatch = 7,
    Capacity = 8,
    InvalidConfiguration = 9,
    RevisionExhausted = 10,
    /// Only valid inside a v7 GM result; a capability is not GM authorization.
    GmNotAuthorized = 11,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeOption {
    pub mode: u8,
    pub team_based: bool,
    pub world_available: bool,
    pub map_id: u32,
    pub minimum_players: u16,
    pub maximum_players: u16,
    pub seconds: u32,
    pub mode_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub room_id: u64,
    pub revision: i64,
    pub settings: PublicDraft,
    pub users: u16,
    pub ready: u16,
    pub satisfied: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub kind: MemberKind,
    pub key: CharacterKey,
    pub level: i64,
    pub class: u16,
    pub sex: u8,
    pub team: u8,
    pub ready: bool,
    pub name: String,
    pub source_name: String,
}
impl Member {
    pub fn identity(&self) -> ParticipantId {
        ParticipantId::new(self.kind, self.key)
    }
    pub fn matches_character(&self, key: CharacterKey) -> bool {
        self.identity().matches_character(key)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomSnapshot {
    pub summary: Summary,
    pub owner: CharacterKey,
    pub members: Vec<Member>,
}
impl RoomSnapshot {
    pub fn member(&self, key: CharacterKey) -> Option<&Member> {
        self.members
            .iter()
            .find(|member| member.matches_character(key))
    }
    pub(crate) fn validate_members(&self, self_key: CharacterKey) -> Result<(), ApplicationError> {
        self.summary.settings.validate()?;
        if !positive(self.summary.room_id)
            || self.summary.revision < 1
            || self.members.is_empty()
            || self.members.len() > MAX_MEMBERS
            || self.members.len() != self.summary.users as usize
            || self.summary.users > self.summary.settings.capacity
            || self.members.iter().filter(|m| m.ready).count() != self.summary.ready as usize
            || self.member(self.owner).is_none()
            || self.member(self_key).is_none()
        {
            return Err(invalid());
        }
        let mut previous = None;
        for m in &self.members {
            let key = m.identity();
            if !key.valid()
                || m.level < 1
                || m.class > 6
                || m.sex > 1
                || m.team > 1
                || !text(&m.name, 256, false)
                || !text(&m.source_name, 256, false)
                || previous.is_some_and(|old| old >= key)
            {
                return Err(invalid());
            }
            previous = Some(key);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub common: Common,
    pub status: Status,
    pub reason: Reason,
    pub list_revision: i64,
    pub options: Vec<ModeOption>,
    pub rooms: Vec<Summary>,
    pub joined: Option<RoomSnapshot>,
}
impl Reply {
    pub fn mode_option(&self, mode: u8, map_id: u32) -> Option<&ModeOption> {
        self.options
            .iter()
            .find(|option| option.mode == mode && option.map_id == map_id)
    }
    pub fn validate(&self) -> Result<(), ApplicationError> {
        // A confirmed room snapshot may originate in a GM envelope. The wire
        // decoder, not this shared snapshot validator, restricts reason 11.
        self.validate_context(true)
    }
    pub(crate) fn validate_gm(&self) -> Result<(), ApplicationError> {
        self.validate_context(true)
    }
    fn validate_context(&self, gm: bool) -> Result<(), ApplicationError> {
        self.common.validate()?;
        if self.list_revision < 1
            || self.rooms.len() > MAX_ROOMS
            || self.options.len() != OPTION_COUNT
            || !match self.status {
                Status::Ok => self.reason == Reason::None,
                Status::Conflict => self.reason == Reason::RevisionMismatch,
                Status::Full => self.reason == Reason::Capacity,
                Status::Unavailable => self.reason == Reason::RevisionExhausted,
                Status::Denied => {
                    (gm && self.reason == Reason::GmNotAuthorized)
                        || matches!(
                            self.reason,
                            Reason::InvalidState
                                | Reason::LevelRestricted
                                | Reason::RoomNotFound
                                | Reason::BadPassword
                                | Reason::NotMember
                                | Reason::NotOwner
                                | Reason::InvalidConfiguration
                        )
                }
            }
        {
            return Err(invalid());
        }
        let mut prior_option = None;
        for option in &self.options {
            let key = (option.mode, option.map_id);
            if option.mode > 5
                || !valid_pair(option.mode, option.map_id)
                || option.team_based != matches!(option.mode, 1 | 2 | 3 | 5)
                || !(1..=100).contains(&option.minimum_players)
                || !(option.minimum_players.max(2)..=100).contains(&option.maximum_players)
                || !(1..=86400).contains(&option.seconds)
                || !text(&option.mode_name, 64, false)
                || prior_option.is_some_and(|old| old >= key)
            {
                return Err(invalid());
            }
            prior_option = Some(key);
        }
        let mut prior_id = 0;
        for room in &self.rooms {
            room.settings.validate()?;
            if !positive(room.room_id)
                || room.room_id <= prior_id
                || room.revision < 1
                || room.users < 1
                || room.users > room.settings.capacity
                || room.ready > room.users
                || self
                    .mode_option(room.settings.mode, room.settings.map_id)
                    .is_none_or(|option| room.settings.capacity > option.maximum_players)
            {
                return Err(invalid());
            }
            if room.satisfied
                && (room.ready != room.users
                    || room.users
                        < self
                            .mode_option(room.settings.mode, room.settings.map_id)
                            .unwrap()
                            .minimum_players)
            {
                return Err(invalid());
            }
            prior_id = room.room_id;
        }
        if let Some(room) = &self.joined {
            if self
                .rooms
                .iter()
                .find(|row| row.room_id == room.summary.room_id)
                != Some(&room.summary)
                || room.members.len() != room.summary.users as usize
                || room.members.len() > MAX_MEMBERS
                || room.members.iter().filter(|member| member.ready).count()
                    != room.summary.ready as usize
            {
                return Err(invalid());
            }
            let mut previous = None;
            for member in &room.members {
                let key = member.identity();
                if !key.valid()
                    || member.level < 1
                    || member.class > 6
                    || member.sex > 1
                    || member.team > 1
                    || !text(&member.name, 256, false)
                    || !text(&member.source_name, 256, false)
                    || previous.is_some_and(|old| old >= key)
                {
                    return Err(invalid());
                }
                previous = Some(key);
            }
            if room.member(room.owner).is_none() || room.member(self.common.context.key).is_none() {
                return Err(invalid());
            }
            let option = self
                .mode_option(room.summary.settings.mode, room.summary.settings.map_id)
                .ok_or_else(invalid)?;
            let first = room
                .members
                .iter()
                .filter(|member| member.team == 0)
                .count();
            let second = room.members.len() - first;
            let satisfied = room.summary.users >= option.minimum_players
                && room.summary.ready == room.summary.users
                && (!option.team_based
                    || (first > 0
                        && second > 0
                        && (room.summary.settings.attributes & 32 == 0 || first == second)
                        && (option.mode != 3
                            || ((15..=50).contains(&first) && (15..=50).contains(&second)))));
            if room.summary.satisfied != satisfied {
                return Err(invalid());
            }
        }
        if self.status == Status::Ok {
            let member_ok = match self.common.action {
                Action::Read => true,
                Action::Leave => self.joined.is_none(),
                Action::Create => self.joined.as_ref().is_some_and(|room| {
                    room.summary.revision == 1 && room.owner == self.common.context.key
                }),
                Action::Join => self.joined.as_ref().is_some_and(|room| {
                    room.summary.room_id == self.common.room_id
                        && Some(room.summary.revision)
                            == self.common.expected_revision.checked_add(1)
                }),
                Action::Configure => self.joined.as_ref().is_some_and(|room| {
                    room.summary.room_id == self.common.room_id
                        && room.owner == self.common.context.key
                        && (room.summary.revision == self.common.expected_revision
                            || Some(room.summary.revision)
                                == self.common.expected_revision.checked_add(1))
                }),
                Action::Team | Action::Ready => self.joined.as_ref().is_some_and(|room| {
                    room.summary.room_id == self.common.room_id
                        && (room.summary.revision == self.common.expected_revision
                            || Some(room.summary.revision)
                                == self.common.expected_revision.checked_add(1))
                        && room.member(self.common.context.key).is_some_and(|member| {
                            if self.common.action == Action::Team {
                                member.team == self.common.value
                            } else {
                                member.ready == (self.common.value != 0)
                            }
                        })
                }),
            };
            if !member_ok {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
impl Expectation {
    pub fn confirms(&self, reply: &Reply) -> bool {
        if reply.validate().is_err() || reply.common != self.common {
            return false;
        }
        if reply.status != Status::Ok {
            return true;
        }
        match self.common.action {
            Action::Read => true,
            Action::Leave => reply.joined.is_none(),
            Action::Create | Action::Configure => reply.joined.as_ref().is_some_and(|room| {
                self.draft.as_ref() == Some(&room.summary.settings)
                    && (self.common.action == Action::Create
                        || room.summary.room_id == self.common.room_id)
                    && (self.common.action != Action::Create || room.summary.revision == 1)
            }),
            Action::Join => reply.joined.as_ref().is_some_and(|room| {
                room.summary.room_id == self.common.room_id
                    && Some(room.summary.revision) == self.common.expected_revision.checked_add(1)
            }),
            Action::Team | Action::Ready => reply.joined.as_ref().is_some_and(|room| {
                room.summary.room_id == self.common.room_id
                    && (room.summary.revision == self.common.expected_revision
                        || Some(room.summary.revision)
                            == self.common.expected_revision.checked_add(1))
                    && room.member(self.common.context.key).is_some_and(|member| {
                        if self.common.action == Action::Team {
                            member.team == self.common.value
                        } else {
                            member.ready == (self.common.value != 0)
                        }
                    })
            }),
        }
    }
}
/// Enforce monotonic versions only within the same selected character/channel.
pub fn follows(prior: Option<&Reply>, next: &Reply) -> bool {
    let Some(prior) = prior else {
        return true;
    };
    if prior.common.context != next.common.context
        || prior.common.channel_id != next.common.channel_id
    {
        return false;
    }
    if next.list_revision < prior.list_revision || next.options != prior.options {
        return false;
    }
    if next.list_revision == prior.list_revision && next.rooms != prior.rooms {
        return false;
    }
    for next_room in &next.rooms {
        if let Some(old) = prior
            .rooms
            .iter()
            .find(|row| row.room_id == next_room.room_id)
        {
            if next_room.revision < old.revision
                || (next_room.revision == old.revision && next_room != old)
            {
                return false;
            }
        }
    }
    if let (Some(old), Some(new)) = (&prior.joined, &next.joined) {
        if old.summary.room_id == new.summary.room_id
            && old.summary.revision == new.summary.revision
            && old != new
        {
            return false;
        }
    }
    true
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    NotRequested,
    Unsupported,
    Ready(Arc<Reply>),
}
impl State {
    pub fn joined(&self) -> Option<&RoomSnapshot> {
        if let Self::Ready(reply) = self {
            reply.joined.as_ref()
        } else {
            None
        }
    }
}
fn positive(value: u64) -> bool {
    value > 0 && value <= i64::MAX as u64
}
fn text(value: &str, limit: usize, password: bool) -> bool {
    value.len() <= limit
        && !value.chars().any(char::is_control)
        && (password || !value.trim().is_empty())
}
fn invalid() -> ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid bounded room frame")
}
pub(crate) struct Encoded(Zeroizing<Vec<u8>>);
impl Encoded {
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.0
    }
}
pub(crate) fn encode_request(id: u32, request: &Request) -> Result<Encoded, ApplicationError> {
    request.validate()?;
    let mut body = Zeroizing::new(Vec::new());
    put_common(&mut body, request.common);
    if let Some(draft) = &request.draft {
        body.extend_from_slice(&draft.public.capacity.to_be_bytes());
        body.extend_from_slice(&[draft.public.mode, draft.public.attributes & 48]);
        body.extend_from_slice(&draft.public.map_id.to_be_bytes());
        put_text(&mut body, &draft.public.name);
        put_text(&mut body, &draft.password);
    } else if let Some(password) = &request.password {
        put_text(&mut body, password);
    }
    let mut frame = Zeroizing::new(app::header(id, REQUEST_KIND, 16 + body.len())?);
    frame.extend_from_slice(&body);
    Ok(Encoded(frame))
}
fn put_common(out: &mut Vec<u8>, value: Common) {
    out.extend_from_slice(&value.context.key.source_id.to_be_bytes());
    out.extend_from_slice(&value.context.key.object_id.to_be_bytes());
    out.extend_from_slice(&value.context.epoch.to_be_bytes());
    out.extend_from_slice(&[value.channel_id, value.action as u8, value.value, 0]);
    out.extend_from_slice(&value.room_id.to_be_bytes());
    out.extend_from_slice(&value.expected_revision.to_be_bytes());
}
fn put_text(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(&(value.len() as u16).to_be_bytes());
    out.extend_from_slice(value.as_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.position.checked_add(count).ok_or_else(invalid)?;
        let value = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(value)
    }
    fn zeros(&mut self, count: usize) -> Result<(), ApplicationError> {
        if self.take(count)?.iter().any(|v| *v != 0) {
            Err(invalid())
        } else {
            Ok(())
        }
    }
    fn u8(&mut self) -> Result<u8, ApplicationError> {
        Ok(self.take(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool, ApplicationError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(invalid()),
        }
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
    fn text(&mut self, max: usize) -> Result<String, ApplicationError> {
        let len = self.u16()? as usize;
        if len > max {
            return Err(invalid());
        }
        let value = std::str::from_utf8(self.take(len)?).map_err(|_| invalid())?;
        if !text(value, max, false) {
            return Err(invalid());
        }
        Ok(value.to_owned())
    }
    fn key(&mut self) -> Result<CharacterKey, ApplicationError> {
        Ok(CharacterKey {
            source_id: self.u32()?,
            object_id: self.u64()?,
        })
    }
    fn summary(&mut self) -> Result<Summary, ApplicationError> {
        let room_id = self.u64()?;
        let revision = self.i64()?;
        let capacity = self.u16()?;
        let mode = self.u8()?;
        let attributes = self.u8()?;
        let map_id = self.u32()?;
        let name = self.text(64)?;
        let users = self.u16()?;
        let ready = self.u16()?;
        let satisfied = self.boolean()?;
        self.zeros(3)?;
        Ok(Summary {
            room_id,
            revision,
            settings: PublicDraft {
                capacity,
                mode,
                attributes,
                map_id,
                name,
            },
            users,
            ready,
            satisfied,
        })
    }
}
pub(crate) fn decode_body_version(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    decode_context(bytes, version, false)
}
pub(crate) fn decode_gm_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if version < 7 {
        return Err(invalid());
    }
    decode_context(bytes, version, true)
}
fn decode_context(bytes: &[u8], version: u16, gm: bool) -> Result<Reply, ApplicationError> {
    if !(1..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    if bytes.len() > MAX_FRAME_BYTES - 16 {
        return Err(invalid());
    }
    let mut input = Reader { bytes, position: 0 };
    let context = SelectionContext {
        key: input.key()?,
        epoch: input.u64()?,
    };
    let channel_id = input.u8()?;
    let action = match input.u8()? {
        0 => Action::Read,
        1 => Action::Create,
        2 => Action::Join,
        3 => Action::Leave,
        4 => Action::Team,
        5 => Action::Ready,
        6 => Action::Configure,
        _ => return Err(invalid()),
    };
    let value = input.u8()?;
    input.zeros(1)?;
    let room_id = input.u64()?;
    let expected_revision = input.i64()?;
    let common = Common {
        context,
        channel_id,
        action,
        value,
        room_id,
        expected_revision,
    };
    common.validate()?;
    let status = match input.u8()? {
        0 => Status::Ok,
        1 => Status::Denied,
        2 => Status::Conflict,
        3 => Status::Full,
        4 => Status::Unavailable,
        _ => return Err(invalid()),
    };
    let reason = match input.u8()? {
        0 => Reason::None,
        1 => Reason::InvalidState,
        2 => Reason::LevelRestricted,
        3 => Reason::RoomNotFound,
        4 => Reason::BadPassword,
        5 => Reason::NotMember,
        6 => Reason::NotOwner,
        7 => Reason::RevisionMismatch,
        8 => Reason::Capacity,
        9 => Reason::InvalidConfiguration,
        10 => Reason::RevisionExhausted,
        11 if gm => Reason::GmNotAuthorized,
        _ => return Err(invalid()),
    };
    input.zeros(2)?;
    let list_revision = input.i64()?;
    let room_count = input.u16()? as usize;
    let option_count = input.u16()? as usize;
    let joined = input.boolean()?;
    input.zeros(3)?;
    if room_count > MAX_ROOMS || option_count != OPTION_COUNT {
        return Err(invalid());
    }
    let mut options = Vec::with_capacity(OPTION_COUNT);
    for _ in 0..OPTION_COUNT {
        let mode = input.u8()?;
        let team_based = input.boolean()?;
        let world_available = input.boolean()?;
        input.zeros(1)?;
        options.push(ModeOption {
            mode,
            team_based,
            world_available,
            map_id: input.u32()?,
            minimum_players: input.u16()?,
            maximum_players: if version >= 19 { input.u16()? } else { 100 },
            seconds: input.u32()?,
            mode_name: input.text(64)?,
        });
    }
    let mut rooms = Vec::with_capacity(room_count);
    for _ in 0..room_count {
        rooms.push(input.summary()?);
    }
    let joined = if joined {
        Some(input.joined(version)?)
    } else {
        None
    };
    if input.position != bytes.len() {
        return Err(invalid());
    }
    let reply = Reply {
        common,
        status,
        reason,
        list_revision,
        options,
        rooms,
        joined,
    };
    reply.validate_context(gm)?;
    Ok(reply)
}
impl Reader<'_> {
    fn joined(&mut self, version: u16) -> Result<RoomSnapshot, ApplicationError> {
        let input = self;
        let summary = input.summary()?;
        let owner = input.key()?;
        let count = input.u16()? as usize;
        input.zeros(2)?;
        if count > MAX_MEMBERS {
            return Err(invalid());
        }
        let mut members = Vec::with_capacity(count);
        for _ in 0..count {
            let kind = if version >= 7 {
                let kind = match input.u8()? {
                    0 => MemberKind::Human,
                    1 => MemberKind::ServerBot,
                    _ => return Err(invalid()),
                };
                input.zeros(3)?;
                kind
            } else {
                MemberKind::Human
            };
            let key = input.key()?;
            let level = input.i64()?;
            let class = input.u16()?;
            let sex = input.u8()?;
            let team = input.u8()?;
            let ready = input.boolean()?;
            input.zeros(3)?;
            members.push(Member {
                kind,
                key,
                level,
                class,
                sex,
                team,
                ready,
                name: input.text(256)?,
                source_name: input.text(256)?,
            });
        }
        Ok(RoomSnapshot {
            summary,
            owner,
            members,
        })
    }
}
pub(crate) fn decode_joined_version(
    bytes: &[u8],
    version: u16,
) -> Result<(RoomSnapshot, usize), ApplicationError> {
    if !(1..=super::world_protocol::CODEC_MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let mut input = Reader { bytes, position: 0 };
    let room = input.joined(version)?;
    Ok((room, input.position))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    pub(crate) fn sample(context: SelectionContext, joined: bool) -> Reply {
        let options = MODE_MAPS
            .iter()
            .enumerate()
            .flat_map(|(mode, maps)| {
                maps.iter().map(move |&map_id| ModeOption {
                    mode: mode as u8,
                    team_based: matches!(mode, 1 | 2 | 3 | 5),
                    world_available: false,
                    map_id,
                    minimum_players: 2,
                    maximum_players: 100,
                    seconds: 300,
                    mode_name: "死鬥".into(),
                })
            })
            .collect();
        let summary = Summary {
            room_id: 5_000_000_001,
            revision: 1,
            settings: PublicDraft {
                capacity: 100,
                mode: 1,
                attributes: 16,
                map_id: 504,
                name: "測試房".into(),
            },
            users: 1,
            ready: 0,
            satisfied: false,
        };
        Reply {
            common: Common::read(context, 0),
            status: Status::Ok,
            reason: Reason::None,
            list_revision: 1,
            options,
            rooms: vec![summary.clone()],
            joined: joined.then(|| RoomSnapshot {
                summary,
                owner: context.key,
                members: vec![Member {
                    kind: MemberKind::Human,
                    key: context.key,
                    level: i64::MAX,
                    class: 6,
                    sex: 1,
                    team: 0,
                    ready: false,
                    name: "勇者".into(),
                    source_name: "太陽神".into(),
                }],
            }),
        }
    }
    fn context() -> SelectionContext {
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    pub(crate) fn frame(reply: &Reply) -> Vec<u8> {
        frame_version(reply, 1)
    }
    pub(crate) fn frame_version(reply: &Reply, version: u16) -> Vec<u8> {
        fn summary(out: &mut Vec<u8>, v: &Summary) {
            out.extend_from_slice(&v.room_id.to_be_bytes());
            out.extend_from_slice(&v.revision.to_be_bytes());
            out.extend_from_slice(&v.settings.capacity.to_be_bytes());
            out.extend_from_slice(&[v.settings.mode, v.settings.attributes]);
            out.extend_from_slice(&v.settings.map_id.to_be_bytes());
            put_text(out, &v.settings.name);
            out.extend_from_slice(&v.users.to_be_bytes());
            out.extend_from_slice(&v.ready.to_be_bytes());
            out.extend_from_slice(&[v.satisfied as u8, 0, 0, 0]);
        }
        let mut out = Vec::new();
        put_common(&mut out, reply.common);
        out.extend_from_slice(&[reply.status as u8, reply.reason as u8, 0, 0]);
        out.extend_from_slice(&reply.list_revision.to_be_bytes());
        out.extend_from_slice(&(reply.rooms.len() as u16).to_be_bytes());
        out.extend_from_slice(&(reply.options.len() as u16).to_be_bytes());
        out.extend_from_slice(&[reply.joined.is_some() as u8, 0, 0, 0]);
        for o in &reply.options {
            out.extend_from_slice(&[o.mode, o.team_based as u8, o.world_available as u8, 0]);
            out.extend_from_slice(&o.map_id.to_be_bytes());
            out.extend_from_slice(&o.minimum_players.to_be_bytes());
            if version >= 19 {
                out.extend_from_slice(&o.maximum_players.to_be_bytes());
            }
            out.extend_from_slice(&o.seconds.to_be_bytes());
            put_text(&mut out, &o.mode_name);
        }
        for room in &reply.rooms {
            summary(&mut out, room);
        }
        if let Some(room) = &reply.joined {
            summary(&mut out, &room.summary);
            out.extend_from_slice(&room.owner.source_id.to_be_bytes());
            out.extend_from_slice(&room.owner.object_id.to_be_bytes());
            out.extend_from_slice(&(room.members.len() as u16).to_be_bytes());
            out.extend_from_slice(&[0, 0]);
            for m in &room.members {
                if version >= 7 {
                    out.extend([m.kind as u8, 0, 0, 0]);
                } else {
                    assert_eq!(m.kind, MemberKind::Human);
                }
                out.extend_from_slice(&m.key.source_id.to_be_bytes());
                out.extend_from_slice(&m.key.object_id.to_be_bytes());
                out.extend_from_slice(&m.level.to_be_bytes());
                out.extend_from_slice(&m.class.to_be_bytes());
                out.extend_from_slice(&[m.sex, m.team, m.ready as u8, 0, 0, 0]);
                put_text(&mut out, &m.name);
                put_text(&mut out, &m.source_name);
            }
        }
        let mut header = app::header(9, RESULT_KIND, 16 + out.len()).unwrap();
        header.extend(out);
        header
    }
    #[test]
    fn request_exact_common_ids_and_password_is_not_an_expectation_or_echo() {
        let request = Request::read(context(), 3).unwrap();
        let frame = encode_request(9, &request).unwrap();
        assert_eq!(frame.bytes().len(), 56);
        assert_eq!(&frame.bytes()[16..20], &7_u32.to_be_bytes());
        assert_eq!(&frame.bytes()[28..36], &u64::MAX.to_be_bytes());
        assert_eq!(&frame.bytes()[36..40], &[3, 0, 0, 0]);
        let mut common = request.common;
        common.action = Action::Create;
        let public = PublicDraft {
            capacity: 2,
            mode: 5,
            attributes: 20,
            map_id: 510,
            name: "房名".into(),
        };
        let request = Request::draft(
            common,
            Draft::new(public.clone(), " synthetic room secret ".into()).unwrap(),
        )
        .unwrap();
        assert!(!format!("{:?}", request.expectation()).contains("synthetic"));
        let frame = encode_request(9, &request).unwrap();
        assert_eq!(&frame.bytes()[58..60], &[5, 16]);
        assert_eq!(request.expectation().draft, Some(public));
    }
    #[test]
    fn v7_tagged_member_collisions_are_distinct_and_legacy_bytes_remain_exact() {
        let legacy = sample(context(), true);
        let old = frame(&legacy);
        for version in 1..=6 {
            assert_eq!(frame_version(&legacy, version), old);
            assert_eq!(decode_body_version(&old[16..], version).unwrap(), legacy);
        }
        let mut mixed = legacy.clone();
        let r = mixed.joined.as_mut().unwrap();
        let mut bot = r.members[0].clone();
        bot.kind = MemberKind::ServerBot;
        bot.name = "BOT01".into();
        bot.team = 1;
        r.members.push(bot);
        r.summary.users = 2;
        mixed.rooms[0] = r.summary.clone();
        mixed.validate().unwrap();
        assert_eq!(
            mixed
                .joined
                .as_ref()
                .unwrap()
                .member(context().key)
                .unwrap()
                .kind,
            MemberKind::Human
        );
        let wire = frame_version(&mixed, 7);
        assert_eq!(decode_body_version(&wire[16..], 7).unwrap(), mixed);
        assert!(decode_body_version(&wire[16..], 6).is_err());
        for end in 0..wire.len() {
            assert!(app::decode_response_version(&wire[..end], 7).is_err());
        }
        let last = mixed.joined.as_ref().unwrap().members.last().unwrap();
        let row_len = 32 + 2 + last.name.len() + 2 + last.source_name.len();
        let offset = wire.len() - row_len;
        assert_eq!(&wire[offset..offset + 4], &[1, 0, 0, 0]);
        for (p, v) in [
            (offset, 2),
            (offset + 1, 1),
            (offset + 2, 1),
            (offset + 3, 1),
        ] {
            let mut bad = wire.clone();
            bad[p] = v;
            assert!(app::decode_response_version(&bad, 7).is_err());
        }
        let mut no_human = mixed.clone();
        no_human.joined.as_mut().unwrap().members[0].kind = MemberKind::ServerBot;
        assert!(
            no_human.validate().is_err(),
            "bot numeric pair cannot satisfy owner/self"
        );
        let mut wrong_order = mixed.clone();
        wrong_order.joined.as_mut().unwrap().members.swap(0, 1);
        assert!(wrong_order.validate().is_err());
        let mut kind_flip = mixed.clone();
        kind_flip.joined.as_mut().unwrap().members[1].kind = MemberKind::Human;
        assert!(!follows(Some(&mixed), &kind_flip));
    }
    #[test]
    fn strict_text_flags_ranges_and_unsigned_epoch() {
        let public = sample(context(), false).rooms[0].settings.clone();
        for name in [
            "".to_owned(),
            "\u{a0}\u{2007}\u{202f}".into(),
            "\0".into(),
            "界".repeat(22),
        ] {
            let mut invalid = public.clone();
            invalid.name = name;
            assert!(Draft::new(invalid, String::new()).is_err());
        }
        let mut private = public.clone();
        private.attributes |= 4;
        assert!(Draft::new(private.clone(), String::new()).is_err());
        assert!(Draft::new(private.clone(), " ".into()).is_ok());
        assert!(Draft::new(private, "x".repeat(65)).is_err());
        let mut common = Common::read(context(), 0);
        common.action = Action::Join;
        common.room_id = i64::MAX as u64;
        common.expected_revision = i64::MAX;
        assert!(Request::join(common, " ".into()).is_ok());
        for mutate in 0..4 {
            let mut bad = common;
            match mutate {
                0 => bad.room_id += 1,
                1 => bad.expected_revision = 0,
                2 => bad.context.epoch = 0,
                _ => bad.value = 1,
            }
            assert!(Request::join(bad, String::new()).is_err());
        }
    }
    #[test]
    fn complete_directory_roundtrip_and_every_truncation_fails() {
        let expected = sample(context(), true);
        let bytes = frame(&expected);
        assert_eq!(
            app::decode_response(&bytes).unwrap().response,
            app::Response::Rooms(expected)
        );
        for end in 0..bytes.len() {
            assert!(app::decode_response(&bytes[..end]).is_err(), "{end}");
        }
        let mut extra = bytes;
        extra.push(0);
        assert!(app::decode_response(&extra).is_err());
    }
    #[test]
    fn v19_capacity_is_exact_and_legacy_directory_bytes_keep_the_old_layout() {
        let legacy = sample(context(), false);
        let mut bounded = legacy.clone();
        bounded.options[0].maximum_players = 48;
        let wire = frame_version(&bounded, 19);
        let old_wire = frame_version(&bounded, 18);
        assert_eq!(wire.len(), old_wire.len() + OPTION_COUNT * 2);
        assert_eq!(frame_version(&legacy, 18), old_wire);
        assert_eq!(&wire[86..88], &48_u16.to_be_bytes());
        assert_eq!(decode_body_version(&wire[16..], 19).unwrap(), bounded);
        for version in 1..=18 {
            assert_eq!(
                decode_body_version(&frame_version(&bounded, version)[16..], version).unwrap(),
                legacy
            );
        }
        assert!(decode_body_version(&wire[16..], 18).is_err());
        assert!(decode_body_version(&old_wire[16..], 19).is_err());
        for maximum in [0_u16, 1, 101, u16::MAX] {
            let mut bad = wire.clone();
            bad[86..88].copy_from_slice(&maximum.to_be_bytes());
            assert!(decode_body_version(&bad[16..], 19).is_err());
        }
        assert!(!follows(Some(&legacy), &bounded));
        let selected = bounded.rooms[0].settings.clone();
        let option = bounded
            .options
            .iter_mut()
            .find(|o| o.mode == selected.mode && o.map_id == selected.map_id)
            .unwrap();
        option.maximum_players = 48;
        assert!(
            bounded.validate().is_err(),
            "directory cannot advertise an over-limit room"
        );
        for (capacity, allowed) in [(48, true), (49, false)] {
            let mut public = selected.clone();
            public.capacity = capacity;
            let mut common = Common::read(context(), 0);
            common.action = Action::Create;
            let request =
                Request::draft(common, Draft::new(public, String::new()).unwrap()).unwrap();
            assert_eq!(request.capacity_allowed(&bounded), allowed);
        }
    }
    #[test]
    fn explicit_world_binding_flag_is_decodable_but_cannot_change_in_same_session() {
        let mut configured = sample(context(), false);
        configured.options[0].world_available = true;
        configured.validate().unwrap();
        assert!(app::decode_response(&frame(&configured)).is_ok());
        let mut changed = configured.clone();
        changed.options[0].world_available = false;
        assert!(!follows(Some(&configured), &changed));
        changed = configured.clone();
        changed.options[0].map_id = 514;
        assert!(changed.validate().is_err());
    }
    #[test]
    fn invalid_shape_identity_world_and_sorted_members_are_rejected() {
        let good = sample(context(), true);
        for case in 0..13 {
            let mut bad = good.clone();
            match case {
                0 => bad.options[0].map_id = 1,
                1 => bad.options.pop().map(|_| ()).unwrap(),
                2 => bad.options.swap(0, 1),
                3 => bad.options[0].seconds = 86401,
                4 => bad.options[0].minimum_players = 0,
                5 => bad.joined.as_mut().unwrap().owner.object_id += 1,
                6 => bad.joined.as_mut().unwrap().members[0].ready = true,
                7 => bad.joined.as_mut().unwrap().members[0].class = 7,
                8 => bad.rooms[0].settings.attributes = 1,
                9 => bad.joined.as_mut().unwrap().summary.revision += 1,
                10 => bad.status = Status::Denied,
                11 => bad.rooms[0].users = 0,
                _ => bad.list_revision = 0,
            }
            assert!(bad.validate().is_err(), "{case}");
            assert!(app::decode_response(&frame(&bad)).is_err(), "wire {case}");
        }
        let mut bytes = frame(&good);
        bytes[39] = 1;
        assert!(app::decode_response(&bytes).is_err());
    }
    #[test]
    fn maximum_count_and_text_room_frame_is_bounded_and_only_kind19_owns_its_limit() {
        let mut reply = sample(context(), true);
        reply.rooms = (1..=100)
            .map(|id| {
                let mut row = reply.rooms[0].clone();
                row.room_id = id;
                row.settings.name = "n".repeat(64);
                row
            })
            .collect();
        let room = reply.joined.as_mut().unwrap();
        room.summary = reply.rooms[0].clone();
        room.summary.users = 100;
        reply.rooms[0] = room.summary.clone();
        room.owner = CharacterKey {
            source_id: 7,
            object_id: 3333,
        };
        room.members = (3333..3433)
            .map(|id| Member {
                kind: MemberKind::Human,
                key: CharacterKey {
                    source_id: 7,
                    object_id: id,
                },
                level: i64::MAX,
                class: 6,
                sex: 1,
                team: 0,
                ready: false,
                name: "n".repeat(256),
                source_name: "s".repeat(256),
            })
            .collect();
        reply.validate().unwrap();
        let bytes = frame(&reply);
        assert!(bytes.len() > 60_000);
        assert!(bytes.len() < MAX_FRAME_BYTES);
        assert!(app::decode_response(&bytes).is_ok());
        let mut other = bytes.clone();
        other[12..14].copy_from_slice(&6_u16.to_be_bytes());
        other[4..8].copy_from_slice(&((app::MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(app::decode_response(&other).is_err());
        let mut oversized = bytes;
        oversized[4..8].copy_from_slice(&((MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        assert!(app::decode_response(&oversized).is_err());
    }
    #[test]
    fn pending_confirmation_checks_draft_and_requested_team_without_optimism() {
        let mut reply = sample(context(), true);
        reply.common.action = Action::Team;
        reply.common.value = 1;
        reply.common.room_id = reply.rooms[0].room_id;
        reply.common.expected_revision = 1;
        let request = Request::simple(reply.common).unwrap();
        assert!(!request.expectation().confirms(&reply));
        reply.joined.as_mut().unwrap().members[0].team = 1;
        assert!(request.expectation().confirms(&reply));
        reply.common.context.epoch -= 1;
        assert!(!request.expectation().confirms(&reply));
    }
    #[test]
    fn versions_cannot_regress_or_change_members_without_revision_and_options_stay_fixed() {
        let prior = sample(context(), true);
        assert!(follows(Some(&prior), &prior));
        let mut next = prior.clone();
        next.joined.as_mut().unwrap().members[0].name = "replacement".into();
        assert!(!follows(Some(&prior), &next));
        next = prior.clone();
        next.options[0].seconds += 1;
        assert!(!follows(Some(&prior), &next));
        next = prior.clone();
        next.rooms[0].revision += 1;
        assert!(!follows(Some(&prior), &next));
        next.list_revision += 1;
        next.joined.as_mut().unwrap().summary = next.rooms[0].clone();
        assert!(follows(Some(&prior), &next));
    }
    #[test]
    fn joined_readiness_matches_teams_equal_sides_and_siege_boundaries() {
        let mut reply = sample(context(), true);
        let mut second = reply.joined.as_ref().unwrap().members[0].clone();
        second.key.object_id += 1;
        second.team = 1;
        second.ready = true;
        let room = reply.joined.as_mut().unwrap();
        room.members[0].ready = true;
        room.members.push(second);
        room.summary.users = 2;
        room.summary.ready = 2;
        room.summary.satisfied = true;
        reply.rooms[0] = room.summary.clone();
        reply.validate().unwrap();
        reply.joined.as_mut().unwrap().members[1].team = 0;
        assert!(reply.validate().is_err());
        reply.joined.as_mut().unwrap().members[1].team = 1;
        reply.joined.as_mut().unwrap().summary.satisfied = false;
        reply.rooms[0].satisfied = false;
        assert!(
            reply.validate().is_err(),
            "all conditions satisfied cannot be falsely reported false"
        );
        let room = reply.joined.as_mut().unwrap();
        room.summary.settings.mode = 3;
        room.summary.settings.map_id = 547;
        room.summary.satisfied = false;
        reply.rooms[0] = room.summary.clone();
        reply.validate().unwrap();
        reply.joined.as_mut().unwrap().summary.satisfied = true;
        reply.rooms[0].satisfied = true;
        assert!(
            reply.validate().is_err(),
            "siege needs at least 15 each side"
        );
    }
}
