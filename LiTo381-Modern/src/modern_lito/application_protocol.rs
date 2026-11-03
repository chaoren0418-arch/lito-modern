//! Modern application frames, exclusively inside an already verified TLS stream.
//! The original HELLO/ACK decoder retains its independent 32-byte limit.
//!
//! All integers are big endian. The 16-byte LTM1 header is unchanged, but only
//! kinds 3..17 belong here. DATA_RESULT allows 256 KiB and EQUIPMENT_RESULT
//! 320 KiB; other application frames retain their independent 64 KiB limit. Credentials
//! and their encoded frame are owned, non-Debug and zeroized on every exit. This
//! does not promise erasure of copies owned by a TLS implementation or caller UI.

use super::character_data::{self, CharacterDataReply};
use super::equipment::{self, Reply as EquipmentReply};
use super::{combat_protocol, record_protocol, rooms, spell_protocol, world_protocol};
use std::fmt;
use std::io::{self, Read};
use zeroize::Zeroizing;

pub const MAX_FRAME_BYTES: usize = 64 * 1024;
pub const MAX_SOURCES: usize = 64;
pub const MAX_CHARACTERS: usize = 64;
const HEADER_BYTES: usize = 16;
const MAX_ACCOUNT_BYTES: usize = 64;
const MAX_PASSWORD_BYTES: usize = 256;
const MAX_NAME_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    MalformedFrame,
    InvalidCredentials,
    InvalidIdentity,
    UnexpectedMessage,
    RequestIdMismatch,
    IdentityMismatch,
    Io,
}

#[derive(Debug)]
pub struct ApplicationError {
    pub kind: ErrorKind,
    detail: &'static str,
}
impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "modern application {:?}: {}", self.kind, self.detail)
    }
}
impl std::error::Error for ApplicationError {}
pub(crate) fn error(kind: ErrorKind, detail: &'static str) -> ApplicationError {
    ApplicationError { kind, detail }
}
fn malformed(detail: &'static str) -> ApplicationError {
    error(ErrorKind::MalformedFrame, detail)
}
fn io_error(cause: io::Error) -> ApplicationError {
    // Never format arbitrary data/backend errors into diagnostics.
    if cause.kind() == io::ErrorKind::UnexpectedEof {
        malformed("truncated application frame")
    } else {
        error(ErrorKind::Io, "application transport read failed")
    }
}

/// Consumes caller-owned strings. No Clone or Debug implementation can expose
/// the password; invalid input is also wrapped before validation and zeroized.
pub struct Credentials {
    account: Zeroizing<String>,
    password: Zeroizing<String>,
}
impl Credentials {
    pub fn new(account: String, password: String) -> Result<Self, ApplicationError> {
        let credentials = Self {
            account: Zeroizing::new(account),
            password: Zeroizing::new(password),
        };
        if !valid_text(&credentials.account, MAX_ACCOUNT_BYTES)
            || credentials.password.is_empty()
            || credentials.password.len() > MAX_PASSWORD_BYTES
            || credentials.password.chars().any(char::is_control)
        {
            return Err(error(
                ErrorKind::InvalidCredentials,
                "invalid credential shape",
            ));
        }
        // Password whitespace is significant and deliberately allowed.
        Ok(credentials)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub id: u32,
    pub name: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Principal {
    pub account: String,
    /// Server-owned, strictly increasing IDs and exact-Unicode unique names.
    pub sources: Vec<Source>,
}
impl Principal {
    pub fn authorizes_source(&self, source_id: u32) -> bool {
        self.sources.iter().any(|source| source.id == source_id)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CharacterKey {
    pub source_id: u32,
    pub object_id: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterSummary {
    pub key: CharacterKey,
    pub name: String,
    /// Profession/type, NOT a classId sprite-resource address.
    pub class: u16,
    pub sex: u8,
    /// Explicit i64 roster field, not proof of a complete i64 combat/stat model.
    pub level: i64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterList {
    pub source_id: u32,
    /// Opaque nonzero u64 selection generation, including values > i64::MAX.
    pub epoch: u64,
    pub characters: Vec<CharacterSummary>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthenticationFailure {
    InvalidCredentials,
    BackendUnavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterFailure {
    Denied,
    BackendUnavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectionContext {
    pub key: CharacterKey,
    pub epoch: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationAction {
    SelectCharacter,
    EnterChannel,
    ReturnCharacters,
    LeaveLobby,
}
impl NavigationAction {
    pub(crate) fn kind(self) -> u16 {
        match self {
            Self::SelectCharacter => 9,
            Self::EnterChannel => 11,
            Self::ReturnCharacters => 12,
            Self::LeaveLobby => 13,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationStatus {
    Ok,
    Denied,
    BackendUnavailable,
    Busy,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationPhase {
    Characters,
    Channels,
    Lobby,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChannelSummary {
    pub id: u8,
    pub name: String,
    pub population: u32,
    pub capacity: u32,
    pub minimum_level: i64,
    pub maximum_level: i64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationSnapshot {
    pub phase: NavigationPhase,
    pub channel_id: Option<u8>,
    pub selected: Option<CharacterSummary>,
    pub channels: Vec<ChannelSummary>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationReply {
    pub action: NavigationAction,
    pub status: NavigationStatus,
    pub context: SelectionContext,
    pub snapshot: NavigationSnapshot,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Response {
    Authentication(Result<Principal, AuthenticationFailure>),
    Characters {
        source_id: u32,
        epoch: u64,
        result: Result<CharacterList, CharacterFailure>,
    },
    Pong,
    Navigation(NavigationReply),
    CharacterData(CharacterDataReply),
    Equipment(EquipmentReply),
    Rooms(rooms::Reply),
    Matchmaking(super::matchmaking_protocol::Reply),
    World(world_protocol::Reply),
    BattleData(super::battle_read::UnvalidatedReply),
    BattleOperation(super::battle_operation::Reply),
    BattleWorld(super::battle_world::Reply),
    VipWorld(super::battle_vip_world::Reply),
    VipEquipment(super::battle_vip_equipment::Reply),
    BattleRevival(super::battle_revival::Reply),
    BattleEquipment(super::battle_equipment::Reply),
    GreaterHaste(super::battle_greater_haste::CandidateReply),
    TargetHaste(super::battle_target_haste::CandidateReply),
    Slow(super::battle_slow::CandidateReply),
    PlayerHealing(super::battle_player_healing::Reply),
    Combat(combat_protocol::Reply),
    Skills(spell_protocol::SkillsReply),
    Cast(spell_protocol::CastReply),
    Record(record_protocol::Reply),
    GmBots(super::gm_protocol::Reply),
    Chat(super::chat_protocol::Reply),
    FilteredChat(super::filtered_chat_protocol::Reply),
    DirectChat(super::direct_chat_protocol::Reply),
    ModeratedDirect(super::direct_chat_protocol::Reply),
    Shop(super::shop_protocol::Reply),
    Item(super::item_protocol::Reply),
    Ammo(super::ammo_protocol::Reply),
    Npc(super::npc_protocol::Reply),
    NpcSpell(super::npc_spell_protocol::Reply),
    NpcDisintegrate(super::npc_disintegrate_protocol::Reply),
    NpcFireStorm(super::npc_fire_storm_protocol::Reply),
    Siege(super::siege_protocol::Reply),
    Disintegrate(super::disintegrate_cast_protocol::Reply),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResponseFrame {
    pub request_id: u32,
    pub response: Response,
}

fn valid_text(value: &str, maximum: usize) -> bool {
    !value.is_empty()
        && value.len() <= maximum
        && !value.trim().is_empty()
        && !value.chars().any(char::is_control)
}
pub(crate) fn valid_source(source_id: u32) -> bool {
    source_id != 0 && source_id <= i32::MAX as u32
}

/// No Debug implementation: even diagnostic formatting of an AUTH frame would
/// reveal the password. Only the TLS owner may borrow its bytes for transmission.
pub(crate) struct SecretFrame(Zeroizing<Vec<u8>>);
impl SecretFrame {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
pub(crate) fn header(request_id: u32, kind: u16, size: usize) -> Result<Vec<u8>, ApplicationError> {
    if request_id == 0 || !(HEADER_BYTES..=frame_limit(kind)).contains(&size) {
        return Err(malformed("invalid request ID or application frame length"));
    }
    let mut frame = Vec::with_capacity(size);
    frame.extend_from_slice(b"LTM1");
    frame.extend_from_slice(&(size as u32).to_be_bytes());
    frame.extend_from_slice(&request_id.to_be_bytes());
    frame.extend_from_slice(&kind.to_be_bytes());
    frame.extend_from_slice(&[0, 0]);
    Ok(frame)
}

fn frame_limit(kind: u16) -> usize {
    frame_limit_version(kind, super::protocol::MAX_VERSION)
}
fn frame_limit_format(kind: u16, format: super::battle_payload::Format) -> usize {
    let old = frame_limit_version(kind, format.version());
    if kind == super::battle_read::RESULT_KIND && old != 0 {
        format.limit(old)
    } else {
        old
    }
}
fn frame_limit_version(kind: u16, version: u16) -> usize {
    if matches!(
        kind,
        super::battle_read::REQUEST_KIND | super::battle_read::RESULT_KIND
    ) {
        return if version < super::battle_read::VERSION {
            0
        } else if kind == super::battle_read::REQUEST_KIND {
            super::battle_read::REQUEST_BYTES
        } else {
            super::battle_read::MAX_FRAME_BYTES
        };
    }
    if matches!(
        kind,
        super::matchmaking_protocol::REQUEST_KIND | super::matchmaking_protocol::RESULT_KIND
    ) {
        return if version < super::matchmaking_protocol::VERSION {
            0
        } else if kind == super::matchmaking_protocol::REQUEST_KIND {
            super::matchmaking_protocol::REQUEST_BYTES
        } else {
            super::matchmaking_protocol::MAX_FRAME_BYTES
        };
    }
    if matches!(
        kind,
        super::direct_chat_protocol::MODERATED_REQUEST_KIND
            | super::direct_chat_protocol::MODERATED_RESULT_KIND
    ) {
        return if version < super::direct_chat_protocol::MODERATED_VERSION {
            0
        } else if kind == super::direct_chat_protocol::MODERATED_REQUEST_KIND {
            super::direct_chat_protocol::MAX_REQUEST_BYTES
        } else {
            super::direct_chat_protocol::MAX_FRAME_BYTES
        };
    }
    if matches!(
        kind,
        super::filtered_chat_protocol::REQUEST_KIND | super::filtered_chat_protocol::RESULT_KIND
    ) {
        return if version < super::filtered_chat_protocol::VERSION {
            0
        } else if kind == super::filtered_chat_protocol::REQUEST_KIND {
            super::filtered_chat_protocol::MAX_REQUEST_BYTES
        } else {
            super::filtered_chat_protocol::MAX_FRAME_BYTES
        };
    }
    if matches!(
        kind,
        super::direct_chat_protocol::REQUEST_KIND | super::direct_chat_protocol::RESULT_KIND
    ) {
        return if version < super::direct_chat_protocol::VERSION {
            0
        } else if kind == super::direct_chat_protocol::REQUEST_KIND {
            super::direct_chat_protocol::MAX_REQUEST_BYTES
        } else {
            super::direct_chat_protocol::MAX_FRAME_BYTES
        };
    }
    if matches!(
        kind,
        super::chat_protocol::REQUEST_KIND | super::chat_protocol::RESULT_KIND
    ) {
        return if version < super::chat_protocol::VERSION {
            0
        } else if kind == super::chat_protocol::REQUEST_KIND {
            super::chat_protocol::MAX_REQUEST_BYTES
        } else {
            super::chat_protocol::MAX_FRAME_BYTES
        };
    }
    if kind == super::npc_fire_storm_protocol::RESULT_KIND {
        return if version >= super::npc_fire_storm_protocol::VERSION {
            super::npc_protocol::frame_limit(version)
        } else {
            0
        };
    }
    if kind == super::npc_disintegrate_protocol::RESULT_KIND {
        return if version >= super::npc_disintegrate_protocol::VERSION {
            super::npc_protocol::frame_limit(version)
        } else {
            0
        };
    }
    if kind == super::disintegrate_cast_protocol::RESULT_KIND {
        return if version >= super::disintegrate_cast_protocol::VERSION {
            combat_protocol::frame_limit(version)
        } else {
            0
        };
    }
    if kind == super::siege_protocol::RESULT_KIND {
        return if version >= super::siege_protocol::VERSION {
            combat_protocol::frame_limit(version)
        } else {
            0
        };
    }
    if kind == super::npc_protocol::RESULT_KIND || kind == super::npc_spell_protocol::RESULT_KIND {
        // Reject each kind from its header before allocating/reading a body.
        let minimum = if kind == super::npc_spell_protocol::RESULT_KIND {
            super::npc_spell_protocol::VERSION
        } else {
            super::npc_protocol::VERSION
        };
        return if version >= minimum {
            super::npc_protocol::frame_limit(version)
        } else {
            0
        };
    }
    if matches!(kind, 23 | 27 | 35 | 37) {
        return combat_protocol::frame_limit(version);
    }
    if kind == super::shop_protocol::RESULT_KIND {
        return super::shop_protocol::MAX_FRAME_BYTES;
    }
    if kind == super::item_protocol::RESULT_KIND {
        return super::item_protocol::MAX_FRAME_BYTES;
    }
    if kind == super::gm_protocol::RESULT_KIND {
        return super::gm_protocol::MAX_FRAME_BYTES;
    }
    if kind == spell_protocol::CAST_RESULT_KIND {
        spell_protocol::MAX_FRAME_BYTES
    } else if kind == combat_protocol::RESULT_KIND {
        combat_protocol::MAX_FRAME_BYTES
    } else if kind == world_protocol::RESULT_KIND {
        world_protocol::MAX_FRAME_BYTES
    } else if kind == rooms::RESULT_KIND {
        rooms::MAX_FRAME_BYTES
    } else if kind == equipment::RESULT_KIND {
        equipment::MAX_FRAME_BYTES
    } else if kind == character_data::RESULT_KIND {
        character_data::MAX_FRAME_BYTES
    } else {
        MAX_FRAME_BYTES
    }
}
fn append_text(frame: &mut Vec<u8>, text: &str) {
    frame.extend_from_slice(&(text.len() as u16).to_be_bytes());
    frame.extend_from_slice(text.as_bytes());
}
pub(crate) fn encode_auth(
    request_id: u32,
    credentials: &Credentials,
) -> Result<SecretFrame, ApplicationError> {
    let size = HEADER_BYTES + 4 + credentials.account.len() + credentials.password.len();
    let mut frame = Zeroizing::new(header(request_id, 3, size)?);
    append_text(&mut frame, &credentials.account);
    append_text(&mut frame, &credentials.password);
    Ok(SecretFrame(frame))
}
pub(crate) fn encode_characters(
    request_id: u32,
    source_id: u32,
    epoch: u64,
) -> Result<Vec<u8>, ApplicationError> {
    if !valid_source(source_id) || epoch == 0 {
        return Err(error(
            ErrorKind::InvalidIdentity,
            "invalid source or selection epoch",
        ));
    }
    let mut frame = header(request_id, 5, 28)?;
    frame.extend_from_slice(&source_id.to_be_bytes());
    frame.extend_from_slice(&epoch.to_be_bytes());
    Ok(frame)
}
pub(crate) fn encode_ping(request_id: u32) -> Result<Vec<u8>, ApplicationError> {
    header(request_id, 7, 16)
}
pub(crate) fn valid_context(context: SelectionContext) -> bool {
    valid_source(context.key.source_id)
        && context.key.object_id != 0
        && context.key.object_id <= i64::MAX as u64
        && context.epoch != 0
}
pub(crate) fn encode_navigation(
    request_id: u32,
    action: NavigationAction,
    context: SelectionContext,
    channel: Option<u8>,
) -> Result<Vec<u8>, ApplicationError> {
    if !valid_context(context)
        || match action {
            NavigationAction::EnterChannel => !matches!(channel, Some(0..=3)),
            _ => channel.is_some(),
        }
    {
        return Err(error(
            ErrorKind::InvalidIdentity,
            "invalid navigation context or channel",
        ));
    }
    let mut frame = header(
        request_id,
        action.kind(),
        if channel.is_some() { 40 } else { 36 },
    )?;
    frame.extend_from_slice(&context.key.source_id.to_be_bytes());
    frame.extend_from_slice(&context.key.object_id.to_be_bytes());
    frame.extend_from_slice(&context.epoch.to_be_bytes());
    if let Some(id) = channel {
        frame.extend_from_slice(&[id, 0, 0, 0]);
    }
    Ok(frame)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or_else(|| malformed("offset overflow"))?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| malformed("truncated payload"))?;
        self.offset = end;
        Ok(bytes)
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
    fn text(&mut self, maximum: usize, empty_allowed: bool) -> Result<String, ApplicationError> {
        let length = self.u16()? as usize;
        if length > maximum {
            return Err(malformed("text exceeds byte limit"));
        }
        let text =
            std::str::from_utf8(self.take(length)?).map_err(|_| malformed("invalid UTF-8"))?;
        if !(empty_allowed && text.is_empty()) && !valid_text(text, maximum) {
            return Err(malformed("invalid text shape"));
        }
        Ok(text.to_owned())
    }
}

fn validate_header(bytes: &[u8]) -> Result<(usize, u32, u16), ApplicationError> {
    validate_header_format(
        bytes,
        super::battle_payload::Format::Legacy(super::protocol::MAX_VERSION),
    )
}
fn validate_header_format(
    bytes: &[u8],
    format: super::battle_payload::Format,
) -> Result<(usize, u32, u16), ApplicationError> {
    let mut cursor = Cursor { bytes, offset: 0 };
    if cursor.take(4)? != b"LTM1" {
        return Err(malformed("not an LTM1 application frame"));
    }
    let length = cursor.u32()? as usize;
    let request_id = cursor.u32()?;
    let kind = cursor.u16()?;
    // Preserve the absolute-header check before the negotiated-version check,
    // including legacy diagnostics. Only selected battle DATA gets extra space.
    let absolute = if kind == super::battle_read::RESULT_KIND {
        format.limit(frame_limit(kind))
    } else {
        frame_limit(kind)
    };
    if !(HEADER_BYTES..=absolute).contains(&length) || request_id == 0 || cursor.u16()? != 0 {
        return Err(malformed("invalid application header"));
    }
    let valid = match kind {
        4 => length >= 22,
        6 => length >= 32,
        8 => length == 16,
        10 => length >= 44,
        character_data::RESULT_KIND => length >= 40,
        equipment::RESULT_KIND => length >= 104,
        rooms::RESULT_KIND => length >= 76,
        super::matchmaking_protocol::RESULT_KIND => {
            length >= super::matchmaking_protocol::MIN_RESULT_BYTES
        }
        world_protocol::RESULT_KIND => length >= 124,
        super::battle_read::RESULT_KIND => length >= super::battle_read::MIN_RESULT_BYTES,
        combat_protocol::RESULT_KIND => length >= 76,
        spell_protocol::SKILLS_RESULT_KIND => length >= 40,
        spell_protocol::CAST_RESULT_KIND => length >= 76,
        record_protocol::RESULT_KIND => length == 64,
        super::gm_protocol::RESULT_KIND => length >= 72,
        super::chat_protocol::RESULT_KIND => length >= 104,
        super::filtered_chat_protocol::RESULT_KIND => {
            length >= super::filtered_chat_protocol::MIN_RESULT_BYTES
        }
        super::direct_chat_protocol::RESULT_KIND => {
            length >= super::direct_chat_protocol::MIN_RESULT_BYTES
        }
        super::direct_chat_protocol::MODERATED_RESULT_KIND => {
            length >= super::direct_chat_protocol::MIN_RESULT_BYTES
        }
        super::shop_protocol::RESULT_KIND => length >= 128,
        super::item_protocol::RESULT_KIND => length >= 76,
        super::ammo_protocol::RESULT_KIND => length >= 76,
        super::npc_protocol::RESULT_KIND => length >= 92,
        super::npc_spell_protocol::RESULT_KIND => length >= 100,
        super::npc_disintegrate_protocol::RESULT_KIND => length >= 100,
        super::npc_fire_storm_protocol::RESULT_KIND => length >= 84,
        super::siege_protocol::RESULT_KIND => length >= 76,
        super::disintegrate_cast_protocol::RESULT_KIND => length >= 92,
        _ => {
            return Err(error(
                ErrorKind::UnexpectedMessage,
                "not an application response kind",
            ));
        }
    };
    if !valid {
        return Err(malformed("incorrect response length"));
    }
    Ok((length, request_id, kind))
}

pub fn decode_response(bytes: &[u8]) -> Result<ResponseFrame, ApplicationError> {
    decode_response_version(bytes, super::protocol::VERSION)
}

/// Decode according to the verified HELLO/ACK selection, never by trying one
/// gear shape then falling back to another. Legacy callers remain strict v1.
pub fn decode_response_version(
    bytes: &[u8],
    version: u16,
) -> Result<ResponseFrame, ApplicationError> {
    decode_response_format(bytes, super::battle_payload::Format::Legacy(version))
}
pub(crate) fn decode_response_selected(
    bytes: &[u8],
    peer: super::protocol::extensions::Selected,
) -> Result<ResponseFrame, ApplicationError> {
    decode_response_format(bytes, super::battle_payload::Format::Selected(peer))
}
fn decode_response_format(
    bytes: &[u8],
    format: super::battle_payload::Format,
) -> Result<ResponseFrame, ApplicationError> {
    let version = format.version();
    if !(super::protocol::VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(malformed("unsupported negotiated application version"));
    }
    let (length, request_id, kind) = validate_header_format(bytes, format)?;
    if bytes.len() > equipment::MAX_FRAME_BYTES
        && kind != super::battle_read::RESULT_KIND
        && kind != super::disintegrate_cast_protocol::RESULT_KIND
        && !(version >= 31 && matches!(kind, 39 | 41 | 47 | 49))
    {
        return Err(malformed("oversize application frame"));
    }
    if length > frame_limit_format(kind, format) {
        return Err(malformed("response exceeds negotiated version budget"));
    }
    if bytes.len() != length {
        return Err(malformed("truncated frame or trailing bytes"));
    }
    let mut cursor = Cursor {
        bytes,
        offset: HEADER_BYTES,
    };
    let response = match kind {
        super::battle_read::RESULT_KIND => {
            let result = match format {
                super::battle_payload::Format::Legacy(v) => {
                    super::battle_read::decode_body(&bytes[HEADER_BYTES..], v)?
                }
                super::battle_payload::Format::Selected(p) => {
                    super::battle_read::decode_body_selected(&bytes[HEADER_BYTES..], p)?
                }
            };
            cursor.offset = bytes.len();
            Response::BattleData(result)
        }
        4 => {
            let status = cursor.u16()?;
            let count = cursor.u16()? as usize;
            if count > MAX_SOURCES {
                return Err(malformed("too many sources"));
            }
            let account = cursor.text(MAX_ACCOUNT_BYTES, status != 0)?;
            if status == 0 {
                if count == 0 {
                    return Err(malformed("authenticated principal has no sources"));
                }
                let mut sources: Vec<Source> = Vec::with_capacity(count);
                let mut previous = 0;
                for _ in 0..count {
                    let id = cursor.u32()?;
                    let name = cursor.text(MAX_NAME_BYTES, false)?;
                    if !valid_source(id)
                        || id <= previous
                        || sources.iter().any(|source| source.name == name)
                    {
                        return Err(malformed("source list is duplicate, unsorted or invalid"));
                    }
                    previous = id;
                    sources.push(Source { id, name });
                }
                Response::Authentication(Ok(Principal { account, sources }))
            } else {
                if count != 0 || !account.is_empty() {
                    return Err(malformed("failed AUTH carries identity data"));
                }
                Response::Authentication(Err(match status {
                    1 => AuthenticationFailure::InvalidCredentials,
                    2 => AuthenticationFailure::BackendUnavailable,
                    _ => return Err(malformed("unknown authentication status")),
                }))
            }
        }
        6 => {
            let source_id = cursor.u32()?;
            let epoch = cursor.u64()?;
            let status = cursor.u16()?;
            let count = cursor.u16()? as usize;
            if !valid_source(source_id) || epoch == 0 || count > MAX_CHARACTERS {
                return Err(malformed("invalid character source, epoch or count"));
            }
            let result = if status == 0 {
                let mut characters = Vec::with_capacity(count);
                let mut previous = 0;
                for _ in 0..count {
                    let object_id = cursor.u64()?;
                    let name = cursor.text(MAX_NAME_BYTES, false)?;
                    let class = cursor.u16()?;
                    let sex = cursor.u8()?;
                    let reserved = cursor.u8()?;
                    let level = cursor.u64()? as i64;
                    if object_id == 0
                        || object_id > i64::MAX as u64
                        || object_id <= previous
                        || sex > 1
                        || reserved != 0
                        || level < 1
                    {
                        return Err(malformed("invalid, duplicate or unsorted character row"));
                    }
                    previous = object_id;
                    characters.push(CharacterSummary {
                        key: CharacterKey {
                            source_id,
                            object_id,
                        },
                        name,
                        class,
                        sex,
                        level,
                    });
                }
                Ok(CharacterList {
                    source_id,
                    epoch,
                    characters,
                })
            } else {
                if count != 0 {
                    return Err(malformed("failed character result carries rows"));
                }
                Err(match status {
                    1 => CharacterFailure::Denied,
                    2 => CharacterFailure::BackendUnavailable,
                    _ => return Err(malformed("unknown character status")),
                })
            };
            Response::Characters {
                source_id,
                epoch,
                result,
            }
        }
        super::siege_protocol::RESULT_KIND => {
            let result = super::siege_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Siege(result)
        }
        8 => Response::Pong,
        10 => Response::Navigation(decode_navigation(&mut cursor)?),
        character_data::RESULT_KIND => {
            let result = character_data::decode_body(&bytes[HEADER_BYTES..])?;
            cursor.offset = bytes.len();
            Response::CharacterData(result)
        }
        equipment::RESULT_KIND => {
            let result = equipment::decode_body(request_id, &bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Equipment(result)
        }
        rooms::RESULT_KIND => {
            let result = rooms::decode_body_version(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Rooms(result)
        }
        super::matchmaking_protocol::RESULT_KIND => {
            let result = super::matchmaking_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Matchmaking(result)
        }
        world_protocol::RESULT_KIND => {
            let result = world_protocol::decode_body_version(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::World(result)
        }
        combat_protocol::RESULT_KIND => {
            let result = combat_protocol::decode_body_version(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Combat(result)
        }
        spell_protocol::SKILLS_RESULT_KIND => {
            let result = spell_protocol::decode_skills(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Skills(result)
        }
        spell_protocol::CAST_RESULT_KIND => {
            let result = spell_protocol::decode_cast(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Cast(result)
        }
        super::gm_protocol::RESULT_KIND => {
            let result = super::gm_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::GmBots(result)
        }
        record_protocol::RESULT_KIND => {
            let result = record_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Record(result)
        }
        super::shop_protocol::RESULT_KIND => {
            let result = super::shop_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Shop(result)
        }
        super::filtered_chat_protocol::RESULT_KIND => {
            let result =
                super::filtered_chat_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::FilteredChat(result)
        }
        super::direct_chat_protocol::RESULT_KIND => {
            let result = super::direct_chat_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::DirectChat(result)
        }
        super::direct_chat_protocol::MODERATED_RESULT_KIND => {
            let result = super::direct_chat_protocol::decode_moderated_body(
                &bytes[HEADER_BYTES..],
                version,
            )?;
            cursor.offset = bytes.len();
            Response::ModeratedDirect(result)
        }
        super::chat_protocol::RESULT_KIND => {
            let result = super::chat_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Chat(result)
        }
        super::npc_protocol::RESULT_KIND => {
            let result = super::npc_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Npc(result)
        }
        super::npc_spell_protocol::RESULT_KIND => {
            let result = super::npc_spell_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::NpcSpell(result)
        }
        super::npc_fire_storm_protocol::RESULT_KIND => {
            let (result, used) =
                super::npc_fire_storm_protocol::decode_reply(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = HEADER_BYTES + used;
            Response::NpcFireStorm(result)
        }
        super::npc_disintegrate_protocol::RESULT_KIND => {
            let (result, used) =
                super::npc_disintegrate_protocol::decode_reply(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = HEADER_BYTES + used;
            Response::NpcDisintegrate(result)
        }
        super::disintegrate_cast_protocol::RESULT_KIND => {
            let result =
                super::disintegrate_cast_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Disintegrate(result)
        }
        super::ammo_protocol::RESULT_KIND => {
            let result = super::ammo_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Ammo(result)
        }
        super::item_protocol::RESULT_KIND => {
            let result = super::item_protocol::decode_body(&bytes[HEADER_BYTES..], version)?;
            cursor.offset = bytes.len();
            Response::Item(result)
        }
        _ => unreachable!("validated response kind"),
    };
    if cursor.offset != length {
        return Err(malformed("trailing response payload"));
    }
    Ok(ResponseFrame {
        request_id,
        response,
    })
}

fn decode_navigation(cursor: &mut Cursor<'_>) -> Result<NavigationReply, ApplicationError> {
    let action = match cursor.u8()? {
        9 => NavigationAction::SelectCharacter,
        11 => NavigationAction::EnterChannel,
        12 => NavigationAction::ReturnCharacters,
        13 => NavigationAction::LeaveLobby,
        _ => return Err(malformed("unknown navigation action")),
    };
    let status = match cursor.u8()? {
        0 => NavigationStatus::Ok,
        1 => NavigationStatus::Denied,
        2 => NavigationStatus::BackendUnavailable,
        3 => NavigationStatus::Busy,
        _ => return Err(malformed("unknown navigation status")),
    };
    let phase = match cursor.u8()? {
        0 => NavigationPhase::Characters,
        1 => NavigationPhase::Channels,
        2 => NavigationPhase::Lobby,
        _ => return Err(malformed("unknown navigation phase")),
    };
    let channel_id = match cursor.u8()? {
        id @ 0..=3 => Some(id),
        255 => None,
        _ => return Err(malformed("invalid selected channel")),
    };
    let context = SelectionContext {
        key: CharacterKey {
            source_id: cursor.u32()?,
            object_id: cursor.u64()?,
        },
        epoch: cursor.u64()?,
    };
    let present = cursor.u8()?;
    let count = cursor.u8()?;
    if !valid_context(context) || present > 1 || count > 4 || cursor.u16()? != 0 {
        return Err(malformed(
            "invalid navigation identity, count or reserved bits",
        ));
    }
    let selected = if present == 1 {
        let object_id = cursor.u64()?;
        let name = cursor.text(MAX_NAME_BYTES, false)?;
        let class = cursor.u16()?;
        let sex = cursor.u8()?;
        let reserved = cursor.u8()?;
        let level = cursor.u64()? as i64;
        if object_id != context.key.object_id || sex > 1 || reserved != 0 || level < 1 {
            return Err(malformed("invalid selected character row"));
        }
        Some(CharacterSummary {
            key: context.key,
            name,
            class,
            sex,
            level,
        })
    } else {
        None
    };
    let mut channels: Vec<ChannelSummary> = Vec::with_capacity(count as usize);
    for index in 0..count {
        let id = cursor.u8()?;
        if id != index || cursor.take(3)? != [0, 0, 0] {
            return Err(malformed("invalid channel order or reserved bits"));
        }
        let population = cursor.u32()?;
        let capacity = cursor.u32()?;
        let minimum_level = cursor.u64()? as i64;
        let maximum_level = cursor.u64()? as i64;
        let name = cursor.text(MAX_NAME_BYTES, false)?;
        if capacity == 0
            || capacity > 65535
            || population > capacity
            || minimum_level < 1
            || maximum_level < minimum_level
            || channels.iter().any(|row| row.name == name)
        {
            return Err(malformed("invalid channel snapshot"));
        }
        channels.push(ChannelSummary {
            id,
            name,
            population,
            capacity,
            minimum_level,
            maximum_level,
        });
    }
    let shape_valid = match phase {
        NavigationPhase::Characters => {
            selected.is_none() && channels.is_empty() && channel_id.is_none()
        }
        NavigationPhase::Channels => {
            selected.is_some() && channels.len() == 4 && channel_id.is_none()
        }
        NavigationPhase::Lobby => selected.is_some() && channels.len() == 4 && channel_id.is_some(),
    };
    let action_valid = match action {
        NavigationAction::SelectCharacter => {
            phase
                == if status == NavigationStatus::Ok {
                    NavigationPhase::Channels
                } else {
                    NavigationPhase::Characters
                }
        }
        NavigationAction::EnterChannel => {
            phase != NavigationPhase::Characters
                && (status != NavigationStatus::Ok || phase == NavigationPhase::Lobby)
        }
        NavigationAction::ReturnCharacters => {
            (status == NavigationStatus::Ok && phase == NavigationPhase::Characters)
                || (status == NavigationStatus::Busy
                    && matches!(phase, NavigationPhase::Channels | NavigationPhase::Lobby))
        }
        NavigationAction::LeaveLobby => {
            status == NavigationStatus::Ok && phase == NavigationPhase::Channels
        }
    };
    if !shape_valid || !action_valid {
        return Err(malformed("inconsistent navigation state"));
    }
    Ok(NavigationReply {
        action,
        status,
        context,
        snapshot: NavigationSnapshot {
            phase,
            channel_id,
            selected,
            channels,
        },
    })
}

/// Reads exactly one response, retaining coalesced subsequent frames. Length,
/// response kind and fixed minimums are checked before a peer-sized allocation.
// Legacy stream fixtures deliberately exercise v1; live TLS always supplies
// its negotiated version and selected capabilities to read_response_with_battle.
#[cfg(test)]
pub(crate) fn read_response(reader: &mut impl Read) -> Result<ResponseFrame, ApplicationError> {
    read_response_version(reader, super::protocol::VERSION)
}

#[cfg(test)]
pub(crate) fn read_response_version(
    reader: &mut impl Read,
    version: u16,
) -> Result<ResponseFrame, ApplicationError> {
    read_response_with_battle(reader, version, false)
}
pub(crate) fn read_response_with_battle(
    reader: &mut impl Read,
    version: u16,
    battle_negotiated: bool,
) -> Result<ResponseFrame, ApplicationError> {
    read_response_format(
        reader,
        super::battle_payload::Format::Legacy(version),
        battle_negotiated,
    )
}
pub(crate) fn read_response_selected(
    reader: &mut impl Read,
    peer: super::protocol::extensions::Selected,
    battle_requested: bool,
) -> Result<ResponseFrame, ApplicationError> {
    read_response_format(
        reader,
        super::battle_payload::Format::Selected(peer),
        battle_requested && super::battle_read::negotiated(&peer.base()),
    )
}
fn read_response_format(
    reader: &mut impl Read,
    format: super::battle_payload::Format,
    battle_negotiated: bool,
) -> Result<ResponseFrame, ApplicationError> {
    let version = format.version();
    if !(super::protocol::VERSION..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(malformed("unsupported negotiated application version"));
    }
    let mut header = [0; HEADER_BYTES];
    reader.read_exact(&mut header[..4]).map_err(io_error)?;
    if &header[..4] != b"LTM1" {
        return Err(malformed("not an LTM1 application frame"));
    }
    reader.read_exact(&mut header[4..]).map_err(io_error)?;
    let (length, _, kind) = validate_header_format(&header, format)?;
    if kind == super::battle_read::RESULT_KIND && !battle_negotiated {
        return Err(malformed("canonical battle data was not negotiated"));
    }
    if length > frame_limit_format(kind, format) {
        return Err(malformed("response exceeds negotiated version budget"));
    }
    let mut frame = vec![0; length];
    frame[..HEADER_BYTES].copy_from_slice(&header);
    reader
        .read_exact(&mut frame[HEADER_BYTES..])
        .map_err(io_error)?;
    decode_response_format(&frame, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(request_id: u32, kind: u16, body: &[u8]) -> Vec<u8> {
        let mut bytes = header(request_id, kind, HEADER_BYTES + body.len()).unwrap();
        bytes.extend_from_slice(body);
        bytes
    }
    fn auth_body(sources: &[(u32, &str)]) -> Vec<u8> {
        let mut body = vec![0, 0];
        body.extend_from_slice(&(sources.len() as u16).to_be_bytes());
        append_text(&mut body, "canonical");
        for (id, name) in sources {
            body.extend_from_slice(&id.to_be_bytes());
            append_text(&mut body, name);
        }
        body
    }
    fn character_body(ids: &[u64]) -> Vec<u8> {
        let mut body = 2_u32.to_be_bytes().to_vec();
        body.extend_from_slice(&u64::MAX.to_be_bytes());
        body.extend_from_slice(&0_u16.to_be_bytes());
        body.extend_from_slice(&(ids.len() as u16).to_be_bytes());
        for id in ids {
            body.extend_from_slice(&id.to_be_bytes());
            append_text(&mut body, "勇者");
            body.extend_from_slice(&5_u16.to_be_bytes());
            body.extend_from_slice(&[1, 0]);
            body.extend_from_slice(&i64::MAX.to_be_bytes());
        }
        body
    }

    fn navigation_context() -> SelectionContext {
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        }
    }
    fn hex_bytes(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    fn navigation_body() -> Vec<u8> {
        let context = navigation_context();
        let mut body = vec![9, 0, 1, 255];
        body.extend_from_slice(&context.key.source_id.to_be_bytes());
        body.extend_from_slice(&context.key.object_id.to_be_bytes());
        body.extend_from_slice(&context.epoch.to_be_bytes());
        body.extend_from_slice(&[1, 4, 0, 0]);
        body.extend_from_slice(&3333u64.to_be_bytes());
        append_text(&mut body, "勇者");
        body.extend_from_slice(&[0, 3, 1, 0]);
        body.extend_from_slice(&4_000_000_000i64.to_be_bytes());
        for id in 0u8..4 {
            body.extend_from_slice(&[id, 0, 0, 0]);
            body.extend_from_slice(&(u32::from(id) + 1).to_be_bytes());
            body.extend_from_slice(&1000u32.to_be_bytes());
            body.extend_from_slice(&(80 - i64::from(id) * 10).to_be_bytes());
            body.extend_from_slice(
                &(if id == 0 {
                    i64::MAX
                } else {
                    89 - i64::from(id) * 10
                })
                .to_be_bytes(),
            );
            append_text(&mut body, &format!("channel{id}"));
        }
        body
    }

    #[test]
    fn navigation_golden_selection_and_return_match_java_with_full_epoch() {
        let selection =
            hex_bytes("4c544d3100000024fedcba9800090000000000070000000000000d05ffffffffffffffff");
        assert_eq!(
            encode_navigation(
                0xfedcba98,
                NavigationAction::SelectCharacter,
                navigation_context(),
                None
            )
            .unwrap(),
            selection
        );
        let returned = hex_bytes(
            "4c544d310000002cfedcba98000a00000c0000ff000000070000000000000d05ffffffffffffffff00000000",
        );
        let Response::Navigation(reply) = decode_response(&returned).unwrap().response else {
            panic!("navigation");
        };
        assert_eq!(reply.context, navigation_context());
        assert_eq!(reply.action, NavigationAction::ReturnCharacters);
        assert_eq!(reply.status, NavigationStatus::Ok);
        assert_eq!(reply.snapshot.phase, NavigationPhase::Characters);
        assert!(reply.snapshot.selected.is_none());
        let mut combined = std::io::Cursor::new([returned.clone(), returned.clone()].concat());
        assert_eq!(
            read_response(&mut combined).unwrap().response,
            Response::Navigation(reply.clone())
        );
        assert_eq!(combined.position(), returned.len() as u64);
        assert_eq!(
            read_response(&mut combined).unwrap().response,
            Response::Navigation(reply)
        );
    }

    #[test]
    fn only_character_data_can_cross_the_old_frame_limit_and_stream_reads_remain_exact() {
        let body = character_data::tests::fixture_body_named(512, &"x".repeat(256));
        let frame = response(17, character_data::RESULT_KIND, &body);
        assert!(frame.len() > MAX_FRAME_BYTES && frame.len() < character_data::MAX_FRAME_BYTES);
        let Response::CharacterData(reply) = decode_response(&frame).unwrap().response else {
            panic!("data");
        };
        assert_eq!(reply.result.unwrap().items.len(), 512);
        let pong = response(18, 8, &[]);
        let mut stream = std::io::Cursor::new([frame.clone(), pong].concat());
        assert!(matches!(
            read_response(&mut stream).unwrap().response,
            Response::CharacterData(_)
        ));
        assert_eq!(stream.position(), frame.len() as u64);
        assert_eq!(read_response(&mut stream).unwrap().response, Response::Pong);
        for kind in [4u16, 6, 8, 10, 14, 16] {
            let mut changed = frame.clone();
            changed[12..14].copy_from_slice(&kind.to_be_bytes());
            assert!(decode_response(&changed).is_err());
            let mut stream = std::io::Cursor::new(changed);
            assert!(read_response(&mut stream).is_err());
            assert_eq!(
                stream.position(),
                16,
                "oversize non-data rejected before body allocation/read"
            );
        }
        let mut oversized = frame[..16].to_vec();
        oversized[4..8]
            .copy_from_slice(&((character_data::MAX_FRAME_BYTES + 1) as u32).to_be_bytes());
        let mut stream = std::io::Cursor::new(oversized);
        assert!(read_response(&mut stream).is_err());
        assert_eq!(stream.position(), 16);
    }

    #[test]
    fn java_complete_character_data_golden_preserves_every_field_and_flags() {
        // Shared verbatim with ModernCharacterDataProtocolTest.DATA_HEX.
        let frame = hex_bytes(concat!(
            "4c544d3100000133fedcba98000f0000000000070000000000000d05ffffffffffffffff000000000000000000000d05",
            "00014e0003010000000000ee6b28000000000000000001000000000000000200000000000000030000000000000004ff",
            "fffffffffffffb0000000000000006000000000000000700000000000000080000000000000009000000000000000a00",
            "0000000000000b000000000000000cfffffffffffffff3000000000000000e000000000000000f000000000000001000",
            "000000000000110000000000000012000000000000001300010000000000000000006300000000000000647fffffffff",
            "ffffff8000000000000000000000000000000500000000000000ff000000000000000600000000000000070000000000",
            "00000800000000000000090005000000022431"
        ));
        assert_eq!(frame.len(), 307);
        let Response::CharacterData(reply) = decode_response(&frame).unwrap().response else {
            panic!("data");
        };
        assert_eq!(reply.context, navigation_context());
        let data = reply.result.unwrap();
        let stats = &data.stats;
        assert_eq!(
            (
                data.character.name.as_str(),
                data.character.class,
                data.character.sex,
                data.character.level
            ),
            ("N", 3, 1, 4_000_000_000)
        );
        assert_eq!(
            [
                stats.max_hp,
                stats.max_mp,
                stats.saved_hp,
                stats.saved_mp,
                stats.armor_class,
                stats.strength,
                stats.dexterity,
                stats.constitution,
                stats.intelligence,
                stats.wisdom,
                stats.charisma,
                stats.experience,
                stats.lawful,
                stats.original_strength,
                stats.original_dexterity,
                stats.original_constitution,
                stats.original_intelligence,
                stats.original_wisdom,
                stats.food225
            ],
            [1, 2, 3, 4, -5, 6, 7, 8, 9, 10, 11, 12, -13, 14, 15, 16, 17, 18, 19]
        );
        assert_eq!(
            data.items,
            vec![character_data::InventoryItem {
                object_id: 99,
                catalog_id: 100,
                quantity: i64::MAX,
                enchantment: i64::MIN,
                durability: 5,
                blessing: 255,
                charge_count: 6,
                inventory_graphic: 7,
                ground_graphic: 8,
                unit_weight: 9,
                kind: character_data::ItemKind::Weapon,
                equipped: true,
                identified: false,
                sealed: true,
                name_token: "$1".into(),
            }]
        );
        let denied = hex_bytes(
            "4c544d3100000028fedcba98000f0000000000070000000000000d05ffffffffffffffff01000000",
        );
        let Response::CharacterData(reply) = decode_response(&denied).unwrap().response else {
            panic!("denied");
        };
        assert_eq!(reply.context, navigation_context());
        assert_eq!(
            reply.result,
            Err(character_data::CharacterDataFailure::Denied)
        );
    }
    #[test]
    fn navigation_selected_i64_and_all_four_server_channel_rows_are_retained() {
        let frame = response(0xfedcba98, 10, &navigation_body());
        let Response::Navigation(reply) = decode_response(&frame).unwrap().response else {
            panic!("navigation");
        };
        assert_eq!(
            reply.snapshot.selected.as_ref().unwrap().level,
            4_000_000_000
        );
        assert_eq!(reply.snapshot.selected.as_ref().unwrap().name, "勇者");
        assert_eq!(reply.context, navigation_context());
        assert_eq!(reply.snapshot.channels.len(), 4);
        assert_eq!(reply.snapshot.channels[0].maximum_level, i64::MAX);
        assert_eq!(reply.snapshot.channels[3].name, "channel3");
        let mut lobby = frame.clone();
        lobby[16] = 11;
        lobby[18] = 2;
        lobby[19] = 3;
        let Response::Navigation(reply) = decode_response(&lobby).unwrap().response else {
            panic!("lobby");
        };
        assert_eq!(reply.snapshot.channel_id, Some(3));
        for status in 1..=3 {
            let mut denied = frame.clone();
            denied[16] = 11;
            denied[17] = status;
            assert!(decode_response(&denied).is_ok());
        }
    }
    #[test]
    fn return_busy_keeps_a_selected_page_but_is_never_a_successful_character_page() {
        for (phase, channel) in [(1, 255), (2, 3)] {
            let mut body = navigation_body();
            body[..4].copy_from_slice(&[12, 3, phase, channel]);
            let frame = response(0xfedcba98, 10, &body);
            let Response::Navigation(reply) = decode_response(&frame).unwrap().response else {
                panic!("busy return");
            };
            assert_eq!(reply.action, NavigationAction::ReturnCharacters);
            assert_eq!(reply.status, NavigationStatus::Busy);
            assert_eq!(reply.context, navigation_context());
            assert_eq!(
                reply.snapshot.selected.as_ref().unwrap().level,
                4_000_000_000
            );
            assert_eq!(reply.snapshot.channels.len(), 4);
            for status in [0, 1, 2] {
                let mut invalid = frame.clone();
                invalid[17] = status;
                assert!(
                    decode_response(&invalid).is_err(),
                    "phase {phase}, status {status}"
                );
            }
            for length in 0..frame.len() {
                assert!(decode_response(&frame[..length]).is_err());
            }
        }
        // Even a correctly shaped Characters page cannot acknowledge BACK Busy.
        let returned = hex_bytes(
            "4c544d310000002cfedcba98000a00000c0000ff000000070000000000000d05ffffffffffffffff00000000",
        );
        assert!(decode_response(&returned).is_ok());
        for status in 1..=3 {
            let mut invalid = returned.clone();
            invalid[17] = status;
            assert!(decode_response(&invalid).is_err());
        }
    }

    #[test]
    fn lobby_leave_wire_is_exact_and_only_a_successful_channel_page_is_accepted() {
        let request =
            hex_bytes("4c544d3100000024fedcba98000d0000000000070000000000000d05ffffffffffffffff");
        assert_eq!(
            encode_navigation(
                0xfedcba98,
                NavigationAction::LeaveLobby,
                navigation_context(),
                None
            )
            .unwrap(),
            request
        );
        assert!(encode_navigation(
            1,
            NavigationAction::LeaveLobby,
            navigation_context(),
            Some(0)
        )
        .is_err());
        let mut body = navigation_body();
        body[0] = 13;
        let frame = response(0xfedcba98, 10, &body);
        let Response::Navigation(reply) = decode_response(&frame).unwrap().response else {
            panic!("leave");
        };
        assert_eq!(reply.action, NavigationAction::LeaveLobby);
        assert_eq!(reply.status, NavigationStatus::Ok);
        assert_eq!(reply.context, navigation_context());
        assert_eq!(reply.snapshot.phase, NavigationPhase::Channels);
        assert_eq!(reply.snapshot.channel_id, None);
        assert_eq!(reply.snapshot.selected.as_ref().unwrap().name, "勇者");
        assert_eq!(
            reply.snapshot.selected.as_ref().unwrap().level,
            4_000_000_000
        );
        assert_eq!(reply.snapshot.channels.len(), 4);
        for (offset, value) in [
            (17, 1),
            (17, 2),
            (17, 3),
            (18, 0),
            (18, 2),
            (19, 0),
            (40, 0),
            (41, 3),
        ] {
            let mut invalid = frame.clone();
            invalid[offset] = value;
            assert!(
                decode_response(&invalid).is_err(),
                "offset {offset}, value {value}"
            );
        }
        for length in 0..frame.len() {
            assert!(decode_response(&frame[..length]).is_err());
        }
    }
    #[test]
    fn navigation_rejects_bad_shape_ids_counts_reserved_channels_and_truncation() {
        let original = response(0xfedcba98, 10, &navigation_body());
        for (offset, value) in [
            (16, 10),
            (17, 4),
            (18, 3),
            (19, 4),
            (40, 2),
            (41, 5),
            (42, 1),
            (43, 1),
            (51, 6),
            (62, 2),
            (63, 1),
            (64, 0x80),
            (72, 1),
            (73, 1),
            (76, 0xff),
            (80, 1),
            (84, 0xff),
        ] {
            let mut changed = original.clone();
            changed[offset] = value;
            assert!(decode_response(&changed).is_err(), "offset {offset}");
        }
        for range in [20..24, 24..32, 32..40, 80..84, 84..92] {
            let mut changed = original.clone();
            changed[range].fill(0);
            assert!(decode_response(&changed).is_err());
        }
        for length in 0..original.len() {
            assert!(decode_response(&original[..length]).is_err());
        }
        let mut extra = original.clone();
        extra.push(0);
        assert!(decode_response(&extra).is_err());
        let mut failed_select = original.clone();
        failed_select[17] = 1;
        assert!(decode_response(&failed_select).is_err());
        let mut failed_return = original;
        failed_return[16] = 12;
        assert!(decode_response(&failed_return).is_err());
    }
    #[test]
    fn navigation_encoders_reject_local_invalid_context_and_channel() {
        let context = navigation_context();
        for invalid in [
            SelectionContext {
                epoch: 0,
                ..context
            },
            SelectionContext {
                key: CharacterKey {
                    source_id: 0,
                    ..context.key
                },
                ..context
            },
            SelectionContext {
                key: CharacterKey {
                    object_id: u64::MAX,
                    ..context.key
                },
                ..context
            },
        ] {
            assert!(
                encode_navigation(1, NavigationAction::SelectCharacter, invalid, None).is_err()
            );
        }
        assert!(encode_navigation(1, NavigationAction::EnterChannel, context, Some(4)).is_err());
        assert!(encode_navigation(1, NavigationAction::EnterChannel, context, None).is_err());
        assert!(
            encode_navigation(1, NavigationAction::ReturnCharacters, context, Some(0)).is_err()
        );
        let bytes = encode_navigation(1, NavigationAction::EnterChannel, context, Some(3)).unwrap();
        assert_eq!(bytes.len(), 40);
        assert_eq!(&bytes[36..], &[3, 0, 0, 0]);
    }

    #[test]
    fn credential_wire_is_exact_utf8_and_preserves_password_whitespace() {
        let credentials = Credentials::new("test".into(), " p ".into()).unwrap();
        assert_eq!(
            encode_auth(2, &credentials).unwrap().as_bytes(),
            &[
                b'L', b'T', b'M', b'1', 0, 0, 0, 27, 0, 0, 0, 2, 0, 3, 0, 0, 0, 4, b't', b'e',
                b's', b't', 0, 3, b' ', b'p', b' '
            ]
        );
        assert!(Credentials::new("test".into(), "   ".into()).is_ok());
        for (account, password) in [("", "p"), (" ", "p"), ("a", ""), ("a", "p\0"), ("a\n", "p")] {
            assert!(Credentials::new(account.into(), password.into()).is_err());
        }
        assert!(Credentials::new("a".repeat(65), "p".into()).is_err());
        assert!(Credentials::new("a".into(), "界".repeat(86)).is_err());
        assert!(Credentials::new("a".into(), "p".repeat(256)).is_ok());
    }

    #[test]
    fn authentication_result_requires_server_owned_sorted_unique_sources() {
        let decoded =
            decode_response(&response(2, 4, &auth_body(&[(1, "太陽神"), (2, "月亮神")]))).unwrap();
        let Response::Authentication(Ok(principal)) = decoded.response else {
            panic!("AUTH success")
        };
        assert_eq!(principal.account, "canonical");
        assert!(principal.authorizes_source(2));
        assert!(!principal.authorizes_source(3));
        for sources in [
            vec![],
            vec![(0, "a")],
            vec![(2, "a"), (1, "b")],
            vec![(1, "a"), (1, "b")],
            vec![(1, "a"), (2, "a")],
            vec![(u32::MAX, "a")],
        ] {
            assert!(decode_response(&response(2, 4, &auth_body(&sources))).is_err());
        }
        assert!(decode_response(&response(2, 4, &auth_body(&[(1, "A"), (2, "a")]))).is_ok());
        assert_eq!(
            decode_response(&response(2, 4, &[0, 1, 0, 0, 0, 0]))
                .unwrap()
                .response,
            Response::Authentication(Err(AuthenticationFailure::InvalidCredentials))
        );
        for body in [
            vec![0, 9, 0, 0, 0, 0],
            vec![0, 1, 0, 1, 0, 0],
            vec![0, 1, 0, 0, 0, 1, b'a'],
        ] {
            assert!(decode_response(&response(2, 4, &body)).is_err());
        }
    }

    #[test]
    fn character_identity_uses_source_and_full_u64_epoch_with_i64_level() {
        let decoded =
            decode_response(&response(3, 6, &character_body(&[1, i64::MAX as u64]))).unwrap();
        let Response::Characters {
            source_id,
            epoch,
            result: Ok(list),
        } = decoded.response
        else {
            panic!("roster")
        };
        assert_eq!(source_id, 2);
        assert_eq!(epoch, u64::MAX);
        assert_eq!(
            list.characters[1].key,
            CharacterKey {
                source_id: 2,
                object_id: i64::MAX as u64
            }
        );
        assert_eq!(list.characters[1].level, i64::MAX);
        assert_eq!(list.characters[0].name, "勇者");
        for ids in [vec![0], vec![u64::MAX], vec![2, 1], vec![1, 1]] {
            assert!(decode_response(&response(3, 6, &character_body(&ids))).is_err());
        }
        // Fixed body prefix 16, objectId 8, UTF-8 name (2 + 6), class 2,
        // sex/reserved 2, level 8. Each mutation targets one actual wire field.
        for (offset, value) in [(34, 2u8), (35, 1), (36, 0x80), (26, 0xff)] {
            let mut body = character_body(&[1]);
            body[offset] = value;
            assert!(decode_response(&response(3, 6, &body)).is_err());
        }
        for range in [0..4, 4..12, 36..44] {
            let mut body = character_body(&[1]);
            body[range].fill(0);
            assert!(decode_response(&response(3, 6, &body)).is_err());
        }
        assert!(encode_characters(4, 2, u64::MAX).is_ok());
        assert!(encode_characters(4, 2, 0).is_err());
        assert!(encode_characters(4, u32::MAX, 1).is_err());
    }

    #[test]
    fn header_payload_utf8_and_stream_bounds_are_strict() {
        let valid = response(2, 4, &auth_body(&[(1, "a")]));
        for length in 0..valid.len() {
            assert!(decode_response(&valid[..length]).is_err());
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        assert!(decode_response(&trailing).is_err());
        for (offset, value) in [(0, 0), (8, 0), (13, 3), (15, 1)] {
            let mut bad = valid.clone();
            bad[offset] = value;
            if offset == 8 {
                bad[8..12].fill(0);
            }
            assert!(decode_response(&bad).is_err());
        }
        let mut bad = valid.clone();
        bad[22] = 0xff;
        assert!(decode_response(&bad).is_err());
        let mut oversized = valid[..16].to_vec();
        oversized[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        let mut input = std::io::Cursor::new(oversized);
        assert!(read_response(&mut input).is_err());
        assert_eq!(input.position(), 16);
        let mut coalesced = valid.clone();
        coalesced.extend_from_slice(&response(3, 8, &[]));
        let mut input = std::io::Cursor::new(coalesced);
        assert_eq!(read_response(&mut input).unwrap().request_id, 2);
        assert_eq!(input.position(), valid.len() as u64);
        assert_eq!(read_response(&mut input).unwrap().response, Response::Pong);
        assert_eq!(encode_ping(9).unwrap().len(), 16);
        assert!(super::super::protocol::decode(&valid).is_err());
    }

    #[test]
    fn count_byte_limits_failed_rosters_and_fragmentation_are_bounded() {
        let owned: Vec<_> = (1..=64).map(|id| (id, format!("source-{id}"))).collect();
        let sources: Vec<_> = owned
            .iter()
            .map(|(id, name)| (*id, name.as_str()))
            .collect();
        assert!(decode_response(&response(2, 4, &auth_body(&sources))).is_ok());
        let mut body = auth_body(&sources);
        body[2..4].copy_from_slice(&65_u16.to_be_bytes());
        assert!(decode_response(&response(2, 4, &body)).is_err());
        for name in [
            "a".repeat(257),
            "界".repeat(86),
            "\u{a0}".into(),
            "a\u{85}".into(),
        ] {
            assert!(decode_response(&response(2, 4, &auth_body(&[(1, &name)]))).is_err());
        }
        let mut body = character_body(&[]);
        body[14..16].copy_from_slice(&65_u16.to_be_bytes());
        assert!(decode_response(&response(3, 6, &body)).is_err());
        for (status, expected) in [
            (1, CharacterFailure::Denied),
            (2, CharacterFailure::BackendUnavailable),
        ] {
            let mut body = character_body(&[]);
            body[12..14].copy_from_slice(&(status as u16).to_be_bytes());
            assert!(
                matches!(decode_response(&response(3, 6, &body)).unwrap().response,
                Response::Characters { result: Err(failure), .. } if failure == expected)
            );
            let mut with_rows = character_body(&[1]);
            with_rows[12..14].copy_from_slice(&(status as u16).to_be_bytes());
            assert!(decode_response(&response(3, 6, &with_rows)).is_err());
        }
        struct Fragmented(std::io::Cursor<Vec<u8>>);
        impl Read for Fragmented {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                let length = output.len().min(1);
                self.0.read(&mut output[..length])
            }
        }
        let bytes = response(3, 6, &character_body(&[1]));
        let mut fragmented = Fragmented(std::io::Cursor::new(bytes.clone()));
        assert_eq!(
            read_response(&mut fragmented).unwrap(),
            decode_response(&bytes).unwrap()
        );
    }
}
