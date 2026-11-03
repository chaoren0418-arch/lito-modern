//! TLS-protected modern HELLO/ACK, authentication and source-scoped rosters.
//!
//! Rustls verifies the chain, current validity, server-auth usage, signatures
//! and DNS name with its normal verifier. The bundled Mozilla roots or one
//! explicit private CA are mutually exclusive. No dangerous verifier, native
//! XOR transport, plaintext fallback, client certificate or 0-RTT is enabled.
//!
//! The same TLS stream stays owned across AUTH, roster requests and PING/PONG.
//! Operations are serialized by one owner; there is no background receiver here.
//! `Negotiated` alone is neither authenticated nor proof that a peer is online.
//! No raw socket or unrestricted application-data writer escapes this boundary.

use std::fmt;
use std::io::{self, Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::{CertificateDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore};

use super::ammo_protocol as ammo;
use super::application_protocol::{
    self as application, ApplicationError, AuthenticationFailure, CharacterFailure, CharacterList,
    Credentials, NavigationAction, NavigationPhase, NavigationReply, NavigationStatus, Principal,
    Response, SelectionContext,
};
use super::character_data::{self, CharacterDataReply};
use super::chat_protocol as chat;
use super::combat_protocol as combat;
use super::disintegrate_cast_protocol as disintegrate;
use super::equipment::{self, Reply as EquipmentReply, Request as EquipmentRequest};
use super::gm_protocol as gm;
use super::npc_protocol as npc;
use super::npc_spell_protocol as npc_spell;
use super::protocol::{Ack, ClientState, Hello, ModernClientSession, ProtocolError};
use super::record_protocol as record;
use super::rooms;
use super::spell_protocol as spell;
use super::world_protocol as world;
use super::{item_protocol as item, shop_protocol as shop};

pub const MAX_PRIVATE_CA_BYTES: usize = 64 * 1024;
const MAX_TLS_HANDSHAKE_WIRE_BYTES: usize = 256 * 1024;
const MAX_NEGOTIATION_WIRE_BYTES: usize = 64 * 1024;
const MAX_TLS_BUFFER_BYTES: usize = 64 * 1024;
const MAX_APPLICATION_WIRE_BYTES: usize = 256 * 1024;
const MAX_CHARACTER_DATA_WIRE_BYTES: usize = 512 * 1024;
const MAX_EQUIPMENT_WIRE_BYTES: usize = 640 * 1024;
const CLOSE_TIMEOUT: Duration = Duration::from_millis(250);

fn npc_wire_budget(version: u16) -> usize {
    // The v31 atomic NPC result may occupy a full 1 MiB application frame.
    // Keep bounded TLS overhead without imposing the older 256 KiB envelope.
    if version >= 31 {
        2 * 1024 * 1024
    } else {
        2 * MAX_APPLICATION_WIRE_BYTES
    }
}

fn combat_denial_preserves_attack_event(
    prior: Option<&combat::Reply>,
    next: &combat::Reply,
) -> bool {
    if next.echo.action != combat::Action::Attack {
        return true;
    }
    let Some(old) = prior.and_then(|r| r.frame.as_ref()) else {
        return false;
    };
    let Some(frame) = next.frame.as_ref() else {
        return false;
    };
    let Some(actor) = frame
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(next.echo.context.key))
    else {
        return false;
    };
    let (Some(a), Some(b)) = (old.vital(actor.entity_id), frame.vital(actor.entity_id)) else {
        return false;
    };
    if next.status == combat::Status::Ok {
        return b.attack_sequence == next.echo.sequence && b.attack_revision > a.attack_revision;
    }
    (b.attack_sequence == a.attack_sequence || b.attack_sequence == next.echo.sequence)
        && b.attack_revision == a.attack_revision
        && b.last_target_entity_id == a.last_target_entity_id
}

#[derive(Clone, Debug, Default)]
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TransportLimits {
    /// TCP connect itself is a bounded OS call; cancellation is rechecked when
    /// it returns, at most this interval later. No detached connect thread.
    pub connect_timeout: Duration,
    /// Single monotonic deadline spanning configuration, connect, TLS and ACK.
    pub total_timeout: Duration,
    /// TLS network waits observe cancellation at this interval (<=250 ms).
    pub io_poll_interval: Duration,
}

impl Default for TransportLimits {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(3),
            total_timeout: Duration::from_secs(10),
            io_poll_interval: Duration::from_millis(100),
        }
    }
}

impl TransportLimits {
    pub fn validate(self) -> Result<Self, TransportError> {
        if self.connect_timeout.is_zero()
            || self.total_timeout.is_zero()
            || self.io_poll_interval.is_zero()
            || self.total_timeout > Duration::from_secs(60)
            || self.connect_timeout > self.total_timeout
            || self.io_poll_interval > Duration::from_millis(250)
            || self.io_poll_interval > self.total_timeout
        {
            return Err(TransportError::InvalidConfiguration(
                "invalid bounded transport deadlines",
            ));
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Connect,
    TlsHandshake,
    Negotiation,
    Authentication,
    Characters,
    Navigation,
    CharacterData,
    Equipment,
    Rooms,
    Matchmaking,
    GmBots,
    Chat,
    DirectChat,
    Shop,
    Item,
    Ammo,
    World,
    BattleData,
    BattleOperation,
    BattleWorld,
    VipWorld,
    VipEquipment,
    BattleRevival,
    BattleEquipment,
    GreaterHaste,
    TargetHaste,
    Slow,
    PlayerHealing,
    Combat,
    Npc,
    Skills,
    Spell,
    Record,
    Ping,
    Close,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

#[derive(Debug)]
pub enum TransportError {
    InvalidConfiguration(&'static str),
    Cancelled,
    DeadlineExceeded(Stage),
    WireBudgetExceeded(Stage),
    UnexpectedEarlyApplicationData,
    Tls(rustls::Error),
    Protocol(ProtocolError),
    Application(ApplicationError),
    AuthenticationRejected(AuthenticationFailure),
    CharactersRejected(CharacterFailure),
    /// Verified non-consuming admission rejection, with an actual READ cached.
    GuardRejected(super::battle_action::Rejection),
    InvalidState(&'static str),
    Io(io::Error),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(reason) => write!(f, "modern TLS configuration: {reason}"),
            Self::Cancelled => write!(f, "modern TLS connection cancelled"),
            Self::DeadlineExceeded(stage) => write!(f, "modern TLS {stage:?} deadline exceeded"),
            Self::WireBudgetExceeded(stage) => {
                write!(f, "modern TLS {stage:?} wire budget exceeded")
            }
            Self::UnexpectedEarlyApplicationData => {
                write!(f, "server sent application data before modern HELLO")
            }
            Self::Tls(error) => write!(f, "TLS verification/record error: {error}"),
            Self::Protocol(error) => write!(f, "modern negotiation: {error}"),
            Self::Application(error) => error.fmt(f),
            Self::AuthenticationRejected(reason) => {
                write!(f, "modern authentication rejected: {reason:?}")
            }
            Self::CharactersRejected(reason) => {
                write!(f, "modern character request rejected: {reason:?}")
            }
            Self::GuardRejected(reason) => write!(f, "battle action discarded: {reason:?}"),
            Self::InvalidState(reason) => write!(f, "modern TLS state: {reason}"),
            Self::Io(error) => write!(f, "modern TLS I/O: {error}"),
        }
    }
}
impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Tls(e) => Some(e),
            Self::Protocol(e) => Some(e),
            Self::Application(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<ProtocolError> for TransportError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}
impl From<ApplicationError> for TransportError {
    fn from(error: ApplicationError) -> Self {
        Self::Application(error)
    }
}

/// Validate a DNS identity, not a URL, socket address, wildcard or lookup.
/// ASCII A-label DNS names only; callers must not silently apply an IDN guess.
pub fn validate_server_name(name: &str) -> Result<(), TransportError> {
    if name.is_empty() || name.len() > 253 || !name.is_ascii() || name.parse::<IpAddr>().is_ok() {
        return Err(TransportError::InvalidConfiguration(
            "server_name must be an ASCII DNS identity",
        ));
    }
    for label in name.split('.') {
        if label.is_empty()
            || label.len() > 63
            || !label.as_bytes()[0].is_ascii_alphanumeric()
            || !label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
            || !label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(TransportError::InvalidConfiguration(
                "invalid DNS label in server_name",
            ));
        }
    }
    Ok(())
}

fn client_config(trusted_root_der: Option<&[u8]>) -> Result<Arc<ClientConfig>, TransportError> {
    let mut roots = RootCertStore::empty();
    if let Some(der) = trusted_root_der {
        if der.is_empty() || der.len() > MAX_PRIVATE_CA_BYTES {
            return Err(TransportError::InvalidConfiguration(
                "private CA must be one nonempty DER certificate <=64 KiB",
            ));
        }
        roots
            .add(CertificateDer::from(der))
            .map_err(TransportError::Tls)?;
    } else {
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
        .map_err(TransportError::Tls)?
        .with_root_certificates(roots)
        .with_no_client_auth();
    config.enable_early_data = false;
    config.resumption = rustls::client::Resumption::disabled();
    Ok(Arc::new(config))
}

#[derive(Clone, Copy, Debug)]
enum AbortReason {
    Cancelled,
    Deadline,
    WireBudget,
}
#[derive(Clone, Copy, Debug)]
struct Abort {
    reason: AbortReason,
    stage: Stage,
}
impl Abort {
    fn error(self) -> TransportError {
        match self.reason {
            AbortReason::Cancelled => TransportError::Cancelled,
            AbortReason::Deadline => TransportError::DeadlineExceeded(self.stage),
            AbortReason::WireBudget => TransportError::WireBudgetExceeded(self.stage),
        }
    }
}
impl fmt::Display for Abort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error().fmt(f)
    }
}
impl std::error::Error for Abort {}

trait TimedIo: Read + Write {
    fn set_timeout(&mut self, timeout: Duration) -> io::Result<()>;
}
impl TimedIo for TcpStream {
    fn set_timeout(&mut self, timeout: Duration) -> io::Result<()> {
        self.set_read_timeout(Some(timeout))?;
        self.set_write_timeout(Some(timeout))
    }
}

struct DeadlineIo<'a, T: TimedIo> {
    stream: &'a mut T,
    deadline: Instant,
    cancellation: &'a Cancellation,
    poll_interval: Duration,
    stage: Stage,
    read_remaining: usize,
    write_remaining: usize,
    last_abort: Option<Abort>,
}

impl<'a, T: TimedIo> DeadlineIo<'a, T> {
    fn new(
        stream: &'a mut T,
        deadline: Instant,
        cancellation: &'a Cancellation,
        poll_interval: Duration,
        stage: Stage,
        wire_limit: usize,
    ) -> Self {
        Self {
            stream,
            deadline,
            cancellation,
            poll_interval,
            stage,
            read_remaining: wire_limit,
            write_remaining: wire_limit,
            last_abort: None,
        }
    }
    fn abort(&mut self, reason: AbortReason) -> io::Error {
        let abort = Abort {
            reason,
            stage: self.stage,
        };
        self.last_abort = Some(abort);
        // Interrupted must not be used: Read::read_exact and rustls retry it.
        io::Error::new(
            if matches!(reason, AbortReason::Deadline) {
                io::ErrorKind::TimedOut
            } else {
                io::ErrorKind::ConnectionAborted
            },
            abort,
        )
    }
    fn check(&mut self) -> io::Result<Duration> {
        if self.cancellation.is_cancelled() {
            return Err(self.abort(AbortReason::Cancelled));
        }
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(self.abort(AbortReason::Deadline));
        }
        Ok(remaining.min(self.poll_interval))
    }
    fn map_io(&self, error: io::Error) -> TransportError {
        if let Some(abort) = self.last_abort {
            return abort.error();
        }
        if let Some(error) = error
            .get_ref()
            .and_then(|e| e.downcast_ref::<rustls::Error>())
        {
            return TransportError::Tls(error.clone());
        }
        TransportError::Io(error)
    }
}

fn transient(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
    )
}
impl<T: TimedIo> Read for DeadlineIo<'_, T> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            let timeout = self.check()?;
            if self.read_remaining == 0 {
                return Err(self.abort(AbortReason::WireBudget));
            }
            self.stream.set_timeout(timeout)?;
            let length = bytes.len().min(self.read_remaining);
            match self.stream.read(&mut bytes[..length]) {
                Ok(count) => {
                    self.read_remaining -= count;
                    return Ok(count);
                }
                Err(error) if transient(&error) => continue,
                Err(error) => return Err(error),
            }
        }
    }
}
impl<T: TimedIo> Write for DeadlineIo<'_, T> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        loop {
            let timeout = self.check()?;
            if self.write_remaining == 0 {
                return Err(self.abort(AbortReason::WireBudget));
            }
            self.stream.set_timeout(timeout)?;
            let length = bytes.len().min(self.write_remaining);
            match self.stream.write(&bytes[..length]) {
                Ok(count) => {
                    self.write_remaining -= count;
                    return Ok(count);
                }
                Err(error) if transient(&error) => continue,
                Err(error) => return Err(error),
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.check()?;
        self.stream.flush()
    }
}

pub struct ModernTlsConnection {
    socket: TcpStream,
    tls: ClientConnection,
    session: ModernClientSession,
    version: TlsVersion,
    cancellation: Cancellation,
    poll_interval: Duration,
    exchange_timeout: Duration,
    last_request_id: u32,
    authentication_attempted: bool,
    principal: Option<Principal>,
    last_roster: Option<CharacterList>,
    navigation: Option<NavigationReply>,
    rooms: Option<rooms::Reply>,
    matchmaking_confirmation: super::matchmaking_protocol::Confirmation,
    chat_confirmation: chat::Confirmation,
    filtered_chat_confirmation: super::filtered_chat_protocol::Confirmation,
    direct_chat_confirmation: super::direct_chat_protocol::Confirmation,
    world: Option<world::Reply>,
    combat: Option<combat::Reply>,
    npc: Option<npc::Reply>,
    battle_state: super::battle_state::Tracker,
    action_guard: Option<super::battle_action::Guard>,
    action_recovery: Option<Response>,
    shop: Option<shop::Reply>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationState {
    Negotiated,
    Authenticated,
    Closed,
}
fn confirms_lobby_leave(prior: Option<&NavigationReply>, reply: &NavigationReply) -> bool {
    prior.is_some_and(|prior| {
        prior.snapshot.phase == NavigationPhase::Lobby
            && prior.snapshot.selected.is_some()
            && reply.action == NavigationAction::LeaveLobby
            && reply.status == NavigationStatus::Ok
            && reply.context == prior.context
            && reply.snapshot.phase == NavigationPhase::Channels
            && reply.snapshot.channel_id.is_none()
            && reply.snapshot.selected == prior.snapshot.selected
    })
}
/// BACK Busy is a refusal, not a committed navigation transition. The server
/// may refresh population counts, but cannot silently replace the reservation,
/// selected row, membership, or channel definitions while acknowledging it.
pub(super) fn confirms_busy_return(
    prior: Option<&NavigationReply>,
    reply: &NavigationReply,
) -> bool {
    prior.is_some_and(|prior| {
        matches!(
            prior.snapshot.phase,
            NavigationPhase::Channels | NavigationPhase::Lobby
        ) && prior.snapshot.selected.is_some()
            && reply.action == NavigationAction::ReturnCharacters
            && reply.status == NavigationStatus::Busy
            && reply.context == prior.context
            && reply.snapshot.phase == prior.snapshot.phase
            && reply.snapshot.channel_id == prior.snapshot.channel_id
            && reply.snapshot.selected == prior.snapshot.selected
            && reply.snapshot.channels.len() == prior.snapshot.channels.len()
            && reply
                .snapshot
                .channels
                .iter()
                .zip(&prior.snapshot.channels)
                .all(|(next, old)| {
                    next.id == old.id
                        && next.name == old.name
                        && next.capacity == old.capacity
                        && next.minimum_level == old.minimum_level
                        && next.maximum_level == old.maximum_level
                })
    })
}
impl fmt::Debug for ModernTlsConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ModernTlsConnection")
            .field("state", &self.session.state())
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}
fn skill_catalog_negotiated(reply: &spell::SkillsReply, ack: &Ack) -> bool {
    let fire = ack.version >= 14 && ack.capabilities & super::protocol::CAP_FIRE_STORM != 0
        || super::npc_fire_storm_protocol::negotiated(ack);
    let dis = super::disintegrate_cast_protocol::negotiated(ack)
        || super::npc_disintegrate_protocol::negotiated(ack);
    reply.skills.iter().all(|s| {
        s.support == spell::Support::Unimplemented
            || (s.skill_id != spell::FIRE_STORM || fire)
                && (s.skill_id != spell::DISINTEGRATE || dis)
                && (s.skill_id != spell::GREATER_HASTE
                    || super::battle_greater_haste::negotiated(ack))
                && (s.skill_id != spell::HASTE || super::battle_target_haste::negotiated(ack))
                && (s.skill_id != spell::SLOW || super::battle_slow::negotiated(ack))
    })
}

impl ModernTlsConnection {
    /// Explicit handshake extension word; None before negotiation/after close.
    /// This is compatibility only, not authentication or an active-world grant.
    pub fn negotiated_extensions(&self) -> Option<u32> {
        self.session.extensions()
    }
    pub fn negotiated_selection(&self) -> Option<super::protocol::extensions::Selected> {
        super::protocol::extensions::Selected::new(self.ack()?, self.negotiated_extensions()?).ok()
    }
    pub fn state(&self) -> ClientState {
        self.session.state()
    }
    pub fn ack(&self) -> Option<Ack> {
        match self.session.state() {
            ClientState::Negotiated(ack) => Some(ack),
            _ => None,
        }
    }
    pub fn tls_version(&self) -> TlsVersion {
        self.version
    }
    pub fn application_state(&self) -> ApplicationState {
        if self.session.state() == ClientState::Closed {
            ApplicationState::Closed
        } else if self.principal.is_some() {
            ApplicationState::Authenticated
        } else {
            ApplicationState::Negotiated
        }
    }
    pub fn principal(&self) -> Option<&Principal> {
        self.principal.as_ref()
    }
    pub fn navigation(&self) -> Option<&NavigationReply> {
        self.navigation.as_ref()
    }

    /// Each following AUTH/CHAR/PING gets one whole-operation deadline. Set this
    /// from the modern profile on the owning worker, never a global/native knob.
    pub fn set_exchange_timeout(&mut self, timeout: Duration) -> Result<(), TransportError> {
        if !(Duration::from_millis(50)..=Duration::from_secs(30)).contains(&timeout) {
            return Err(TransportError::InvalidConfiguration(
                "application exchange timeout must be 50ms..30s",
            ));
        }
        self.exchange_timeout = timeout;
        Ok(())
    }

    fn abort(&mut self) {
        self.battle_state = super::battle_state::Tracker::default();
        self.action_guard = None;
        self.action_recovery = None;
        self.shop = None;
        self.principal = None;
        self.last_roster = None;
        self.navigation = None;
        self.rooms = None;
        self.world = None;
        self.combat = None;
        self.npc = None;
        self.session.close();
        let _ = self.socket.shutdown(Shutdown::Both);
    }
    fn next_request_id(&mut self) -> Result<u32, TransportError> {
        if self.session.state() == ClientState::Closed {
            return Err(TransportError::InvalidState("connection is closed"));
        }
        match self.last_request_id.checked_add(1) {
            Some(id) => {
                self.last_request_id = id;
                Ok(id)
            }
            None => {
                self.abort();
                Err(TransportError::InvalidState(
                    "request ID exhausted; reconnect required",
                ))
            }
        }
    }

    /// Only private, typed encoders call this method. All application bytes use
    /// the exact connection whose TLS chain/name and HELLO were verified.
    fn exchange(
        &mut self,
        frame: &[u8],
        request_id: u32,
        stage: Stage,
    ) -> Result<Response, TransportError> {
        self.exchange_with_operation(frame, request_id, stage, None)
    }
    fn exchange_with_operation(
        &mut self,
        frame: &[u8],
        request_id: u32,
        stage: Stage,
        operation: Option<super::battle_action::Guard>,
    ) -> Result<Response, TransportError> {
        let weight_enabled = self.supports_weight();
        let timed_enabled = self.supports_timed();
        let arrow_enabled = self.supports_arrow();
        let regen_enabled = self.supports_regen();
        let honor_enabled = self.supports_honor();
        let fire_enabled = self.supports_fire_storm();
        let descriptor_ack = self.ack();
        let battle_enabled = self.supports_battle_data();
        let action_enabled = self.supports_battle_actions();
        // Once plaintext reaches rustls, cancellation is an uncertain exchange,
        // even if the OS has not yet reported a successful write. Do not drain it.
        let mut application_queued = false;
        let result = (|| {
            if self.session.state() == ClientState::Closed {
                return Err(TransportError::InvalidState("connection is closed"));
            }
            let peer = self
                .negotiated_selection()
                .ok_or(TransportError::InvalidState(
                    "application requires complete negotiated selection",
                ))?;
            let version = peer.base().version;
            let format = super::battle_payload::Format::Selected(peer);
            let guarded = if action_enabled && super::battle_action::mutation(frame) {
                if descriptor_ack
                    .as_ref()
                    .is_some_and(super::battle_operation::negotiated)
                {
                    return Err(TransportError::InvalidState(
                        "v38 requires canonical operation endpoint",
                    ));
                }
                let guard = self
                    .action_guard
                    .take()
                    .ok_or(TransportError::InvalidState(
                        "life-bound action has no queued proof",
                    ))?;
                Some((guard, super::battle_action::encode(guard, frame, version)?))
            } else {
                if self.action_guard.is_some() {
                    return Err(TransportError::InvalidState(
                        "queued proof does not describe this operation",
                    ));
                }
                None
            };
            let frame = guarded.as_ref().map_or(frame, |(_, wire)| wire.as_slice());
            let deadline = Instant::now() + self.exchange_timeout;
            let mut io = DeadlineIo::new(
                &mut self.socket,
                deadline,
                &self.cancellation,
                self.poll_interval,
                stage,
                if stage == Stage::VipWorld {
                    2 * super::battle_vip_world::MAX_RESULT_BYTES
                } else if stage == Stage::VipEquipment {
                    2 * super::battle_vip_equipment::MAX_RESULT_BYTES
                } else if stage == Stage::PlayerHealing {
                    2 * super::battle_player_healing::MAX_RESULT_BYTES
                } else if stage == Stage::Slow {
                    2 * format.limit(super::battle_slow::MAX_RESULT_BYTES)
                } else if stage == Stage::TargetHaste {
                    2 * format.limit(super::battle_target_haste::MAX_RESULT_BYTES)
                } else if stage == Stage::GreaterHaste {
                    2 * format.limit(super::battle_greater_haste::MAX_RESULT_BYTES)
                } else if stage == Stage::BattleEquipment {
                    2 * format.limit(super::battle_equipment::MAX_RESULT_BYTES)
                } else if stage == Stage::BattleRevival {
                    2 * format.limit(super::battle_revival::MAX_RESULT_BYTES)
                } else if stage == Stage::BattleWorld {
                    2 * format.limit(super::battle_world::MAX_RESULT_BYTES)
                } else if operation.is_some() {
                    2 * format.limit(super::battle_operation::MAX_RESULT_BYTES)
                } else if guarded.is_some() {
                    2 * super::battle_action::MAX_RESULT_BYTES
                } else if stage == Stage::BattleData {
                    2 * format.limit(super::battle_read::MAX_FRAME_BYTES)
                } else if stage == Stage::Npc {
                    npc_wire_budget(version)
                } else if version >= 14
                    && matches!(
                        stage,
                        Stage::Combat | Stage::Item | Stage::Spell | Stage::Ammo
                    )
                {
                    2 * 1024 * 1024 // bounded TLS overhead for a v14 1 MiB application frame
                } else if stage == Stage::Equipment {
                    MAX_EQUIPMENT_WIRE_BYTES
                } else if stage == Stage::CharacterData
                    || stage == Stage::Shop
                    || version >= 10
                        && matches!(
                            stage,
                            Stage::Combat | Stage::Item | Stage::Spell | Stage::Ammo
                        )
                {
                    MAX_CHARACTER_DATA_WIRE_BYTES
                } else {
                    MAX_APPLICATION_WIRE_BYTES
                },
            );
            io.check().map_err(|error| io.map_io(error))?;
            application_queued = true;
            self.tls
                .writer()
                .write_all(frame)
                .map_err(TransportError::Io)?;
            self.tls
                .complete_io(&mut io)
                .map_err(|error| io.map_io(error))?;
            let (rejection, response) = if stage == Stage::VipWorld {
                super::battle_vip_world::read_reply(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::VipWorld(reply),
                        },
                    )
                })
            } else if stage == Stage::VipEquipment {
                super::battle_vip_equipment::read_reply(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::VipEquipment(reply),
                        },
                    )
                })
            } else if stage == Stage::PlayerHealing {
                let expected = super::battle_player_healing::decode_candidate_intent(frame)?;
                super::battle_player_healing::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    &expected,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::PlayerHealing(reply),
                        },
                    )
                })
            } else if stage == Stage::Slow {
                super::battle_slow::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::Slow(reply),
                        },
                    )
                })
            } else if stage == Stage::TargetHaste {
                super::battle_target_haste::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::TargetHaste(reply),
                        },
                    )
                })
            } else if stage == Stage::GreaterHaste {
                super::battle_greater_haste::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::GreaterHaste(reply),
                        },
                    )
                })
            } else if stage == Stage::BattleEquipment {
                super::battle_equipment::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::BattleEquipment(reply),
                        },
                    )
                })
            } else if stage == Stage::BattleRevival {
                super::battle_revival::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::BattleRevival(reply),
                        },
                    )
                })
            } else if stage == Stage::BattleWorld {
                super::battle_world::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::BattleWorld(reply),
                        },
                    )
                })
            } else if let Some(guard) = operation {
                super::battle_operation::read_reply_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    guard,
                    frame,
                    peer,
                )
                .map(|reply| {
                    (
                        super::battle_action::Rejection::None,
                        application::ResponseFrame {
                            request_id,
                            response: Response::BattleOperation(reply),
                        },
                    )
                })
            } else if let Some((guard, expected)) = &guarded {
                super::battle_action::read_reply(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    *guard,
                    expected,
                    version,
                )
            } else {
                application::read_response_selected(
                    &mut rustls::Stream::new(&mut self.tls, &mut io),
                    peer,
                    battle_enabled && stage == Stage::BattleData,
                )
                .map(|r| (super::battle_action::Rejection::None, r))
            }
            .map_err(|error| {
                #[cfg(test)]
                eprintln!(
                    "application response rejected: stage={stage:?} version={version} kind={:?}",
                    error.kind
                );
                io.last_abort
                    .map(Abort::error)
                    .unwrap_or(TransportError::Application(error))
            })?;
            io.check().map_err(|error| io.map_io(error))?;
            drop(io);
            if response.request_id != request_id {
                return Err(application::error(
                    application::ErrorKind::RequestIdMismatch,
                    "application response request ID mismatch",
                )
                .into());
            }
            if matches!(&response.response,Response::BattleData(r) if !super::battle_read::reply_negotiated_selected(r,peer))
            {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "canonical data exceeds negotiated features",
                )
                .into());
            }
            if matches!(&response.response,Response::BattleOperation(r) if !super::battle_operation::reply_negotiated_selected(r,peer))
            {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "canonical operation exceeds negotiated features",
                )
                .into());
            }
            if matches!(&response.response,Response::BattleEquipment(r) if !super::battle_equipment::reply_negotiated_selected(r,peer))
            {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "canonical equipment exceeds negotiated features",
                )
                .into());
            }
            if matches!(&response.response,Response::BattleRevival(r) if !super::battle_revival::reply_negotiated_selected(r,peer))
            {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "canonical revival exceeds negotiated features",
                )
                .into());
            }
            if matches!(&response.response,Response::BattleWorld(r) if !super::battle_world::reply_negotiated_selected(r,peer))
            {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "canonical lifecycle exceeds negotiated features",
                )
                .into());
            }
            if !world::descriptor_negotiated(&response.response, descriptor_ack.as_ref().unwrap()) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "match descriptor does not match negotiated capability or world feature scope",
                )
                .into());
            }
            if !super::weight_protocol::negotiated(&response.response, weight_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "weight snapshot does not match negotiated capability",
                )
                .into());
            }
            if !super::timed_protocol::negotiated(&response.response, timed_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "timed state does not match negotiated capability",
                )
                .into());
            }
            if !ammo::negotiated(&response.response, arrow_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "ammunition does not match negotiated capability",
                )
                .into());
            }
            if !super::regen_protocol::negotiated(&response.response, regen_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "natural recovery does not match negotiated capability",
                )
                .into());
            }
            if !super::honor_protocol::negotiated(&response.response, honor_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "Honor state does not match negotiated capability and mode",
                )
                .into());
            }
            if !super::fire_storm_protocol::negotiated(&response.response, fire_enabled) {
                return Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "Fire Storm state was not negotiated",
                )
                .into());
            }
            if rejection != super::battle_action::Rejection::None {
                self.accept_action_recovery(response.response)?;
                return Err(TransportError::GuardRejected(rejection));
            }
            Ok(response.response)
        })();
        if result.is_err() && !matches!(&result, Err(TransportError::GuardRejected(_))) {
            if !application_queued && matches!(&result, Err(TransportError::Cancelled)) {
                // The worker's stop can arrive after its loop check but before
                // this pre-write check. We are still at a confirmed boundary:
                // send only a bounded close alert, never the cancelled frame.
                // Preserve Cancelled, the original stop token and no receipt.
                let _ = self.close_with_token(&Cancellation::new());
            } else {
                self.abort();
            }
        }
        result
    }

    /// Consumes credentials once, after verified TLS and HELLO, on this stream.
    /// Rejected AUTH closes the connection and cannot be retried on that stream.
    pub fn authenticate(&mut self, credentials: Credentials) -> Result<Principal, TransportError> {
        if self.authentication_attempted || self.application_state() != ApplicationState::Negotiated
        {
            self.abort();
            return Err(TransportError::InvalidState(
                "authentication requires one fresh negotiated session",
            ));
        }
        self.authentication_attempted = true;
        let request_id = self.next_request_id()?;
        let frame = application::encode_auth(request_id, &credentials)?;
        drop(credentials);
        let response = self.exchange(frame.as_bytes(), request_id, Stage::Authentication);
        drop(frame);
        match response? {
            Response::Authentication(Ok(principal)) => {
                self.principal = Some(principal.clone());
                Ok(principal)
            }
            Response::Authentication(Err(reason)) => {
                self.abort();
                Err(TransportError::AuthenticationRejected(reason))
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::UnexpectedMessage,
                    "expected AUTH_RESULT",
                )
                .into())
            }
        }
    }

    pub fn characters(
        &mut self,
        source_id: u32,
        epoch: u64,
    ) -> Result<CharacterList, TransportError> {
        if self.navigation.is_some() {
            return Err(TransportError::InvalidState(
                "return from the selected character before listing a source",
            ));
        }
        let Some(principal) = self.principal.as_ref() else {
            self.abort();
            return Err(TransportError::InvalidState(
                "character roster requires authentication",
            ));
        };
        if !principal.authorizes_source(source_id) {
            // A local invalid selection never leaks a repository request. It is
            // not malformed peer data and does not invalidate an existing login.
            return Err(TransportError::CharactersRejected(CharacterFailure::Denied));
        }
        if epoch == 0 {
            return Err(application::error(
                application::ErrorKind::InvalidIdentity,
                "zero selection epoch",
            )
            .into());
        }
        let request_id = self.next_request_id()?;
        let frame = application::encode_characters(request_id, source_id, epoch)?;
        self.last_roster = None;
        match self.exchange(&frame, request_id, Stage::Characters)? {
            Response::Characters {
                source_id: returned_source,
                epoch: returned_epoch,
                result,
            } if returned_source == source_id && returned_epoch == epoch => {
                let roster = result.map_err(TransportError::CharactersRejected)?;
                self.last_roster = Some(roster.clone());
                Ok(roster)
            }
            Response::Characters { .. } => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "character source or epoch mismatch",
                )
                .into())
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::UnexpectedMessage,
                    "expected CHAR_RESULT",
                )
                .into())
            }
        }
    }

    pub fn select_character(
        &mut self,
        context: SelectionContext,
    ) -> Result<NavigationReply, TransportError> {
        let current = self.last_roster.as_ref().is_some_and(|roster| {
            roster.source_id == context.key.source_id
                && roster.epoch == context.epoch
                && roster.characters.iter().any(|row| row.key == context.key)
        });
        if self.navigation.is_some() || !current {
            return Err(TransportError::InvalidState(
                "selection must reference the currently confirmed roster",
            ));
        }
        self.navigate(NavigationAction::SelectCharacter, context, None)
    }
    pub fn enter_channel(&mut self, channel_id: u8) -> Result<NavigationReply, TransportError> {
        let context = self
            .navigation
            .as_ref()
            .ok_or(TransportError::InvalidState(
                "select a character before entering a channel",
            ))?
            .context;
        if channel_id > 3 {
            return Err(TransportError::InvalidState("channel must be 0..3"));
        }
        self.navigate(NavigationAction::EnterChannel, context, Some(channel_id))
    }
    pub fn return_characters(&mut self) -> Result<NavigationReply, TransportError> {
        let context = self
            .navigation
            .as_ref()
            .ok_or(TransportError::InvalidState(
                "no selected character to release",
            ))?
            .context;
        self.navigate(NavigationAction::ReturnCharacters, context, None)
    }
    /// Release only lobby membership, retaining the server-selected character.
    /// No local page transition is committed until the matching ACK arrives.
    pub fn leave_lobby(&mut self) -> Result<NavigationReply, TransportError> {
        let context = self
            .navigation
            .as_ref()
            .filter(|reply| reply.snapshot.phase == NavigationPhase::Lobby)
            .ok_or(TransportError::InvalidState(
                "join a lobby before leaving it",
            ))?
            .context;
        self.navigate(NavigationAction::LeaveLobby, context, None)
    }
    pub fn supports_character_data(&self) -> bool {
        self.ack()
            .is_some_and(|ack| ack.capabilities & super::protocol::CAP_CHARACTER_DATA != 0)
    }
    pub fn supports_equipment(&self) -> bool {
        self.ack()
            .is_some_and(|ack| ack.capabilities & super::protocol::CAP_EQUIPMENT != 0)
    }
    pub fn supports_rooms(&self) -> bool {
        self.ack()
            .is_some_and(|ack| ack.capabilities & super::protocol::CAP_ROOMS != 0)
    }
    pub fn supports_gm_bots(&self) -> bool {
        self.ack().is_some_and(|ack| {
            ack.version >= 7 && ack.capabilities & super::protocol::CAP_GM_BOTS != 0
        })
    }
    pub fn room_snapshot(&self) -> Option<&rooms::Reply> {
        self.rooms.as_ref()
    }
    pub fn supports_world(&self) -> bool {
        self.ack()
            .is_some_and(|ack| ack.capabilities & super::protocol::CAP_WORLD != 0)
    }
    pub fn world_snapshot(&self) -> Option<&world::Reply> {
        self.world.as_ref()
    }
    pub fn supports_battle_actions(&self) -> bool {
        self.ack()
            .is_some_and(|a| super::battle_action::negotiated(&a))
    }
    pub(crate) fn arm_battle_action(
        &mut self,
        guard: super::battle_action::Guard,
    ) -> Result<(), TransportError> {
        if !self.supports_battle_actions()
            || self.action_guard.is_some()
            || self.action_recovery.is_some()
        {
            return Err(TransportError::InvalidState(
                "battle action proof was not consumed",
            ));
        }
        self.action_guard = Some(guard);
        Ok(())
    }
    pub(crate) fn take_action_recovery(&mut self) -> Option<Response> {
        self.action_recovery.take()
    }
    fn accept_action_recovery(&mut self, response: Response) -> Result<(), TransportError> {
        // Validate monotonic authority BEFORE changing any cached component.
        let valid = match &response {
            Response::Npc(r) => {
                npc::follows(self.npc.as_ref(), r)
                    && r.frame.as_ref().is_some_and(|f| f.resources.is_some())
                    && combat::follows(self.combat.as_ref(), &r.combat_reply())
                    && r.combat_reply()
                        .world_reply()
                        .as_ref()
                        .is_some_and(|w| world::follows(self.world.as_ref(), w))
            }
            Response::Combat(r) => {
                self.npc.is_none()
                    && combat::follows(self.combat.as_ref(), r)
                    && r.world_reply()
                        .as_ref()
                        .is_some_and(|w| world::follows(self.world.as_ref(), w))
            }
            Response::World(r) => {
                self.npc.is_none()
                    && self.combat.is_none()
                    && world::follows(self.world.as_ref(), r)
            }
            _ => false,
        };
        if !valid {
            return Err(application::error(
                application::ErrorKind::IdentityMismatch,
                "battle recovery regressed authority",
            )
            .into());
        }
        match &response {
            Response::Npc(r) => {
                let c = r.combat_reply();
                self.world = c.world_reply();
                self.combat = Some(c);
                self.npc = Some(r.clone());
            }
            Response::Combat(r) => {
                self.world = r.world_reply();
                self.combat = Some(r.clone());
            }
            Response::World(r) => {
                self.world = Some(r.clone());
            }
            _ => unreachable!(),
        }
        self.action_recovery = Some(response);
        Ok(())
    }
    pub fn supports_combat(&self) -> bool {
        self.ack()
            .is_some_and(|ack| ack.capabilities & super::protocol::CAP_COMBAT != 0)
    }
    pub fn combat_snapshot(&self) -> Option<&combat::Reply> {
        self.combat.as_ref()
    }
    pub fn supports_npc(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= npc::RESOURCE_VERSION && a.capabilities & super::protocol::CAP_NPC != 0
        })
    }
    pub fn npc_snapshot(&self) -> Option<&npc::Reply> {
        self.npc.as_ref()
    }
    pub fn supports_spells(&self) -> bool {
        self.ack().is_some_and(|ack| ack.version >= 4) && self.supports_combat()
    }
    pub fn supports_light_arrow(&self) -> bool {
        self.ack().is_some_and(|ack| ack.version >= 6) && self.supports_combat()
    }
    pub fn supports_record(&self) -> bool {
        self.ack().is_some_and(|ack| ack.version >= 5)
    }
    pub fn supports_shop(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 8 && a.capabilities & super::protocol::CAP_SHOP != 0)
    }
    pub fn supports_weight(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 9 && a.capabilities & super::protocol::CAP_WEIGHT != 0)
    }
    pub fn supports_timed(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 10 && a.capabilities & super::protocol::CAP_TIMED != 0)
    }
    pub fn supports_arrow(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 11 && a.capabilities & super::protocol::CAP_ARROW != 0)
    }
    pub fn supports_regen(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 12 && a.capabilities & super::protocol::CAP_REGEN != 0)
    }
    pub fn supports_honor(&self) -> bool {
        self.ack()
            .is_some_and(|a| a.version >= 13 && a.capabilities & super::protocol::CAP_HONOR != 0)
    }
    pub fn supports_fire_storm(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= 14 && a.capabilities & super::protocol::CAP_FIRE_STORM != 0
        })
    }
    pub fn supports_match_descriptor(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= 15 && a.capabilities & super::protocol::CAP_MATCH_DESCRIPTOR != 0
        })
    }
    pub fn select_ammo(&mut self, request: ammo::Request) -> Result<ammo::Reply, TransportError> {
        request.validate()?;
        if !self.supports_arrow()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel_id)
            })
            || !self
                .world
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| {
                    f.match_id == request.match_id
                        && f.generation == request.generation
                        && f.feature_enabled(world::MatchDescriptor::AMMO, true)
                })
        {
            return Err(TransportError::InvalidState(
                "ammunition requires the selected world",
            ));
        }
        if request.action == ammo::Action::Select
            && !self
                .combat
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| {
                    self.world.as_ref().and_then(|w| w.frame.as_ref()) == Some(&f.world)
                        && f.world
                            .actors
                            .iter()
                            .find(|a| a.matches_character(request.context.key))
                            .and_then(|a| f.ammo(a.entity_id))
                            .is_some_and(|s| {
                                s.request_sequence.checked_add(1) == Some(request.sequence)
                            })
                })
        {
            return Err(TransportError::InvalidState(
                "arrow selection requires current atomic sequence",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = ammo::encode_request(id, request, self.ack().unwrap().version)?;
        match self.exchange(&bytes, id, Stage::Ammo)? {
            Response::Ammo(reply)
                if reply.echo == request && reply.validate().is_ok() && reply.frame.is_none() =>
            {
                Ok(reply)
            }
            Response::Ammo(reply) if reply.confirms(&request, self.combat.as_ref()) => {
                let next = reply.combat_reply();
                self.world = next.world_reply();
                self.combat = Some(next);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "arrow selection reply changed identity or authoritative state",
                )
                .into())
            }
        }
    }
    pub fn shop_snapshot(&self) -> Option<&shop::Reply> {
        self.shop.as_ref()
    }
    pub fn shop(&mut self, request: shop::Request) -> Result<shop::Reply, TransportError> {
        request.validate()?;
        if !self.supports_shop()
            || self.principal.is_none()
            || self.world.as_ref().is_some_and(|w| w.frame.is_some())
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel_id)
            })
        {
            return Err(TransportError::InvalidState(
                "shop requires the selected lobby outside world",
            ));
        }
        if request.action != shop::Action::Read
            && !self
                .shop
                .as_ref()
                .and_then(|r| r.snapshot.as_deref())
                .is_some_and(|s| {
                    s.round_id == request.round_id
                        && s.revision == request.expected_revision
                        && s.token == request.token
                })
        {
            return Err(TransportError::InvalidState(
                "purchase requires the confirmed shop round and token",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = shop::encode_request(id, request, self.ack().unwrap().version)?;
        match self.exchange(&bytes, id, Stage::Shop)? {
            Response::Shop(reply)
                if reply.confirms(&request, self.shop.as_ref(), self.rooms.as_ref())
                    && reply
                        .snapshot
                        .as_ref()
                        .is_none_or(|s| !s.materials_enabled || self.supports_slow()) =>
            {
                if let Some(room) = &reply.room {
                    self.rooms = Some(room.clone());
                }
                if reply.snapshot.is_some() {
                    self.shop = Some(reply.clone());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "shop reply does not confirm transaction and selected identity",
                )
                .into())
            }
        }
    }
    pub fn use_item(&mut self, request: item::Request) -> Result<item::Reply, TransportError> {
        request.validate()?;
        if !self.supports_shop()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel_id)
            })
            || !self
                .world
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|w| {
                    w.match_id == request.match_id
                        && w.generation == request.generation
                        && w.feature_enabled(world::MatchDescriptor::ITEMS, true)
                })
        {
            return Err(TransportError::InvalidState(
                "item use requires the selected world",
            ));
        }
        if request.action == item::Action::Use {
            let current = self.combat.as_ref().and_then(|r| r.frame.as_ref());
            if !current.is_some_and(|f| {
                self.world.as_ref().and_then(|r| r.frame.as_ref()) == Some(&f.world)
                    && f.world
                        .actors
                        .iter()
                        .find(|a| a.matches_character(request.context.key))
                        .and_then(|a| f.item(a.entity_id))
                        .is_some_and(|s| {
                            s.request_sequence.checked_add(1) == Some(request.sequence)
                        })
            }) {
                return Err(TransportError::InvalidState(
                    "item use requires current atomic sequence",
                ));
            }
        }
        let id = self.next_request_id()?;
        let bytes = item::encode_request(id, request, self.ack().unwrap().version)?;
        match self.exchange(&bytes, id, Stage::Item)? {
            Response::Item(reply)
                if reply.echo == request && reply.validate().is_ok() && reply.frame.is_none() =>
            {
                Ok(reply)
            }
            Response::Item(reply)
                if reply.confirms(&request, self.combat.as_ref())
                    && reply
                        .combat_reply()
                        .world_reply()
                        .as_ref()
                        .is_some_and(|next| world::follows(self.world.as_ref(), next)) =>
            {
                let next = reply.combat_reply();
                self.world = next.world_reply();
                self.combat = Some(next);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "item reply does not confirm atomic use and health",
                )
                .into())
            }
        }
    }
    /// Central numeric-key totals, not inferred from a terminal match receipt.
    pub fn record(&mut self) -> Result<record::Reply, TransportError> {
        if !self.supports_record() || self.principal.is_none() {
            return Err(TransportError::InvalidState(
                "record requires negotiated v5 authentication",
            ));
        }
        let context = self
            .navigation
            .as_ref()
            .ok_or(TransportError::InvalidState(
                "record requires a selected character",
            ))?
            .context;
        let id = self.next_request_id()?;
        let bytes = record::encode_request(id, context)?;
        match self.exchange(&bytes, id, Stage::Record)? {
            Response::Record(reply) if reply.context == context => Ok(reply),
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "record reply context mismatch",
                )
                .into())
            }
        }
    }
    pub fn skills(&mut self) -> Result<spell::SkillsReply, TransportError> {
        if !self.supports_spells() || self.principal.is_none() {
            return Err(TransportError::InvalidState(
                "skills require negotiated v4 authentication",
            ));
        }
        let context = self
            .navigation
            .as_ref()
            .ok_or(TransportError::InvalidState(
                "skills require a selected character",
            ))?
            .context;
        let id = self.next_request_id()?;
        let bytes = spell::encode_skills(id, context)?;
        match self.exchange(&bytes, id, Stage::Skills)? {
            Response::Skills(reply)
                if reply.context == context
                    && self
                        .ack()
                        .is_some_and(|a| skill_catalog_negotiated(&reply, &a)) =>
            {
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "skill reply context mismatch",
                )
                .into())
            }
        }
    }
    pub fn cast(
        &mut self,
        request: spell::CastRequest,
    ) -> Result<spell::CastReply, TransportError> {
        request.validate()?;
        if !self.supports_spells()
            || self
                .world
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| !f.feature_enabled(world::MatchDescriptor::SPELLS, true))
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
        {
            return Err(TransportError::InvalidState(
                "spell requires this selected world",
            ));
        }
        let before = self.combat.as_ref().and_then(|r| r.frame.as_ref()).ok_or(
            TransportError::InvalidState("spell requires atomic combat state"),
        )?;
        let caster = before
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(request.context.key))
            .ok_or(TransportError::InvalidState("spell caster absent"))?;
        let required_feature = match request.skill_id {
            1 => Some(world::MatchDescriptor::HONOR),
            4 => Some(world::MatchDescriptor::LIGHT_ARROW),
            70 => Some(world::MatchDescriptor::FIRE_STORM),
            _ => None,
        };
        if required_feature.is_some_and(|feature| !before.world.feature_enabled(feature, true)) {
            return Err(TransportError::InvalidState(
                "spell is disabled in this match",
            ));
        }
        if self.world.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.world)
            || before.world.match_id != request.match_id
            || before.world.generation != request.generation
            || !before
                .world
                .cast(caster.entity_id)
                .is_some_and(|c| c.request_sequence.checked_add(1) == Some(request.sequence))
        {
            return Err(TransportError::InvalidState(
                "spell requires current world and cast sequence",
            ));
        }
        let before = before.clone();
        let id = self.next_request_id()?;
        let bytes = spell::encode_cast_version(
            id,
            &request,
            self.ack()
                .ok_or(TransportError::InvalidState(
                    "spell requires negotiated version",
                ))?
                .version,
        )?;
        match self.exchange(&bytes, id, Stage::Spell)? {
            Response::Cast(reply) if reply.confirms(&request) && reply.frame.is_none() => Ok(reply),
            Response::Cast(reply)
                if reply.confirms(&request)
                    && reply
                        .combat_reply()
                        .as_ref()
                        .is_some_and(|next| combat::follows(self.combat.as_ref(), next))
                    && spell::confirms_transition(&before, &reply) =>
            {
                let next = reply.combat_reply().unwrap();
                self.world = next.world_reply();
                self.combat = Some(next);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "spell reply identity or authority transition mismatch",
                )
                .into())
            }
        }
    }
    pub fn disintegrate(
        &mut self,
        request: disintegrate::Request,
    ) -> Result<disintegrate::Reply, TransportError> {
        request.validate()?;
        let ack = self.ack().ok_or(TransportError::InvalidState(
            "Disintegrate requires negotiation",
        ))?;
        let version = ack.version;
        let before = self.combat.as_ref().and_then(|r| r.frame.as_ref()).ok_or(
            TransportError::InvalidState("Disintegrate requires atomic combat state"),
        )?;
        if !disintegrate::negotiated(&ack)
            || !self.supports_spells()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self.world.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.world)
            || !request.ready_in(before)
        {
            return Err(TransportError::InvalidState(
                "Disintegrate requires selected v27 scope, life and cast sequence",
            ));
        }
        let before = before.clone();
        let id = self.next_request_id()?;
        let bytes = disintegrate::encode_request(id, &request, version)?;
        match self.exchange(&bytes, id, Stage::Spell)? {
            Response::Disintegrate(reply) if reply.confirms(&request) && reply.frame.is_none() => {
                Ok(reply)
            }
            Response::Disintegrate(reply)
                if reply.confirms(&request)
                    && disintegrate::confirms_transition(&before, &reply)
                    && combat::follows(self.combat.as_ref(), &reply.combat_reply()) =>
            {
                let next = reply.combat_reply();
                self.world = next.world_reply();
                self.combat = Some(next);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "Disintegrate reply identity or authority transition mismatch",
                )
                .into())
            }
        }
    }
    pub fn cast_npc(
        &mut self,
        request: npc_spell::Request,
    ) -> Result<npc_spell::Reply, TransportError> {
        request.validate()?;
        let version = self.ack().map_or(0, |ack| ack.version);
        let before = self.npc.as_ref().and_then(|r| r.frame.as_ref()).ok_or(
            TransportError::InvalidState("NPC spell requires an atomic NPC snapshot"),
        )?;
        if version < npc_spell::VERSION
            || !self.supports_npc()
            || !self.supports_spells()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self.world.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat.world)
            || self.combat.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat)
            || !request.ready_in(before)
        {
            return Err(TransportError::InvalidState(
                "NPC spell requires selected v24 scope, life and cast sequence",
            ));
        }
        let before = before.clone();
        let id = self.next_request_id()?;
        let bytes = npc_spell::encode_request(id, &request, version)?;
        match self.exchange(&bytes, id, Stage::Npc)? {
            Response::NpcSpell(reply)
                if reply.confirms(&request)
                    && npc_spell::confirms_transition(&before, &reply)
                    && npc::follows(self.npc.as_ref(), &reply.npc_reply())
                    && reply.frame.as_ref().is_some_and(|f| f.resources.is_some()) =>
            {
                let snapshot = reply.npc_reply();
                let combat = snapshot.combat_reply();
                self.world = combat.world_reply();
                self.combat = Some(combat);
                self.npc = Some(snapshot);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "NPC spell identity, resources or authority transition mismatch",
                )
                .into())
            }
        }
    }
    pub fn disintegrate_npc(
        &mut self,
        request: super::npc_disintegrate_protocol::Request,
    ) -> Result<super::npc_disintegrate_protocol::Reply, TransportError> {
        request.validate()?;
        let version = self.ack().map_or(0, |ack| ack.version);
        let before = self.npc.as_ref().and_then(|r| r.frame.as_ref()).ok_or(
            TransportError::InvalidState("NPC77 requires an atomic NPC snapshot"),
        )?;
        if !self
            .ack()
            .is_some_and(|ack| super::npc_disintegrate_protocol::negotiated(&ack))
            || version < super::npc_disintegrate_protocol::VERSION
            || !self.supports_npc()
            || !self.supports_spells()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self.world.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat.world)
            || self.combat.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat)
            || !request.ready_in(before)
        {
            return Err(TransportError::InvalidState(
                "NPC77 requires selected v29 scope, life and cast sequence",
            ));
        }
        let before = before.clone();
        let id = self.next_request_id()?;
        let bytes = super::npc_disintegrate_protocol::encode_request(id, &request, version)?;
        match self.exchange(&bytes, id, Stage::Npc)? {
            Response::NpcDisintegrate(reply)
                if reply.confirms(&request)
                    && super::npc_disintegrate_protocol::confirms_transition(&before, &reply)
                    && npc::follows(self.npc.as_ref(), &reply.npc_reply())
                    && reply
                        .frame
                        .as_ref()
                        .is_some_and(|f| f.resources.is_some() && f.names.is_some()) =>
            {
                let snapshot = reply.npc_reply();
                let combat = snapshot.combat_reply();
                self.world = combat.world_reply();
                self.combat = Some(combat);
                self.npc = Some(snapshot);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "NPC77 identity, resources or authority transition mismatch",
                )
                .into())
            }
        }
    }
    pub fn fire_storm_npc(
        &mut self,
        request: super::npc_fire_storm_protocol::Request,
    ) -> Result<super::npc_fire_storm_protocol::Reply, TransportError> {
        request.validate()?;
        let version = self.ack().map_or(0, |ack| ack.version);
        let before = self.npc.as_ref().and_then(|r| r.frame.as_ref()).ok_or(
            TransportError::InvalidState("NPC70 requires an atomic NPC snapshot"),
        )?;
        if !self
            .ack()
            .is_some_and(|ack| super::npc_fire_storm_protocol::negotiated(&ack))
            || version < super::npc_fire_storm_protocol::VERSION
            || !self.supports_npc()
            || !self.supports_spells()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self.world.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat.world)
            || self.combat.as_ref().and_then(|r| r.frame.as_ref()) != Some(&before.combat)
            || !request.ready_in(before)
        {
            return Err(TransportError::InvalidState(
                "NPC70 requires selected v31 scope, life and cast sequence",
            ));
        }
        let before = before.clone();
        let id = self.next_request_id()?;
        let bytes = super::npc_fire_storm_protocol::encode_request(id, &request, version)?;
        match self.exchange(&bytes, id, Stage::Npc)? {
            Response::NpcFireStorm(reply)
                if reply.confirms(&request)
                    && super::npc_fire_storm_protocol::confirms_transition(&before, &reply)
                    && npc::follows(self.npc.as_ref(), &reply.npc_reply())
                    && reply
                        .frame
                        .as_ref()
                        .is_some_and(|f| f.resources.is_some() && f.names.is_some()) =>
            {
                let snapshot = reply.npc_reply();
                let combat = snapshot.combat_reply();
                self.world = combat.world_reply();
                self.combat = Some(combat);
                self.npc = Some(snapshot);
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "NPC70 identity, resources or authority transition mismatch",
                )
                .into())
            }
        }
    }
    pub fn npc(&mut self, request: npc::Request) -> Result<npc::Reply, TransportError> {
        request.validate()?;
        let current = self.world.as_ref().and_then(|r| r.frame.as_ref());
        if !self.supports_npc()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || !current.is_some_and(|w| {
                w.match_id == request.world.match_id
                    && w.generation == request.world.generation
                    && w.room.summary.room_id == request.world.room_id
                    && w.descriptor
                        .is_some_and(|d| d.has(world::MatchDescriptor::NPC))
            })
        {
            return Err(TransportError::InvalidState(
                "NPC requires the selected v18 encounter",
            ));
        }
        if request.action == npc::Action::AttackObjective
            && !self.ack().is_some_and(|ack| ack.version >= 21)
        {
            return Err(TransportError::InvalidState("Quest attack requires v21"));
        }
        if request.action == npc::Action::ConfirmDeath
            && !self.ack().is_some_and(|ack| ack.version >= 22)
        {
            return Err(TransportError::InvalidState(
                "Quest death confirmation requires v22",
            ));
        }
        if request.action == npc::Action::AttackTower
            && (!self.ack().is_some_and(|ack| ack.version >= 25)
                || !current.is_some_and(|w| w.time_attack.is_some()))
        {
            return Err(TransportError::InvalidState(
                "cooperative tower attack requires v25",
            ));
        }
        if request.action != npc::Action::Read
            && !self
                .npc
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| {
                    Some(&f.combat.world) == current
                        && f.combat.world.phase == world::Phase::Active
                        && f.resources.is_some()
                        && match request.action {
                            npc::Action::Attack => request
                                .target()
                                .and_then(|t| f.actor(t))
                                .is_some_and(|n| n.alive() && n.life == request.life),
                            npc::Action::AttackObjective => request
                                .objective()
                                .and_then(|t| super::quest_protocol::target(&f.combat.world, t))
                                .is_some_and(|(_, a)| a.hp > 0),
                            npc::Action::ConfirmDeath => {
                                super::quest_life::life(&f.combat.world, request.serial)
                                    .is_some_and(|l| {
                                        l.dead()
                                            && l.incarnation() == request.life
                                            && l.death_revision as u64 == request.sequence
                                    })
                            }
                            npc::Action::Read => false,
                            npc::Action::AttackTower => f
                                .combat
                                .world
                                .time_attack
                                .as_ref()
                                .is_some_and(|d| d.tower_hp > 0),
                        }
                        && f.combat
                            .world
                            .actors
                            .iter()
                            .find(|a| a.matches_character(request.context.key))
                            .is_some_and(|a| {
                                f.combat.vital(a.entity_id).is_some_and(|v| {
                                    if request.action == npc::Action::ConfirmDeath {
                                        v.hp == 0 && a.entity_id == request.serial
                                    } else {
                                        v.hp > 0
                                            && f.combat
                                                .world
                                                .time_attack
                                                .as_ref()
                                                .is_none_or(|d| !d.departed(a.entity_id))
                                            && v.attack_sequence.checked_add(1)
                                                == Some(request.sequence)
                                    }
                                })
                            })
                })
        {
            return Err(TransportError::InvalidState(
                "NPC attack requires exact typed target/life and atomic sequence",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = npc::encode_request(id, &request)?;
        match self.exchange(&bytes, id, Stage::Npc)? {
            Response::Npc(reply)
                if reply.confirms(&request)
                    && npc::follows(self.npc.as_ref(), &reply)
                    && reply.frame.as_ref().is_none_or(|f| f.resources.is_some())
                    && reply
                        .combat_reply()
                        .world_reply()
                        .as_ref()
                        .is_none_or(|next| world::follows(self.world.as_ref(), next))
                    && combat::follows(self.combat.as_ref(), &reply.combat_reply()) =>
            {
                let next = reply.combat_reply();
                if let Some(w) = next.world_reply() {
                    self.world = Some(w);
                    self.combat = Some(next);
                }
                self.npc = Some(reply.clone());
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "NPC response authority/preflight mismatch",
                )
                .into())
            }
        }
    }
    pub fn siege(
        &mut self,
        request: super::siege_protocol::Request,
    ) -> Result<super::siege_protocol::Reply, TransportError> {
        use super::siege_protocol as siege;
        request.validate()?;
        let prior = self.combat.as_ref().filter(|r| {
            r.frame.as_ref().is_some_and(|f| {
                self.world.as_ref().and_then(|w| w.frame.as_ref()) == Some(&f.world)
                    && request.ready_in(f)
            })
        });
        if !self.ack().is_some_and(|a| a.version >= siege::VERSION)
            || !self.supports_match_descriptor()
            || !self.supports_combat()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || prior.is_none()
        {
            return Err(TransportError::InvalidState(
                "siege requires current atomic world and role",
            ));
        }
        let prior = prior.unwrap().clone();
        let id = self.next_request_id()?;
        let bytes = siege::encode_request(id, &request)?;
        match self.exchange(&bytes, id, Stage::Combat)? {
            Response::Siege(reply)
                if reply.confirms(&request, &prior)
                    && reply
                        .combat_reply()
                        .world_reply()
                        .as_ref()
                        .is_none_or(|next| world::follows(self.world.as_ref(), next)) =>
            {
                // A null denial is not a new world. Keep the last confirmed
                // authority available for the next read; never retry a mutation.
                if reply.frame.is_some() {
                    let combat = reply.combat_reply();
                    self.world = combat.world_reply();
                    self.combat = Some(combat);
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "siege response identity or authority mismatch",
                )
                .into())
            }
        }
    }
    pub fn combat(&mut self, request: combat::Request) -> Result<combat::Reply, TransportError> {
        request.validate()?;
        if !self.supports_combat()
            || self
                .world
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| !f.combat_enabled(true))
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || !self
                .world
                .as_ref()
                .and_then(|w| w.frame.as_ref())
                .is_some_and(|w| {
                    w.match_id == request.match_id && w.generation == request.generation
                })
        {
            return Err(TransportError::InvalidState(
                "combat requires this selected world",
            ));
        }
        if request.action == combat::Action::Attack {
            let valid = self
                .combat
                .as_ref()
                .and_then(|r| r.frame.as_ref())
                .is_some_and(|f| {
                    let self_actor = f
                        .world
                        .actors
                        .iter()
                        .find(|a| a.matches_character(request.context.key));
                    self.world.as_ref().and_then(|w| w.frame.as_ref()) == Some(&f.world)
                        && self_actor.is_some_and(|a| {
                            a.entity_id != request.target_entity_id
                                && super::honor_protocol::target(f, request.target_entity_id)
                                    .is_some()
                                && f.vital(a.entity_id).is_some_and(|v| {
                                    v.attack_sequence.checked_add(1) == Some(request.sequence)
                                })
                        })
                });
            if !valid {
                return Err(TransportError::InvalidState(
                    "attack requires current atomic combat sequence and target",
                ));
            }
        }
        let id = self.next_request_id()?;
        let bytes = combat::encode_request(id, &request)?;
        match self.exchange(&bytes, id, Stage::Combat)? {
            Response::Combat(reply)
                if reply.confirms(&request)
                    && combat::follows(self.combat.as_ref(), &reply)
                    && reply
                        .world_reply()
                        .as_ref()
                        .is_none_or(|next| world::follows(self.world.as_ref(), next))
                    && combat_denial_preserves_attack_event(self.combat.as_ref(), &reply)
                    && ammo::confirms_attack(self.combat.as_ref(), &reply) =>
            {
                if let Some(next) = reply.world_reply() {
                    self.world = Some(next)
                }
                self.combat = Some(reply.clone());
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "combat response identity or authority sequence mismatch",
                )
                .into())
            }
        }
    }
    pub fn supports_battle_data(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_read::negotiated)
    }
    pub(crate) fn supports_battle_operations(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_operation::negotiated)
    }
    pub(crate) fn supports_battle_world(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_world::negotiated)
    }
    pub(crate) fn supports_battle_equipment(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_equipment::negotiated)
    }
    pub(crate) fn supports_player_healing(&self) -> bool {
        self.negotiated_selection()
            .is_some_and(super::battle_player_healing::negotiated)
    }
    /// Same authenticated TLS owner and confirmed scene. Invalid local intents
    /// do not send or retire the connection; uncertain exchanges never retry.
    pub(crate) fn player_healing(
        &mut self,
        mut request: super::battle_player_healing::Intent,
    ) -> Result<super::battle_player_healing::Reply, TransportError> {
        let s = request.stamp;
        if !self.supports_player_healing()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self
                .battle_state
                .admits_operation(super::battle_action::Guard {
                    stamp: s,
                    target: None,
                })
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "player healing requires negotiated selected lobby",
            ));
        }
        let before = self
            .battle_state
            .snapshot()
            .and_then(|s| s.observation())
            .ok_or(TransportError::InvalidState(
                "player healing requires confirmed world",
            ))?
            .clone();
        if !request.current(&before) {
            return Err(TransportError::InvalidState(
                "player healing requires current painted proof",
            ));
        }
        let peer = self.negotiated_selection().unwrap();
        request.request_id = self.next_request_id()?;
        let mut pending = super::battle_player_healing::LivePending::begin(&before, request, peer)?;
        let frame = super::battle_player_healing::encode_intent_selected(pending.intent(), peer)?;
        match self.exchange(&frame, request.request_id, Stage::PlayerHealing)? {
            Response::PlayerHealing(reply) => {
                if let Err(error) = pending.accept(&mut self.battle_state, &reply, peer) {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "player healing response mismatch",
                ))
            }
        }
    }
    pub(crate) fn supports_greater_haste(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_greater_haste::negotiated)
    }
    /// Exact pending intent, atomic chronology, no optimistic effect or retry.
    pub(crate) fn greater_haste(
        &mut self,
        mut request: super::battle_greater_haste::Intent,
    ) -> Result<super::battle_greater_haste::CandidateReply, TransportError> {
        let s = request.stamp;
        if !self.supports_greater_haste()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "greater haste requires negotiated selected lobby",
            ));
        }
        let before = self
            .battle_state
            .snapshot()
            .and_then(|s| s.observation())
            .ok_or(TransportError::InvalidState(
                "greater haste requires confirmed world",
            ))?;
        request.validate_before(before.data())?;
        let actor = before
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or(TransportError::InvalidState(
                "greater haste actor unavailable",
            ))?;
        if actor.spells.sequence.checked_add(1) != Some(request.sequence) {
            return Err(TransportError::InvalidState(
                "greater haste sequence unavailable",
            ));
        }
        request.request_id = self.next_request_id()?;
        let frame =
            super::battle_greater_haste::encode_intent(&request, self.ack().unwrap().version)?;
        match self.exchange(&frame, request.request_id, Stage::GreaterHaste)? {
            Response::GreaterHaste(reply) => {
                if let Err(error) = self.battle_state.accept_greater_haste(
                    &reply,
                    &request,
                    self.negotiated_selection().unwrap(),
                ) {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "greater haste response mismatch",
                ))
            }
        }
    }
    pub(crate) fn supports_target_haste(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_target_haste::negotiated)
    }
    /// Exact pending intent, atomic chronology, no optimistic effect or retry.
    pub(crate) fn target_haste(
        &mut self,
        mut request: super::battle_target_haste::Intent,
    ) -> Result<super::battle_target_haste::CandidateReply, TransportError> {
        let s = request.stamp;
        if !self.supports_target_haste()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "target haste requires negotiated selected lobby",
            ));
        }
        let before = self
            .battle_state
            .snapshot()
            .and_then(|s| s.observation())
            .ok_or(TransportError::InvalidState(
                "target haste requires confirmed world",
            ))?;
        request.validate_before(before.data())?;
        let actor = before
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or(TransportError::InvalidState(
                "target haste actor unavailable",
            ))?;
        if actor.spells.sequence.checked_add(1) != Some(request.sequence) {
            return Err(TransportError::InvalidState(
                "target haste sequence unavailable",
            ));
        }
        request.request_id = self.next_request_id()?;
        let frame =
            super::battle_target_haste::encode_intent(&request, self.ack().unwrap().version)?;
        match self.exchange(&frame, request.request_id, Stage::TargetHaste)? {
            Response::TargetHaste(reply) => {
                if let Err(error) = self.battle_state.accept_target_haste(
                    &reply,
                    &request,
                    self.negotiated_selection().unwrap(),
                ) {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "target haste response mismatch",
                ))
            }
        }
    }
    pub(crate) fn supports_slow(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_slow::negotiated)
    }
    /// Exact pending intent, atomic chronology, no optimistic effect or retry.
    pub(crate) fn slow(
        &mut self,
        mut request: super::battle_slow::Intent,
    ) -> Result<super::battle_slow::CandidateReply, TransportError> {
        let s = request.stamp;
        if !self.supports_slow()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "slow requires negotiated selected lobby",
            ));
        }
        let before = self
            .battle_state
            .snapshot()
            .and_then(|s| s.observation())
            .ok_or(TransportError::InvalidState(
                "slow requires confirmed world",
            ))?;
        request.validate_before(before.data())?;
        let actor = before
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or(TransportError::InvalidState("slow actor unavailable"))?;
        if actor.spells.sequence.checked_add(1) != Some(request.sequence) {
            return Err(TransportError::InvalidState("slow sequence unavailable"));
        }
        request.request_id = self.next_request_id()?;
        let frame = super::battle_slow::encode_intent(&request, self.ack().unwrap().version)?;
        match self.exchange(&frame, request.request_id, Stage::Slow)? {
            Response::Slow(reply) => {
                if let Err(error) = self.battle_state.accept_slow(
                    &reply,
                    &request,
                    self.negotiated_selection().unwrap(),
                ) {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState("slow response mismatch"))
            }
        }
    }
    /// No automatic retry, optimistic cache update, or synthetic legacy frame.
    /// The response's complete world and lifecycle are admitted atomically.
    pub(crate) fn battle_equipment(
        &mut self,
        request: super::battle_equipment::Request,
    ) -> Result<super::battle_equipment::Reply, TransportError> {
        request.validate()?;
        let s = request.stamp;
        if !self.supports_battle_equipment()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self
                .battle_state
                .admits_operation(super::battle_action::Guard {
                    stamp: s,
                    target: None,
                })
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "live equipment requires negotiated selected lobby",
            ));
        }
        let id = self.next_request_id()?;
        let frame =
            super::battle_equipment::encode_request(id, &request, self.ack().unwrap().version)?;
        match self.exchange(&frame, id, Stage::BattleEquipment)? {
            Response::BattleEquipment(reply) if reply.echo == request => {
                if let Err(error) = self
                    .battle_state
                    .accept_equipment(&reply, self.negotiated_selection().unwrap())
                {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "live equipment response mismatch",
                ))
            }
        }
    }
    pub(crate) fn supports_battle_revival(&self) -> bool {
        self.ack()
            .as_ref()
            .is_some_and(super::battle_revival::negotiated)
    }
    pub(crate) fn battle_revival(
        &mut self,
        request: super::battle_revival::Request,
    ) -> Result<super::battle_revival::Reply, TransportError> {
        request.validate()?;
        let s = request.stamp;
        if !self.supports_battle_revival()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self
                .battle_state
                .admits_operation(super::battle_action::Guard {
                    stamp: s,
                    target: None,
                })
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "death confirmation requires negotiated selected lobby",
            ));
        }
        let id = self.next_request_id()?;
        let frame =
            super::battle_revival::encode_request(id, &request, self.ack().unwrap().version)?;
        match self.exchange(&frame, id, Stage::BattleRevival)? {
            Response::BattleRevival(reply) if reply.echo == request => {
                if let Err(error) = self
                    .battle_state
                    .accept_revival(&reply, self.negotiated_selection().unwrap())
                {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "death confirmation response mismatch",
                ))
            }
        }
    }
    pub(crate) fn battle_snapshot(&self) -> Option<&Arc<super::battle_state::Snapshot>> {
        self.battle_state.snapshot()
    }
    pub(crate) fn supports_vip_world(&self) -> bool {
        self.negotiated_selection()
            .is_some_and(super::battle_vip_world::negotiated)
    }
    /// Explicit VIP lifecycle on this authenticated TLS owner. Uses the same
    /// bounded, cancellable exchange and atomic service chronology as ordinary
    /// traffic; no retry, legacy frame, renderer or LOAD permission is created.
    pub(crate) fn vip_world(
        &mut self,
        request: super::battle_world::Request,
    ) -> Result<super::battle_vip_world::Reply, TransportError> {
        request.validate()?;
        if !self.supports_vip_world()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.battle_state.admits_vip_world(&request)
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "full VIP lifecycle requires negotiated selected lobby",
            ));
        }
        let peer = self.negotiated_selection().unwrap();
        let id = self.next_request_id()?;
        let frame = super::battle_vip_world::encode_request(id, &request, peer)?;
        match self.exchange(&frame, id, Stage::VipWorld)? {
            Response::VipWorld(reply) if reply.echo() == &request => {
                if let Err(error) = self.battle_state.accept_vip_world(&reply, peer) {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "full VIP lifecycle response mismatch",
                ))
            }
        }
    }
    pub(crate) fn vip_equipment(
        &mut self,
        request: super::battle_vip_equipment::Request,
    ) -> Result<
        (
            super::battle_vip_equipment::Reply,
            Option<super::battle_vip_equipment::Quote>,
        ),
        TransportError,
    > {
        let s = request.stamp;
        if !self.supports_vip_world()
            || !self
                .negotiated_selection()
                .is_some_and(|p| p.allows_vip_equipment())
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.battle_state.admits_vip_equipment(&request)
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "full VIP equipment requires negotiated selected attachment",
            ));
        }
        let peer = self.negotiated_selection().unwrap();
        let id = self.next_request_id()?;
        let frame = super::battle_vip_equipment::encode_request(id, &request, peer)?;
        match self.exchange(&frame, id, Stage::VipEquipment)? {
            Response::VipEquipment(reply) if reply.echo == request => {
                match self
                    .battle_state
                    .accept_vip_equipment(&reply, &request, peer)
                {
                    Ok(quote) => Ok((reply, quote)),
                    Err(error) => {
                        self.abort();
                        Err(error.into())
                    }
                }
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "full VIP equipment response mismatch",
                ))
            }
        }
    }
    /// Full service/engine chronology is confirmed atomically before returning
    /// the receipt. No renderer, LOAD permission or legacy cache is fabricated.
    pub(crate) fn battle_world(
        &mut self,
        request: super::battle_world::Request,
    ) -> Result<super::battle_world::Reply, TransportError> {
        request.validate()?;
        if !self.supports_battle_world()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || !self.battle_state.admits_world(&request)
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "canonical lifecycle requires negotiated selected lobby",
            ));
        }
        let id = self.next_request_id()?;
        let frame = super::battle_world::encode_request(id, &request, self.ack().unwrap().version)?;
        match self.exchange(&frame, id, Stage::BattleWorld)? {
            Response::BattleWorld(reply) if reply.echo == request => {
                if let Err(error) = self
                    .battle_state
                    .accept_world(&reply, self.negotiated_selection().unwrap())
                {
                    self.abort();
                    return Err(error.into());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "canonical lifecycle response mismatch",
                ))
            }
        }
    }
    /// Same authenticated connection and painted life proof. Returns a full
    /// canonical receipt without fabricating legacy caches or retrying a SET.
    pub(crate) fn battle_operation(
        &mut self,
        guard: super::battle_action::Guard,
        build: impl FnOnce(u32, u16) -> Result<Vec<u8>, application::ApplicationError>,
    ) -> Result<super::battle_operation::Reply, TransportError> {
        let s = guard.stamp;
        if !self.supports_battle_operations()
            || self.principal.is_none()
            || self.action_guard.is_some()
            || self.supports_battle_world() && !self.battle_state.admits_operation(guard)
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == s.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(s.channel)
            })
        {
            return Err(TransportError::InvalidState(
                "canonical operation requires negotiated selected lobby",
            ));
        }
        let version = self.ack().unwrap().version;
        let id = self.next_request_id()?;
        let inner = build(id, version)?;
        let frame = super::battle_operation::encode(guard, &inner, version)?;
        if frame[8..12] != id.to_be_bytes() {
            return Err(TransportError::InvalidState(
                "canonical command id mismatch",
            ));
        }
        match self.exchange_with_operation(&frame, id, Stage::BattleOperation, Some(guard))? {
            Response::BattleOperation(reply) => {
                if self.supports_battle_world() {
                    if let Err(error) = self
                        .battle_state
                        .accept_operation(&reply, self.negotiated_selection().unwrap())
                    {
                        self.abort();
                        return Err(error.into());
                    }
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(TransportError::InvalidState(
                    "canonical command response mismatch",
                ))
            }
        }
    }
    /// Read the canonical service publication without touching legacy caches.
    /// The caller receives unadmitted typed data, not a render/command permit.
    pub fn read_battle(
        &mut self,
        request: super::battle_read::Request,
    ) -> Result<super::battle_read::UnvalidatedReply, TransportError> {
        if self.supports_battle_world() {
            return Err(TransportError::InvalidState(
                "v39 requires canonical lifecycle endpoint",
            ));
        }
        request.validate()?;
        if !self.supports_battle_data()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel_id)
            })
        {
            return Err(TransportError::InvalidState(
                "battle read requires negotiated selected lobby",
            ));
        }
        let id = self.next_request_id()?;
        let frame = super::battle_read::encode_request(id, &request)?;
        match self.exchange(&frame, id, Stage::BattleData)? {
            Response::BattleData(reply) if reply.echo == request => Ok(reply),
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "battle data does not confirm selected source and match",
                )
                .into())
            }
        }
    }
    /// Same authenticated TLS owner and selected channel. No automatic mutation retries.
    pub fn world(&mut self, request: world::Request) -> Result<world::Reply, TransportError> {
        if self.supports_battle_world() {
            return Err(TransportError::InvalidState(
                "v39 requires canonical lifecycle endpoint",
            ));
        }
        request.validate()?;
        if !self.supports_world()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
        {
            return Err(TransportError::InvalidState(
                "world request requires this selected lobby",
            ));
        }
        let id = self.next_request_id()?;
        let frame = world::encode_request(id, &request)?;
        match self.exchange(&frame, id, Stage::World)? {
            Response::World(reply)
                if reply.confirms(&request) && world::follows(self.world.as_ref(), &reply) =>
            {
                self.world = Some(reply.clone());
                if reply.frame.is_none() {
                    self.combat = None;
                    self.npc = None;
                }
                if self.npc.as_ref().is_some_and(|n| {
                    !reply.frame.as_ref().is_some_and(|f| {
                        n.echo.world.match_id == f.match_id
                            && n.echo.world.generation == f.generation
                            && n.echo.world.room_id == f.room.summary.room_id
                    })
                }) {
                    self.npc = None;
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "world response does not confirm request or monotonic match",
                )
                .into())
            }
        }
    }
    /// Room passwords never leave this ephemeral TLS-only request owner. Any
    /// uncertain mutation closes the connection; callers must not auto-resend.
    pub fn rooms(&mut self, request: rooms::Request) -> Result<rooms::Reply, TransportError> {
        request.validate()?;
        if self
            .rooms
            .as_ref()
            .is_some_and(|directory| !request.capacity_allowed(directory))
        {
            return Err(TransportError::InvalidState(
                "room capacity exceeds the confirmed map limit",
            ));
        }
        if !self.supports_rooms()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.common.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.common.channel_id)
            })
        {
            return Err(TransportError::InvalidState(
                "room requests require this selected lobby",
            ));
        }
        let expected = request.expectation();
        let id = self.next_request_id()?;
        let frame = rooms::encode_request(id, &request)?;
        drop(request);
        match self.exchange(frame.bytes(), id, Stage::Rooms)? {
            Response::Rooms(reply)
                if expected.confirms(&reply) && rooms::follows(self.rooms.as_ref(), &reply) =>
            {
                if reply.status == rooms::Status::Ok
                    && matches!(
                        expected.common.action,
                        rooms::Action::Create | rooms::Action::Join | rooms::Action::Leave
                    )
                {
                    self.matchmaking_confirmation.retire();
                }
                self.rooms = Some(reply.clone());
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "room response does not confirm request or monotonic membership",
                )
                .into())
            }
        }
    }
    pub fn supports_matchmaking(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= super::matchmaking_protocol::VERSION
                && a.capabilities & super::protocol::CAP_MATCHMAKING != 0
        })
    }
    /// Never auto-replay START or a poll that may have committed a cohort.
    /// A matching reply atomically updates the same authoritative room cache.
    pub fn matchmaking(
        &mut self,
        request: super::matchmaking_protocol::Request,
    ) -> Result<super::matchmaking_protocol::Reply, TransportError> {
        use super::matchmaking_protocol as matching;
        request.validate()?;
        if !self.supports_matchmaking()
            || self.principal.is_none()
            || !self.matchmaking_confirmation.permits(&request)
            || !self.navigation.as_ref().is_some_and(|n| {
                n.context == request.context
                    && n.snapshot.phase == NavigationPhase::Lobby
                    && n.snapshot.channel_id == Some(request.channel_id)
            })
            || request.action == matching::Action::Start
                && self.rooms.as_ref().is_none_or(|r| {
                    r.joined.is_some()
                        || !r
                            .options
                            .iter()
                            .any(|o| request.criteria.unwrap().accepts(o))
                })
        {
            return Err(TransportError::InvalidState(
                "matchmaking requires confirmed lobby, admission and qualified map",
            ));
        }
        let id = self.next_request_id()?;
        let frame = matching::encode_request(id, &request, self.ack().unwrap().version)?;
        match self.exchange(&frame, id, Stage::Matchmaking)? {
            Response::Matchmaking(reply)
                if reply.confirms(&request)
                    && reply
                        .snapshot
                        .as_ref()
                        .is_none_or(|s| rooms::follows(self.rooms.as_ref(), &s.rooms))
                    && self.matchmaking_confirmation.accept(&reply) =>
            {
                if let Some(s) = &reply.snapshot {
                    self.rooms = Some(s.rooms.clone());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "matchmaking reply does not confirm admission or room membership",
                )
                .into())
            }
        }
    }
    pub fn supports_chat(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= chat::VERSION && a.capabilities & super::protocol::CAP_CHAT != 0
        })
    }
    /// One bounded exchange, with no automatic resend. A caller must bootstrap
    /// a fresh token after navigation/room changes, then retain the returned cursor.
    pub fn chat(&mut self, request: chat::Request) -> Result<chat::Reply, TransportError> {
        request.validate()?;
        if self.supports_filtered_chat()
            || !self.supports_chat()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self
                .rooms
                .as_ref()
                .and_then(|r| r.joined.as_ref())
                .map_or(0, |r| r.summary.room_id)
                != request.room_id
        {
            return Err(TransportError::InvalidState(
                "chat requires the confirmed selected audience",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = chat::encode_request(id, &request, self.ack().unwrap().version)?;
        match self.exchange(&bytes, id, Stage::Chat)? {
            Response::Chat(reply)
                if reply.confirms(&request) && self.chat_confirmation.accept(&reply) =>
            {
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "chat response does not confirm the request",
                )
                .into())
            }
        }
    }
    pub fn supports_filtered_chat(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= super::filtered_chat_protocol::VERSION
                && a.capabilities & super::protocol::CAP_FILTERED_CHAT != 0
        })
    }
    /// Recipient-local v33 exchange. A confirmed admission and cursor are never
    /// inferred from the legacy shared log, and uncertain sends are not retried.
    pub fn filtered_chat(
        &mut self,
        request: super::filtered_chat_protocol::Request,
    ) -> Result<super::filtered_chat_protocol::Reply, TransportError> {
        request.validate()?;
        if !self.supports_filtered_chat()
            || self.principal.is_none()
            || !self.filtered_chat_confirmation.permits(&request)
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self
                .rooms
                .as_ref()
                .and_then(|r| r.joined.as_ref())
                .map_or(0, |r| r.summary.room_id)
                != request.room_id
        {
            return Err(TransportError::InvalidState(
                "filtered chat requires confirmed identity, audience and sequence",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = super::filtered_chat_protocol::encode_request(
            id,
            &request,
            self.ack().unwrap().version,
        )?;
        match self.exchange(&bytes, id, Stage::Chat)? {
            Response::FilteredChat(reply)
                if reply.confirms(&request) && self.filtered_chat_confirmation.accept(&reply) =>
            {
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "filtered chat response does not confirm the request",
                )
                .into())
            }
        }
    }
    pub fn supports_direct_chat(&self) -> bool {
        self.ack().is_some_and(|a| {
            a.version >= super::direct_chat_protocol::VERSION
                && a.capabilities & super::protocol::CAP_DIRECT_CHAT != 0
        })
    }
    /// One qualified private exchange; unknown mutation results are never retried.
    pub fn direct_chat(
        &mut self,
        request: super::direct_chat_protocol::Request,
    ) -> Result<super::direct_chat_protocol::Reply, TransportError> {
        request.validate()?;
        let moderated = self.supports_filtered_chat();
        let mut public = super::filtered_chat_protocol::Request::read(
            request.context,
            request.channel_id,
            request.room_id,
        );
        public.membership = request.membership;
        if !self.supports_direct_chat()
            || moderated && !self.filtered_chat_confirmation.permits(&public)
            || self.principal.is_none()
            || !self.direct_chat_confirmation.permits(&request)
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self
                .rooms
                .as_ref()
                .and_then(|r| r.joined.as_ref())
                .map_or(0, |r| r.summary.room_id)
                != request.room_id
        {
            return Err(TransportError::InvalidState(
                "direct chat requires confirmed identity, audience and sequence",
            ));
        }
        let id = self.next_request_id()?;
        let version = self.ack().unwrap().version;
        let bytes = if moderated {
            super::direct_chat_protocol::encode_moderated_request(id, &request, version)?
        } else {
            super::direct_chat_protocol::encode_request(id, &request, version)?
        };
        match (moderated, self.exchange(&bytes, id, Stage::DirectChat)?) {
            (false, Response::DirectChat(reply)) | (true, Response::ModeratedDirect(reply))
                if reply.confirms(&request) && self.direct_chat_confirmation.accept(&reply) =>
            {
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "direct chat response does not confirm the request",
                )
                .into())
            }
        }
    }
    /// Fresh server GM authorization; a bot never becomes a selection context.
    pub fn gm_bots(&mut self, request: gm::Request) -> Result<gm::Reply, TransportError> {
        request.validate()?;
        if !self.supports_gm_bots()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
            || self.world.as_ref().is_some_and(|r| r.frame.is_some())
            || !self
                .rooms
                .as_ref()
                .and_then(|r| r.joined.as_ref())
                .is_some_and(|r| {
                    r.summary.room_id == request.room_id
                        && r.summary.revision == request.expected_revision
                        && r.member(request.context.key).is_some()
                })
        {
            return Err(TransportError::InvalidState(
                "GM request requires the confirmed selected room outside world",
            ));
        }
        let id = self.next_request_id()?;
        let bytes = gm::encode_request(id, request, self.ack().unwrap().version)?;
        match self.exchange(&bytes, id, Stage::GmBots)? {
            Response::GmBots(reply) if reply.confirms(&request, self.rooms.as_ref()) => {
                if let Some(room) = &reply.room {
                    self.rooms = Some(room.clone());
                }
                Ok(reply)
            }
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "GM response does not confirm request or tagged room identity",
                )
                .into())
            }
        }
    }
    /// Authority READ/SET share this verified TLS owner. No automatic resend is
    /// safe after an uncertain SET: exchange failures abort and reconnect must READ.
    pub fn equipment(
        &mut self,
        request: EquipmentRequest,
    ) -> Result<EquipmentReply, TransportError> {
        request.validate()?;
        if !self.supports_equipment()
            || self.principal.is_none()
            || !self.navigation.as_ref().is_some_and(|prior| {
                prior.context == request.context
                    && matches!(
                        prior.snapshot.phase,
                        NavigationPhase::Channels | NavigationPhase::Lobby
                    )
            })
        {
            return Err(TransportError::InvalidState(
                "selected equipment capability is required",
            ));
        }
        let request_id = self.next_request_id()?;
        let frame = equipment::encode_request(request_id, request)?;
        match self.exchange(&frame, request_id, Stage::Equipment)? {
            Response::Equipment(reply) if reply.request == request => Ok(reply),
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "equipment reply does not confirm the complete request",
                )
                .into())
            }
        }
    }
    /// Read only source-persisted character/items. A successful query does not
    /// update the navigation snapshot or authorize any item/equipment mutation.
    pub fn character_data(&mut self) -> Result<CharacterDataReply, TransportError> {
        if !self.supports_character_data() || self.principal.is_none() {
            return Err(TransportError::InvalidState(
                "character data capability was not negotiated",
            ));
        }
        let context = self
            .navigation
            .as_ref()
            .ok_or(TransportError::InvalidState(
                "select a character before reading its data",
            ))?
            .context;
        let request_id = self.next_request_id()?;
        let frame = character_data::encode_request(request_id, context)?;
        match self.exchange(&frame, request_id, Stage::CharacterData)? {
            Response::CharacterData(reply) if reply.context == context => Ok(reply),
            _ => {
                self.abort();
                Err(application::error(
                    application::ErrorKind::IdentityMismatch,
                    "character data reply does not confirm the requested source, object and epoch",
                )
                .into())
            }
        }
    }
    fn navigate(
        &mut self,
        action: NavigationAction,
        context: SelectionContext,
        channel: Option<u8>,
    ) -> Result<NavigationReply, TransportError> {
        if matches!(
            action,
            NavigationAction::LeaveLobby | NavigationAction::ReturnCharacters
        ) && self
            .rooms
            .as_ref()
            .is_some_and(|reply| reply.joined.is_some())
        {
            return Err(TransportError::InvalidState(
                "leave the room before leaving this lobby",
            ));
        }
        if !self
            .principal
            .as_ref()
            .is_some_and(|principal| principal.authorizes_source(context.key.source_id))
            || !self.ack().is_some_and(|ack| {
                ack.capabilities & super::protocol::CAP_TOURNAMENT_NAVIGATION != 0
            })
        {
            return Err(TransportError::InvalidState(
                "authenticated navigation capability is required",
            ));
        }
        let request_id = self.next_request_id()?;
        let frame = application::encode_navigation(request_id, action, context, channel)?;
        let response = self.exchange(&frame, request_id, Stage::Navigation)?;
        let valid = match &response {
            Response::Navigation(reply) if reply.action == action && reply.context == context => {
                match action {
                    NavigationAction::EnterChannel if reply.status == NavigationStatus::Ok => {
                        reply.snapshot.channel_id == channel
                    }
                    NavigationAction::EnterChannel => {
                        self.navigation.as_ref().is_some_and(|prior| {
                            reply.snapshot.phase == prior.snapshot.phase
                                && reply.snapshot.channel_id == prior.snapshot.channel_id
                        })
                    }
                    NavigationAction::LeaveLobby => {
                        confirms_lobby_leave(self.navigation.as_ref(), reply)
                    }
                    NavigationAction::ReturnCharacters
                        if reply.status == NavigationStatus::Busy =>
                    {
                        confirms_busy_return(self.navigation.as_ref(), reply)
                    }
                    _ => true,
                }
            }
            _ => false,
        };
        if !valid {
            self.abort();
            return Err(application::error(
                application::ErrorKind::IdentityMismatch,
                "navigation reply does not confirm the requested identity and transition",
            )
            .into());
        }
        let Response::Navigation(reply) = response else {
            unreachable!("validated navigation response")
        };
        // Never discard a successful mutation ACK as an obsolete read. The owner
        // serializes navigation, and any uncertain exchange already aborts TCP.
        self.navigation = if reply.snapshot.phase == NavigationPhase::Characters {
            None
        } else {
            Some(reply.clone())
        };
        if action == NavigationAction::ReturnCharacters && reply.status == NavigationStatus::Ok {
            self.last_roster = None;
        }
        if reply.status == NavigationStatus::Ok {
            self.battle_state.retire();
            self.direct_chat_confirmation.reset_inbox();
            self.matchmaking_confirmation.retire();
            self.shop = None;
            self.rooms = None;
            self.world = None;
            self.combat = None;
            self.npc = None;
        }
        Ok(reply)
    }

    /// Liveness of this one exchange only, allowed before or after AUTH. The
    /// owning worker schedules heartbeat deadlines independently of UI paint.
    pub fn ping(&mut self) -> Result<(), TransportError> {
        let request_id = self.next_request_id()?;
        let frame = application::encode_ping(request_id)?;
        match self.exchange(&frame, request_id, Stage::Ping)? {
            Response::Pong => Ok(()),
            _ => {
                self.abort();
                Err(
                    application::error(application::ErrorKind::UnexpectedMessage, "expected PONG")
                        .into(),
                )
            }
        }
    }
    /// Best-effort close_notify with a hard 250ms I/O budget, then always close
    /// the TCP socket. Cancelled connections skip network waits entirely.
    pub fn close(&mut self) -> Result<(), TransportError> {
        let cancellation = self.cancellation.clone();
        self.close_with_token(&cancellation)
    }

    /// The session worker may call this ONLY after its command loop returns at
    /// a confirmed frame boundary. Its shared stop token is already cancelled,
    /// but no request remains in flight. A separate, bounded close-only token
    /// permits the TLS alert without reviving application I/O or acknowledging
    /// any pending mutation. Failed/uncertain exchanges keep their hard abort.
    pub(super) fn close_idle_worker(&mut self) -> Result<(), TransportError> {
        self.close_with_token(&Cancellation::new())
    }

    fn close_with_token(&mut self, cancellation: &Cancellation) -> Result<(), TransportError> {
        self.battle_state = super::battle_state::Tracker::default();
        self.shop = None;
        self.principal = None;
        self.last_roster = None;
        self.navigation = None;
        self.rooms = None;
        self.world = None;
        self.combat = None;
        self.npc = None;
        if self.session.state() == ClientState::Closed {
            return Ok(());
        }
        self.session.close();
        self.tls.send_close_notify();
        let result = if cancellation.is_cancelled() {
            Ok(())
        } else {
            let deadline = Instant::now() + CLOSE_TIMEOUT;
            let mut io = DeadlineIo::new(
                &mut self.socket,
                deadline,
                cancellation,
                self.poll_interval,
                Stage::Close,
                MAX_NEGOTIATION_WIRE_BYTES,
            );
            self.tls
                .complete_io(&mut io)
                .map(|_| ())
                .map_err(|e| io.map_io(e))
        };
        let shutdown = self.socket.shutdown(Shutdown::Both);
        result?;
        if let Err(error) = shutdown {
            if error.kind() != io::ErrorKind::NotConnected {
                return Err(TransportError::Io(error));
            }
        }
        Ok(())
    }
}
impl Drop for ModernTlsConnection {
    fn drop(&mut self) {
        self.battle_state = super::battle_state::Tracker::default();
        self.principal = None;
        self.last_roster = None;
        self.navigation = None;
        self.rooms = None;
        self.world = None;
        self.combat = None;
        self.npc = None;
        self.session.close();
        // No waiting, callbacks, logging or panic in the destructor.
        let _ = self.socket.shutdown(Shutdown::Both);
    }
}

/// Connect to a pre-resolved numeric endpoint. No synchronous DNS, settings,
/// credentials, fallback server, system trust-store mutation or native packet.
pub fn connect_and_negotiate(
    endpoint: SocketAddr,
    server_name: &str,
    trusted_root_der: Option<&[u8]>,
    hello: Hello,
    limits: TransportLimits,
    cancellation: Cancellation,
) -> Result<ModernTlsConnection, TransportError> {
    connect_and_negotiate_inner(
        endpoint,
        server_name,
        trusted_root_der,
        hello,
        None,
        limits,
        cancellation,
    )
}

/// Explicit extended handshake over the same verified TLS and deadline path.
/// Never retries a legacy handshake after a rejection or missing extension ACK.
pub fn connect_and_negotiate_extended(
    endpoint: SocketAddr,
    server_name: &str,
    trusted_root_der: Option<&[u8]>,
    hello: Hello,
    extensions: super::protocol::extensions::Offer,
    limits: TransportLimits,
    cancellation: Cancellation,
) -> Result<ModernTlsConnection, TransportError> {
    connect_and_negotiate_inner(
        endpoint,
        server_name,
        trusted_root_der,
        hello,
        Some(extensions),
        limits,
        cancellation,
    )
}

fn connect_and_negotiate_inner(
    endpoint: SocketAddr,
    server_name: &str,
    trusted_root_der: Option<&[u8]>,
    hello: Hello,
    extensions: Option<super::protocol::extensions::Offer>,
    limits: TransportLimits,
    cancellation: Cancellation,
) -> Result<ModernTlsConnection, TransportError> {
    let limits = limits.validate()?;
    let deadline = Instant::now()
        .checked_add(limits.total_timeout)
        .ok_or(TransportError::InvalidConfiguration("deadline overflow"))?;
    if cancellation.is_cancelled() {
        return Err(TransportError::Cancelled);
    }
    validate_server_name(server_name)?;
    if endpoint.port() == 0
        || endpoint.ip().is_unspecified()
        || endpoint.ip().is_multicast()
        || endpoint.ip() == IpAddr::V4(std::net::Ipv4Addr::BROADCAST)
        || matches!(endpoint, SocketAddr::V6(address) if address.scope_id() != 0 || address.flowinfo() != 0)
    {
        return Err(TransportError::InvalidConfiguration(
            "endpoint must be a concrete unicast address and nonzero port",
        ));
    }
    let mut session = match extensions {
        Some(e) => ModernClientSession::new_extended(hello, e)?,
        None => ModernClientSession::new(hello)?,
    };
    let config = client_config(trusted_root_der)?;
    let name = ServerName::try_from(server_name.to_ascii_lowercase())
        .map_err(|_| TransportError::InvalidConfiguration("invalid TLS server name"))?;
    let mut tls = ClientConnection::new(config, name).map_err(TransportError::Tls)?;
    tls.set_buffer_limit(Some(MAX_TLS_BUFFER_BYTES));
    let remaining = deadline.saturating_duration_since(Instant::now());
    if cancellation.is_cancelled() {
        return Err(TransportError::Cancelled);
    }
    if remaining.is_zero() {
        return Err(TransportError::DeadlineExceeded(Stage::Connect));
    }
    let connect = TcpStream::connect_timeout(&endpoint, remaining.min(limits.connect_timeout));
    if cancellation.is_cancelled() {
        return Err(TransportError::Cancelled);
    }
    let mut socket = connect.map_err(|error| {
        if matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ) {
            TransportError::DeadlineExceeded(Stage::Connect)
        } else {
            TransportError::Io(error)
        }
    })?;
    socket.set_nodelay(true).map_err(TransportError::Io)?;
    {
        let mut io = DeadlineIo::new(
            &mut socket,
            deadline,
            &cancellation,
            limits.io_poll_interval,
            Stage::TlsHandshake,
            MAX_TLS_HANDSHAKE_WIRE_BYTES,
        );
        tls.complete_io(&mut io).map_err(|error| io.map_io(error))?;
        io.check().map_err(|error| io.map_io(error))?;
    }
    if tls.is_handshaking() || tls.peer_certificates().is_none() {
        return Err(TransportError::InvalidConfiguration(
            "TLS did not establish verified server credentials",
        ));
    }
    let version = match tls.protocol_version() {
        Some(rustls::ProtocolVersion::TLSv1_2) => TlsVersion::Tls12,
        Some(rustls::ProtocolVersion::TLSv1_3) => TlsVersion::Tls13,
        _ => {
            return Err(TransportError::InvalidConfiguration(
                "TLS version must be 1.2 or 1.3",
            ));
        }
    };
    // Do not turn bytes sent before our HELLO into a response to it.
    match tls.reader().read(&mut [0u8; 1]) {
        Ok(0) => return Err(TransportError::Io(io::ErrorKind::UnexpectedEof.into())),
        Ok(_) => return Err(TransportError::UnexpectedEarlyApplicationData),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
        Err(error) => return Err(TransportError::Io(error)),
    }
    {
        let mut io = DeadlineIo::new(
            &mut socket,
            deadline,
            &cancellation,
            limits.io_poll_interval,
            Stage::Negotiation,
            MAX_NEGOTIATION_WIRE_BYTES,
        );
        io.check().map_err(|error| io.map_io(error))?;
        let hello_frame = session.begin()?;
        // Application bytes enter rustls only after verified TLS completion.
        tls.writer()
            .write_all(&hello_frame)
            .map_err(TransportError::Io)?;
        tls.complete_io(&mut io).map_err(|error| io.map_io(error))?;
        let result = session.accept_ack_from(&mut rustls::Stream::new(&mut tls, &mut io));
        result.map_err(|error| {
            io.last_abort
                .map(Abort::error)
                .unwrap_or(TransportError::Protocol(error))
        })?;
        io.check().map_err(|error| io.map_io(error))?;
    }
    Ok(ModernTlsConnection {
        socket,
        tls,
        session,
        version,
        cancellation,
        poll_interval: limits.io_poll_interval,
        exchange_timeout: Duration::from_secs(5),
        last_request_id: hello.request_id,
        authentication_attempted: false,
        principal: None,
        last_roster: None,
        navigation: None,
        rooms: None,
        matchmaking_confirmation: super::matchmaking_protocol::Confirmation::default(),
        chat_confirmation: chat::Confirmation::default(),
        filtered_chat_confirmation: super::filtered_chat_protocol::Confirmation::default(),
        direct_chat_confirmation: super::direct_chat_protocol::Confirmation::default(),
        world: None,
        combat: None,
        npc: None,
        battle_state: super::battle_state::Tracker::default(),
        action_guard: None,
        action_recovery: None,
        shop: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    mod battle_revival_interop_tests {
        include!("battle_revival_interop_tests.rs");
    }
    include!("transport_npc_fire_storm_tests.rs");
    include!("transport_npc_action_tests.rs");
    include!("transport_chat_tests.rs");
    include!("transport_direct_chat_tests.rs");
    include!("transport_filtered_chat_tests.rs");
    include!("transport_direct_chat_fault_tests.rs");
    include!("transport_direct_chat_state_tests.rs");
    include!("transport_chat_controller_tests.rs");
    include!("transport_close_controller_tests.rs");
    include!("transport_haste_read_tests.rs");
    include!("transport_extensions_tests.rs");
    include!("transport_player_healing_tests.rs");
    include!("transport_vip_world_tests.rs");
    mod profile_probe_tests {
        include!("profile_probe_tls_tests.rs");
    }
    #[test]
    fn denied_attack_sequence_may_be_consumed_but_never_creates_attack_animation() {
        let old = combat::tests::active_reply();
        let mut next = old.clone();
        next.echo.action = combat::Action::Attack;
        next.echo.sequence = 1;
        next.echo.target_entity_id = 2;
        next.status = combat::Status::Denied;
        next.reason = combat::Reason::TooFast;
        let f = next.frame.as_mut().unwrap();
        f.world.revision += 1;
        f.vitals[0].attack_sequence = 1;
        assert!(combat_denial_preserves_attack_event(Some(&old), &next));
        let f = next.frame.as_mut().unwrap();
        f.vitals[0].attack_revision = f.world.revision;
        f.vitals[0].last_target_entity_id = 2;
        assert!(!combat_denial_preserves_attack_event(Some(&old), &next));
        next.status = combat::Status::Ok;
        next.reason = combat::Reason::None;
        assert!(combat_denial_preserves_attack_event(Some(&old), &next));
        next.frame.as_mut().unwrap().vitals[0].attack_sequence = 2;
        assert!(!combat_denial_preserves_attack_event(Some(&old), &next));
    }
    use std::ffi::OsString;
    use std::io::{BufRead, BufReader, Cursor};
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::sync::mpsc;
    use std::thread;

    const MAX_EXPECTED_CHARACTER_DATA_BYTES: usize = 512 * 1024;

    fn read_expected_character_data(mut input: impl Read) -> Result<String, &'static str> {
        let mut bytes = Vec::new();
        input
            .by_ref()
            .take(MAX_EXPECTED_CHARACTER_DATA_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "cannot read independent character data")?;
        if bytes.len() > MAX_EXPECTED_CHARACTER_DATA_BYTES {
            return Err("independent character data exceeds its byte limit");
        }
        String::from_utf8(bytes).map_err(|_| "independent character data must be strict UTF-8")
    }

    // These two explicit inputs are alternatives, not a fallback chain. Never
    // report their values or filesystem errors: expected rows and paths do not
    // belong in failed-test diagnostics. Injecting the inputs also avoids tests
    // racing over process-wide environment variables.
    fn load_expected_character_data(
        inline: Option<OsString>,
        file: Option<OsString>,
    ) -> Result<String, &'static str> {
        match (inline, file) {
            (Some(_), Some(_)) => Err("independent character data inputs are mutually exclusive"),
            (None, None) => Err("independently probed source character data is required"),
            (Some(inline), None) => {
                let text = inline
                    .into_string()
                    .map_err(|_| "independent character data must be strict UTF-8")?;
                if text.len() > MAX_EXPECTED_CHARACTER_DATA_BYTES {
                    return Err("independent character data exceeds its byte limit");
                }
                Ok(text)
            }
            (None, Some(file)) => {
                let path = PathBuf::from(file);
                if !path.is_absolute() {
                    return Err("independent character data requires an absolute regular file");
                }
                let metadata = std::fs::symlink_metadata(&path)
                    .map_err(|_| "cannot inspect independent character data file")?;
                if !metadata.is_file() {
                    return Err("independent character data requires an absolute regular file");
                }
                let mut options = std::fs::OpenOptions::new();
                options.read(true);
                #[cfg(windows)]
                {
                    use std::os::windows::fs::OpenOptionsExt;
                    // Keep the opened fixture stable while checking its length
                    // and reading it; do not share write/delete access.
                    // FILE_SHARE_READ = 0x1. This std-only test reader does not
                    // enable an additional production windows crate feature.
                    options.share_mode(0x1);
                }
                let mut input = options
                    .open(&path)
                    .map_err(|_| "cannot open independent character data file")?;
                let opened = input
                    .metadata()
                    .map_err(|_| "cannot inspect opened independent character data file")?;
                if !opened.is_file() {
                    return Err("independent character data requires an absolute regular file");
                }
                if opened.len() > MAX_EXPECTED_CHARACTER_DATA_BYTES as u64 {
                    return Err("independent character data exceeds its byte limit");
                }
                read_expected_character_data(&mut input)
            }
        }
    }

    #[test]
    fn independent_character_data_inputs_reject_ambiguity_and_non_absolute_files() {
        assert_eq!(
            load_expected_character_data(None, None),
            Err("independently probed source character data is required")
        );
        assert_eq!(
            load_expected_character_data(
                Some("sensitive input".into()),
                Some("hidden path".into())
            ),
            Err("independent character data inputs are mutually exclusive")
        );
        assert_eq!(
            load_expected_character_data(None, Some("relative.json".into())),
            Err("independent character data requires an absolute regular file")
        );
        // The original JSON number literals are returned unchanged, without a
        // floating-point round trip or line-ending normalization.
        let text = "[9223372036854775807,-9223372036854775808]\r\n";
        assert_eq!(
            load_expected_character_data(Some(text.into()), None).unwrap(),
            text
        );
        assert!(load_expected_character_data(
            Some("x".repeat(MAX_EXPECTED_CHARACTER_DATA_BYTES + 1).into()),
            None
        )
        .is_err());
    }

    #[test]
    fn independent_character_data_reader_is_strict_and_stops_at_limit_plus_one() {
        assert!(read_expected_character_data(Cursor::new([0xff])).is_err());
        let exact = vec![b' '; MAX_EXPECTED_CHARACTER_DATA_BYTES];
        assert_eq!(
            read_expected_character_data(Cursor::new(&exact))
                .unwrap()
                .len(),
            MAX_EXPECTED_CHARACTER_DATA_BYTES
        );
        let mut oversized = Cursor::new(vec![b' '; MAX_EXPECTED_CHARACTER_DATA_BYTES + 32]);
        assert_eq!(
            read_expected_character_data(&mut oversized),
            Err("independent character data exceeds its byte limit")
        );
        assert_eq!(
            oversized.position(),
            MAX_EXPECTED_CHARACTER_DATA_BYTES as u64 + 1
        );
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("private path must never be reported"))
            }
        }
        assert_eq!(
            read_expected_character_data(FailingReader),
            Err("cannot read independent character data")
        );
    }

    #[test]
    fn dns_identity_rejects_urls_ips_wildcards_controls_and_ambiguous_labels() {
        for valid in [
            "localhost",
            "game.example.com",
            "xn--example-9za.test",
            "SERVER-1.EXAMPLE",
        ] {
            validate_server_name(valid).unwrap();
        }
        for bad in [
            "",
            "127.0.0.1",
            "::1",
            "https://game.test",
            "game.test:443",
            "*.test",
            "a..b",
            ".test",
            "test.",
            "-bad.test",
            "bad-.test",
            "with space",
            "with\0nul",
            "遊戲.test",
            "a/b",
            "a\\b",
        ] {
            assert!(validate_server_name(bad).is_err(), "{bad:?}");
        }
        assert!(validate_server_name(&"a".repeat(64)).is_err());
        assert!(validate_server_name(&format!(
            "{}.{}.{}.{}",
            "a".repeat(63),
            "b".repeat(63),
            "c".repeat(63),
            "d".repeat(63)
        ))
        .is_err());
    }

    #[test]
    fn invalid_limits_and_pre_cancelled_requests_do_not_connect() {
        for limits in [
            TransportLimits {
                connect_timeout: Duration::ZERO,
                ..TransportLimits::default()
            },
            TransportLimits {
                total_timeout: Duration::from_secs(61),
                ..TransportLimits::default()
            },
            TransportLimits {
                io_poll_interval: Duration::ZERO,
                ..TransportLimits::default()
            },
            TransportLimits {
                io_poll_interval: Duration::from_secs(1),
                ..TransportLimits::default()
            },
            TransportLimits {
                connect_timeout: Duration::from_secs(11),
                ..TransportLimits::default()
            },
        ] {
            assert!(limits.validate().is_err());
        }
        let cancellation = Cancellation::new();
        cancellation.clone().cancel();
        assert!(matches!(
            connect_and_negotiate(
                "127.0.0.1:1".parse().unwrap(),
                "localhost",
                None,
                Hello::current(1),
                TransportLimits::default(),
                cancellation
            ),
            Err(TransportError::Cancelled)
        ));
        for bad_root in [&[][..], &[0, 1, 2][..]] {
            assert!(client_config(Some(bad_root)).is_err());
        }
        assert!(client_config(Some(&vec![0; MAX_PRIVATE_CA_BYTES + 1])).is_err());
        let config = client_config(None).unwrap();
        assert!(!config.enable_early_data);
        for endpoint in [
            "127.0.0.1:0",
            "0.0.0.0:443",
            "224.0.0.1:443",
            "255.255.255.255:443",
            "[::]:443",
            "[ff02::1]:443",
            "[fe80::1%3]:443",
        ] {
            assert!(matches!(
                connect_and_negotiate(
                    endpoint.parse().unwrap(),
                    "localhost",
                    None,
                    Hello::current(1),
                    TransportLimits::default(),
                    Cancellation::new(),
                ),
                Err(TransportError::InvalidConfiguration(_))
            ));
        }
    }

    struct FakeIo {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
        chunk: usize,
        cancel_on_timeout: Option<Cancellation>,
    }
    impl TimedIo for FakeIo {
        fn set_timeout(&mut self, timeout: Duration) -> io::Result<()> {
            assert!(timeout > Duration::ZERO && timeout <= Duration::from_millis(100));
            if let Some(token) = self.cancel_on_timeout.take() {
                token.cancel();
            }
            Ok(())
        }
    }
    impl Read for FakeIo {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.chunk == 0 {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let length = output.len().min(self.chunk);
            self.input.read(&mut output[..length])
        }
    }
    impl Write for FakeIo {
        fn write(&mut self, input: &[u8]) -> io::Result<usize> {
            let length = input.len().min(self.chunk);
            self.output.extend_from_slice(&input[..length]);
            Ok(length)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    fn fake() -> FakeIo {
        FakeIo {
            input: Cursor::new(vec![1, 2, 3, 4, 5]),
            output: Vec::new(),
            chunk: 1,
            cancel_on_timeout: None,
        }
    }

    #[test]
    fn bounded_io_preserves_fragmented_bytes_and_refuses_excess_wire_data() {
        let token = Cancellation::new();
        let mut raw = fake();
        let mut io = DeadlineIo::new(
            &mut raw,
            Instant::now() + Duration::from_secs(1),
            &token,
            Duration::from_millis(100),
            Stage::Negotiation,
            4,
        );
        let mut bytes = [0; 4];
        io.read_exact(&mut bytes).unwrap();
        assert_eq!(bytes, [1, 2, 3, 4]);
        io.write_all(&bytes).unwrap();
        assert!(io.read(&mut [0; 1]).is_err());
        assert!(matches!(
            io.last_abort.unwrap().error(),
            TransportError::WireBudgetExceeded(Stage::Negotiation)
        ));
        assert!(io.write(&[5]).is_err());
        assert_eq!(raw.output, bytes);
        assert_eq!(raw.input.position(), 4);
    }

    #[test]
    fn deadline_and_cancellation_abort_read_exact_instead_of_retrying_interrupted() {
        let token = Cancellation::new();
        let mut raw = fake();
        let mut io = DeadlineIo::new(
            &mut raw,
            Instant::now(),
            &token,
            Duration::from_millis(100),
            Stage::TlsHandshake,
            4,
        );
        assert!(matches!(
            io.read(&mut [0; 1]).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        ));
        assert_eq!(raw.input.position(), 0);
        let token = Cancellation::new();
        let mut raw = fake();
        raw.chunk = 0;
        raw.cancel_on_timeout = Some(token.clone());
        let mut io = DeadlineIo::new(
            &mut raw,
            Instant::now() + Duration::from_secs(1),
            &token,
            Duration::from_millis(100),
            Stage::TlsHandshake,
            4,
        );
        let error = io.read_exact(&mut [0; 4]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted);
        assert!(matches!(io.map_io(error), TransportError::Cancelled));
    }

    // These opt-in tests launch only the test-owned ephemeral loopback peer.
    // Prepare its certificates with JDK 17 ModernTlsTestPeer under server target,
    // then set LITO_MODERN_TLS_TEST_FIXTURES, LITO_MODERN_TEST_JAVA_CLASSPATH and
    // LITO_MODERN_TEST_JAVA (explicit JDK 17 java.exe, not a PATH Java 8 fallback).
    // No system trust, game resources, account database or native game is changed.
    struct JavaPeer {
        child: Child,
        endpoint: SocketAddr,
        stdout: Option<BufReader<std::process::ChildStdout>>,
        process_job: Option<crate::modern_lito::world_integration_tests::OwnedProcessJob>,
    }
    impl JavaPeer {
        fn start(mode: &str) -> Self {
            Self::start_class("local.lito381.server.ModernTlsTestPeer", mode, "5000")
        }
        fn start_application(mode: &str) -> Self {
            Self::start_class(
                "local.lito381.server.ModernApplicationTestPeer",
                mode,
                "15000",
            )
        }
        fn start_class(class: &str, mode: &str, lifetime: &str) -> Self {
            let process_job = (class == "local.lito381.server.ModernVipMainSqlTestPeer")
                .then(crate::modern_lito::world_integration_tests::OwnedProcessJob::new);
            let directory = fixture_directory();
            let executable = std::env::var_os("LITO_MODERN_TEST_JAVA")
                .expect("set LITO_MODERN_TEST_JAVA to the explicit JDK 17 java executable");
            let classpath = std::env::var_os("LITO_MODERN_TEST_JAVA_CLASSPATH")
                .expect("set LITO_MODERN_TEST_JAVA_CLASSPATH to server test-classes + classes");
            let working_directory = if process_job.is_some() {
                let root = PathBuf::from(
                    std::env::var_os("LITO_MODERN_TEST_SERVER_ROOT")
                        .expect("set explicit server root for owned SQL schema files"),
                );
                assert!(root.is_absolute() && root.is_dir());
                root
            } else {
                std::env::current_dir().expect("existing test working directory")
            };
            let child = Command::new(executable)
                .current_dir(working_directory)
                .arg("-cp")
                .arg(classpath)
                .arg(class)
                .arg("serve")
                .arg(directory)
                .arg(mode)
                .arg(lifetime)
                // Real-DB opt-in credentials belong only to the Rust client and
                // travel over verified TLS, never as inherited peer environment.
                .env_remove("LITO_MODERN_PROBE_ACCOUNT")
                .env_remove("LITO_MODERN_PROBE_PASSWORD")
                .stdin(
                    if process_job.is_some()
                        || class == "local.lito381.server.ModernSessionFailureTestPeer"
                    {
                        Stdio::piped()
                    } else {
                        Stdio::null()
                    },
                )
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("launch test-owned Java TLS peer");
            let mut peer = Self {
                child,
                endpoint: "127.0.0.1:0".parse().unwrap(),
                stdout: None,
                process_job,
            };
            if let Some(job) = &peer.process_job {
                job.assign(&peer.child);
                peer.child
                    .stdin
                    .take()
                    .expect("owned SQL gate")
                    .write_all(b"GO\n")
                    .expect("release only Job-owned SQL descendants");
            }
            peer.stdout = Some(BufReader::new(peer.child.stdout.take().unwrap()));
            let line = peer.read_bounded_line("READY");
            let port = line
                .trim_end()
                .strip_prefix("READY ")
                .expect("unexpected Java READY output")
                .parse::<u16>()
                .expect("numeric Java READY port");
            assert_ne!(port, 0);
            peer.endpoint.set_port(port);
            peer
        }

        fn read_bounded_line(&mut self, stage: &str) -> String {
            let stdout = self.stdout.take().expect("owned peer stdout");
            let (sender, receiver) = mpsc::sync_channel(1);
            let reader = thread::spawn(move || {
                // Retain the same buffered reader for an optional commit barrier;
                // even a broken fixture cannot supply an unbounded status line.
                let mut limited = stdout.take(128);
                let mut line = String::new();
                let result = limited.read_line(&mut line).map(|_| line);
                let _ = sender.send((result, limited.into_inner()));
            });
            let seconds = if self.process_job.is_some() && stage == "READY" {
                20
            } else {
                5
            };
            let line = receiver.recv_timeout(Duration::from_secs(seconds));
            if line.is_err() {
                // Close our own child to release the owned stdout reader before
                // joining. No name-based process search or unrelated termination.
                self.terminate_owned();
            }
            reader.join().expect("owned peer status reader thread");
            let (line, stdout) = line.unwrap_or_else(|_| panic!("Java peer {stage} deadline"));
            self.stdout = Some(stdout);
            let line = line.expect("read bounded peer status");
            assert!(line.ends_with('\n'), "truncated Java {stage} line");
            line
        }

        fn expect_successful_exit(&mut self) {
            let deadline = Instant::now() + Duration::from_secs(6);
            loop {
                if let Some(status) = self.child.try_wait().expect("poll owned Java peer") {
                    assert!(status.success(), "Java peer exited with {status}");
                    if let Some(job) = &self.process_job {
                        while job.active_processes() != 0 {
                            assert!(
                                Instant::now() < deadline,
                                "SQL descendant did not retire normally"
                            );
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                    return;
                }
                assert!(Instant::now() < deadline, "Java peer did not exit on close");
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn terminate_owned(&mut self) {
            drop(self.process_job.take());
            if !matches!(self.child.try_wait(), Ok(Some(_))) {
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
    }
    impl Drop for JavaPeer {
        fn drop(&mut self) {
            self.terminate_owned();
        }
    }

    fn fixture_directory() -> PathBuf {
        let directory = PathBuf::from(
            std::env::var_os("LITO_MODERN_TLS_TEST_FIXTURES")
                .expect("set LITO_MODERN_TLS_TEST_FIXTURES to prepared server target fixtures"),
        );
        assert!(directory.is_absolute());
        directory
    }

    fn private_root() -> Vec<u8> {
        let file = std::fs::File::open(fixture_directory().join("root.der")).unwrap();
        let mut bytes = Vec::new();
        file.take(MAX_PRIVATE_CA_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(!bytes.is_empty() && bytes.len() <= MAX_PRIVATE_CA_BYTES);
        bytes
    }

    // Explicit test capability only. The runtime launcher still offers no NPC
    // capability, and this does not exercise account login or a production provider.
    #[test]
    #[ignore = "requires prepared Java 17 loopback NPC engine/codec TLS peer"]
    fn java_npc_v17_typed_engine_snapshots_attack_cooldown_and_abort_over_verified_tls() {
        use super::super::application_protocol::CharacterKey;
        use super::super::npc_protocol as npc;
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernNpcTestPeer", "npc-v17", "5000");
        let mut hello = Hello::current(100);
        // This historical peer selects v17: future optional features must not
        // become required merely because the current implementation knows them.
        hello.max_version = 17;
        hello.supported_capabilities =
            super::super::protocol::RUNTIME_CAPABILITIES | super::super::protocol::CAP_NPC;
        hello.required_capabilities = hello.supported_capabilities;
        let mut connection = connect_and_negotiate(
            peer.endpoint,
            "localhost",
            Some(&private_root()),
            hello,
            integration_limits(),
            Cancellation::new(),
        )
        .unwrap();
        assert_eq!(connection.ack().unwrap().version, 17);
        assert_ne!(
            connection.ack().unwrap().capabilities & super::super::protocol::CAP_NPC,
            0
        );
        let mut request = npc::Request {
            context: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 1,
                },
                epoch: 1,
            },
            channel_id: 0,
            action: npc::Action::Read,
            world: npc::Scope {
                room_id: 1,
                match_id: 1,
                generation: 6,
            },
            sequence: 0,
            serial: 0,
            life: 0,
        };
        let mut prior: Option<npc::Reply> = None;
        for index in 0..4 {
            if index == 1 || index == 2 {
                request.action = npc::Action::Attack;
                request.sequence = index;
                request.serial = 1;
                request.life = 1;
            } else {
                request.action = npc::Action::Read;
                request.sequence = 0;
                request.serial = 0;
                request.life = 0;
            }
            let id = connection.next_request_id().unwrap();
            let encoded = npc::encode_request(id, &request).unwrap();
            let Response::Npc(reply) = connection.exchange(&encoded, id, Stage::Combat).unwrap()
            else {
                panic!("NPC response");
            };
            assert!(reply.confirms(&request));
            assert!(npc::follows(prior.as_ref(), &reply));
            let frame = reply.frame.as_ref().unwrap();
            match index {
                0 => {
                    assert_eq!((frame.actors.len(), frame.exchanges.len()), (2, 2));
                    assert_eq!(frame.actors[0].hp, 9_007_199_254_740_993);
                    assert_eq!(frame.combat.vitals[0].hp, 9_007_199_254_740_965);
                    assert_eq!(frame.combat.vitals[2].hp, 12);
                    assert_eq!(
                        frame.exchanges[0].participant.kind,
                        super::super::participant::MemberKind::Human
                    );
                    assert_eq!(
                        frame.exchanges[1].participant.kind,
                        super::super::participant::MemberKind::ServerBot
                    );
                }
                1 => {
                    assert_eq!(reply.status, npc::Status::Ok);
                    assert_eq!(frame.actors[0].hp, 9_007_199_254_740_979);
                    assert_eq!(frame.exchanges.len(), 3);
                    assert_eq!(frame.combat.vitals[0].attack_sequence, 1);
                    assert_eq!(frame.combat.vitals[0].last_target_entity_id, 0);
                }
                2 => {
                    assert_eq!(reply.status, npc::Status::Denied);
                    assert_eq!(reply.reason, npc::Reason::TooFast);
                    assert_eq!(
                        frame.actors,
                        prior.as_ref().unwrap().frame.as_ref().unwrap().actors
                    );
                    assert_eq!(
                        frame.exchanges,
                        prior.as_ref().unwrap().frame.as_ref().unwrap().exchanges
                    );
                    assert_eq!(frame.combat.vitals[0].attack_sequence, 2);
                }
                _ => {
                    assert!(frame.closed);
                    assert!(frame.actors.is_empty());
                    assert_eq!(frame.combat.world.phase, world::Phase::Aborted);
                    assert!(frame.combat.world.completion.is_none());
                }
            }
            prior = Some(reply);
        }
        connection.close().unwrap();
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires Java 17 NPC service and real shared map554/PC/NPC/SPR/WAV; real controller/runtime gate, no HWND"]
    fn java_npc_v19_controllers_real_shared_preflight_loaded_attack_return_and_source_switch() {
        npc_shared_controller_flow(false, false, false, false, false);
    }

    #[test]
    #[ignore = "requires Java 17 staged Quest service and real shared map554/PC/NPC/SPR/WAV; no HWND or final objective provider"]
    fn java_quest_v20_controllers_real_shared_isolated_four_waves_and_no_false_victory() {
        npc_shared_controller_flow(true, false, false, false, false);
    }

    #[test]
    #[ignore = "requires Java 17 v21 Quest, real shared map554/PC/NPC/tower SPR/WAV and durable outbox; fixture profiles/store, no HWND"]
    fn java_quest_v21_controllers_real_shared_guardian_destruction_result_and_return() {
        npc_shared_controller_flow(true, true, false, false, false);
    }

    #[test]
    #[ignore = "requires Java 17 v22 Quest and real shared map554/PC/NPC/SPR/WAV; two real controllers, fixture profiles/store, no HWND"]
    fn java_quest_v22_controllers_real_shared_death_confirmation_respawn_result_and_return() {
        npc_shared_controller_flow(true, true, true, false, false);
    }

    #[test]
    #[ignore = "v29 real NPC77 two controllers, shared map554/SPR/717 preflight, cast/kill/retire/source switch; fixture profiles, no HWND/SQL"]
    fn java_npc77_v29_controllers_real_shared_cast_kill_retire_and_source_switch() {
        npc_shared_controller_flow(false, false, false, true, false);
    }
    fn npc_shared_controller_flow(
        quest: bool,
        objectives: bool,
        lives: bool,
        npc77: bool,
        npc70: bool,
    ) {
        npc_shared_controller_flow_with_chat(quest, objectives, lives, npc77, npc70, false);
    }
    #[test]
    #[ignore = "v32 chat with real NPC70 controllers, shared map554/SPR/710, loaded/active/cast/kill/return; fixture profiles, no HWND or SQL"]
    fn java_chat_world_shared_npc70_cast_and_return() {
        npc_shared_controller_flow_with_chat(false, false, false, false, true, true);
    }
    fn npc_shared_controller_flow_with_chat(
        quest: bool,
        objectives: bool,
        lives: bool,
        npc77: bool,
        npc70: bool,
        chat_enabled: bool,
    ) {
        npc_shared_controller_flow_with_conversations(
            quest,
            objectives,
            lives,
            npc77,
            npc70,
            chat_enabled,
            false,
            false,
        );
    }
    #[test]
    #[ignore = "v32 private chat with two real NPC70 controllers and shared map554/SPR710, loaded/active/cast/return; no HWND or SQL"]
    fn java_direct_chat_world_shared_npc70_cast_and_return() {
        npc_shared_controller_flow_with_conversations(
            false, false, false, false, true, true, true, false,
        );
    }
    #[test]
    #[ignore = "v37 real TLS/two controllers/shared map554/SPR/WAV; deferred pre-death command rejects after real Quest revival, then all waves/result/return; no HWND/SQL"]
    fn java_v37_life_bound_controller_deferred_action_rejects_refreshes_and_finishes() {
        npc_shared_controller_flow_with_conversations(
            true, true, true, false, false, false, false, true,
        );
    }
    fn npc_shared_controller_flow_with_conversations(
        quest: bool,
        objectives: bool,
        lives: bool,
        npc77: bool,
        npc70: bool,
        chat_enabled: bool,
        private_enabled: bool,
        life_actions: bool,
    ) {
        use super::super::{
            login_window::{self, LoginCommand, LoginController, LoginPhase, LoginSnapshot},
            profile::ModernProfile,
            session::SessionController,
            world_runtime::{RenderedWorld, RuntimeStatus, WorldRuntime},
        };
        use base64::Engine;
        use std::sync::Arc;
        struct Client {
            controller: SessionController,
            runtime: WorldRuntime,
            rendered: Option<Arc<RenderedWorld>>,
            auto_ack: bool,
            acks: std::collections::BTreeSet<(u64, u64)>,
            skill77_audio: std::collections::BTreeSet<u64>,
            skill_audio_id: u32,
        }
        impl Client {
            fn pump(&mut self) {
                self.controller.poll();
                let view = self.controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view
                    .world
                    .frame()
                    .is_none_or(|f| f.phase == world::Phase::Aborted)
                {
                    self.runtime.clear();
                } else if let Some(reply) = &view.npc {
                    if let Some(frame) = &reply.frame {
                        if view.world.frame() == Some(&frame.combat.world)
                            && view.combat.as_deref() == Some(&frame.combat)
                        {
                            let key = view.navigation.as_ref().unwrap().context.key;
                            let entity = frame
                                .combat
                                .world
                                .actors
                                .iter()
                                .find(|a| a.matches_character(key))
                                .unwrap()
                                .entity_id;
                            login_window::update_world_runtime(&mut self.runtime, &view, entity)
                                .unwrap();
                        }
                    }
                }
                self.rendered = match self.runtime.poll() {
                    RuntimeStatus::Ready(r) => Some(r),
                    RuntimeStatus::Failed(message) => panic!("real NPC resource worker: {message}"),
                    _ => None,
                };
                if let Some(audio) = self.rendered.as_ref().and_then(|r| r.audio.as_ref()) {
                    let sound = self.skill_audio_id;
                    self.skill77_audio.extend(
                        audio
                            .events
                            .iter()
                            .filter(|e| e.group.candidates() == [sound])
                            .map(|e| e.sequence),
                    );
                }
                if self.auto_ack {
                    if let Some(rendered) = &self.rendered {
                        if let Some(request) = login_window::loaded_request(&view, rendered) {
                            if !self.acks.contains(&(request.match_id, request.generation)) {
                                self.controller
                                    .command(LoginCommand::World(request))
                                    .unwrap();
                                assert!(self.acks.insert((request.match_id, request.generation)));
                            }
                        }
                    }
                }
            }
            fn view(&self) -> LoginSnapshot {
                self.controller.snapshot()
            }
            fn command(&mut self, command: LoginCommand) {
                self.controller.command(command).unwrap();
            }
        }
        #[track_caller]
        fn assert_pending_cast_is_not_predicted(
            before: &super::super::combat_protocol::Frame,
            view: &LoginSnapshot,
            sequence: u64,
        ) {
            let current = view.combat.as_ref().unwrap();
            assert!(view.spell_pending);
            assert_eq!(current.world.match_id, before.world.match_id);
            assert_eq!(current.world.generation, before.world.generation);
            assert!(current.world.revision >= before.world.revision);
            assert_eq!(view.world.frame(), Some(&current.world));
            assert_eq!(
                &view.npc.as_ref().unwrap().frame.as_ref().unwrap().combat,
                current.as_ref()
            );
            // command() first polls legitimate background READ publication. That
            // may advance world revision / NPC damage / natural recovery before
            // reserving this cast. It must NOT advance this player's cast proof.
            assert_eq!(current.world.cast(1), before.world.cast(1));
            assert!(current.world.cast(1).unwrap().request_sequence < sequence);
            assert_eq!(current.npc_fires, before.npc_fires);
            assert_eq!(current.npc_disintegrations, before.npc_disintegrations);
            for prior in &before.vitals {
                let next = current.vital(prior.entity_id).unwrap();
                let a = before.regen(prior.entity_id).unwrap();
                let b = current.regen(prior.entity_id).unwrap();
                assert_eq!(
                    next.mp - prior.mp,
                    (b.mp_gain - a.mp_gain) - (b.mp_loss - a.mp_loss),
                    "no unconfirmed MP deduction"
                );
                assert_eq!(
                    next.hp - prior.hp,
                    (b.hp_gain - a.hp_gain) - (b.hp_loss - a.hp_loss) + (b.item_gain - a.item_gain)
                        - (b.damage_loss - a.damage_loss),
                    "HP changes require the confirmed recovery/damage ledger"
                );
            }
            if current.world.revision == before.world.revision {
                assert_eq!(current.as_ref(), before);
            }
        }
        #[track_caller]
        fn wait(
            clients: &mut [Client; 2],
            deadline: Instant,
            condition: impl Fn(&[Client; 2]) -> bool,
        ) {
            let call = std::panic::Location::caller().line();
            let started = Instant::now();
            let mut diagnostic = false;
            loop {
                for client in clients.iter_mut() {
                    client.pump();
                    if let world::State::Ready(reply) = client.view().world {
                        assert_ne!(
                            reply.status,
                            world::Status::Unavailable,
                            "NPC admission: {:?}",
                            reply.reason
                        );
                    }
                }
                if condition(clients) {
                    return;
                }
                if !diagnostic && started.elapsed() > Duration::from_secs(5) {
                    diagnostic = true;
                    for (i, c) in clients.iter().enumerate() {
                        let v = c.view();
                        eprintln!("npc-stage {call} client {i}: world={:?} npc={:?} rendered={:?} pending={}/{}",
                        v.world.frame().map(|f|(f.phase,f.revision)),v.npc.as_ref().and_then(|r|r.frame.as_ref()).map(|f|(f.combat.world.phase,f.combat.world.revision,f.actors.len())),
                        c.rendered.as_ref().map(|r|(r.revision,r.npcs.len(),r.npc.as_ref().and_then(|n|n.frame.as_ref()).map(|f|f.combat.world.phase))),v.world_pending,v.combat_pending);
                    }
                }
                assert!(
                    Instant::now() < deadline,
                    "NPC controller/runtime stage {call} deadline: {} / {}",
                    clients[0].view().status,
                    clients[1].view().status
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn room(
            clients: &mut [Client; 2],
            index: usize,
            deadline: Instant,
            request: rooms::Request,
        ) {
            clients[index].command(LoginCommand::Rooms(request));
            wait(clients, deadline, |cs| !cs[index].view().room_pending);
        }
        fn chat_exchange(
            clients: &mut [Client; 2],
            deadline: Instant,
            sender: usize,
            text: &str,
            private_enabled: bool,
        ) {
            if private_enabled {
                use super::super::{
                    direct_chat_protocol as dc, direct_chat_state as ds, direct_chat_window as dw,
                };
                wait(clients, deadline, |cs| {
                    cs.iter().all(|c| {
                        let v = c.view();
                        v.direct_chat.token().is_some()
                            && !v.conversation_pending()
                            && !v.world_pending
                            && !v.room_pending
                            && !v.equipment_pending
                            && !v.spell_pending
                            && !v.combat_pending
                            && !v.record_pending
                            && !v.shop_pending
                            && !v.item_pending
                            && !v.ammo_pending
                    })
                });
                let before = clients[sender].view();
                let token = before.direct_chat.token().unwrap();
                let recipient = clients[1 - sender]
                    .view()
                    .direct_chat
                    .owner()
                    .unwrap()
                    .clone();
                let text = if text == "戰鬥世界可收送" {
                    ".de 20"
                } else {
                    text
                };
                clients[sender].command(
                    login_window::direct_chat_command(
                        &before,
                        dw::Intent {
                            token,
                            action: dc::Action::Send,
                            target: Some(dc::Address {
                                source_id: recipient.key.source_id,
                                name: recipient.character_name.clone(),
                            }),
                            preferences: None,
                            text: text.into(),
                        },
                    )
                    .unwrap(),
                );
                assert!(clients[sender].view().direct_chat_pending);
                assert_eq!(
                    clients[sender].view().direct_chat,
                    before.direct_chat,
                    "private send is never predicted"
                );
                wait(clients, deadline, |cs| {
                    !cs[sender].view().direct_chat_pending
                        && cs.iter().all(|c| {
                            c.view().direct_chat.messages().iter().any(|m| {
                                m.text == text
                                    && m.sender.key == token.enrollment.context.key
                                    && m.recipient == recipient
                            })
                        })
                });
                assert_eq!(
                    clients[sender].view().direct_chat.outcome(),
                    ds::Outcome::Confirmed(dc::Action::Send)
                );
                for c in clients.iter() {
                    assert!(
                        c.view().chat.messages().is_empty(),
                        "private message never reaches public transcript"
                    );
                    let v = c.view();
                    let t = v.direct_chat.token().unwrap();
                    assert_eq!(t.room_id, v.rooms.joined().map_or(0, |r| r.summary.room_id));
                    assert_eq!(t.enrollment.context, v.navigation.as_ref().unwrap().context);
                }
                assert_eq!(
                    clients[sender].view().direct_chat.token(),
                    Some(token),
                    "world activity does not replace the private enrollment"
                );
                return;
            }
            wait(clients, deadline, |cs| {
                cs.iter().all(|c| c.view().chat.token().is_some())
            });
            let before = clients[sender].view();
            let token = before.chat.token().unwrap();
            clients[sender].command(
                super::super::login_window::chat_command(
                    &before,
                    super::super::chat_window::Intent {
                        token,
                        text: text.into(),
                        kind: super::super::chat_window::Kind::Message,
                    },
                )
                .unwrap(),
            );
            assert!(clients[sender].view().chat_pending);
            assert_eq!(
                clients[sender].view().chat.messages(),
                before.chat.messages()
            );
            wait(clients, deadline, |cs| {
                !cs[sender].view().chat_pending
                    && cs.iter().all(|c| {
                        c.view()
                            .chat
                            .messages()
                            .iter()
                            .any(|m| m.text == text && m.sender == token.scope.context.key)
                    })
            });
            assert_eq!(
                clients[sender].view().chat.delivery(),
                super::super::chat_state::Delivery::Confirmed
            );
            for c in clients.iter() {
                assert_eq!(
                    c.view().chat.token().unwrap().scope.room_id,
                    token.scope.room_id
                );
            }
        }
        let shared = PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared resource root"),
        );
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernNpcServiceTestPeer",
            if chat_enabled {
                "npc70-service-shared-v32-chat"
            } else if npc70 {
                "npc70-service-shared-v31"
            } else if npc77 {
                "npc77-service-shared-v29"
            } else if life_actions {
                "quest-lives-shared-v37"
            } else if lives {
                "quest-lives-shared-v22"
            } else if objectives {
                "quest-objectives-shared-v21"
            } else if quest {
                "quest-stages-shared-v20"
            } else {
                "npc-service-shared-v19"
            },
            "60000",
        );
        let deadline = Instant::now() + Duration::from_secs(55);
        let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1, "profile_id":"npc-shared-controller", "display_name":"Isolated NPC shared controller",
            "shared_resource_root":shared, "endpoint":peer.endpoint.to_string(), "tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500,"exchange_timeout_ms":3000
        })).unwrap()).unwrap();
        let mut clients = std::array::from_fn(|_| Client {
            controller: SessionController::new(profile.clone()),
            runtime: WorldRuntime::new(shared.clone()).unwrap(),
            rendered: None,
            auto_ack: false,
            acks: Default::default(),
            skill77_audio: Default::default(),
            skill_audio_id: if npc70 { 710 } else { 717 },
        });
        for index in 0..2 {
            clients[index].command(LoginCommand::Connect);
            wait(&mut clients, deadline, |cs| {
                cs[index].view().phase == LoginPhase::LoginReady
            });
            clients[index].command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            });
            wait(&mut clients, deadline, |cs| {
                cs[index].view().phase == LoginPhase::ChoosingSource
            });
            clients[index].command(LoginCommand::SelectSource(if index == 0 { 7 } else { 42 }));
            wait(&mut clients, deadline, |cs| {
                cs[index].view().phase == LoginPhase::CharactersReady
            });
            let view = clients[index].view();
            let context = SelectionContext {
                key: application::CharacterKey {
                    source_id: view.selected_source.unwrap(),
                    object_id: view.characters[0].object_id,
                },
                epoch: view.roster_epoch,
            };
            clients[index].command(LoginCommand::SelectCharacter(context));
            wait(&mut clients, deadline, |cs| {
                cs[index].view().phase == LoginPhase::ChoosingChannel
                    && !cs[index].view().record_pending
                    && !cs[index].view().spell_pending
            });
            clients[index].command(LoginCommand::EnterChannel {
                context,
                channel_id: 0,
            });
            wait(&mut clients, deadline, |cs| {
                cs[index].view().phase == LoginPhase::InLobby
            });
        }
        let contexts = clients
            .each_ref()
            .map(|c| c.view().navigation.unwrap().context);
        if chat_enabled {
            chat_exchange(
                &mut clients,
                deadline,
                0,
                "v32 大廳中文 👋",
                private_enabled,
            );
        }
        let mut create = rooms::Common::read(contexts[0], 0);
        create.action = rooms::Action::Create;
        let maximum_players = if quest { 4 } else { 48 };
        for client in &clients {
            let rooms::State::Ready(directory) = client.view().rooms else {
                panic!("missing confirmed room directory")
            };
            assert_eq!(
                directory.mode_option(2, 554).unwrap().maximum_players,
                maximum_players
            );
        }
        let oversized = rooms::Request::draft(
            create,
            rooms::Draft::new(
                rooms::PublicDraft {
                    capacity: maximum_players + 1,
                    mode: 2,
                    map_id: 554,
                    attributes: 16,
                    name: "Must not send".into(),
                },
                String::new(),
            )
            .unwrap(),
        )
        .unwrap();
        assert!(clients[0]
            .controller
            .command(LoginCommand::Rooms(oversized))
            .is_err());
        assert!(!clients[0].view().room_pending);
        room(
            &mut clients,
            0,
            deadline,
            rooms::Request::draft(
                create,
                rooms::Draft::new(
                    rooms::PublicDraft {
                        capacity: 4,
                        mode: 2,
                        map_id: 554,
                        attributes: 16,
                        name: "Real shared NPC".into(),
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap(),
        );
        let summary = clients[0].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut clients,
            1,
            deadline,
            rooms::Request::join(
                rooms::Common {
                    context: contexts[1],
                    channel_id: 0,
                    action: rooms::Action::Join,
                    value: 0,
                    room_id: summary.room_id,
                    expected_revision: summary.revision,
                },
                String::new(),
            )
            .unwrap(),
        );
        for index in 0..2 {
            room(
                &mut clients,
                index,
                deadline,
                rooms::Request::read(contexts[index], 0).unwrap(),
            );
            let summary = clients[index]
                .view()
                .rooms
                .joined()
                .unwrap()
                .summary
                .clone();
            room(
                &mut clients,
                index,
                deadline,
                rooms::Request::simple(rooms::Common {
                    context: contexts[index],
                    channel_id: 0,
                    action: rooms::Action::Ready,
                    value: 1,
                    room_id: summary.room_id,
                    expected_revision: summary.revision,
                })
                .unwrap(),
            );
        }
        clients[0].auto_ack = true;
        wait(&mut clients, deadline, |cs| {
            cs.iter().all(|c| {
                c.rendered
                    .as_ref()
                    .is_some_and(|r| r.ready_to_ack && r.npc.is_some())
                    && c.view().npc.as_ref().is_some_and(|n| {
                        n.frame
                            .as_ref()
                            .is_some_and(|f| c.view().world.frame() == Some(&f.combat.world))
                    })
            }) && cs[0].view().world.frame().is_some_and(|f| {
                f.actors
                    .iter()
                    .any(|a| a.matches_character(contexts[0].key) && a.loaded)
            })
        });
        for (index, client) in clients.iter().enumerate() {
            let view = client.view();
            let npc = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
            assert_eq!(npc.combat.world.phase, world::Phase::Loading);
            assert_eq!(npc.combat.world.room.members.len(), 2);
            assert_eq!(npc.combat.world.audience_lane, quest.then_some(index as u8));
            if quest {
                assert_eq!(npc.combat.world.actors.len(), 1);
                assert_eq!(npc.combat.world.actors[0].entity_id, index as u64 + 1);
                assert!(npc.combat.world.actors[0].matches_character(contexts[index].key));
                assert_eq!(npc.combat.vitals.len(), 1);
                assert_eq!(npc.combat.vitals[0].entity_id, index as u64 + 1);
                assert_eq!(
                    npc.combat.vitals[0].max_hp,
                    if lives { 1 } else { 9_007_199_254_740_993 }
                );
            }
            assert_eq!(
                npc.resources,
                Some(if npc77 || npc70 {
                    vec![1649]
                } else {
                    vec![1649, 2158]
                })
            );
            assert!(npc.actors.is_empty());
            assert_eq!(npc.spawned, 0);
            assert!(client.rendered.as_ref().unwrap().npcs.is_empty());
            assert!(client.rendered.as_ref().unwrap().quests.is_empty());
            assert_eq!(npc.combat.world.quest.is_some(), objectives);
            if objectives {
                let q = npc.combat.world.quest.as_ref().unwrap();
                assert_eq!(q.targets.len(), 1);
                assert_eq!(q.targets[0].lane(), index as u8);
                assert!(q.actors.is_empty());
            }
        }
        assert_eq!(clients[0].acks.len(), 1);
        assert!(clients[1].acks.is_empty());
        let guest_view = clients[1].view();
        let proof = clients[1].rendered.as_ref().unwrap();
        assert!(login_window::loaded_request(&guest_view, proof).is_some());
        let mut bad = (**proof).clone();
        bad.npc = None;
        assert!(login_window::loaded_request(&guest_view, &bad).is_none());
        let mut bad = (**proof).clone();
        bad.manifest_digest[0] ^= 1;
        assert!(login_window::loaded_request(&guest_view, &bad).is_none());
        let mut bad = (**proof).clone();
        bad.ready_to_ack = false;
        assert!(login_window::loaded_request(&guest_view, &bad).is_none());
        let mut bad = (**proof).clone();
        bad.revision += 1;
        assert!(login_window::loaded_request(&guest_view, &bad).is_none());
        let mut bad = (**proof).clone();
        Arc::make_mut(bad.npc.as_mut().unwrap()).echo.channel_id = 1;
        assert!(login_window::loaded_request(&guest_view, &bad).is_none());
        let mut bad_view = guest_view.clone();
        bad_view.combat = None;
        assert!(login_window::loaded_request(&bad_view, proof).is_none());
        if chat_enabled {
            chat_exchange(&mut clients, deadline, 0, "真實資源載入中", private_enabled);
            if private_enabled {
                assert_eq!(clients[0].view().direct_chat.messages().len(), 2);
            }
            assert_eq!(clients[0].acks.len(), 1);
            assert!(clients[1].acks.is_empty());
            assert!(clients
                .iter()
                .all(|c| c.view().world.frame().unwrap().phase == world::Phase::Loading));
        }
        clients[1].auto_ack = true;
        wait(&mut clients, deadline, |cs| {
            cs.iter().all(|c| {
                c.rendered.as_ref().is_some_and(|r| {
                    r.npcs.len() == 1
                        && r.npc.as_ref().is_some_and(|n| {
                            n.frame.as_ref().unwrap().combat.world.phase == world::Phase::Active
                        })
                })
            })
        });
        if chat_enabled {
            chat_exchange(&mut clients, deadline, 1, "戰鬥世界可收送", private_enabled);
            if private_enabled {
                assert_eq!(clients[0].view().rooms.joined().unwrap().members.len(), 2);
            }
            assert_eq!(
                clients[0]
                    .view()
                    .combat
                    .as_ref()
                    .unwrap()
                    .world
                    .cast(1)
                    .unwrap()
                    .request_sequence,
                0
            );
        }
        let view = clients[0].view();
        let frame = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
        let target = npc::Id {
            scope: frame.scope,
            serial: frame.actors[0].serial,
        };
        let life = frame.actors[0].life;
        let hp = frame.actors[0].hp;
        assert_eq!(
            frame.combat.vitals[0].max_hp,
            if npc77 || npc70 {
                10000
            } else if lives {
                1
            } else {
                9_007_199_254_740_993
            }
        );
        let deferred = if life_actions {
            wait(&mut clients, deadline, |cs| {
                let v = cs[0].view();
                v.action_supported
                    && super::super::battle_action::for_command(
                        &v,
                        &LoginCommand::NpcAttack { target, life },
                    )
                    .ok()
                    .flatten()
                    .is_some()
            });
            let v = clients[0].view();
            let guard = super::super::battle_action::for_command(
                &v,
                &LoginCommand::NpcAttack { target, life },
            )
            .unwrap()
            .unwrap();
            assert_eq!(guard.stamp.life, 1);
            Some((
                npc::Request {
                    context: contexts[0],
                    channel_id: 0,
                    action: npc::Action::Attack,
                    world: target.scope,
                    sequence: 1,
                    serial: target.serial,
                    life,
                },
                guard,
            ))
        } else {
            None
        };
        if lives {
            use super::super::quest_life;
            wait(&mut clients, deadline, |cs| {
                cs.iter().all(|c| {
                    !c.view().combat_pending
                        && c.rendered.as_ref().is_some_and(|r| {
                            r.combat.as_ref().is_some_and(|f| {
                                f.vitals[0].hp == 0
                                    && quest_life::confirmation(f, f.vitals[0].entity_id).is_some()
                            })
                        })
                })
            });
            let proofs = clients.each_ref().map(|c| {
                let view = c.view();
                let f = view.combat.as_ref().unwrap();
                assert!(f.world.completion.is_none());
                assert_eq!(f.world.phase, world::Phase::Active);
                assert_eq!(f.world.quest.as_ref().unwrap().lives.len(), 1);
                quest_life::confirmation(f, f.vitals[0].entity_id).unwrap()
            });
            for index in 0..2 {
                let before = clients[index].view().combat.unwrap();
                let mut wrong = proofs[index];
                wrong.incarnation += 1;
                assert!(clients[index]
                    .controller
                    .command(LoginCommand::QuestRevive { proof: wrong })
                    .is_err());
                assert!(clients[index]
                    .controller
                    .command(LoginCommand::QuestRevive {
                        proof: proofs[1 - index]
                    })
                    .is_err());
                clients[index].command(LoginCommand::QuestRevive {
                    proof: proofs[index],
                });
                // command() first consumes already-confirmed background reads.
                // The world clock may advance between the rejected proofs and
                // enqueueing, but none of this actor's resurrection state may.
                let pending = clients[index].view();
                assert!(pending.combat_pending);
                let current = pending.combat.as_ref().unwrap();
                let entity = proofs[index].entity_id;
                assert_eq!(
                    (current.world.match_id, current.world.generation),
                    (before.world.match_id, before.world.generation)
                );
                assert!(current.world.revision >= before.world.revision);
                assert_eq!(
                    current.vital(entity),
                    before.vital(entity),
                    "no predicted HP or acknowledgment"
                );
                assert_eq!(
                    quest_life::life(&current.world, entity),
                    quest_life::life(&before.world, entity),
                    "no predicted life, respawn or HP gain"
                );
                assert_eq!(
                    current.world.actors.iter().find(|a| a.entity_id == entity),
                    before.world.actors.iter().find(|a| a.entity_id == entity),
                    "no predicted movement to spawn"
                );
                assert!(clients[index]
                    .controller
                    .command(LoginCommand::QuestRevive {
                        proof: proofs[index]
                    })
                    .is_err());
                wait(&mut clients, deadline, |cs| {
                    !cs[index].view().combat_pending
                        && cs[index].rendered.as_ref().is_some_and(|r| {
                            r.combat.as_ref().is_some_and(|f| {
                                quest_life::life(&f.world, proofs[index].entity_id)
                                    .is_some_and(|l| l.incarnation() == 2 && !l.dead())
                            })
                        })
                });
                let after = clients[index].view();
                let f = after.combat.as_ref().unwrap();
                let state = quest_life::life(&f.world, proofs[index].entity_id).unwrap();
                assert_eq!((state.deaths, state.respawns, state.hp_gain), (1, 1, 1));
                assert_eq!(f.vitals[0].hp, 1);
                assert_eq!(f.world.actors[0].position, state.spawn);
                assert_eq!(
                    f.world.actors[0].sequence,
                    before.world.actors[0].sequence + 1
                );
                assert_eq!(f.vitals[0].attack_sequence, 0);
                assert!(after
                    .npc
                    .as_ref()
                    .unwrap()
                    .frame
                    .as_ref()
                    .unwrap()
                    .exchanges
                    .iter()
                    .any(|e| e.participant_life == 1 && e.outcome.death));
                assert!(clients[index]
                    .controller
                    .command(LoginCommand::QuestRevive {
                        proof: proofs[index]
                    })
                    .is_err());
            }
        }
        if let Some((request, guard)) = deferred {
            assert_eq!(guard.stamp.life, 1);
            clients[0]
                .controller
                .submit_deferred_npc_fixture(request, guard)
                .unwrap();
            wait(&mut clients, deadline, |cs| !cs[0].view().combat_pending);
            let refreshed = clients[0].view();
            assert_eq!(refreshed.phase, LoginPhase::InLobby);
            assert!(
                refreshed.status.contains("StaleLife"),
                "{}",
                refreshed.status
            );
            assert_eq!(
                refreshed.combat.as_ref().unwrap().vitals[0].attack_sequence,
                0,
                "rejection must not consume the deferred request"
            );
            assert_eq!(
                super::super::quest_life::life(&refreshed.combat.as_ref().unwrap().world, 1)
                    .unwrap()
                    .incarnation(),
                2
            );
        }
        if quest {
            let mut seen = std::collections::BTreeSet::new();
            for index in 0..2 {
                for stage in 1..=4 {
                    wait(&mut clients, deadline, |cs| {
                        cs[index].rendered.as_ref().is_some_and(|r| {
                            r.npc.as_ref().is_some_and(|n| {
                                n.frame.as_ref().is_some_and(|f| {
                                    f.actors.iter().any(|a| {
                                        a.hp > 0 && a.lane == index as u8 && a.stage == stage
                                    })
                                })
                            })
                        })
                    });
                    let view = clients[index].view();
                    let frame = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                    let actor = frame
                        .actors
                        .iter()
                        .find(|a| a.hp > 0 && a.lane == index as u8)
                        .unwrap();
                    assert_eq!(actor.stage, stage);
                    assert_eq!(actor.graphic, if stage % 2 == 1 { 1649 } else { 2158 });
                    assert_eq!(frame.resources, Some(vec![1649, 2158]));
                    assert!(seen.insert(actor.serial));
                    let target = npc::Id {
                        scope: frame.scope,
                        serial: actor.serial,
                    };
                    let life = actor.life;
                    for (viewer, client) in clients.iter().enumerate() {
                        assert_eq!(
                            client.rendered.as_ref().unwrap().npcs.iter().any(|n| {
                                n.id == target
                                    && n.life == life
                                    && n.alive
                                    && n.graphics_id == actor.graphic
                            }),
                            viewer == index,
                            "later wave must have a real shared sprite placement"
                        );
                    }
                    if life_actions {
                        wait(&mut clients, deadline, |cs| {
                            super::super::battle_action::for_command(
                                &cs[index].view(),
                                &LoginCommand::NpcAttack { target, life },
                            )
                            .ok()
                            .flatten()
                            .is_some()
                        });
                    }
                    clients[index].command(LoginCommand::NpcAttack { target, life });
                    wait(&mut clients, deadline, |cs| {
                        !cs[index].view().combat_pending
                            && cs[index].view().combat.as_ref().is_some_and(|f| {
                                f.vitals.len() == 1
                                    && f.vitals[0].entity_id == index as u64 + 1
                                    && f.vitals[0].attack_sequence == u64::from(stage)
                            })
                    });
                    let view = clients[index].view();
                    let frame = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                    assert!(frame.exchanges.iter().any(|e| {
                        e.npc.serial == target.serial
                            && e.direction == npc::Direction::PlayerToNpc
                            && e.request_sequence == u64::from(stage)
                            && e.outcome.death
                    }));
                    assert_eq!(frame.combat.world.phase, world::Phase::Active);
                    assert!(frame.combat.world.completion.is_none());
                    assert!(frame.actors.iter().all(|a| a.lane == index as u8));
                    if index == 0 {
                        let other = clients[1].view();
                        let other = other.npc.as_ref().unwrap().frame.as_ref().unwrap();
                        assert!(other
                            .actors
                            .iter()
                            .any(|a| a.hp > 0 && a.lane == 1 && a.stage == 1));
                        assert_eq!(other.combat.vitals[0].entity_id, 2);
                        assert_eq!(other.combat.vitals[0].attack_sequence, 0);
                    }
                    // Keep pumping both real controllers while the server's
                    // attack clock elapses; no synthetic client timestamps.
                    let cooldown = Instant::now() + Duration::from_millis(710);
                    wait(&mut clients, deadline, |_| Instant::now() >= cooldown);
                }
            }
            wait(&mut clients, deadline, |cs| {
                cs.iter().all(|c| {
                    c.rendered.as_ref().is_some_and(|r| {
                        r.npcs.is_empty()
                            && r.npc.as_ref().is_some_and(|n| {
                                n.frame
                                    .as_ref()
                                    .is_some_and(|f| f.spawned == 8 && f.actors.is_empty())
                            })
                    })
                })
            });
            assert_eq!(seen.len(), 8);
            for client in &clients {
                let view = client.view();
                let frame = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                assert_eq!(frame.resources, Some(vec![1649, 2158]));
                assert_eq!(frame.combat.world.phase, world::Phase::Active);
                assert!(frame.combat.world.completion.is_none());
                assert_eq!(
                    client.acks.len(),
                    1,
                    "later waves must not reload the world"
                );
            }
            if objectives {
                wait(&mut clients, deadline, |cs| {
                    cs.iter().all(|c| {
                        c.rendered
                            .as_ref()
                            .is_some_and(|r| r.quests.len() == 1 && r.quests[0].alive)
                    })
                });
                let id = clients[0].rendered.as_ref().unwrap().quests[0].id;
                let guest = clients[1].rendered.as_ref().unwrap().quests[0].id;
                assert_eq!((id.slot, guest.slot), (4, 8));
                assert!(clients[0]
                    .controller
                    .command(LoginCommand::QuestAttack { target: guest })
                    .is_err());
                clients[0].command(LoginCommand::QuestAttack { target: id });
                wait(&mut clients, deadline, |cs| {
                    cs.iter().all(|c| {
                        !c.view().combat_pending
                            && c.view().world.frame().is_some_and(|f| {
                                f.phase == world::Phase::Finished
                                    && f.completion
                                        .as_ref()
                                        .is_some_and(|r| r.state == world::SettlementState::Saved)
                            })
                    })
                });
                let winner = clients[0].view();
                let loser = clients[1].view();
                let a = winner.world.frame().unwrap();
                let b = loser.world.frame().unwrap();
                assert_eq!(a.completion, b.completion);
                assert_eq!(a.completion.as_ref().unwrap().event.mode, 2);
                assert_eq!(a.completion.as_ref().unwrap().event.participants.len(), 2);
                assert_eq!(winner.combat.as_ref().unwrap().vitals[0].attack_sequence, 5);
                assert_eq!(loser.combat.as_ref().unwrap().vitals[0].attack_sequence, 4);
                assert_eq!(a.quest.as_ref().unwrap().winning_lane, Some(0));
                assert_eq!(b.quest.as_ref().unwrap().winning_lane, Some(0));
                assert_eq!(a.quest.as_ref().unwrap().actors[0].hp, 0);
                assert_eq!(b.quest.as_ref().unwrap().actors[0].hp, 1);
                wait(&mut clients, deadline, |cs| {
                    cs.iter().all(|c| {
                        c.rendered.as_ref().is_some_and(|r| {
                            r.combat
                                .as_ref()
                                .is_some_and(|f| f.world.phase == world::Phase::Finished)
                        })
                    })
                });
                assert!(!clients[0].rendered.as_ref().unwrap().quests[0].alive);
                assert!(clients[1].rendered.as_ref().unwrap().quests[0].alive);
                assert!(clients[0].rendered.as_ref().unwrap().objectives.is_empty());
                assert!(clients[0]
                    .controller
                    .command(LoginCommand::QuestAttack { target: id })
                    .is_err());
            }
        } else if npc70 {
            assert!(clients[0].view().skills.fire_storm().is_some());
            assert!(clients[1].view().skills.fire_storm().is_none());
            let guest_request =
                login_window::npc_fire_storm_input_request(&clients[1].view()).unwrap();
            assert!(clients[1]
                .controller
                .command(LoginCommand::CastNpcFireStorm(guest_request))
                .is_err());
            let mut count = 0;
            for attempt in 0..8 {
                if attempt >= 2 {
                    let cooldown = Instant::now() + Duration::from_millis(2100);
                    wait(&mut clients, deadline, |_| Instant::now() >= cooldown);
                }
                let q = login_window::npc_fire_storm_input_request(&clients[0].view()).unwrap();
                let before = clients[0].view().combat.unwrap();
                clients[0].command(LoginCommand::CastNpcFireStorm(q));
                assert_pending_cast_is_not_predicted(&before, &clients[0].view(), q.sequence);
                assert!(clients[0]
                    .controller
                    .command(LoginCommand::CastNpcFireStorm(q))
                    .is_err());
                wait(&mut clients, deadline, |cs| {
                    !cs[0].view().spell_pending
                        && cs[0].view().combat.as_ref().is_some_and(|f| {
                            f.world.cast(1).unwrap().request_sequence == q.sequence
                        })
                });
                if attempt != 1 {
                    count += 1;
                }
                let view = clients[0].view();
                let f = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                assert_eq!(f.combat.npc_fire(1).unwrap().count, count);
                assert_eq!(f.combat.npc_fire(1).unwrap().mp_spent, count * 48);
                assert_eq!(f.combat.npc_fire(2).unwrap().count, 0);
                if attempt == 1 {
                    assert!(view.status.contains("TooFast"));
                } else {
                    wait(&mut clients, deadline, |cs| {
                        cs.iter().all(|c| c.skill77_audio.len() >= count as usize)
                    });
                }
                if attempt >= 2 && f.actor(target).is_none_or(|a| !a.alive()) {
                    break;
                }
            }
            wait(&mut clients, deadline, |cs| {
                cs.iter().all(|c| {
                    c.rendered.as_ref().is_some_and(|r| {
                        r.npcs.is_empty()
                            && r.npc
                                .as_ref()
                                .is_some_and(|n| n.frame.as_ref().unwrap().actors.is_empty())
                    })
                })
            });
            for c in &clients {
                assert_eq!(c.acks.len(), 1, "no resource reload after casts");
                assert_eq!(c.skill77_audio.len(), count as usize);
                assert_eq!(
                    c.view().combat.as_ref().unwrap().npc_fire(1).unwrap().count,
                    count
                );
            }
        } else if npc77 {
            assert!(clients
                .iter()
                .all(|c| c.view().skills.disintegrate().is_some()));
            let request = |c: &Client| {
                let view = c.view();
                let f = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                super::super::npc_disintegrate_protocol::Request {
                    context: contexts[0],
                    channel_id: 0,
                    target,
                    caster_life: 1,
                    target_life: life,
                    sequence: f.combat.world.cast(1).unwrap().request_sequence + 1,
                }
            };
            for (index, expected_count) in [(0, 1), (1, 1), (2, 2)] {
                if index == 2 {
                    let cooldown = Instant::now() + Duration::from_millis(5100);
                    wait(&mut clients, deadline, |_| Instant::now() >= cooldown);
                }
                let q = request(&clients[0]);
                if index == 0 {
                    assert!(clients[0]
                        .controller
                        .command(LoginCommand::CastNpcDisintegrate(
                            super::super::npc_disintegrate_protocol::Request {
                                target_life: life + 1,
                                ..q
                            }
                        ))
                        .is_err());
                }
                let old = clients[0].view().combat.unwrap();
                clients[0].command(LoginCommand::CastNpcDisintegrate(q));
                assert_pending_cast_is_not_predicted(&old, &clients[0].view(), q.sequence);
                assert!(clients[0]
                    .controller
                    .command(LoginCommand::CastNpcDisintegrate(q))
                    .is_err());
                wait(&mut clients, deadline, |cs| {
                    !cs[0].view().spell_pending
                        && cs[0].view().combat.as_ref().is_some_and(|f| {
                            f.world.cast(1).unwrap().request_sequence == q.sequence
                        })
                });
                let view = clients[0].view();
                let f = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
                let s = f.combat.npc_disintegrate(1).unwrap();
                assert_eq!(
                    (s.count, s.mp_spent, s.lawful),
                    (expected_count, 70 * expected_count, 500 + expected_count)
                );
                assert_eq!(f.combat.npc_disintegrate(2).unwrap().count, 0);
                if index == 1 {
                    assert!(view.status.contains("TooFast"));
                } else {
                    wait(&mut clients, deadline, |cs| {
                        cs.iter()
                            .all(|c| c.skill77_audio.len() >= expected_count as usize)
                    });
                }
            }
            wait(&mut clients, deadline, |cs| {
                cs.iter().all(|c| {
                    c.rendered.as_ref().is_some_and(|r| {
                        r.npcs.is_empty()
                            && r.npc
                                .as_ref()
                                .is_some_and(|n| n.frame.as_ref().unwrap().actors.is_empty())
                    })
                })
            });
            for client in &clients {
                let view = client.view();
                let f = view.combat.as_ref().unwrap();
                assert!(f.npc_disintegrate(1).unwrap().hit.as_ref().unwrap().death);
                assert_eq!(f.vital(1).unwrap().mp, 860);
                assert_eq!(client.skill77_audio.len(), 2);
                assert_eq!(client.acks.len(), 1);
            }
            assert!(clients[0]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(request(&clients[0])))
                .is_err());
        } else {
            clients[0].command(LoginCommand::NpcAttack { target, life });
            wait(&mut clients, deadline, |cs| {
                !cs[0].view().combat_pending
                    && cs[0]
                        .view()
                        .combat
                        .as_ref()
                        .is_some_and(|f| f.vitals[0].attack_sequence == 1)
            });
            let view = clients[0].view();
            let frame = view.npc.as_ref().unwrap().frame.as_ref().unwrap();
            assert!(frame.actors[0].hp < hp);
            assert_eq!(frame.combat.vitals[0].last_target_entity_id, 0);
            assert_eq!(frame.combat.vitals[1].attack_sequence, 0);
        }
        assert_eq!(clients[1].acks.len(), 1);
        let mut leave = world::Request::read(contexts[0], 0);
        leave.action = world::Action::Leave;
        leave.match_id = target.scope.match_id;
        leave.generation = target.scope.generation;
        clients[0].command(LoginCommand::World(leave));
        if objectives {
            wait(&mut clients, deadline, |cs| {
                cs[0].view().world.frame().is_none() && !cs[0].view().world_pending
            });
            assert_eq!(
                clients[1].view().world.frame().unwrap().phase,
                world::Phase::Finished
            );
            let mut guest_leave = leave;
            guest_leave.context = contexts[1];
            clients[1].command(LoginCommand::World(guest_leave));
        }
        wait(&mut clients, deadline, |cs| {
            cs[0].view().world.frame().is_none()
                && !cs[0].view().world_pending
                && (if objectives {
                    cs[1].view().world.frame().is_none() && !cs[1].view().world_pending
                } else {
                    cs[1]
                        .view()
                        .npc
                        .as_ref()
                        .is_some_and(|r| r.frame.as_ref().is_some_and(|f| f.closed))
                })
                && cs
                    .iter()
                    .all(|c| c.runtime.is_quiescent() && c.rendered.is_none())
        });
        assert!(clients[0].view().npc.is_none());
        if chat_enabled {
            chat_exchange(
                &mut clients,
                deadline,
                0,
                "已退出世界仍在房間",
                private_enabled,
            );
            if private_enabled {
                assert_eq!(clients[0].view().direct_chat.messages().len(), 4);
            }
        }
        assert!(clients[0]
            .controller
            .command(LoginCommand::NpcAttack { target, life })
            .is_err());
        let summary = clients[0].view().rooms.joined().unwrap().summary.clone();
        room(
            &mut clients,
            0,
            deadline,
            rooms::Request::simple(rooms::Common {
                context: contexts[0],
                channel_id: 0,
                action: rooms::Action::Leave,
                value: 0,
                room_id: summary.room_id,
                expected_revision: summary.revision,
            })
            .unwrap(),
        );
        clients[0].command(LoginCommand::LeaveLobby(contexts[0]));
        wait(&mut clients, deadline, |cs| {
            cs[0].view().phase == LoginPhase::ChoosingChannel
        });
        clients[0].command(LoginCommand::ReturnCharacters(contexts[0]));
        wait(&mut clients, deadline, |cs| {
            cs[0].view().phase == LoginPhase::CharactersReady
        });
        clients[0].command(LoginCommand::SelectSource(42));
        wait(&mut clients, deadline, |cs| {
            cs[0].view().phase == LoginPhase::CharactersReady
                && cs[0].view().selected_source == Some(42)
        });
        assert!(clients[0].view().npc.is_none());
        assert!(clients[0].runtime.is_quiescent());
        if chat_enabled {
            assert!(clients[0].view().chat.messages().is_empty());
            assert!(clients[0].view().chat.token().is_none());
            if private_enabled {
                assert!(clients[0].view().direct_chat.owner().is_none());
                assert!(clients[0].view().direct_chat.messages().is_empty());
                assert!(clients[0].view().direct_chat.token().is_none());
            }
        }
        for client in &mut clients {
            client.command(LoginCommand::Shutdown);
        }
        wait(&mut clients, deadline, |cs| {
            cs.iter()
                .all(|c| c.controller.is_quiescent() && c.runtime.is_quiescent())
        });
        peer.expect_successful_exit();
    }

    fn integration_limits() -> TransportLimits {
        TransportLimits {
            connect_timeout: Duration::from_millis(500),
            total_timeout: Duration::from_secs(4),
            io_poll_interval: Duration::from_millis(25),
        }
    }

    #[test]
    #[ignore = "requires Java 17 real NPC service TLS peer; synthetic geometry/profiles, no GUI or resource ACK proof"]
    fn java_npc_v18_public_connection_auth_prepare_preflight_attack_leave_and_retire() {
        fn login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
            let mut hello = Hello::current(100);
            hello.supported_capabilities |= super::super::protocol::CAP_NPC;
            let mut c = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&private_root()),
                hello,
                integration_limits(),
                Cancellation::new(),
            )
            .unwrap();
            assert_eq!(c.ack().unwrap().version, 18);
            assert!(c.supports_npc());
            c.authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = c.characters(source, 1).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch: 1,
            };
            c.select_character(context).unwrap();
            c.enter_channel(0).unwrap();
            (c, context)
        }
        fn ready(c: &mut ModernTlsConnection, context: SelectionContext) {
            let room = c
                .rooms(rooms::Request::read(context, 0).unwrap())
                .unwrap()
                .joined
                .unwrap();
            let request = rooms::Common {
                context,
                channel_id: 0,
                action: rooms::Action::Ready,
                value: 1,
                room_id: room.summary.room_id,
                expected_revision: room.summary.revision,
            };
            assert_eq!(
                c.rooms(rooms::Request::simple(request).unwrap())
                    .unwrap()
                    .status,
                rooms::Status::Ok
            );
        }
        fn read_request(context: SelectionContext, frame: &world::Frame) -> npc::Request {
            npc::Request {
                context,
                channel_id: 0,
                action: npc::Action::Read,
                world: npc::Scope {
                    room_id: frame.room.summary.room_id,
                    match_id: frame.match_id,
                    generation: frame.generation,
                },
                sequence: 0,
                serial: 0,
                life: 0,
            }
        }
        fn ack(c: &mut ModernTlsConnection, context: SelectionContext, frame: &world::Frame) {
            // Protocol barrier test only: no renderer is installed in this synthetic-map fixture.
            let mut request = world::Request::read(context, 0);
            request.action = world::Action::Loaded;
            request.match_id = frame.match_id;
            request.generation = frame.generation;
            request.manifest_digest = frame.manifest.digest;
            assert_eq!(c.world(request).unwrap().status, world::Status::Ok);
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernNpcServiceTestPeer",
            "npc-service-v18",
            "30000",
        );
        let (mut owner, a) = login(&peer, 7);
        let (mut guest, b) = login(&peer, 42);
        let mut common = rooms::Common::read(a, 0);
        common.action = rooms::Action::Create;
        let room = owner
            .rooms(
                rooms::Request::draft(
                    common,
                    rooms::Draft::new(
                        rooms::PublicDraft {
                            capacity: 4,
                            mode: 2,
                            attributes: 16,
                            map_id: 554,
                            name: "NPC service TLS".into(),
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
            .joined
            .unwrap();
        let join = rooms::Common {
            context: b,
            channel_id: 0,
            action: rooms::Action::Join,
            value: 0,
            room_id: room.summary.room_id,
            expected_revision: room.summary.revision,
        };
        assert_eq!(
            guest
                .rooms(rooms::Request::join(join, String::new()).unwrap())
                .unwrap()
                .status,
            rooms::Status::Ok
        );
        ready(&mut owner, a);
        ready(&mut guest, b);
        let room = owner
            .rooms(rooms::Request::read(a, 0).unwrap())
            .unwrap()
            .joined
            .unwrap();
        let mut prepare = world::Request::read(a, 0);
        prepare.action = world::Action::Prepare;
        prepare.room_id = room.summary.room_id;
        prepare.expected_room_revision = room.summary.revision;
        assert_eq!(owner.world(prepare).unwrap().status, world::Status::Ok);
        let deadline = Instant::now() + Duration::from_secs(5);
        let loading = loop {
            let frame = owner
                .world(world::Request::read(a, 0))
                .unwrap()
                .frame
                .unwrap();
            if frame.phase != world::Phase::Preparing {
                break frame;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(loading.phase, world::Phase::Loading);
        guest.world(world::Request::read(b, 0)).unwrap();
        let read = read_request(a, &loading);
        let initial = owner.npc(read).unwrap();
        let plan = super::super::npc_presentation::Plan::from_reply(&initial).unwrap();
        assert!(initial.frame.as_ref().unwrap().actors.is_empty());
        assert_eq!(
            initial.frame.as_ref().unwrap().resources,
            Some(vec![1649, 2158])
        );
        ack(&mut owner, a, &loading);
        assert!(owner.npc(read).unwrap().frame.unwrap().actors.is_empty());
        ack(&mut guest, b, &loading);
        let active = owner.npc(read).unwrap();
        assert!(plan.validate(&active).is_ok());
        let frame = active.frame.unwrap();
        assert_eq!(frame.combat.world.phase, world::Phase::Active);
        assert_eq!(frame.actors.len(), 1);
        assert_eq!(frame.combat.vitals[0].max_hp, 9_007_199_254_740_993);
        let target = &frame.actors[0];
        let mut attack = npc::Request {
            action: npc::Action::Attack,
            sequence: 1,
            serial: target.serial,
            life: target.life,
            ..read
        };
        let hit = owner.npc(attack).unwrap();
        assert_eq!(hit.status, npc::Status::Ok);
        assert_eq!(
            hit.frame.as_ref().unwrap().combat.vitals[0].last_target_entity_id,
            0
        );
        assert_eq!(
            owner.world_snapshot().unwrap().frame.as_ref(),
            Some(&hit.frame.as_ref().unwrap().combat.world)
        );
        assert_eq!(
            owner.combat_snapshot().unwrap().frame.as_ref(),
            Some(&hit.frame.as_ref().unwrap().combat)
        );
        attack.sequence += 1;
        assert_eq!(owner.npc(attack).unwrap().reason, npc::Reason::TooFast);
        let guest_read = read_request(b, &loading);
        let other = guest.npc(guest_read).unwrap();
        assert_eq!(other.frame.unwrap().combat.vitals[1].attack_sequence, 0);
        let mut forged = read;
        forged.context = b;
        assert!(owner.npc(forged).is_err());
        assert!(owner.npc_snapshot().is_some()); // Rejected locally, no uncertain mutation.
        let mut leave = world::Request::read(a, 0);
        leave.action = world::Action::Leave;
        leave.match_id = loading.match_id;
        leave.generation = loading.generation;
        assert!(owner.world(leave).unwrap().frame.is_none());
        assert!(owner.npc_snapshot().is_none());
        assert!(owner.npc(read).is_err());
        let closed = guest.npc(guest_read).unwrap();
        assert!(closed.frame.as_ref().unwrap().closed);
        assert!(closed.frame.as_ref().unwrap().actors.is_empty());
        assert!(plan.validate(&closed).is_ok());
        guest.close().unwrap();
        owner.close().unwrap();
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires Java 17 real NPC77 service TLS peer; synthetic geometry/profiles, not player UI admission"]
    fn java_npc77_v29_public_connection_qualification_cast_cooldown_recast_leave_and_retire() {
        use super::super::npc_disintegrate_protocol as npc77;
        fn login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
            let mut hello = Hello::current(100);
            hello.max_version = 29;
            hello.supported_capabilities |=
                super::super::protocol::CAP_NPC | super::super::protocol::CAP_NPC_DISINTEGRATE;
            let mut c = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&private_root()),
                hello,
                integration_limits(),
                Cancellation::new(),
            )
            .unwrap();
            assert_eq!(c.ack().unwrap().version, 29);
            assert!(npc77::negotiated(&c.ack().unwrap()));
            assert_eq!(
                c.ack().unwrap().capabilities & super::super::protocol::CAP_DISINTEGRATE,
                0
            );
            assert!(c.supports_npc());
            c.authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = c.characters(source, 1).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch: 1,
            };
            c.select_character(context).unwrap();
            c.enter_channel(0).unwrap();
            (c, context)
        }
        fn ready(c: &mut ModernTlsConnection, context: SelectionContext) {
            let room = c
                .rooms(rooms::Request::read(context, 0).unwrap())
                .unwrap()
                .joined
                .unwrap();
            let request = rooms::Common {
                context,
                channel_id: 0,
                action: rooms::Action::Ready,
                value: 1,
                room_id: room.summary.room_id,
                expected_revision: room.summary.revision,
            };
            assert_eq!(
                c.rooms(rooms::Request::simple(request).unwrap())
                    .unwrap()
                    .status,
                rooms::Status::Ok
            );
        }
        fn read_request(context: SelectionContext, frame: &world::Frame) -> npc::Request {
            npc::Request {
                context,
                channel_id: 0,
                action: npc::Action::Read,
                world: npc::Scope {
                    room_id: frame.room.summary.room_id,
                    match_id: frame.match_id,
                    generation: frame.generation,
                },
                sequence: 0,
                serial: 0,
                life: 0,
            }
        }
        fn ack(c: &mut ModernTlsConnection, context: SelectionContext, frame: &world::Frame) {
            // Protocol barrier test only: no renderer is installed in this synthetic-map fixture.
            let mut request = world::Request::read(context, 0);
            request.action = world::Action::Loaded;
            request.match_id = frame.match_id;
            request.generation = frame.generation;
            request.manifest_digest = frame.manifest.digest;
            assert_eq!(c.world(request).unwrap().status, world::Status::Ok);
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernNpcServiceTestPeer",
            "npc77-service-v29",
            "30000",
        );
        let (mut owner, a) = login(&peer, 7);
        let (mut guest, b) = login(&peer, 42);
        assert!(owner
            .skills()
            .unwrap()
            .skills
            .iter()
            .any(|s| s.skill_id == 77 && s.support == spell::Support::Disintegrate));
        let mut common = rooms::Common::read(a, 0);
        common.action = rooms::Action::Create;
        let room = owner
            .rooms(
                rooms::Request::draft(
                    common,
                    rooms::Draft::new(
                        rooms::PublicDraft {
                            capacity: 4,
                            mode: 2,
                            attributes: 16,
                            map_id: 554,
                            name: "NPC77 service TLS".into(),
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap()
            .joined
            .unwrap();
        let join = rooms::Common {
            context: b,
            channel_id: 0,
            action: rooms::Action::Join,
            value: 0,
            room_id: room.summary.room_id,
            expected_revision: room.summary.revision,
        };
        assert_eq!(
            guest
                .rooms(rooms::Request::join(join, String::new()).unwrap())
                .unwrap()
                .status,
            rooms::Status::Ok
        );
        ready(&mut owner, a);
        ready(&mut guest, b);
        let room = owner
            .rooms(rooms::Request::read(a, 0).unwrap())
            .unwrap()
            .joined
            .unwrap();
        let mut prepare = world::Request::read(a, 0);
        prepare.action = world::Action::Prepare;
        prepare.room_id = room.summary.room_id;
        prepare.expected_room_revision = room.summary.revision;
        assert_eq!(owner.world(prepare).unwrap().status, world::Status::Ok);
        let deadline = Instant::now() + Duration::from_secs(5);
        let loading = loop {
            let frame = owner
                .world(world::Request::read(a, 0))
                .unwrap()
                .frame
                .unwrap();
            if frame.phase != world::Phase::Preparing {
                break frame;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(loading.phase, world::Phase::Loading);
        guest.world(world::Request::read(b, 0)).unwrap();
        let read = read_request(a, &loading);
        let initial = owner.npc(read).unwrap();
        // Protocol preflight only; actual resource loading is separately tested.
        let plan = super::super::npc_presentation::Plan::from_reply(&initial).unwrap();
        assert!(plan.validate(&initial).is_ok());
        assert!(initial.frame.as_ref().unwrap().actors.is_empty());
        assert_eq!(initial.frame.as_ref().unwrap().resources, Some(vec![1649]));
        ack(&mut owner, a, &loading);
        assert!(owner.npc(read).unwrap().frame.unwrap().actors.is_empty());
        ack(&mut guest, b, &loading);
        let active = owner.npc(read).unwrap();

        let frame = active.frame.unwrap();
        assert_eq!(frame.combat.world.phase, world::Phase::Active);
        assert_eq!(frame.actors.len(), 1);
        assert_eq!(frame.combat.vitals[0].max_hp, 10000);
        let target = &frame.actors[0];
        let cast = npc77::Request {
            context: a,
            channel_id: 0,
            target: npc::Id {
                scope: frame.scope,
                serial: target.serial,
            },
            sequence: 1,
            caster_life: 1,
            target_life: target.life,
        };
        let hit = owner.disintegrate_npc(cast).unwrap();
        assert_eq!(hit.status, npc::Status::Ok);
        let hit_frame = hit.frame.as_ref().unwrap();
        let ledger = hit_frame.combat.npc_disintegrate(1).unwrap();
        assert_eq!((ledger.count, ledger.mp_spent, ledger.lawful), (1, 70, 501));
        assert_eq!(hit_frame.combat.vital(1).unwrap().mp, 930);
        assert_eq!(hit_frame.combat.vital(1).unwrap().last_target_entity_id, 0);
        assert!(hit_frame.actor(cast.target).unwrap().hp < 10000);
        assert_eq!(
            owner.world_snapshot().unwrap().frame.as_ref(),
            Some(&hit_frame.combat.world)
        );
        assert_eq!(
            owner.combat_snapshot().unwrap().frame.as_ref(),
            Some(&hit_frame.combat)
        );
        let denied = owner
            .disintegrate_npc(npc77::Request {
                sequence: 2,
                ..cast
            })
            .unwrap();
        assert_eq!(denied.reason, npc77::Reason::TooFast);
        assert_eq!(
            denied.frame.as_ref().unwrap().combat.npc_disintegrations,
            hit_frame.combat.npc_disintegrations
        );
        // Replay and wrong ownership/life are rejected before the network call.
        assert!(owner.disintegrate_npc(cast).is_err());
        for forged in [
            npc77::Request {
                context: b,
                sequence: 3,
                ..cast
            },
            npc77::Request {
                caster_life: 2,
                sequence: 3,
                ..cast
            },
            npc77::Request {
                target_life: 2,
                sequence: 3,
                ..cast
            },
            npc77::Request {
                channel_id: 1,
                sequence: 3,
                ..cast
            },
        ] {
            assert!(owner.disintegrate_npc(forged).is_err());
            assert!(owner.npc_snapshot().is_some());
        }
        thread::sleep(Duration::from_millis(5100));
        let second = owner
            .disintegrate_npc(npc77::Request {
                sequence: 3,
                ..cast
            })
            .unwrap();
        assert_eq!(second.status, npc::Status::Ok);
        let second_frame = second.frame.as_ref().unwrap();
        let ledger = second_frame.combat.npc_disintegrate(1).unwrap();
        assert_eq!(
            (ledger.count, ledger.mp_spent, ledger.lawful),
            (2, 140, 502)
        );
        assert_eq!(second_frame.combat.vital(1).unwrap().mp, 860);
        let guest_read = read_request(b, &loading);
        let other = guest.npc(guest_read).unwrap().frame.unwrap();
        assert_eq!(other.combat.npc_disintegrate(1).unwrap().count, 2);
        assert_eq!(other.combat.npc_disintegrate(2).unwrap().count, 0);
        let mut leave = world::Request::read(a, 0);
        leave.action = world::Action::Leave;
        leave.match_id = loading.match_id;
        leave.generation = loading.generation;
        assert!(owner.world(leave).unwrap().frame.is_none());
        assert!(owner.npc_snapshot().is_none());
        assert!(owner.npc(read).is_err());
        let closed = guest.npc(guest_read).unwrap();
        assert!(closed.frame.as_ref().unwrap().closed);
        assert!(closed.frame.as_ref().unwrap().actors.is_empty());
        assert_eq!(
            closed
                .frame
                .as_ref()
                .unwrap()
                .combat
                .npc_disintegrate(1)
                .unwrap()
                .count,
            2
        );
        guest.close().unwrap();
        owner.close().unwrap();
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 loopback TLS peer and explicit test environment"]
    fn java_tls_verified_chain_fragmented_ack_tls12_and_retained_connection() {
        let root = private_root();
        for (index, mode) in ["tls", "fragmented", "tls12"].into_iter().enumerate() {
            let mut peer = JavaPeer::start(mode);
            let hello = Hello::current(100 + index as u32);
            let mut connection = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&root),
                hello,
                integration_limits(),
                Cancellation::new(),
            )
            .unwrap_or_else(|error| panic!("{mode}: {error:?}"));
            let ack = connection.ack().expect("validated ACK is retained");
            assert_eq!(ack.request_id, hello.request_id);
            assert_eq!(connection.state(), ClientState::Negotiated(ack));
            assert_eq!(
                connection.tls_version(),
                if mode == "tls12" {
                    TlsVersion::Tls12
                } else {
                    TlsVersion::Tls13
                }
            );
            assert!(peer.child.try_wait().unwrap().is_none());
            connection.close().expect("bounded TLS close_notify");
            assert_eq!(connection.state(), ClientState::Closed);
            assert!(connection.ack().is_none());
            connection.close().expect("idempotent close");
            peer.expect_successful_exit();
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 loopback TLS peer and explicit test environment"]
    fn java_tls_rejects_wrong_dns_untrusted_expired_and_plaintext() {
        let root = private_root();
        for (mode, name, trusted) in [
            ("tls", "wrong.invalid", true),
            ("tls", "localhost", false),
            ("expired", "localhost", true),
            ("plain", "localhost", true),
        ] {
            let peer = JavaPeer::start(mode);
            let result = connect_and_negotiate(
                peer.endpoint,
                name,
                trusted.then_some(root.as_slice()),
                Hello::current(200),
                integration_limits(),
                Cancellation::new(),
            );
            let error = result.expect_err("invalid server must never negotiate");
            if mode == "plain" {
                assert!(matches!(error, TransportError::Tls(_)), "{error:?}");
            } else {
                assert!(
                    matches!(
                        error,
                        TransportError::Tls(rustls::Error::InvalidCertificate(_))
                    ),
                    "{mode}, {name}, trusted={trusted}: {error:?}"
                );
            }
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 loopback TLS peer and explicit test environment"]
    fn java_tls_deadlines_and_cancellation_cover_handshake_and_ack_stalls() {
        let root = private_root();
        // slow-ack keeps sending valid partial bytes; it must not restart the
        // single absolute deadline on each read and keep negotiation alive.
        for mode in ["handshake-stall", "stall", "slow-ack"] {
            let peer = JavaPeer::start(mode);
            let started = Instant::now();
            let result = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&root),
                Hello::current(300),
                TransportLimits {
                    total_timeout: Duration::from_millis(1500),
                    ..integration_limits()
                },
                Cancellation::new(),
            );
            assert!(
                matches!(result, Err(TransportError::DeadlineExceeded(_))),
                "{mode}: {result:?}"
            );
            assert!(started.elapsed() < Duration::from_secs(3));

            let peer = JavaPeer::start(mode);
            let token = Cancellation::new();
            let cancel = token.clone();
            let interrupter = thread::spawn(move || {
                thread::sleep(Duration::from_millis(300));
                cancel.cancel();
            });
            let started = Instant::now();
            let result = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&root),
                Hello::current(301),
                integration_limits(),
                token,
            );
            interrupter.join().unwrap();
            assert!(
                matches!(result, Err(TransportError::Cancelled)),
                "{mode}: {result:?}"
            );
            assert!(started.elapsed() < Duration::from_secs(2));
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 loopback TLS peer and explicit test environment"]
    fn java_tls_rejects_oversized_frames_and_mismatched_ack_request_ids() {
        let root = private_root();
        for mode in ["oversized", "bad-ack-id"] {
            let peer = JavaPeer::start(mode);
            let result = connect_and_negotiate(
                peer.endpoint,
                "localhost",
                Some(&root),
                Hello::current(400),
                integration_limits(),
                Cancellation::new(),
            );
            assert!(
                matches!(result, Err(TransportError::Protocol(_))),
                "{mode}: {result:?}"
            );
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 loopback TLS peer and explicit test environment"]
    fn java_tls_controller_negotiates_profile_context_then_disconnects_quiescently() {
        use super::super::connection::{ConnectionController, ConnectionState};
        use super::super::profile::ModernProfile;
        use base64::Engine;

        let mut peer = JavaPeer::start("tls");
        let document = serde_json::json!({
            "schema": 1,
            "profile_id": "tls-interop",
            "display_name": "TLS loopback test (not an account login)",
            "shared_resource_root": if cfg!(windows) { "D:\\synthetic-test" } else { "/synthetic-test" },
            "endpoint": peer.endpoint.to_string(),
            "tls_server_name": "localhost",
            "trust": {
                "kind": "private_ca",
                "certificate_der_base64": base64::engine::general_purpose::STANDARD.encode(private_root())
            },
            "connect_timeout_ms": 500,
            "exchange_timeout_ms": 3500
        });
        let profile = ModernProfile::parse(&serde_json::to_vec(&document).unwrap()).unwrap();
        let context = profile.context_key();
        let mut controller = ConnectionController::default();
        let epoch = controller.start(profile).unwrap();
        assert_eq!(controller.state(), ConnectionState::Connecting(epoch));
        assert_eq!(controller.context_key(), Some(context));
        let deadline = Instant::now() + Duration::from_secs(5);
        while matches!(controller.state(), ConnectionState::Connecting(_)) {
            controller.poll();
            assert!(Instant::now() < deadline, "controller negotiation deadline");
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            matches!(controller.state(), ConnectionState::Negotiated(current, _) if current == epoch),
            "state={:?}, error={:?}",
            controller.state(),
            controller.error()
        );
        assert_eq!(controller.context_key(), Some(context));
        assert!(controller.is_quiescent());
        controller.disconnect();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !controller.is_quiescent() {
            controller.poll();
            assert!(Instant::now() < deadline, "controller disconnect deadline");
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(controller.state(), ConnectionState::Disconnected);
        assert!(controller.context_key().is_none());
        assert!(controller.error().is_none());
        peer.expect_successful_exit();
    }

    fn fixture_credentials(password: &str) -> Credentials {
        Credentials::new("fixture-user".into(), password.into()).unwrap()
    }
    fn assert_synthetic_character_data(
        data: &character_data::CharacterData,
        source: u32,
        large: bool,
    ) {
        let base = 4_000_000_000 + i64::from(source) * 100;
        let stats = &data.stats;
        assert_eq!(data.context.key.source_id, source);
        assert_eq!(data.context.key.object_id, 3333);
        assert_eq!(
            data.character.name,
            if source == 7 {
                "Sun Knight"
            } else {
                "Moon Mage"
            }
        );
        assert_eq!(data.character.class, if source == 7 { 1 } else { 3 });
        assert_eq!(data.character.sex, if source == 7 { 0 } else { 1 });
        assert_eq!(data.character.level, if source == 7 { 99 } else { 120001 });
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
            [
                base + 1,
                base + 2,
                base + 101,
                base + 102,
                -(base + 3),
                base + 4,
                base + 5,
                base + 6,
                base + 7,
                base + 8,
                base + 9,
                i64::MAX - i64::from(source),
                -(base + 10),
                base + 11,
                base + 12,
                base + 13,
                base + 14,
                base + 15,
                225
            ]
        );
        assert_eq!(data.items.len(), if large { 512 } else { 3 });
        for (index, item) in data.items.iter().enumerate() {
            let offset = i64::from(source) + index as i64;
            assert_eq!(item.object_id, 5_000_000_000 + index as u64 + 1);
            assert_eq!(item.catalog_id, 6_000_000_000 + index as u64 + 1);
            assert_eq!(
                [
                    item.quantity,
                    item.enchantment,
                    item.durability,
                    item.blessing,
                    item.charge_count,
                    item.inventory_graphic,
                    item.ground_graphic,
                    item.unit_weight
                ],
                [
                    7_000_000_000 + offset,
                    -(8_000_000_000 + offset),
                    9_000_000_000 + offset,
                    index as i64 % 256,
                    10_000_000_000 + offset,
                    11_000_000_000 + offset,
                    12_000_000_000 + offset,
                    13_000_000_000 + offset
                ]
            );
            assert_eq!(
                item.kind,
                [
                    character_data::ItemKind::Weapon,
                    character_data::ItemKind::Armor,
                    character_data::ItemKind::Etc
                ][index % 3]
            );
            assert_eq!(
                (item.equipped, item.identified, item.sealed),
                (index < 2, index % 2 == 0, index % 3 == 0)
            );
            let expected = if large {
                let prefix = format!("S{source}:{index}:");
                format!("{prefix}{}", "x".repeat(256 - prefix.len()))
            } else {
                format!("${source}-{index}")
            };
            assert_eq!(item.name_token, expected);
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_character_data_reads_all_source_fields_and_large_inventory_without_replacing_navigation(
    ) {
        for mode in ["data", "data-large", "auth"] {
            let mut peer =
                JavaPeer::start_class("local.lito381.server.ModernServerTestPeer", mode, "30000");
            let mut connection = application_connection(&peer, Cancellation::new());
            assert!(connection.supports_character_data());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            assert!(
                connection.character_data().is_err(),
                "no data before selected identity"
            );
            for (index, source) in [7, 42].into_iter().enumerate() {
                let epoch = index as u64 + 1;
                let roster = connection.characters(source, epoch).unwrap();
                let selected = connection
                    .select_character(SelectionContext {
                        key: roster.characters[0].key,
                        epoch,
                    })
                    .unwrap();
                for lobby in [false, true] {
                    if lobby {
                        connection.enter_channel(0).unwrap();
                    }
                    let navigation = connection.navigation().cloned().unwrap();
                    let reply = connection.character_data().unwrap();
                    assert_eq!(reply.context, selected.context);
                    if mode == "auth" {
                        assert_eq!(
                            reply.result,
                            Err(character_data::CharacterDataFailure::BackendUnavailable)
                        );
                    } else {
                        let data = reply.result.unwrap();
                        assert_eq!(data.context, selected.context);
                        assert_synthetic_character_data(&data, source, mode == "data-large");
                        assert_ne!(
                            data.character.level,
                            navigation.snapshot.selected.as_ref().unwrap().level
                        );
                    }
                    assert_eq!(connection.navigation(), Some(&navigation));
                    connection.ping().unwrap();
                }
                connection.leave_lobby().unwrap();
                connection.return_characters().unwrap();
                assert!(connection.character_data().is_err());
            }
            connection.close().unwrap();
            peer.expect_successful_exit();
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_character_data_mismatched_context_and_deadline_abort_the_actual_selected_session() {
        for mode in [
            "wrong-data-epoch",
            "wrong-data-object",
            "wrong-data-source",
            "silent-data",
        ] {
            let mut peer = JavaPeer::start_class(
                "local.lito381.server.ModernNavigationTestPeer",
                mode,
                "15000",
            );
            let mut connection = application_connection(&peer, Cancellation::new());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = connection.characters(7, 1).unwrap();
            connection
                .select_character(SelectionContext {
                    key: roster.characters[0].key,
                    epoch: 1,
                })
                .unwrap();
            if mode == "silent-data" {
                connection
                    .set_exchange_timeout(Duration::from_millis(180))
                    .unwrap();
            }
            let error = connection.character_data().unwrap_err();
            if mode == "silent-data" {
                assert!(
                    matches!(
                        error,
                        TransportError::DeadlineExceeded(Stage::CharacterData)
                    ),
                    "{error:?}"
                );
            } else {
                assert!(
                    matches!(error, TransportError::Application(ref error) if error.kind == application::ErrorKind::IdentityMismatch),
                    "{error:?}"
                );
            }
            assert_eq!(connection.application_state(), ApplicationState::Closed);
            assert!(connection.principal().is_none());
            assert!(connection.navigation().is_none());
            peer.expect_successful_exit();
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_character_data_is_not_sent_when_the_negotiated_capabilities_exclude_it() {
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernServerTestPeer", "auth", "15000");
        // This uses the CURRENT codec with an explicit capabilities=7 offer,
        // not a claim that an unreconstructed old codec accepts unknown bit 8.
        let hello = Hello {
            supported_capabilities: 7,
            ..Hello::current(100)
        };
        let mut connection = connect_and_negotiate(
            peer.endpoint,
            "localhost",
            Some(&private_root()),
            hello,
            integration_limits(),
            Cancellation::new(),
        )
        .unwrap();
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        let roster = connection.characters(7, 1).unwrap();
        connection
            .select_character(SelectionContext {
                key: roster.characters[0].key,
                epoch: 1,
            })
            .unwrap();
        assert!(!connection.supports_character_data());
        assert!(matches!(
            connection.character_data(),
            Err(TransportError::InvalidState(_))
        ));
        connection.ping().unwrap();
        assert!(connection.navigation().is_some());
        connection.close().unwrap();
        peer.expect_successful_exit();
    }
    #[test]
    fn busy_return_requires_the_exact_previous_reservation_and_only_population_may_change() {
        let context = SelectionContext {
            key: application::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        };
        for phase in [NavigationPhase::Channels, NavigationPhase::Lobby] {
            let prior = NavigationReply {
                action: NavigationAction::EnterChannel,
                status: NavigationStatus::Ok,
                context,
                snapshot: application::NavigationSnapshot {
                    phase,
                    channel_id: (phase == NavigationPhase::Lobby).then_some(2),
                    selected: Some(application::CharacterSummary {
                        key: context.key,
                        name: "勇者".into(),
                        class: 5,
                        sex: 1,
                        level: i64::MAX,
                    }),
                    channels: (0..4)
                        .map(|id| application::ChannelSummary {
                            id,
                            name: format!("channel{id}"),
                            population: 1,
                            capacity: 100,
                            minimum_level: 1,
                            maximum_level: i64::MAX,
                        })
                        .collect(),
                },
            };
            let mut reply = prior.clone();
            reply.action = NavigationAction::ReturnCharacters;
            reply.status = NavigationStatus::Busy;
            reply.snapshot.channels[3].population = 99;
            assert!(confirms_busy_return(Some(&prior), &reply));
            assert!(
                confirms_busy_return(Some(&reply), &reply),
                "repeated Busy is retryable"
            );
            assert!(!confirms_busy_return(None, &reply));
            for index in 0..19 {
                let mut invalid = reply.clone();
                match index {
                    0 => invalid.context.epoch -= 1,
                    1 => invalid.context.key.source_id = 42,
                    2 => invalid.context.key.object_id += 1,
                    3 => invalid.snapshot.selected.as_mut().unwrap().name = "changed".into(),
                    4 => invalid.snapshot.selected.as_mut().unwrap().level -= 1,
                    5 => invalid.snapshot.selected.as_mut().unwrap().class += 1,
                    6 => invalid.snapshot.selected.as_mut().unwrap().sex = 0,
                    7 => invalid.snapshot.selected.as_mut().unwrap().key.source_id = 42,
                    8 => invalid.snapshot.selected.as_mut().unwrap().key.object_id += 1,
                    9 => invalid.snapshot.selected = None,
                    10 => invalid.snapshot.phase = NavigationPhase::Characters,
                    11 => invalid.snapshot.channel_id = Some(3),
                    12 => invalid.status = NavigationStatus::Ok,
                    13 => invalid.action = NavigationAction::EnterChannel,
                    14 => invalid.snapshot.channels[0].name = "different".into(),
                    15 => invalid.snapshot.channels[0].capacity -= 1,
                    16 => invalid.snapshot.channels[0].minimum_level += 1,
                    17 => invalid.snapshot.channels[0].maximum_level -= 1,
                    _ => invalid.snapshot.channels.swap(0, 1),
                }
                assert!(
                    !confirms_busy_return(Some(&prior), &invalid),
                    "{phase:?}, case {index}"
                );
            }
            let mut changed_phase = reply.clone();
            changed_phase.snapshot.phase = if phase == NavigationPhase::Lobby {
                NavigationPhase::Channels
            } else {
                NavigationPhase::Lobby
            };
            changed_phase.snapshot.channel_id =
                (changed_phase.snapshot.phase == NavigationPhase::Lobby).then_some(2);
            assert!(
                !confirms_busy_return(Some(&prior), &changed_phase),
                "well-shaped but unconfirmed page change"
            );
            let mut missing_channel = reply.clone();
            missing_channel.snapshot.channels.pop();
            assert!(!confirms_busy_return(Some(&prior), &missing_channel));
        }
    }

    #[test]
    fn lobby_leave_ack_requires_the_same_selected_row_and_context_without_a_lobby_membership() {
        let context = SelectionContext {
            key: application::CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch: u64::MAX,
        };
        let prior = NavigationReply {
            action: NavigationAction::EnterChannel,
            status: NavigationStatus::Ok,
            context,
            snapshot: application::NavigationSnapshot {
                phase: NavigationPhase::Lobby,
                channel_id: Some(2),
                selected: Some(application::CharacterSummary {
                    key: context.key,
                    name: "勇者".into(),
                    class: 5,
                    sex: 1,
                    level: i64::MAX,
                }),
                channels: vec![], // The decoder separately validates all four rows.
            },
        };
        let mut reply = prior.clone();
        reply.action = NavigationAction::LeaveLobby;
        reply.snapshot.phase = NavigationPhase::Channels;
        reply.snapshot.channel_id = None;
        assert!(confirms_lobby_leave(Some(&prior), &reply));
        assert!(!confirms_lobby_leave(None, &reply));
        assert!(!confirms_lobby_leave(Some(&reply), &reply));
        for index in 0..10 {
            let mut invalid = reply.clone();
            match index {
                0 => invalid.context.epoch -= 1,
                1 => invalid.context.key.source_id = 42,
                2 => invalid.context.key.object_id += 1,
                3 => invalid.snapshot.selected.as_mut().unwrap().name = "changed".into(),
                4 => invalid.snapshot.selected.as_mut().unwrap().level -= 1,
                5 => invalid.snapshot.selected.as_mut().unwrap().class += 1,
                6 => invalid.snapshot.selected.as_mut().unwrap().sex = 0,
                7 => invalid.snapshot.channel_id = Some(2),
                8 => invalid.status = NavigationStatus::Busy,
                _ => invalid.action = NavigationAction::ReturnCharacters,
            }
            assert!(
                !confirms_lobby_leave(Some(&prior), &invalid),
                "case {index}"
            );
        }
    }
    fn application_connection(peer: &JavaPeer, token: Cancellation) -> ModernTlsConnection {
        let mut connection = connect_and_negotiate(
            peer.endpoint,
            "localhost",
            Some(&private_root()),
            Hello::current(100),
            integration_limits(),
            token,
        )
        .unwrap();
        connection
            .set_exchange_timeout(Duration::from_secs(2))
            .unwrap();
        connection
    }

    #[test]
    #[ignore = "requires prepared Java 17 real production room TLS peer"]
    fn java_rooms_two_tls_owners_password_team_ready_configure_conflict_leave_and_reentry() {
        fn login(peer: &JavaPeer, source: u32) -> (ModernTlsConnection, SelectionContext) {
            let mut connection = application_connection(peer, Cancellation::new());
            assert_eq!(
                connection.ack().unwrap().version,
                super::super::protocol::MAX_VERSION
            );
            assert_eq!(connection.ack().unwrap().capabilities, 303);
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = connection.characters(source, 1).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch: 1,
            };
            connection.select_character(context).unwrap();
            connection.enter_channel(0).unwrap();
            connection
                .rooms(rooms::Request::read(context, 0).unwrap())
                .unwrap();
            (connection, context)
        }
        fn refresh(
            connection: &mut ModernTlsConnection,
            context: SelectionContext,
        ) -> rooms::Reply {
            connection
                .rooms(rooms::Request::read(context, 0).unwrap())
                .unwrap()
        }
        fn common(reply: &rooms::Reply, action: rooms::Action, value: u8) -> rooms::Common {
            let room = reply
                .joined
                .as_ref()
                .map(|room| &room.summary)
                .unwrap_or(&reply.rooms[0]);
            rooms::Common {
                context: reply.common.context,
                channel_id: 0,
                action,
                value,
                room_id: room.room_id,
                expected_revision: room.revision,
            }
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernRoomsTestPeer",
            "normal",
            "30000",
        );
        let (mut owner, owner_context) = login(&peer, 7);
        let (mut guest, guest_context) = login(&peer, 42);
        let mut create = rooms::Common::read(owner_context, 0);
        create.action = rooms::Action::Create;
        let public = rooms::PublicDraft {
            capacity: 4,
            mode: 1,
            attributes: 20,
            map_id: 504,
            name: "Actual TLS 房間".into(),
        };
        let created = owner
            .rooms(
                rooms::Request::draft(
                    create,
                    rooms::Draft::new(public.clone(), "fixture-room-password".into()).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(created.status, rooms::Status::Ok);
        assert_eq!(created.joined.as_ref().unwrap().members.len(), 1);
        assert!(owner.leave_lobby().is_err());
        assert!(owner.return_characters().is_err());
        let directory = refresh(&mut guest, guest_context);
        let join = common(&directory, rooms::Action::Join, 0);
        let denied = guest
            .rooms(rooms::Request::join(join, "wrong-fixture-password".into()).unwrap())
            .unwrap();
        assert_eq!(denied.reason, rooms::Reason::BadPassword);
        assert!(denied.joined.is_none());
        let joined = guest
            .rooms(rooms::Request::join(join, "fixture-room-password".into()).unwrap())
            .unwrap();
        assert_eq!(joined.status, rooms::Status::Ok);
        assert_eq!(joined.joined.as_ref().unwrap().members.len(), 2);
        assert_eq!(joined.joined.as_ref().unwrap().members[0].key.source_id, 7);
        assert_eq!(joined.joined.as_ref().unwrap().members[1].key.source_id, 42);
        assert_eq!(joined.joined.as_ref().unwrap().members[1].level, 120000);
        let current = refresh(&mut owner, owner_context);
        let owner_team = owner
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Team, 0)).unwrap())
            .unwrap();
        assert_eq!(owner_team.status, rooms::Status::Ok);
        let current = refresh(&mut guest, guest_context);
        let guest_team = guest
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Team, 1)).unwrap())
            .unwrap();
        assert_eq!(guest_team.status, rooms::Status::Ok);
        let current = refresh(&mut owner, owner_context);
        owner
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Ready, 1)).unwrap())
            .unwrap();
        let current = refresh(&mut guest, guest_context);
        let ready = guest
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Ready, 1)).unwrap())
            .unwrap();
        assert!(ready.joined.as_ref().unwrap().summary.satisfied);
        let current = refresh(&mut owner, owner_context);
        let mut edited = public;
        edited.name = "重新設定房間".into();
        edited.attributes |= 32;
        let configured = owner
            .rooms(
                rooms::Request::draft(
                    common(&current, rooms::Action::Configure, 0),
                    rooms::Draft::new(edited.clone(), "fixture-room-password".into()).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(configured.status, rooms::Status::Ok);
        assert_eq!(configured.joined.as_ref().unwrap().summary.settings, edited);
        assert!(configured
            .joined
            .as_ref()
            .unwrap()
            .members
            .iter()
            .all(|m| !m.ready));
        let mut stale = common(&configured, rooms::Action::Ready, 1);
        stale.expected_revision = 1;
        let conflict = owner.rooms(rooms::Request::simple(stale).unwrap()).unwrap();
        assert_eq!(conflict.status, rooms::Status::Conflict);
        let current = refresh(&mut guest, guest_context);
        let left = guest
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Leave, 0)).unwrap())
            .unwrap();
        assert!(left.joined.is_none());
        let current = refresh(&mut owner, owner_context);
        assert_eq!(current.joined.as_ref().unwrap().members.len(), 1);
        let left = owner
            .rooms(rooms::Request::simple(common(&current, rooms::Action::Leave, 0)).unwrap())
            .unwrap();
        assert!(left.joined.is_none());
        assert!(left.rooms.is_empty());
        for (connection, old_context) in [(&mut owner, owner_context), (&mut guest, guest_context)]
        {
            connection.leave_lobby().unwrap();
            connection.return_characters().unwrap();
            let roster = connection.characters(old_context.key.source_id, 2).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch: 2,
            };
            connection.select_character(context).unwrap();
            connection.enter_channel(0).unwrap();
            let empty = refresh(connection, context);
            assert!(empty.joined.is_none());
            assert!(empty.rooms.is_empty());
            connection.leave_lobby().unwrap();
            connection.return_characters().unwrap();
            connection.close().unwrap();
        }
        peer.expect_successful_exit();
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "requires Java real room/equipment authorities and production TLS listener"]
    fn java_rooms_controller_periodic_read_and_gear_set_publish_confirmed_unready_together() {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;
        fn wait(controller: &mut impl LoginController, phase: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(6);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == phase && !view.equipment_pending && !view.room_pending {
                    return;
                }
                assert!(Instant::now() < deadline, "{}", view.status);
                thread::sleep(Duration::from_millis(10));
            }
        }
        fn command(controller: &mut impl LoginController, action: rooms::Action, value: u8) {
            let rooms::State::Ready(reply) = controller.snapshot().rooms else {
                panic!()
            };
            let common = if action == rooms::Action::Read {
                rooms::Common::read(reply.common.context, 0)
            } else {
                let room = reply.joined.as_ref().unwrap();
                rooms::Common {
                    context: reply.common.context,
                    channel_id: 0,
                    action,
                    value,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                }
            };
            controller
                .command(LoginCommand::Rooms(rooms::Request::simple(common).unwrap()))
                .unwrap();
            wait(controller, LoginPhase::InLobby);
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernRoomsTestPeer",
            "equipment",
            "30000",
        );
        let profile=ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1,"profile_id":"room-gear-owner","display_name":"Room gear owner",
            "shared_resource_root":"D:\\synthetic-test","endpoint":peer.endpoint.to_string(),"tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500,"exchange_timeout_ms":2000
        })).unwrap()).unwrap();
        let mut controller = SessionController::new(profile);
        controller.command(LoginCommand::Connect).unwrap();
        wait(&mut controller, LoginPhase::LoginReady);
        controller
            .command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            })
            .unwrap();
        wait(&mut controller, LoginPhase::ChoosingSource);
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        wait(&mut controller, LoginPhase::CharactersReady);
        let rows = controller.snapshot();
        let context = SelectionContext {
            key: application::CharacterKey {
                source_id: 7,
                object_id: rows.characters[0].object_id,
            },
            epoch: rows.roster_epoch,
        };
        controller
            .command(LoginCommand::SelectCharacter(context))
            .unwrap();
        wait(&mut controller, LoginPhase::ChoosingChannel);
        controller
            .command(LoginCommand::EnterChannel {
                context,
                channel_id: 0,
            })
            .unwrap();
        wait(&mut controller, LoginPhase::InLobby);
        let mut common = rooms::Common::read(context, 0);
        common.action = rooms::Action::Create;
        controller
            .command(LoginCommand::Rooms(
                rooms::Request::draft(
                    common,
                    rooms::Draft::new(
                        rooms::PublicDraft {
                            capacity: 4,
                            mode: 1,
                            map_id: 504,
                            attributes: 16,
                            name: "Gear readiness".into(),
                        },
                        String::new(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ))
            .unwrap();
        wait(&mut controller, LoginPhase::InLobby);
        let mut guest = application_connection(&peer, Cancellation::new());
        assert_eq!(
            guest.ack().unwrap().version,
            super::super::protocol::MAX_VERSION
        );
        assert_eq!(guest.ack().unwrap().capabilities, 319);
        guest
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        let rows = guest.characters(42, 1).unwrap();
        let guest_context = SelectionContext {
            key: rows.characters[0].key,
            epoch: 1,
        };
        guest.select_character(guest_context).unwrap();
        guest.enter_channel(0).unwrap();
        let directory = guest
            .rooms(rooms::Request::read(guest_context, 0).unwrap())
            .unwrap();
        let room = &directory.rooms[0];
        guest
            .rooms(
                rooms::Request::join(
                    rooms::Common {
                        context: guest_context,
                        channel_id: 0,
                        action: rooms::Action::Join,
                        value: 0,
                        room_id: room.room_id,
                        expected_revision: room.revision,
                    },
                    String::new(),
                )
                .unwrap(),
            )
            .unwrap();
        // No UI polling or command is involved in this sleep. The independent
        // TLS owner must learn the second member from its periodic room READ.
        thread::sleep(Duration::from_millis(2200));
        controller.poll();
        assert_eq!(
            controller.snapshot().rooms.joined().unwrap().members.len(),
            2
        );
        command(&mut controller, rooms::Action::Ready, 1);
        assert!(
            controller
                .snapshot()
                .rooms
                .joined()
                .unwrap()
                .member(context.key)
                .unwrap()
                .ready
        );
        let before = controller.snapshot();
        controller
            .command(LoginCommand::Equipment(equipment::Request::read(context)))
            .unwrap();
        wait(&mut controller, LoginPhase::InLobby);
        assert!(
            controller
                .snapshot()
                .rooms
                .joined()
                .unwrap()
                .member(context.key)
                .unwrap()
                .ready,
            "gear READ must not unready"
        );
        let equipment::State::Ready(gear) = controller.snapshot().equipment else {
            panic!()
        };
        controller
            .command(LoginCommand::Equipment(equipment::Request::set(
                &gear,
                5_000_000_002,
                true,
            )))
            .unwrap();
        assert!(controller.snapshot().equipment_pending);
        assert_eq!(controller.snapshot().rooms, before.rooms);
        wait(&mut controller, LoginPhase::InLobby);
        let after = controller.snapshot();
        let equipment::State::Ready(confirmed) = after.equipment else {
            panic!()
        };
        assert_eq!(confirmed.revision, 1);
        assert_eq!(confirmed.equipped(5_000_000_002), Some(true));
        assert!(
            !after
                .rooms
                .joined()
                .unwrap()
                .member(context.key)
                .unwrap()
                .ready,
            "SET must READ room before publishing its completed epoch"
        );
        assert!(
            after.rooms.joined().unwrap().summary.revision
                > before.rooms.joined().unwrap().summary.revision
        );
        guest.ping().unwrap();
        let directory = guest
            .rooms(rooms::Request::read(guest_context, 0).unwrap())
            .unwrap();
        let room = directory.joined.as_ref().unwrap();
        guest
            .rooms(
                rooms::Request::simple(rooms::Common {
                    context: guest_context,
                    channel_id: 0,
                    action: rooms::Action::Leave,
                    value: 0,
                    room_id: room.summary.room_id,
                    expected_revision: room.summary.revision,
                })
                .unwrap(),
            )
            .unwrap();
        guest.leave_lobby().unwrap();
        guest.return_characters().unwrap();
        guest.close().unwrap();
        command(&mut controller, rooms::Action::Read, 0);
        command(&mut controller, rooms::Action::Leave, 0);
        controller
            .command(LoginCommand::LeaveLobby(context))
            .unwrap();
        wait(&mut controller, LoginPhase::ChoosingChannel);
        controller
            .command(LoginCommand::ReturnCharacters(context))
            .unwrap();
        wait(&mut controller, LoginPhase::CharactersReady);
        controller.command(LoginCommand::Shutdown).unwrap();
        wait(&mut controller, LoginPhase::Disconnected);
        assert!(controller.is_quiescent());
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_authority_set_conflict_denied_source_isolation_and_back_reload() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentTestPeer",
            "normal",
            "30000",
        );
        let mut connection = application_connection(&peer, Cancellation::new());
        assert!(connection.supports_equipment());
        assert_eq!(connection.ack().unwrap().capabilities, 31);
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        for (source, epoch, expected_revision) in [(7, 1, 0), (42, 2, 0), (7, 3, 1)] {
            let roster = connection.characters(source, epoch).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch,
            };
            connection.select_character(context).unwrap();
            let sql = connection.character_data().unwrap().result.unwrap();
            assert!(
                sql.items
                    .iter()
                    .find(|item| item.object_id == 5_000_000_001)
                    .unwrap()
                    .equipped
            );
            assert!(
                !sql.items
                    .iter()
                    .find(|item| item.object_id == 5_000_000_002)
                    .unwrap()
                    .equipped
            );
            let read = connection
                .equipment(equipment::Request::read(context))
                .unwrap();
            assert_eq!(read.status, equipment::Status::Ok);
            let before = read.snapshot.unwrap();
            assert_eq!(before.revision, expected_revision);
            assert_eq!(before.equipped(5_000_000_002), Some(expected_revision == 1));
            assert_eq!(
                before.modifiers[0],
                if expected_revision == 1 { 22 } else { 11 }
            );
            assert_eq!(
                before.modifiers[6],
                if expected_revision == 1 {
                    5_000_000_042
                } else {
                    5_000_000_007
                }
            );
            if source == 7 && epoch == 1 {
                connection.enter_channel(2).unwrap();
                let prior_navigation = connection.navigation().cloned();
                let request = equipment::Request::set(&before, 5_000_000_002, true);
                let result = connection.equipment(request).unwrap();
                assert!(result.changed);
                let after = result.snapshot.unwrap();
                assert_eq!(after.revision, 1);
                assert_eq!(after.equipped(5_000_000_001), Some(false));
                assert_eq!(after.equipped(5_000_000_002), Some(true));
                assert_eq!(after.modifiers[0], 22);
                assert_eq!(after.modifiers[6], 5_000_000_042);
                assert_ne!(before.view_token, after.view_token);
                assert_eq!(connection.navigation(), prior_navigation.as_ref());
                let conflict = connection.equipment(request).unwrap();
                assert_eq!(conflict.status, equipment::Status::Conflict);
                assert!(!conflict.changed);
                assert_eq!(conflict.snapshot.as_ref().unwrap().revision, 1);
                let denied = connection
                    .equipment(equipment::Request::set(&after, 5_000_000_003, true))
                    .unwrap();
                assert_eq!(denied.status, equipment::Status::Denied);
                assert_eq!(denied.reason, Some(equipment::Reason::NotEquipment));
                let unchanged = denied.snapshot.unwrap();
                assert_eq!(unchanged.revision, 1);
                assert_eq!(unchanged.equipped(5_000_000_003), Some(false));
                let no_op = connection
                    .equipment(equipment::Request::set(&unchanged, 5_000_000_002, true))
                    .unwrap();
                assert_eq!(no_op.status, equipment::Status::Ok);
                assert!(!no_op.changed);
                assert_eq!(no_op.snapshot.unwrap().revision, 1);
                // Original DATA still reports SQL flags; it cannot overwrite gear's confirmed loadout.
                assert!(connection.character_data().unwrap().result.unwrap().items[0].equipped);
                connection.ping().unwrap();
                connection.leave_lobby().unwrap();
            }
            assert_eq!(
                connection.return_characters().unwrap().status,
                NavigationStatus::Ok
            );
            assert!(connection
                .equipment(equipment::Request::read(context))
                .is_err());
        }
        connection.close().unwrap();
        peer.expect_successful_exit();
    }
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_reconnect_full_read_recovers_confirmed_private_revision() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentTestPeer",
            "reconnect",
            "30000",
        );
        for epoch in [1, 2] {
            let mut connection = application_connection(&peer, Cancellation::new());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = connection.characters(7, epoch).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch,
            };
            connection.select_character(context).unwrap();
            connection.character_data().unwrap();
            let before = connection
                .equipment(equipment::Request::read(context))
                .unwrap()
                .snapshot
                .unwrap();
            assert_eq!(before.revision, epoch as i64 - 1);
            if epoch == 1 {
                let after = connection
                    .equipment(equipment::Request::set(&before, 5_000_000_002, true))
                    .unwrap();
                assert_eq!(after.status, equipment::Status::Ok);
                assert!(after.changed);
            } else {
                assert_eq!(before.equipped(5_000_000_001), Some(false));
                assert_eq!(before.equipped(5_000_000_002), Some(true));
                assert_eq!(before.modifiers[6], 5_000_000_042);
            }
            connection.close().unwrap();
        }
        peer.expect_successful_exit();
    }
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_uncertain_atomic_commit_closes_without_ack_or_automatic_resend() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentTestPeer",
            "uncertain",
            "15000",
        );
        let mut connection = application_connection(&peer, Cancellation::new());
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        let roster = connection.characters(7, 1).unwrap();
        let context = SelectionContext {
            key: roster.characters[0].key,
            epoch: 1,
        };
        connection.select_character(context).unwrap();
        let initial = connection
            .equipment(equipment::Request::read(context))
            .unwrap()
            .snapshot
            .unwrap();
        let request = equipment::Request::set(&initial, 5_000_000_002, true);
        assert!(
            connection.equipment(request).is_err(),
            "post-rename uncertainty must never masquerade as a failure ACK or success"
        );
        assert_eq!(connection.state(), ClientState::Closed);
        let last = connection.last_request_id;
        assert!(connection.equipment(request).is_err());
        assert_eq!(connection.last_request_id, last);
        assert_eq!(
            initial.equipped(5_000_000_002),
            Some(false),
            "unconfirmed old snapshot is immutable"
        );
        peer.expect_successful_exit(); // Peer independently reopens store and verifies durable revision.
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_controller_ack_only_updates_and_keeps_owned_heartbeat_and_shutdown() {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;
        fn wait(controller: &mut impl LoginController, phase: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(6);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == phase && !view.equipment_pending {
                    return;
                }
                assert!(Instant::now() < deadline, "{}", view.status);
                thread::sleep(Duration::from_millis(10));
            }
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentTestPeer",
            "normal",
            "30000",
        );
        let profile=ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1,"profile_id":"equipment-owner-test","display_name":"Equipment authority controller test",
            "shared_resource_root":"D:\\synthetic-test","endpoint":peer.endpoint.to_string(),"tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500,"exchange_timeout_ms":2000
        })).unwrap()).unwrap();
        let mut controller = SessionController::new(profile);
        controller.command(LoginCommand::Connect).unwrap();
        wait(&mut controller, LoginPhase::LoginReady);
        controller
            .command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            })
            .unwrap();
        wait(&mut controller, LoginPhase::ChoosingSource);
        for (source, expected_revision) in [(7, 0), (42, 0), (7, 1)] {
            controller
                .command(LoginCommand::SelectSource(source))
                .unwrap();
            wait(&mut controller, LoginPhase::CharactersReady);
            let rows = controller.snapshot();
            let context = SelectionContext {
                key: application::CharacterKey {
                    source_id: source,
                    object_id: rows.characters[0].object_id,
                },
                epoch: rows.roster_epoch,
            };
            controller
                .command(LoginCommand::SelectCharacter(context))
                .unwrap();
            wait(&mut controller, LoginPhase::ChoosingChannel);
            let equipment::State::Ready(initial) = controller.snapshot().equipment else {
                panic!("SELECT must load equipment on the same TLS owner")
            };
            assert_eq!(initial.revision, expected_revision);
            if source == 7 && expected_revision == 0 {
                let request = equipment::Request::set(&initial, 5_000_000_002, true);
                controller
                    .command(LoginCommand::Equipment(request))
                    .unwrap();
                assert!(controller.snapshot().equipment_pending);
                let equipment::State::Ready(retained) = controller.snapshot().equipment else {
                    panic!()
                };
                assert!(Arc::ptr_eq(&retained, &initial));
                assert!(controller
                    .command(LoginCommand::ReturnCharacters(context))
                    .is_err());
                assert!(controller
                    .command(LoginCommand::Equipment(request))
                    .is_err());
                wait(&mut controller, LoginPhase::ChoosingChannel);
                let equipment::State::Ready(current) = controller.snapshot().equipment else {
                    panic!()
                };
                assert_eq!(current.revision, 1);
                assert_eq!(current.equipped(5_000_000_002), Some(true));
                controller
                    .command(LoginCommand::Equipment(equipment::Request::set(
                        &current,
                        5_000_000_003,
                        true,
                    )))
                    .unwrap();
                wait(&mut controller, LoginPhase::ChoosingChannel);
                assert!(controller.snapshot().status.contains("拒絕"));
                controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id: 2,
                    })
                    .unwrap();
                wait(&mut controller, LoginPhase::InLobby);
                thread::sleep(Duration::from_millis(2200));
                controller
                    .command(LoginCommand::Equipment(equipment::Request::read(context)))
                    .unwrap();
                wait(&mut controller, LoginPhase::InLobby);
                let equipment::State::Ready(current) = controller.snapshot().equipment else {
                    panic!()
                };
                assert_eq!(current.revision, 1);
                controller
                    .command(LoginCommand::LeaveLobby(context))
                    .unwrap();
                wait(&mut controller, LoginPhase::ChoosingChannel);
            }
            controller
                .command(LoginCommand::ReturnCharacters(context))
                .unwrap();
            wait(&mut controller, LoginPhase::CharactersReady);
            assert_eq!(
                controller.snapshot().equipment,
                equipment::State::NotRequested
            );
        }
        controller.command(LoginCommand::Shutdown).unwrap();
        wait(&mut controller, LoginPhase::Disconnected);
        assert!(controller.is_quiescent());
        peer.expect_successful_exit();
    }
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_back_busy_keeps_channels_lobby_roster_and_data_until_retry() {
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentBusyTestPeer",
            "back-busy",
            "30000",
        );
        let mut connection = application_connection(&peer, Cancellation::new());
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        for (source, epoch, lobby) in [(7, 1, false), (42, u64::MAX, true)] {
            let roster = connection.characters(source, epoch).unwrap();
            let context = SelectionContext {
                key: roster.characters[0].key,
                epoch,
            };
            connection.select_character(context).unwrap();
            if lobby {
                connection.enter_channel(2).unwrap();
            }
            let prior = connection.navigation().cloned().unwrap();
            let data = connection.character_data().unwrap();
            assert!(data.result.is_ok());
            let prior_roster = connection.last_roster.clone();
            assert!(prior_roster.is_some());
            let busy = connection.return_characters().unwrap();
            assert_eq!(busy.status, NavigationStatus::Busy);
            assert!(confirms_busy_return(Some(&prior), &busy));
            assert_eq!(connection.navigation(), Some(&busy));
            assert_eq!(
                connection.last_roster, prior_roster,
                "refusal cannot clear confirmed source roster"
            );
            assert_eq!(
                connection.application_state(),
                ApplicationState::Authenticated
            );
            // Neither this refused local command nor the Busy response may send
            // CHAR. The peer independently rejects actual reads before retry.
            assert!(matches!(
                connection.characters(source, 2),
                Err(TransportError::InvalidState(_))
            ));
            connection.ping().unwrap();
            assert_eq!(connection.character_data().unwrap(), data);
            let returned = connection.return_characters().unwrap();
            assert_eq!(returned.status, NavigationStatus::Ok);
            assert_eq!(returned.snapshot.phase, NavigationPhase::Characters);
            assert_eq!(returned.context, context);
            assert!(connection.navigation().is_none());
            assert!(connection.last_roster.is_none());
        }
        connection.close().unwrap();
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_back_busy_wrong_phase_or_selected_row_aborts_the_real_owner() {
        for mode in ["back-busy-wrong-phase", "back-busy-wrong-selected"] {
            for lobby in [false, true] {
                let mut peer = JavaPeer::start_class(
                    "local.lito381.server.ModernEquipmentBusyTestPeer",
                    mode,
                    "15000",
                );
                let mut connection = application_connection(&peer, Cancellation::new());
                connection
                    .authenticate(fixture_credentials("fixture-password"))
                    .unwrap();
                let roster = connection.characters(7, u64::MAX).unwrap();
                connection
                    .select_character(SelectionContext {
                        key: roster.characters[0].key,
                        epoch: roster.epoch,
                    })
                    .unwrap();
                if lobby {
                    connection.enter_channel(2).unwrap();
                }
                let error = connection.return_characters().unwrap_err();
                assert!(
                    matches!(error, TransportError::Application(ref error) if error.kind == application::ErrorKind::IdentityMismatch),
                    "{mode}, lobby {lobby}: {error:?}"
                );
                assert_eq!(connection.application_state(), ApplicationState::Closed);
                assert!(connection.principal().is_none());
                assert!(connection.navigation().is_none());
                assert!(connection.last_roster.is_none());
                peer.expect_successful_exit(); // Requires real held operation and selection retirement.
            }
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_navigation_rejects_wrong_source_epoch_channel_and_closes_uncertain_commit()
    {
        for mode in [
            "wrong-source",
            "wrong-object",
            "wrong-action",
            "wrong-epoch",
            "wrong-channel",
            "denied-channel",
            "silent-enter",
            "wrong-leave-epoch",
            "wrong-leave-selected",
            "silent-leave",
        ] {
            let mut peer = JavaPeer::start_class(
                "local.lito381.server.ModernNavigationTestPeer",
                mode,
                "15000",
            );
            let mut connection = application_connection(&peer, Cancellation::new());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let roster = connection.characters(7, 1).unwrap();
            let context = application::SelectionContext {
                key: roster.characters[0].key,
                epoch: 1,
            };
            let failure = if matches!(
                mode,
                "wrong-source" | "wrong-object" | "wrong-action" | "wrong-epoch"
            ) {
                connection.select_character(context).unwrap_err()
            } else if matches!(
                mode,
                "wrong-leave-epoch" | "wrong-leave-selected" | "silent-leave"
            ) {
                connection.select_character(context).unwrap();
                connection.enter_channel(0).unwrap();
                if mode == "silent-leave" {
                    connection
                        .set_exchange_timeout(Duration::from_millis(180))
                        .unwrap();
                }
                connection.leave_lobby().unwrap_err()
            } else {
                connection.select_character(context).unwrap();
                if mode == "silent-enter" {
                    connection
                        .set_exchange_timeout(Duration::from_millis(180))
                        .unwrap();
                }
                connection.enter_channel(0).unwrap_err()
            };
            if matches!(mode, "silent-enter" | "silent-leave") {
                assert!(
                    matches!(failure, TransportError::DeadlineExceeded(Stage::Navigation)),
                    "{failure:?}"
                );
            } else {
                assert!(
                    matches!(failure, TransportError::Application(ref error) if error.kind == application::ErrorKind::IdentityMismatch),
                    "{failure:?}"
                );
            }
            assert_eq!(connection.application_state(), ApplicationState::Closed);
            assert!(connection.principal().is_none());
            assert!(connection.navigation().is_none());
            peer.expect_successful_exit(); // Requires real server selection/membership cleanup, even after committed JOIN.
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_navigation_cycles_all_four_channels_and_clears_server_presence_on_disconnect(
    ) {
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernServerTestPeer", "auth", "30000");
        let mut connection = application_connection(&peer, Cancellation::new());
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        let mut epoch = 1;
        for source in [7, 42, 7] {
            for channel in 0..4 {
                let roster = connection.characters(source, epoch).unwrap();
                let context = application::SelectionContext {
                    key: roster.characters[0].key,
                    epoch,
                };
                let selected = connection.select_character(context).unwrap();
                assert_eq!(selected.status, application::NavigationStatus::Ok);
                assert_eq!(
                    selected.snapshot.phase,
                    application::NavigationPhase::Channels
                );
                assert_eq!(selected.snapshot.channels.len(), 4);
                assert!(selected
                    .snapshot
                    .channels
                    .iter()
                    .all(|row| row.population == 0));
                assert!(
                    connection.characters(source, epoch + 1).is_err(),
                    "selected identity must be released before source reload"
                );
                assert!(
                    connection.select_character(context).is_err(),
                    "no duplicate enter mutation"
                );
                let lobby = connection.enter_channel(channel).unwrap();
                assert_eq!(lobby.status, application::NavigationStatus::Ok);
                assert_eq!(lobby.snapshot.phase, application::NavigationPhase::Lobby);
                assert_eq!(lobby.snapshot.channel_id, Some(channel));
                assert_eq!(lobby.snapshot.channels[channel as usize].population, 1);
                assert_eq!(lobby.snapshot.selected.as_ref().unwrap().key, context.key);
                connection.ping().unwrap();
                let left = connection.leave_lobby().unwrap();
                assert_eq!(left.action, NavigationAction::LeaveLobby);
                assert_eq!(left.context, context);
                assert_eq!(left.snapshot.phase, NavigationPhase::Channels);
                assert_eq!(left.snapshot.channel_id, None);
                assert_eq!(left.snapshot.selected, lobby.snapshot.selected);
                assert!(left.snapshot.channels.iter().all(|row| row.population == 0));
                assert_eq!(connection.navigation(), Some(&left));
                assert!(
                    connection.leave_lobby().is_err(),
                    "only a lobby has membership to leave"
                );
                assert!(
                    connection.characters(source, epoch + 1).is_err(),
                    "leaving lobby keeps the character reservation"
                );
                let rejoined = connection.enter_channel(channel).unwrap();
                assert_eq!(rejoined.snapshot.channel_id, Some(channel));
                assert_eq!(rejoined.snapshot.channels[channel as usize].population, 1);
                assert_eq!(rejoined.context, context);
                assert_eq!(rejoined.snapshot.selected, lobby.snapshot.selected);
                connection.leave_lobby().unwrap();
                let returned = connection.return_characters().unwrap();
                assert_eq!(
                    returned.snapshot.phase,
                    application::NavigationPhase::Characters
                );
                assert!(connection.navigation().is_none());
                assert!(
                    connection.select_character(context).is_err(),
                    "return requires a new server-confirmed roster"
                );
                epoch += 1;
            }
        }
        let roster = connection.characters(42, epoch).unwrap();
        connection
            .select_character(application::SelectionContext {
                key: roster.characters[0].key,
                epoch,
            })
            .unwrap();
        connection.enter_channel(0).unwrap();
        connection.close().unwrap(); // Deliberately no BACK: server finally must clear its real locks/counts.
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_real_database_readonly_all_characters_four_channels_and_return() {
        // This is separately opt-in: never infer a database configuration or
        // embed credentials in a test artifact. The Java peer independently
        // validates the exact native files and numeric loopback SQL endpoint.
        let configuration = PathBuf::from(
            std::env::var_os("LITO_MODERN_REAL_DB_SERVER_CONFIG")
                .expect("explicit readonly database configuration is required"),
        );
        assert!(configuration.is_absolute());
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ExpectedCharacter {
            object_id: u64,
            class: u16,
            sex: u8,
            level: i64,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ExpectedSource {
            source_id: u32,
            source_name: String,
            characters: Vec<ExpectedCharacter>,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ExpectedItem {
            object_id: u64,
            catalog_id: u64,
            count: i64,
            equipped: bool,
            enchant: i64,
            durability: i64,
            identified: bool,
            blessing: i64,
            sealed: bool,
            charges: i64,
            kind: String,
            inventory_gfx: i64,
            ground_gfx: i64,
            catalog_weight_units: i64,
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ExpectedData {
            phase: String,
            source_id: u32,
            object_id: u64,
            values_kind: String,
            values: std::collections::BTreeMap<String, i64>,
            inventory_scope: String,
            item_count: usize,
            items: Vec<ExpectedItem>,
        }
        fn matches_data(expected: &ExpectedData, data: &character_data::CharacterData) -> bool {
            let stats = &data.stats;
            let values = [
                ("TYPE", i64::from(data.character.class)),
                ("SEX", i64::from(data.character.sex)),
                ("LEVEL", data.character.level),
                ("MAX_HP", stats.max_hp),
                ("MAX_MP", stats.max_mp),
                ("CURRENT_HP", stats.saved_hp),
                ("CURRENT_MP", stats.saved_mp),
                ("AC", stats.armor_class),
                ("STR", stats.strength),
                ("DEX", stats.dexterity),
                ("CON", stats.constitution),
                ("INT", stats.intelligence),
                ("WIS", stats.wisdom),
                ("CHA", stats.charisma),
                ("EXP", stats.experience),
                ("LAWFUL", stats.lawful),
                ("ORIGINAL_STR", stats.original_strength),
                ("ORIGINAL_DEX", stats.original_dexterity),
                ("ORIGINAL_CON", stats.original_constitution),
                ("ORIGINAL_INT", stats.original_intelligence),
                ("ORIGINAL_WIS", stats.original_wisdom),
                ("FOOD225", stats.food225),
            ];
            expected.source_id == data.context.key.source_id
                && expected.object_id == data.context.key.object_id
                && values
                    .iter()
                    .all(|(key, value)| expected.values.get(*key) == Some(value))
                && data.items.len() == expected.items.len()
                && data
                    .items
                    .iter()
                    .zip(&expected.items)
                    .all(|(item, expected)| {
                        item.object_id == expected.object_id
                            && item.catalog_id == expected.catalog_id
                            && item.quantity == expected.count
                            && item.equipped == expected.equipped
                            && item.enchantment == expected.enchant
                            && item.durability == expected.durability
                            && item.identified == expected.identified
                            && item.blessing == expected.blessing
                            && item.sealed == expected.sealed
                            && item.charge_count == expected.charges
                            && item.inventory_graphic == expected.inventory_gfx
                            && item.ground_graphic == expected.ground_gfx
                            && item.unit_weight == expected.catalog_weight_units
                            && expected.kind
                                == match item.kind {
                                    character_data::ItemKind::Weapon => "WEAPON",
                                    character_data::ItemKind::Armor => "ARMOR",
                                    character_data::ItemKind::Etc => "ETC",
                                }
                    })
        }
        // Expected IDs/counts/type/sex/level come from the separate readonly
        // probe, not from this TLS reply itself. Never put account or character
        // names in this environment value or copy raw DB rows into artifacts.
        let expected_json =
            std::env::var("LITO_MODERN_REAL_DB_EXPECTED_ROSTERS").unwrap_or_else(|_| {
                panic!("explicit independently checked source rosters are required")
            });
        assert!(
            expected_json.len() <= 8192,
            "bounded expected source rosters required"
        );
        let expected_sources: Vec<ExpectedSource> = serde_json::from_str(&expected_json)
            .unwrap_or_else(|_| panic!("invalid expected source roster shape"));
        assert!(!expected_sources.is_empty() && expected_sources.len() <= 16);
        assert!(expected_sources.iter().all(|source| {
            source.source_id > 0
                && source.source_id <= i32::MAX as u32
                && !source.source_name.is_empty()
                && source.source_name.len() <= 256
                && !source.characters.is_empty()
                && source.characters.len() <= 64
                && source.characters.iter().all(|row| {
                    row.object_id > 0
                        && row.object_id <= i64::MAX as u64
                        && row.sex <= 1
                        && row.level >= 15
                })
                && source
                    .characters
                    .windows(2)
                    .all(|rows| rows[0].object_id < rows[1].object_id)
        }));
        assert!(expected_sources
            .windows(2)
            .all(|sources| sources[0].source_id < sources[1].source_id));
        let expected_json = load_expected_character_data(
            std::env::var_os("LITO_MODERN_REAL_DB_EXPECTED_DATA"),
            std::env::var_os("LITO_MODERN_REAL_DB_EXPECTED_DATA_FILE"),
        )
        .unwrap_or_else(|message| panic!("{message}"));
        let expected_data: Vec<ExpectedData> = serde_json::from_str(&expected_json)
            .unwrap_or_else(|_| panic!("invalid expected source data shape"));
        assert_eq!(
            expected_data.len(),
            expected_sources
                .iter()
                .map(|source| source.characters.len())
                .sum::<usize>()
        );
        let mut data_keys = std::collections::BTreeSet::new();
        for data in &expected_data {
            assert!(
                data.phase == "character-data"
                    && data.values_kind == "persisted_not_effective"
                    && data.inventory_scope == "character_items_only_no_private_sidecar"
                    && data.values.len() == 23
                    && data.values.contains_key("CLASS_SPRITE")
                    && data.item_count == data.items.len()
                    && data.items.len() <= character_data::MAX_ITEMS
            );
            assert!(data_keys.insert((data.source_id, data.object_id)));
            assert!(expected_sources.iter().any(|source| {
                source.source_id == data.source_id
                    && source
                        .characters
                        .iter()
                        .any(|row| row.object_id == data.object_id)
            }));
        }
        let credentials = application::Credentials::new(
            std::env::var("LITO_MODERN_PROBE_ACCOUNT")
                .unwrap_or_else(|_| panic!("explicit test account is required")),
            std::env::var("LITO_MODERN_PROBE_PASSWORD")
                .unwrap_or_else(|_| panic!("explicit test password is required")),
        )
        .expect("invalid explicit test credential shape");
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernDatabaseNavigationTestPeer",
            "readonly",
            "30000",
        );
        let mut connection = application_connection(&peer, Cancellation::new());
        connection
            .set_exchange_timeout(Duration::from_secs(6))
            .unwrap();
        let principal = connection
            .authenticate(credentials)
            .expect("real database AUTH failed");
        assert!(
            principal.sources.len() == expected_sources.len()
                && principal
                    .sources
                    .iter()
                    .zip(&expected_sources)
                    .all(|(source, expected)| source.id == expected.source_id
                        && source.name == expected.source_name),
            "real database authorization must include exactly the explicit expected sources"
        );
        let mut epoch = 1;
        for expected in expected_sources.iter().chain(expected_sources.first()) {
            let source = expected.source_id;
            let roster = connection
                .characters(source, epoch)
                .expect("real source roster failed");
            assert!(
                roster.characters.len() == expected.characters.len()
                    && roster.characters.iter().zip(&expected.characters).all(
                        |(row, expected)| row.key.source_id == source
                            && row.key.object_id == expected.object_id
                            && row.class == expected.class
                            && row.sex == expected.sex
                            && row.level == expected.level
                    ),
                "real source must return every independently checked character with correct fields"
            );
            let keys: Vec<_> = roster.characters.iter().map(|row| row.key).collect();
            for key in keys {
                for channel in 0..4 {
                    // BACK invalidates the old roster. Reload from the same DB,
                    // then SELECT independently qualifies the row a second time.
                    epoch += 1;
                    let fresh = connection
                        .characters(source, epoch)
                        .expect("real roster reload failed");
                    assert!(
                        fresh.characters == roster.characters,
                        "complete same-source roster changed during readonly navigation"
                    );
                    let selected = connection
                        .select_character(application::SelectionContext { key, epoch })
                        .expect("real character selection failed");
                    assert_eq!(selected.status, application::NavigationStatus::Ok);
                    assert!(
                        selected.snapshot.selected.as_ref()
                            == roster.characters.iter().find(|row| row.key == key),
                        "fresh selection must preserve the correct source character fields"
                    );
                    assert!(selected
                        .snapshot
                        .channels
                        .iter()
                        .all(|row| row.population == 0));
                    let data_reply = connection
                        .character_data()
                        .expect("real source character data exchange failed");
                    assert_eq!(data_reply.context, selected.context);
                    let data = data_reply
                        .result
                        .expect("real source character data is unavailable");
                    let expected_data = expected_data
                        .iter()
                        .find(|data| data.source_id == source && data.object_id == key.object_id)
                        .expect("missing independently probed character");
                    assert!(
                        matches_data(expected_data, &data),
                        "persisted values and every source item must equal the separate read-only SQL probe"
                    );
                    assert_eq!(
                        connection.navigation(),
                        Some(&selected),
                        "fresh data is not a navigation mutation"
                    );
                    let lobby = connection
                        .enter_channel(channel)
                        .expect("real source channel entry failed");
                    assert_eq!(lobby.status, application::NavigationStatus::Ok);
                    assert_eq!(lobby.snapshot.phase, application::NavigationPhase::Lobby);
                    assert_eq!(lobby.snapshot.channel_id, Some(channel));
                    assert_eq!(lobby.snapshot.channels[channel as usize].population, 1);
                    connection
                        .ping()
                        .expect("real source lobby heartbeat failed");
                    let left = connection
                        .leave_lobby()
                        .expect("real source lobby leave failed");
                    assert_eq!(left.action, NavigationAction::LeaveLobby);
                    assert_eq!(left.context, lobby.context);
                    assert_eq!(left.snapshot.phase, NavigationPhase::Channels);
                    assert_eq!(left.snapshot.channel_id, None);
                    assert_eq!(left.snapshot.selected, lobby.snapshot.selected);
                    assert!(left.snapshot.channels.iter().all(|row| row.population == 0));
                    let rejoined = connection
                        .enter_channel(channel)
                        .expect("real source lobby reentry failed");
                    assert_eq!(rejoined.status, NavigationStatus::Ok);
                    assert_eq!(rejoined.context, lobby.context);
                    assert_eq!(rejoined.snapshot.selected, lobby.snapshot.selected);
                    assert_eq!(rejoined.snapshot.channel_id, Some(channel));
                    assert_eq!(rejoined.snapshot.channels[channel as usize].population, 1);
                    connection
                        .leave_lobby()
                        .expect("real source second lobby leave failed");
                    connection
                        .return_characters()
                        .expect("real source return failed");
                    assert!(connection.navigation().is_none());
                }
            }
            epoch += 1;
        }
        connection
            .close()
            .expect("real database test connection close failed");
        assert!(connection.principal().is_none());
        peer.expect_successful_exit(); // True listener/backend/presence retirement, not UI-only cleanup.
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_production_listener_authenticates_switches_sources_and_retires_workers() {
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernServerTestPeer", "auth", "15000");
        let mut connection = application_connection(&peer, Cancellation::new());
        connection.ping().unwrap();
        let principal = connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        assert_eq!(principal.account, "sun-owner");
        assert_eq!(
            principal
                .sources
                .iter()
                .map(|source| source.id)
                .collect::<Vec<_>>(),
            vec![7, 42]
        );
        let sun = connection.characters(7, 1).unwrap();
        let moon = connection.characters(42, 2).unwrap();
        let again = connection.characters(7, 3).unwrap();
        assert_eq!(sun.characters[0].key.object_id, 3333);
        assert_eq!(moon.characters[0].key.object_id, 3333);
        assert_ne!(sun.characters[0].key, moon.characters[0].key);
        assert_eq!(sun.characters[0].name, "Sun Knight");
        assert_eq!(moon.characters[0].name, "Moon Mage");
        assert_eq!(moon.characters[0].level, 120_000);
        assert_eq!(again.characters, sun.characters);
        connection.ping().unwrap();
        connection.close().unwrap();
        assert!(connection.principal().is_none());
        // The child exits successfully only after the production listener,
        // accepted sessions and actual backend workers have all terminated.
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_ping_auth_and_source_switch_keep_distinct_same_objid_characters() {
        let mut peer = JavaPeer::start_application("auth");
        let mut connection = application_connection(&peer, Cancellation::new());
        assert_eq!(connection.application_state(), ApplicationState::Negotiated);
        connection.ping().unwrap();
        assert!(connection.principal().is_none());
        let principal = connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        assert_eq!(principal.account, "sun-owner");
        assert_eq!(
            principal.sources,
            vec![
                application::Source {
                    id: 7,
                    name: "太陽神".into()
                },
                application::Source {
                    id: 42,
                    name: "月亮".into()
                },
            ]
        );
        assert_eq!(
            connection.application_state(),
            ApplicationState::Authenticated
        );
        let sun = connection.characters(7, 1).unwrap();
        let moon = connection.characters(42, 2).unwrap();
        let sun_again = connection.characters(7, u64::MAX).unwrap();
        assert_eq!((sun.epoch, moon.epoch, sun_again.epoch), (1, 2, u64::MAX));
        assert_eq!(
            (
                sun.characters.len(),
                moon.characters.len(),
                sun_again.characters.len()
            ),
            (1, 1, 1)
        );
        let sun_row = &sun.characters[0];
        let moon_row = &moon.characters[0];
        assert_eq!(
            sun_row.key,
            application::CharacterKey {
                source_id: 7,
                object_id: 3333
            }
        );
        assert_eq!(
            moon_row.key,
            application::CharacterKey {
                source_id: 42,
                object_id: 3333
            }
        );
        assert_ne!(sun_row.key, moon_row.key);
        assert_eq!(sun_row.name, "Sun Knight");
        assert_eq!((sun_row.class, sun_row.sex, sun_row.level), (1, 0, 80));
        assert_eq!(moon_row.name, "Moon Mage");
        assert_eq!(
            (moon_row.class, moon_row.sex, moon_row.level),
            (3, 1, 120_000)
        );
        assert_eq!(sun_again.characters, sun.characters);
        connection.ping().unwrap();
        assert_eq!(connection.principal(), Some(&principal));
        connection.close().unwrap();
        assert_eq!(connection.application_state(), ApplicationState::Closed);
        assert!(connection.principal().is_none());
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_rejected_auth_clears_principal_and_cannot_retry() {
        for (mode, password, expected) in [
            (
                "auth",
                "wrong-synthetic-password",
                AuthenticationFailure::InvalidCredentials,
            ),
            (
                "auth-backend",
                "fixture-password",
                AuthenticationFailure::BackendUnavailable,
            ),
        ] {
            let mut peer = JavaPeer::start_application(mode);
            let mut connection = application_connection(&peer, Cancellation::new());
            let result = connection.authenticate(fixture_credentials(password));
            assert!(
                matches!(result, Err(TransportError::AuthenticationRejected(reason)) if reason == expected),
                "{result:?}"
            );
            assert_eq!(connection.application_state(), ApplicationState::Closed);
            assert!(connection.principal().is_none());
            assert!(connection.ping().is_err());
            assert!(connection
                .authenticate(fixture_credentials("fixture-password"))
                .is_err());
            peer.expect_successful_exit();
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_character_denial_and_backend_failure_preserve_authenticated_session() {
        for mode in ["auth", "characters-backend"] {
            let mut peer = JavaPeer::start_application(mode);
            let mut connection = application_connection(&peer, Cancellation::new());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            let previous_id = connection.last_request_id;
            assert!(matches!(
                connection.characters(99, 1),
                Err(TransportError::CharactersRejected(CharacterFailure::Denied))
            ));
            assert_eq!(
                connection.last_request_id, previous_id,
                "local unauthorized selection sends nothing"
            );
            // Test-only access deliberately bypasses the client's local source
            // check to verify the REAL server gate rejects an unauthorized source.
            let request_id = connection.next_request_id().unwrap();
            let frame = application::encode_characters(request_id, 99, 9).unwrap();
            assert!(matches!(
                connection
                    .exchange(&frame, request_id, Stage::Characters)
                    .unwrap(),
                Response::Characters {
                    source_id: 99,
                    epoch: 9,
                    result: Err(CharacterFailure::Denied)
                }
            ));
            if mode == "characters-backend" {
                assert!(matches!(
                    connection.characters(7, 10),
                    Err(TransportError::CharactersRejected(
                        CharacterFailure::BackendUnavailable
                    ))
                ));
            } else {
                assert_eq!(
                    connection.characters(7, 10).unwrap().characters[0]
                        .key
                        .source_id,
                    7
                );
            }
            assert_eq!(
                connection.application_state(),
                ApplicationState::Authenticated
            );
            connection.ping().unwrap();
            connection.close().unwrap();
            peer.expect_successful_exit();
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_cancel_and_whole_roster_deadline_clear_all_authenticated_state() {
        for cancelled in [false, true] {
            let peer = JavaPeer::start_application("characters-slow");
            let token = Cancellation::new();
            let mut connection = application_connection(&peer, token.clone());
            connection
                .authenticate(fixture_credentials("fixture-password"))
                .unwrap();
            connection
                .set_exchange_timeout(if cancelled {
                    Duration::from_secs(2)
                } else {
                    Duration::from_millis(150)
                })
                .unwrap();
            let interrupter = cancelled.then(|| {
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(100));
                    token.cancel();
                })
            });
            let started = Instant::now();
            let result = connection.characters(7, 1);
            if let Some(interrupter) = interrupter {
                interrupter.join().unwrap();
            }
            if cancelled {
                assert!(
                    matches!(result, Err(TransportError::Cancelled)),
                    "{result:?}"
                );
            } else {
                assert!(
                    matches!(
                        result,
                        Err(TransportError::DeadlineExceeded(Stage::Characters))
                    ),
                    "{result:?}"
                );
            }
            assert!(started.elapsed() < Duration::from_secs(1));
            assert!(connection.principal().is_none());
            assert_eq!(connection.application_state(), ApplicationState::Closed);
        }
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_request_id_does_not_wrap_and_authentication_is_one_shot() {
        let mut peer = JavaPeer::start_application("auth");
        let mut connection = connect_and_negotiate(
            peer.endpoint,
            "localhost",
            Some(&private_root()),
            Hello::current(u32::MAX),
            integration_limits(),
            Cancellation::new(),
        )
        .unwrap();
        assert!(matches!(
            connection.ping(),
            Err(TransportError::InvalidState(_))
        ));
        assert_eq!(connection.application_state(), ApplicationState::Closed);
        peer.expect_successful_exit();

        let mut peer = JavaPeer::start_application("auth");
        let mut connection = application_connection(&peer, Cancellation::new());
        connection
            .authenticate(fixture_credentials("fixture-password"))
            .unwrap();
        assert!(matches!(
            connection.authenticate(fixture_credentials("fixture-password")),
            Err(TransportError::InvalidState(_))
        ));
        assert!(connection.principal().is_none());
        assert_eq!(connection.application_state(), ApplicationState::Closed);
        peer.expect_successful_exit();
    }

    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_wrong_request_source_or_epoch_closes_instead_of_publishing_old_identity() {
        for mode in [
            "auth-wrong-id",
            "characters-wrong-epoch",
            "characters-wrong-source",
        ] {
            let peer = JavaPeer::start_application(mode);
            let mut connection = application_connection(&peer, Cancellation::new());
            let result = if mode == "auth-wrong-id" {
                connection
                    .authenticate(fixture_credentials("fixture-password"))
                    .map(|_| ())
            } else {
                connection
                    .authenticate(fixture_credentials("fixture-password"))
                    .unwrap();
                connection.characters(7, 5).map(|_| ())
            };
            assert!(
                matches!(result, Err(TransportError::Application(ref error))
                if error.kind == application::ErrorKind::RequestIdMismatch || error.kind == application::ErrorKind::IdentityMismatch),
                "{mode}: {result:?}"
            );
            assert_eq!(connection.application_state(), ApplicationState::Closed);
            assert!(connection.principal().is_none());
            assert!(connection.ack().is_none());
            assert!(connection.ping().is_err());
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_controller_cancel_after_server_commit_before_ack_releases_owner() {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;
        fn wait_phase(controller: &mut impl LoginController, expected: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == expected {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "phase {:?} != {expected:?}: {}",
                    view.phase,
                    view.status
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        for (mode, action) in [
            ("cancel-select", 9),
            ("cancel-enter", 11),
            ("cancel-leave", 13),
            ("cancel-data", 14),
        ] {
            let mut peer = JavaPeer::start_class(
                "local.lito381.server.ModernNavigationTestPeer",
                mode,
                "15000",
            );
            let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
                "schema":1, "profile_id":"cancel-committed-navigation", "display_name":"Owned navigation cancellation test",
                "shared_resource_root":"D:\\synthetic-test", "endpoint":peer.endpoint.to_string(), "tls_server_name":"localhost",
                "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
                "connect_timeout_ms":500, "exchange_timeout_ms":10000
            })).unwrap()).unwrap();
            let mut controller = SessionController::new(profile);
            controller.command(LoginCommand::Connect).unwrap();
            wait_phase(&mut controller, LoginPhase::LoginReady);
            controller
                .command(LoginCommand::Login {
                    account: "fixture-user".into(),
                    password: "fixture-password".into(),
                })
                .unwrap();
            wait_phase(&mut controller, LoginPhase::ChoosingSource);
            controller.command(LoginCommand::SelectSource(7)).unwrap();
            wait_phase(&mut controller, LoginPhase::CharactersReady);
            let roster = controller.snapshot();
            let context = application::SelectionContext {
                key: application::CharacterKey {
                    source_id: 7,
                    object_id: roster.characters[0].object_id,
                },
                epoch: roster.roster_epoch,
            };
            let mut mutation_started = Instant::now();
            controller
                .command(LoginCommand::SelectCharacter(context))
                .unwrap();
            if action == 11 || action == 13 {
                wait_phase(&mut controller, LoginPhase::ChoosingChannel);
                mutation_started = Instant::now();
                controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id: 0,
                    })
                    .unwrap();
            }
            if action == 13 {
                wait_phase(&mut controller, LoginPhase::InLobby);
                mutation_started = Instant::now();
                controller
                    .command(LoginCommand::LeaveLobby(context))
                    .unwrap();
            }
            // Unlike a sleep, this test-only stdout marker proves the exact
            // server authority commit happened before cancellation; ACK is withheld.
            assert_eq!(
                peer.read_bounded_line("COMMITTED").trim_end(),
                format!("COMMITTED {action}")
            );
            assert_eq!(
                controller.snapshot().phase,
                if action == 9 || action == 14 {
                    LoginPhase::SelectingCharacter
                } else if action == 13 {
                    LoginPhase::LeavingLobby
                } else {
                    LoginPhase::JoiningLobby
                }
            );
            controller.command(LoginCommand::Shutdown).unwrap();
            wait_phase(&mut controller, LoginPhase::Disconnected);
            assert!(
                mutation_started.elapsed() < Duration::from_secs(2),
                "cancellation must finish well before the 10s application deadline"
            );
            assert!(controller.is_quiescent());
            assert!(controller.snapshot().navigation.is_none());
            assert!(controller.snapshot().characters.is_empty());
            // Local quiescence is not used as server cleanup evidence. The peer
            // independently requires EOF, actual backend retirement and zero owners.
            peer.expect_successful_exit();
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_controller_navigation_returns_reload_roster_and_preserves_background_lobby()
    {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;
        fn wait_phase(controller: &mut impl LoginController, expected: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == expected {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "phase {:?}, expected {expected:?}: {}",
                    view.phase,
                    view.status
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        let mut peer =
            JavaPeer::start_class("local.lito381.server.ModernServerTestPeer", "data", "30000");
        let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1, "profile_id":"navigation-controller", "display_name":"Owned navigation controller test",
            "shared_resource_root":"D:\\synthetic-test", "endpoint":peer.endpoint.to_string(), "tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500, "exchange_timeout_ms":2000
        })).unwrap()).unwrap();
        let mut controller = SessionController::new(profile);
        controller.command(LoginCommand::Connect).unwrap();
        wait_phase(&mut controller, LoginPhase::LoginReady);
        controller
            .command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            })
            .unwrap();
        wait_phase(&mut controller, LoginPhase::ChoosingSource);
        for source in [7, 42, 7] {
            controller
                .command(LoginCommand::SelectSource(source))
                .unwrap();
            wait_phase(&mut controller, LoginPhase::CharactersReady);
            for channel_id in 0..4 {
                let roster = controller.snapshot();
                let context = application::SelectionContext {
                    key: application::CharacterKey {
                        source_id: source,
                        object_id: roster.characters[0].object_id,
                    },
                    epoch: roster.roster_epoch,
                };
                assert_ne!(context.epoch, 0);
                controller
                    .command(LoginCommand::SelectCharacter(context))
                    .unwrap();
                assert!(controller
                    .command(LoginCommand::SelectCharacter(context))
                    .is_err());
                wait_phase(&mut controller, LoginPhase::ChoosingChannel);
                let character_data::CharacterDataState::Ready(saved_data) =
                    controller.snapshot().character_data
                else {
                    panic!("successful selection must publish its actual source data");
                };
                assert_synthetic_character_data(&saved_data, source, false);
                assert_eq!(saved_data.context, context);
                assert!(controller
                    .command(LoginCommand::SelectSource(if source == 7 { 42 } else { 7 }))
                    .is_err());
                controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id,
                    })
                    .unwrap();
                assert!(controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id
                    })
                    .is_err());
                wait_phase(&mut controller, LoginPhase::InLobby);
                let character_data::CharacterDataState::Ready(joined_data) =
                    controller.snapshot().character_data
                else {
                    panic!("joined data");
                };
                assert!(Arc::ptr_eq(&saved_data, &joined_data));
                let view = controller.snapshot();
                let navigation = view.navigation.unwrap();
                assert_eq!(navigation.context, context);
                assert_eq!(navigation.snapshot.channel_id, Some(channel_id));
                assert_eq!(
                    navigation.snapshot.channels[channel_id as usize].population,
                    1
                );
                assert_eq!(
                    navigation.snapshot.selected.as_ref().unwrap().key.source_id,
                    source
                );
                if source == 42 && channel_id == 3 {
                    // Production listener's read deadline is 5s. No UI polling,
                    // commands or paint for over two whole deadline windows.
                    thread::sleep(Duration::from_millis(10_600));
                }
                controller
                    .command(LoginCommand::LeaveLobby(context))
                    .unwrap();
                assert!(controller
                    .command(LoginCommand::LeaveLobby(context))
                    .is_err());
                wait_phase(&mut controller, LoginPhase::ChoosingChannel);
                let left = controller.snapshot();
                let character_data::CharacterDataState::Ready(left_data) = &left.character_data
                else {
                    panic!("left data");
                };
                assert!(Arc::ptr_eq(&saved_data, left_data));
                assert_eq!(left.roster_epoch, 0);
                assert!(
                    left.characters.is_empty(),
                    "leave cannot expose a stale roster"
                );
                let left = left.navigation.unwrap();
                assert_eq!(left.context, context);
                assert_eq!(left.snapshot.selected, navigation.snapshot.selected);
                assert_eq!(left.snapshot.channel_id, None);
                assert!(left.snapshot.channels.iter().all(|row| row.population == 0));
                controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id,
                    })
                    .unwrap();
                wait_phase(&mut controller, LoginPhase::InLobby);
                let rejoined = controller.snapshot().navigation.unwrap();
                assert_eq!(rejoined.context, context);
                assert_eq!(rejoined.snapshot.selected, navigation.snapshot.selected);
                assert_eq!(
                    rejoined.snapshot.channels[channel_id as usize].population,
                    1
                );
                controller
                    .command(LoginCommand::LeaveLobby(context))
                    .unwrap();
                wait_phase(&mut controller, LoginPhase::ChoosingChannel);
                controller
                    .command(LoginCommand::ReturnCharacters(context))
                    .unwrap();
                wait_phase(&mut controller, LoginPhase::CharactersReady);
                let returned = controller.snapshot();
                assert!(returned.navigation.is_none());
                assert_eq!(
                    returned.character_data,
                    character_data::CharacterDataState::NotRequested
                );
                assert!(
                    returned.roster_epoch > context.epoch,
                    "same source is actually reloaded after server BACK ACK"
                );
                assert!(
                    controller
                        .command(LoginCommand::SelectCharacter(context))
                        .is_err(),
                    "old UI event must not reselect stale roster"
                );
            }
        }
        controller.command(LoginCommand::Shutdown).unwrap();
        wait_phase(&mut controller, LoginPhase::Disconnected);
        assert!(controller.is_quiescent());
        assert!(controller.snapshot().navigation.is_none());
        peer.expect_successful_exit();
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_equipment_back_busy_controller_preserves_data_arc_and_retries_without_automatic_reload()
    {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;
        fn wait_phase(controller: &mut impl LoginController, expected: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == expected {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "phase {:?}, expected {expected:?}: {}",
                    view.phase,
                    view.status
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        let mut peer = JavaPeer::start_class(
            "local.lito381.server.ModernEquipmentBusyTestPeer",
            "back-busy",
            "30000",
        );
        let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1, "profile_id":"equipment-busy-controller", "display_name":"Owned equipment Busy controller test",
            "shared_resource_root":"D:\\synthetic-test", "endpoint":peer.endpoint.to_string(), "tls_server_name":"localhost",
            "trust":{"kind":"private_ca","certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(private_root())},
            "connect_timeout_ms":500, "exchange_timeout_ms":2000
        })).unwrap()).unwrap();
        let mut controller = SessionController::new(profile);
        controller.command(LoginCommand::Connect).unwrap();
        wait_phase(&mut controller, LoginPhase::LoginReady);
        controller
            .command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            })
            .unwrap();
        wait_phase(&mut controller, LoginPhase::ChoosingSource);
        for (source, lobby) in [(7, false), (42, true)] {
            controller
                .command(LoginCommand::SelectSource(source))
                .unwrap();
            wait_phase(&mut controller, LoginPhase::CharactersReady);
            let roster = controller.snapshot();
            let context = SelectionContext {
                key: application::CharacterKey {
                    source_id: source,
                    object_id: roster.characters[0].object_id,
                },
                epoch: roster.roster_epoch,
            };
            controller
                .command(LoginCommand::SelectCharacter(context))
                .unwrap();
            wait_phase(&mut controller, LoginPhase::ChoosingChannel);
            if lobby {
                controller
                    .command(LoginCommand::EnterChannel {
                        context,
                        channel_id: 2,
                    })
                    .unwrap();
                wait_phase(&mut controller, LoginPhase::InLobby);
            }
            let original = controller.snapshot();
            let character_data::CharacterDataState::Ready(original_data) = &original.character_data
            else {
                panic!("fixture must supply actual selected data");
            };
            controller
                .command(LoginCommand::ReturnCharacters(context))
                .unwrap();
            assert_eq!(controller.snapshot().phase, LoginPhase::ReturningCharacters);
            wait_phase(&mut controller, original.phase);
            // No paint/poll required for heartbeat. A wrong automatic CHAR is
            // rejected by this peer rather than masked by a successful reload.
            thread::sleep(Duration::from_millis(2200));
            controller.poll();
            let busy = controller.snapshot();
            assert_eq!(busy.phase, original.phase);
            assert_eq!(busy.sources, original.sources);
            assert_eq!(busy.selected_source, original.selected_source);
            assert_eq!(busy.roster_epoch, original.roster_epoch);
            assert_eq!(busy.characters, original.characters);
            assert!(busy.status.contains("裝備保存中"));
            assert!(busy.status.contains("稍後再返回"));
            let reply = busy.navigation.as_ref().unwrap();
            assert!(confirms_busy_return(original.navigation.as_ref(), reply));
            let character_data::CharacterDataState::Ready(busy_data) = &busy.character_data else {
                panic!("Busy cannot clear loaded data");
            };
            assert!(Arc::ptr_eq(original_data, busy_data));
            assert!(controller
                .command(LoginCommand::SelectSource(if source == 7 { 42 } else { 7 }))
                .is_err());
            controller
                .command(LoginCommand::ReturnCharacters(context))
                .unwrap();
            wait_phase(&mut controller, LoginPhase::CharactersReady);
            let returned = controller.snapshot();
            assert_eq!(returned.selected_source, Some(source));
            assert!(returned.roster_epoch > context.epoch);
            assert!(returned.navigation.is_none());
            assert_eq!(
                returned.character_data,
                character_data::CharacterDataState::NotRequested
            );
            assert!(controller
                .command(LoginCommand::SelectCharacter(context))
                .is_err());
        }
        controller.command(LoginCommand::Shutdown).unwrap();
        wait_phase(&mut controller, LoginPhase::Disconnected);
        assert!(controller.is_quiescent());
        peer.expect_successful_exit();
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires prepared Java 17 application TLS peer and explicit test environment"]
    fn java_application_controller_login_switch_idle_heartbeat_disconnect_and_shutdown() {
        use super::super::login_window::{LoginCommand, LoginController, LoginPhase};
        use super::super::profile::ModernProfile;
        use super::super::session::SessionController;
        use base64::Engine;

        fn wait_phase(controller: &mut impl LoginController, expected: LoginPhase) {
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                controller.poll();
                let view = controller.snapshot();
                assert_ne!(view.phase, LoginPhase::Failed, "{}", view.status);
                if view.phase == expected {
                    return;
                }
                assert!(
                    Instant::now() < deadline,
                    "controller phase deadline: {:?} != {expected:?}",
                    view.phase
                );
                thread::sleep(Duration::from_millis(10));
            }
        }
        // This peer enforces a 3000ms application read deadline. Surviving more
        // than two such windows requires the worker's own PINGs, not UI polling.
        let mut peer = JavaPeer::start_application("heartbeat");
        let profile = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema": 1,
            "profile_id": "application-interop",
            "display_name": "Owned loopback controller test",
            "shared_resource_root": "D:\\synthetic-test",
            "endpoint": peer.endpoint.to_string(),
            "tls_server_name": "localhost",
            "trust": { "kind": "private_ca",
                "certificate_der_base64": base64::engine::general_purpose::STANDARD.encode(private_root()) },
            "connect_timeout_ms": 500,
            "exchange_timeout_ms": 2000
        })).unwrap()).unwrap();
        let mut controller = SessionController::new(profile);
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
        controller.command(LoginCommand::Connect).unwrap();
        wait_phase(&mut controller, LoginPhase::LoginReady);
        assert!(controller.snapshot().sources.is_empty());
        assert!(controller.snapshot().characters.is_empty());
        controller
            .command(LoginCommand::Login {
                account: "fixture-user".into(),
                password: "fixture-password".into(),
            })
            .unwrap();
        wait_phase(&mut controller, LoginPhase::ChoosingSource);
        assert_eq!(
            controller
                .snapshot()
                .sources
                .iter()
                .map(|source| source.id)
                .collect::<Vec<_>>(),
            [7, 42]
        );
        for (source_id, name) in [(7, "Sun Knight"), (42, "Moon Mage"), (7, "Sun Knight")] {
            controller
                .command(LoginCommand::SelectSource(source_id))
                .unwrap();
            let loading = controller.snapshot();
            assert_eq!(loading.phase, LoginPhase::LoadingCharacters);
            assert_eq!(loading.selected_source, Some(source_id));
            assert!(
                loading.characters.is_empty(),
                "previous source rows are cleared immediately"
            );
            wait_phase(&mut controller, LoginPhase::CharactersReady);
            let view = controller.snapshot();
            assert_eq!(view.selected_source, Some(source_id));
            assert_eq!(view.characters.len(), 1);
            assert_eq!(
                (view.characters[0].source_id, view.characters[0].object_id),
                (source_id, 3333)
            );
            assert!(view.characters[0].label.starts_with(name));
        }
        // Deliberately do NOT poll, issue commands, paint or pump any UI. The
        // server would close after 3000ms without real application traffic.
        // After 6400ms, a fresh successful roster proves heartbeat progress on
        // this same authenticated TLS connection independently of the UI.
        thread::sleep(Duration::from_millis(6400));
        controller.command(LoginCommand::SelectSource(42)).unwrap();
        wait_phase(&mut controller, LoginPhase::CharactersReady);
        assert_eq!(controller.snapshot().characters[0].source_id, 42);
        controller.command(LoginCommand::Disconnect).unwrap();
        wait_phase(&mut controller, LoginPhase::Disconnected);
        assert!(controller.is_quiescent());
        let disconnected = controller.snapshot();
        assert!(disconnected.sources.is_empty() && disconnected.characters.is_empty());
        assert!(disconnected.selected_source.is_none());
        controller.command(LoginCommand::Shutdown).unwrap();
        controller.poll();
        assert!(controller.is_quiescent());
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
        assert!(controller.command(LoginCommand::Connect).is_err());
        peer.expect_successful_exit();
    }
}
