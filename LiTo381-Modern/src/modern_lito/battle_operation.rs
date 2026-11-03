//! v38 canonical action receipt. Exact echoed intent, owner-scoped schema-2
//! data and service settlement are validated together. Parsing does not install
//! a renderer, acknowledge a different pending command or permit equipment SET.
use super::{
    application_protocol as app, battle_action as action,
    battle_payload::{self as payload, data as d},
    protocol, world_protocol as world,
};
use std::io::Read;
pub const VERSION: u16 = 38;
pub const REQUEST_KIND: u16 = 64;
pub const RESULT_KIND: u16 = 65;
pub const TRAILER_BYTES: usize = 32;
pub const MIN_RESULT_BYTES: usize = 188 + TRAILER_BYTES + 16;
pub const MAX_RESULT_BYTES: usize =
    action::MAX_REQUEST_BYTES + TRAILER_BYTES + payload::MAX_WIRE_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Disposition {
    Unavailable,
    Applied,
    Denied,
    Rejected,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    StalePosition,
    TooFast,
    Blocked,
    Occupied,
    Dead,
    TargetDead,
    SameTeam,
    OutOfRange,
    Unsupported,
    Numeric,
    TooHeavy,
    StaleEffect,
    NoAmmo,
    StaleRegen,
    Unlearned,
    NoMp,
    NoDestination,
    Empty,
    ClassRestricted,
    Attacker,
    NotAtPortal,
    WrongLane,
    StaleState,
    Lawful,
    StaleLife,
    Wait,
    NoTarget,
    Budget,
}
impl Reason {
    fn parse(v: u8) -> Result<Self, app::ApplicationError> {
        use Reason::*;
        [
            None,
            NotActive,
            StalePosition,
            TooFast,
            Blocked,
            Occupied,
            Dead,
            TargetDead,
            SameTeam,
            OutOfRange,
            Unsupported,
            Numeric,
            TooHeavy,
            StaleEffect,
            NoAmmo,
            StaleRegen,
            Unlearned,
            NoMp,
            NoDestination,
            Empty,
            ClassRestricted,
            Attacker,
            NotAtPortal,
            WrongLane,
            StaleState,
            Lawful,
            StaleLife,
            Wait,
            NoTarget,
            Budget,
        ]
        .get(v as usize)
        .copied()
        .ok_or_else(invalid)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lifecycle {
    pub revision: i64,
    pub phase: world::Phase,
    pub abort: world::AbortReason,
    pub settlement: Option<world::SettlementState>,
    pub race_settlement: Option<world::SettlementState>,
    pub race_terminal_revision: i64,
}
impl Lifecycle {
    pub(crate) fn read(b: &[u8]) -> Result<Self, app::ApplicationError> {
        if b.len() != 24 || b[20..24] != [0; 4] {
            return Err(invalid());
        }
        let phase = match b[8] {
            0 => world::Phase::Preparing,
            1 => world::Phase::Loading,
            2 => world::Phase::Countdown,
            3 => world::Phase::Active,
            4 => world::Phase::Aborted,
            5 => world::Phase::Finished,
            _ => return Err(invalid()),
        };
        let abort = match b[9] {
            0 => world::AbortReason::None,
            1 => world::AbortReason::Disconnected,
            2 => world::AbortReason::LoadTimeout,
            3 => world::AbortReason::ActiveTimeout,
            4 => world::AbortReason::ServerAbort,
            5 => world::AbortReason::ClockRegressed,
            6 => world::AbortReason::PreparationCancelled,
            7 => world::AbortReason::PreparationFailed,
            _ => return Err(invalid()),
        };
        Ok(Self {
            revision: i64_at(b, 0),
            phase,
            abort,
            settlement: settlement(b[10])?,
            race_settlement: settlement(b[11])?,
            race_terminal_revision: i64_at(b, 12),
        })
    }
    pub(crate) fn validate(
        &self,
        data: &payload::UnvalidatedData,
    ) -> Result<(), app::ApplicationError> {
        let aborted = self.phase == world::Phase::Aborted;
        let overlay = aborted && data.world.phase != d::Phase::Finished && data.result.is_none();
        if self.revision <= data.world.revision
            || self.phase == world::Phase::Preparing
            || aborted != (self.abort != world::AbortReason::None)
            || (!overlay
                && (self.phase as u8 != data.world.phase as u8 + 1
                    || self.abort as u8 != data.world.abort as u8))
            || self.settlement.is_some() != data.result.is_some()
            || self.settlement.is_some() && self.phase != world::Phase::Finished
            || self.race_terminal_revision < 0
            || self.race_terminal_revision > self.revision
        {
            return Err(invalid());
        }
        match &data.race {
            None => {
                if self.race_settlement.is_some() || self.race_terminal_revision != 0 {
                    return Err(invalid());
                }
            }
            Some(r) => {
                let complete = r.phase == d::RacePhase::Complete;
                let terminal = if r.terminal_revision == 0 {
                    0
                } else {
                    r.terminal_revision.checked_add(1).ok_or_else(invalid)?
                };
                if complete != self.race_settlement.is_some()
                    || complete && self.phase != world::Phase::Finished
                    || (!overlay && self.race_terminal_revision != terminal)
                    || overlay && self.race_terminal_revision == 0
                {
                    return Err(invalid());
                }
                if complete {
                    let expected = terminal
                        .checked_add(i64::from(
                            self.race_settlement != Some(world::SettlementState::Pending),
                        ))
                        .ok_or_else(invalid)?;
                    if self.revision != expected {
                        return Err(invalid());
                    }
                }
            }
        }
        Ok(())
    }
}
fn settlement(b: u8) -> Result<Option<world::SettlementState>, app::ApplicationError> {
    Ok(match b {
        0 => None,
        1 => Some(world::SettlementState::Pending),
        2 => Some(world::SettlementState::Saved),
        3 => Some(world::SettlementState::Failed),
        _ => return Err(invalid()),
    })
}
#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub guard: action::Guard,
    pub rejection: action::Rejection,
    pub disposition: Disposition,
    pub reason: Reason,
    pub lifecycle: Lifecycle,
    pub data: payload::UnvalidatedData,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CanonicalBattleOperationReply[redacted]")
    }
}
pub fn negotiated(ack: &protocol::Ack) -> bool {
    ack.version >= VERSION && action::negotiated(ack) && super::battle_read::negotiated(ack)
}
pub(crate) fn reply_negotiated(reply: &Reply, ack: &protocol::Ack) -> bool {
    reply_negotiated_selected(reply, protocol::extensions::Selected::from(*ack))
}
pub(crate) fn reply_negotiated_selected(
    reply: &Reply,
    peer: protocol::extensions::Selected,
) -> bool {
    let ack = &peer.base();
    let d = &reply.data;
    let needed = world::MatchDescriptor::feature_capabilities(d.world.descriptor.features as u32);
    negotiated(ack)
        && super::battle_read::command_data_negotiated_selected(d, peer)
        && d.presentation.is_some()
        && ack.capabilities & needed == needed
        && (d.manifest.sectors.0.len() <= 16 || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
}
pub fn encode(
    guard: action::Guard,
    inner: &[u8],
    version: u16,
) -> Result<Vec<u8>, app::ApplicationError> {
    if !(VERSION..=protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    let mut frame = action::encode(guard, inner, version)?;
    frame[12..14].copy_from_slice(&REQUEST_KIND.to_be_bytes());
    Ok(frame)
}
/// The exact request is known before reading; reject bad kind/id/length before
/// allocation. Coalesced following responses remain unread in the stream.
pub fn read_reply(
    reader: &mut impl Read,
    guard: action::Guard,
    expected: &[u8],
    version: u16,
) -> Result<Reply, app::ApplicationError> {
    read_reply_format(reader, guard, expected, payload::Format::Legacy(version))
}
pub fn read_reply_selected(
    reader: &mut impl Read,
    guard: action::Guard,
    expected: &[u8],
    peer: crate::modern_lito::protocol::extensions::Selected,
) -> Result<Reply, app::ApplicationError> {
    read_reply_format(reader, guard, expected, payload::Format::Selected(peer))
}
fn read_reply_format(
    reader: &mut impl Read,
    guard: action::Guard,
    expected: &[u8],
    format: payload::Format,
) -> Result<Reply, app::ApplicationError> {
    let version = format.version();
    if !(VERSION..=protocol::MAX_VERSION).contains(&version)
        || !(188..=action::MAX_REQUEST_BYTES).contains(&expected.len())
        || encode(guard, &expected[116..], version)? != expected
    {
        return Err(invalid());
    }
    let mut header = [0; 16];
    reader.read_exact(&mut header).map_err(|_| invalid())?;
    let length = u32_at(&header, 4) as usize;
    if &header[..4] != b"LTM1"
        || u16_at(&header, 12) != RESULT_KIND
        || header[14..16] != [0; 2]
        || header[8..12] != expected[8..12]
        || !(MIN_RESULT_BYTES..=format.limit(MAX_RESULT_BYTES)).contains(&length)
    {
        return Err(invalid());
    }
    let mut body = vec![0; length - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    let n = expected.len() - 16;
    if body.len() < n + TRAILER_BYTES + 16 || body[..n] != expected[16..] {
        return Err(invalid());
    }
    let meta = &body[n..n + TRAILER_BYTES];
    if meta[3] != 0 || u32_at(meta, 28) as usize != body.len() - n - TRAILER_BYTES {
        return Err(invalid());
    }
    let rejection = match meta[0] {
        0 => action::Rejection::None,
        1 => action::Rejection::NotActive,
        2 => action::Rejection::StaleLife,
        3 => action::Rejection::Dead,
        4 => action::Rejection::StaleTarget,
        _ => return Err(invalid()),
    };
    let disposition = match meta[1] {
        0 => Disposition::Unavailable,
        1 => Disposition::Applied,
        2 => Disposition::Denied,
        3 => Disposition::Rejected,
        _ => return Err(invalid()),
    };
    let reason = Reason::parse(meta[2])?;
    if (rejection != action::Rejection::None) != (disposition == Disposition::Rejected)
        || match disposition {
            Disposition::Applied | Disposition::Rejected => reason != Reason::None,
            Disposition::Unavailable => reason != Reason::NotActive,
            Disposition::Denied => reason == Reason::None,
        }
    {
        return Err(invalid());
    }
    let lifecycle = Lifecycle::read(&meta[4..28])?;
    let data = format.decode_command(&body[n + TRAILER_BYTES..])?;
    let s = guard.stamp;
    let player = d::Player {
        source: s.context.key.source_id as i32,
        object: s.context.key.object_id as i64,
        epoch: s.context.epoch as i64,
        channel: i32::from(s.channel),
    };
    let world = d::WorldId {
        room: s.world[0],
        match_id: s.world[1],
        generation: s.world[2],
    };
    data.check_scene_values(&player, &world)?;
    if data.presentation.is_none()
        || data.personal.entity != s.entity
        || data.world.revision < s.revision
    {
        return Err(invalid());
    }
    lifecycle.validate(&data)?;
    let actor = data
        .actors
        .0
        .iter()
        .find(|a| a.entity == s.entity)
        .ok_or_else(invalid)?;
    let life = super::battle_window::life(actor);
    if match rejection {
        action::Rejection::None => false,
        action::Rejection::NotActive => lifecycle.phase == world::Phase::Active,
        action::Rejection::StaleLife => life == s.life,
        action::Rejection::Dead => actor.vital.as_ref().is_none_or(|v| v.values.hp != 0),
        action::Rejection::StaleTarget => guard.target.is_none(),
    } {
        return Err(invalid());
    }
    if matches!(disposition, Disposition::Applied | Disposition::Denied)
        && lifecycle.phase != world::Phase::Aborted
    {
        let inner = &expected[116..];
        let kind = u16_at(inner, 12);
        let (sequence, actual) = match kind {
            20 => (i64_at(inner, 72), actor.last_move_sequence),
            22 => (
                i64_at(inner, 56),
                actor.vital.as_ref().ok_or_else(invalid)?.attack_sequence,
            ),
            26 | 44 => (i64_at(inner, 56), actor.spells.sequence),
            34 => (
                i64_at(inner, 56),
                data.personal
                    .supplies
                    .as_ref()
                    .ok_or_else(invalid)?
                    .request_sequence,
            ),
            36 => (
                i64_at(inner, 56),
                data.personal
                    .ammo
                    .as_ref()
                    .ok_or_else(invalid)?
                    .request_sequence,
            ),
            38 => (
                i64_at(inner, 64),
                actor.vital.as_ref().ok_or_else(invalid)?.attack_sequence,
            ),
            40 | 46 | 48 => (i64_at(inner, 64), actor.spells.sequence),
            42 => (
                i64_at(inner, 64),
                if inner[37] == 1 {
                    actor.last_move_sequence
                } else {
                    actor.vital.as_ref().ok_or_else(invalid)?.attack_sequence
                },
            ),
            _ => return Err(invalid()),
        };
        if life != s.life || actual != sequence || data.world.revision <= s.revision {
            return Err(invalid());
        }
    }
    Ok(Reply {
        guard,
        rejection,
        disposition,
        reason,
        lifecycle,
        data,
    })
}
fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(b[at..at + 2].try_into().unwrap())
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
        "invalid canonical battle operation receipt",
    )
}

#[cfg(test)]
#[path = "battle_operation_tests.rs"]
mod tests;
