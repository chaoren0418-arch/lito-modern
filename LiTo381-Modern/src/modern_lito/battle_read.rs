//! v35 read-only canonical data over the existing authenticated TLS owner.
//! Returned data remains unadmitted: no command ACK, saved receipt, renderer
//! update or legacy World/Combat conversion is implied by a successful read.
use super::{
    application_protocol as app,
    battle_payload::{self as payload, data},
    protocol,
};
pub const VERSION: u16 = 35;
pub const PRESENTATION_VERSION: u16 = 36;
pub const REQUEST_KIND: u16 = 60;
pub const RESULT_KIND: u16 = 61;
pub const REQUEST_BYTES: usize = 64;
pub const MIN_RESULT_BYTES: usize = 72;
pub const MAX_FRAME_BYTES: usize = MIN_RESULT_BYTES + payload::MAX_WIRE_BYTES;

#[derive(Clone, PartialEq, Eq)]
pub struct Request {
    pub context: app::SelectionContext,
    pub channel_id: u8,
    pub world: data::WorldId,
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleReadRequest[redacted]")
    }
}
impl Request {
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.world.room <= 0
            || self.world.match_id <= 0
            || self.world.generation <= 0
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn player(&self) -> data::Player {
        data::Player {
            source: self.context.key.source_id as i32,
            object: self.context.key.object_id as i64,
            epoch: self.context.epoch as i64,
            channel: i32::from(self.channel_id),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Preparing,
    Data,
    Unavailable,
}
#[derive(Clone, PartialEq, Eq)]
pub struct UnvalidatedReply {
    pub echo: Request,
    pub state: State,
    pub data: Option<payload::UnvalidatedData>,
}
impl std::fmt::Debug for UnvalidatedReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnvalidatedBattleRead[redacted]")
    }
}
impl UnvalidatedReply {
    /// Owner-only checked input for a future atomic inventory/UI update. This
    /// is not a complete world admission, durable receipt or SET capability.
    pub fn personal_view(
        &self,
    ) -> Result<Option<payload::personal::PersonalView>, app::ApplicationError> {
        self.echo.validate()?;
        match (&self.state, &self.data) {
            (State::Data, Some(d)) => d
                .personal_view(&self.echo.player(), &self.echo.world)
                .map(Some),
            (State::Preparing | State::Unavailable, None) => Ok(None),
            _ => Err(invalid()),
        }
    }
}
pub fn negotiated(ack: &protocol::Ack) -> bool {
    ack.version >= VERSION && ack.capabilities & protocol::CAP_BATTLE_DATA != 0
}
pub(crate) fn reply_negotiated(r: &UnvalidatedReply, ack: &protocol::Ack) -> bool {
    negotiated(ack) && r.data.as_ref().is_none_or(|d| data_negotiated(d, ack))
}
/// One format/capability gate shared by every canonical DATA reply family.
pub(crate) fn data_negotiated(d: &payload::UnvalidatedData, ack: &protocol::Ack) -> bool {
    if d.vip.is_some()
        || d.world.descriptor.features & payload::vip::FEATURE != 0
        || d.armor.is_some()
        || d.healing.is_some()
        || d.world.descriptor.features & (payload::healing::FEATURE | payload::armor::FEATURE) != 0
    {
        return false;
    }
    let needed = super::world_protocol::MatchDescriptor::feature_capabilities(
        d.world.descriptor.features as u32,
    );
    negotiated(ack)
        && ack.version <= protocol::MAX_VERSION
        && super::battle_greater_haste::data_negotiated(d, ack)
        && (if d.world.descriptor.features & payload::speed::FEATURE != 0 {
            ack.version >= payload::speed::VERSION
                && ack.capabilities & protocol::CAP_SLOW_DATA != 0
                && (d.world.descriptor.features & payload::haste::FEATURE == 0
                    || ack.capabilities & protocol::CAP_HASTE != 0)
                && d.speed.is_some()
                && d.haste.is_none()
                && d.npc_motion.is_none()
        } else if d.speed.is_some() {
            false
        } else if d.world.descriptor.features & payload::haste::FEATURE != 0 {
            ack.version >= payload::haste::minimum_version(d.world.descriptor.features)
                && ack.capabilities & protocol::CAP_HASTE != 0
                && d.haste.is_some()
                && d.npc_motion.is_none()
        } else {
            d.haste.is_none()
                && d.npc_motion.is_some() == (ack.version >= payload::npc_motion::VERSION)
        })
        && (d.presentation.is_some() == (ack.version >= PRESENTATION_VERSION))
        && ack.capabilities & needed == needed
        && (d.manifest.sectors.0.len() <= 16 || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
}
pub(crate) fn data_negotiated_selected(
    d: &payload::UnvalidatedData,
    peer: protocol::extensions::Selected,
) -> bool {
    let f = d.world.descriptor.features;
    let status = if let Some(v) = &d.vip {
        if !peer.allows_vip_data()
            || v.appearances.is_some() != peer.allows_vip_appearance()
            || f & payload::vip::FEATURE == 0
            || d.armor.is_some()
            || d.healing.is_some()
            || (f & payload::healing::FEATURE != 0) != v.healing.is_some()
            || v.healing.is_some() && !peer.allows_data()
            || (f & payload::armor::FEATURE != 0) != v.presentation.is_some()
            || v.presentation.is_some() && !peer.allows_armor_presentation()
        {
            return false;
        }
        &v.status
    } else if f & payload::vip::FEATURE != 0 {
        return false;
    } else if let Some(a) = &d.armor {
        if !peer.allows_armor()
            || a.presentation.is_some() != peer.allows_armor_presentation()
            || f & payload::armor::FEATURE == 0
            || d.healing.is_some()
            || (f & payload::healing::FEATURE != 0) != a.healing.is_some()
            || a.healing.is_some() && !peer.allows_data()
        {
            return false;
        }
        &a.status
    } else if let Some(h) = &d.healing {
        if !peer.allows_data()
            || f & payload::healing::FEATURE == 0
            || f & payload::armor::FEATURE != 0
        {
            return false;
        }
        &h.status
    } else {
        return data_negotiated(d, &peer.base());
    };
    let ack = peer.base();
    let needed = super::world_protocol::MatchDescriptor::feature_capabilities(f as u32);
    d.speed.is_none()
        && d.haste.is_none()
        && d.npc_motion.is_none()
        && d.presentation.is_some()
        && ack.capabilities & needed == needed
        && (d.manifest.sectors.0.len() <= 16 || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
        && match status {
            payload::healing::Status::Speed(_) => f & payload::speed::FEATURE != 0,
            payload::healing::Status::Haste(_) => {
                f & payload::speed::FEATURE == 0 && f & payload::haste::FEATURE != 0
            }
            payload::healing::Status::Motion(m) => {
                f & (payload::speed::FEATURE | payload::haste::FEATURE) == 0
                    && m.is_some() == d.npcs.is_some()
            }
        }
}
/// Existing commands do not carry the distinct saved/runtime VIP admission.
/// A read capability must never implicitly enable a mutation capability.
pub(crate) fn command_data_negotiated_selected(
    d: &payload::UnvalidatedData,
    peer: protocol::extensions::Selected,
) -> bool {
    d.vip.is_none()
        && d.world.descriptor.features & payload::vip::FEATURE == 0
        && data_negotiated_selected(d, peer)
}
pub(crate) fn reply_negotiated_selected(
    r: &UnvalidatedReply,
    peer: protocol::extensions::Selected,
) -> bool {
    negotiated(&peer.base())
        && r.data
            .as_ref()
            .is_none_or(|d| data_negotiated_selected(d, peer))
}
pub(crate) fn encode_request(id: u32, r: &Request) -> Result<Vec<u8>, app::ApplicationError> {
    r.validate()?;
    let mut b = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(r.context.key.source_id.to_be_bytes());
    b.extend(r.context.key.object_id.to_be_bytes());
    b.extend(r.context.epoch.to_be_bytes());
    b.extend(u32::from(r.channel_id).to_be_bytes());
    b.extend(r.world.room.to_be_bytes());
    b.extend(r.world.match_id.to_be_bytes());
    b.extend(r.world.generation.to_be_bytes());
    Ok(b)
}
pub(crate) fn decode_body(
    b: &[u8],
    version: u16,
) -> Result<UnvalidatedReply, app::ApplicationError> {
    decode_body_format(b, payload::Format::Legacy(version))
}
pub(crate) fn decode_body_selected(
    b: &[u8],
    peer: protocol::extensions::Selected,
) -> Result<UnvalidatedReply, app::ApplicationError> {
    decode_body_format(b, payload::Format::Selected(peer))
}
fn decode_body_format(
    b: &[u8],
    format: payload::Format,
) -> Result<UnvalidatedReply, app::ApplicationError> {
    let version = format.version();
    if version < VERSION
        || version > protocol::MAX_VERSION
        || b.len() < 56
        || b.len() > format.limit(MAX_FRAME_BYTES) - 16
    {
        return Err(invalid());
    }
    let i64_at = |at: usize| i64::from_be_bytes(b[at..at + 8].try_into().unwrap());
    let u32_at = |at: usize| u32::from_be_bytes(b[at..at + 4].try_into().unwrap());
    let echo = Request {
        context: app::SelectionContext {
            key: app::CharacterKey {
                source_id: u32_at(0),
                object_id: i64_at(4) as u64,
            },
            epoch: i64_at(12) as u64,
        },
        channel_id: u32_at(20).try_into().map_err(|_| invalid())?,
        world: data::WorldId {
            room: i64_at(24),
            match_id: i64_at(32),
            generation: i64_at(40),
        },
    };
    echo.validate()?;
    let state = match b[48] {
        0 => State::Preparing,
        1 => State::Data,
        2 => State::Unavailable,
        _ => return Err(invalid()),
    };
    let length = u32_at(52) as usize;
    if b[49..52] != [0, 0, 0] || length != b.len() - 56 || (state == State::Data) != (length != 0) {
        return Err(invalid());
    }
    let data = if state == State::Data {
        let d = format.decode(&b[56..])?;
        if d.presentation.is_some() != (version >= PRESENTATION_VERSION) {
            return Err(invalid());
        }
        d.check_scene_values(&echo.player(), &echo.world)?;
        Some(d)
    } else {
        None
    };
    Ok(UnvalidatedReply { echo, state, data })
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid canonical battle read",
    )
}
