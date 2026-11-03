//! v41 selected/life-bound equipment CAS. A durable write is not necessarily
//! a live-world success. Nested source equipment never replaces combat facts.
use super::{
    application_protocol as app,
    battle_follow::Observation,
    battle_operation::Lifecycle,
    battle_payload::{self as payload, data as d},
    battle_window::Stamp,
    equipment, protocol, world_protocol as world,
};
use std::io::Read;

pub const VERSION: u16 = 41;
pub const REQUEST_KIND: u16 = 70;
pub const RESULT_KIND: u16 = 71;
pub const REQUEST_BYTES: usize = 160;
pub const MIN_RESULT_BYTES: usize = 204;
pub const MAX_RESULT_BYTES: usize =
    MIN_RESULT_BYTES + equipment::MAX_FRAME_BYTES + payload::MAX_WIRE_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    StaleLife,
    Dead,
    Busy,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Disposition {
    NotCommitted,
    Unchanged,
    Applied,
    PersistedRetired,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Request {
    pub stamp: Stamp,
    pub equipment: equipment::Request,
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleEquipmentRequest[redacted]")
    }
}
impl Request {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        self.equipment.validate()?;
        let s = self.stamp;
        require(
            app::valid_context(s.context)
                && s.channel <= 3
                && s.world.iter().all(|v| *v > 0)
                && (1..=100).contains(&s.entity)
                && s.life > 0
                && s.revision > 0
                && self.equipment.context == s.context
                && self.equipment.action == equipment::Action::Set,
        )
    }
    /// The CAS token comes from the real equipment response, never synthesized
    /// from a canonical view. Both must still describe the same source/gear.
    pub(crate) fn from_observation(
        o: &Observation,
        snapshot: &equipment::Snapshot,
        item: u64,
        desired: bool,
    ) -> Result<Self, app::ApplicationError> {
        matches_snapshot(o.data(), snapshot)?;
        let q = Self {
            stamp: Stamp::of(o),
            equipment: equipment::Request::set(snapshot, item, desired),
        };
        q.validate()?;
        require(q.current(o))?;
        Ok(q)
    }
    pub(crate) fn current(&self, o: &Observation) -> bool {
        o.legacy_commands_supported()
            && self.stamp.current(o)
            && o.data().world.phase == d::Phase::Active
            && o.data()
                .personal
                .equipment
                .as_ref()
                .is_some_and(|e| e.revision == self.equipment.expected_revision)
            && o.data().actors.0.iter().any(|a| {
                a.entity == self.stamp.entity && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
            })
    }
    fn player(&self) -> d::Player {
        let s = self.stamp;
        d::Player {
            source: s.context.key.source_id as i32,
            object: s.context.key.object_id as i64,
            epoch: s.context.epoch as i64,
            channel: s.channel.into(),
        }
    }
    fn world(&self) -> d::WorldId {
        d::WorldId {
            room: self.stamp.world[0],
            match_id: self.stamp.world[1],
            generation: self.stamp.world[2],
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub reason: Reason,
    pub disposition: Disposition,
    pub durable_revision: i64,
    pub equipment: Option<equipment::Reply>,
    pub lifecycle: Lifecycle,
    pub data: payload::UnvalidatedData,
}
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleEquipmentReply[redacted]")
    }
}
impl Reply {
    pub(crate) fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = self.echo;
        q.validate()?;
        let s = q.stamp;
        self.data.check_scene_values(&q.player(), &q.world())?;
        self.lifecycle.validate(&self.data)?;
        require(
            self.data.presentation.is_some()
                && self.data.personal.entity == s.entity
                && self.data.world.revision >= s.revision,
        )?;
        let actor = self
            .data
            .actors
            .0
            .iter()
            .find(|a| a.entity == s.entity)
            .ok_or_else(invalid)?;
        // Shared incarnation proof, including honor/siege respawns. This is
        // the same ledger used by painted Stamp and server command admission.
        let life = super::battle_motion::life(actor).ok_or_else(invalid)?;
        if self.reason != Reason::None {
            require(
                self.equipment.is_none()
                    && self.disposition == Disposition::NotCommitted
                    && self.durable_revision == -1,
            )?;
            return require(match self.reason {
                Reason::NotActive => self.lifecycle.phase != world::Phase::Active,
                Reason::StaleLife => life != s.life,
                Reason::Dead => {
                    life == s.life && actor.vital.as_ref().is_some_and(|v| v.values.hp == 0)
                }
                Reason::Busy | Reason::Unavailable => true,
                Reason::None => false,
            });
        }
        let gear = self.equipment.as_ref().ok_or_else(invalid)?;
        gear.validate()?;
        require(gear.request == q.equipment)?;
        match self.disposition {
            Disposition::NotCommitted => {
                require(gear.status != equipment::Status::Ok && self.durable_revision == -1)
            }
            Disposition::Unchanged | Disposition::Applied => {
                let changed = self.disposition == Disposition::Applied;
                let snapshot = gear.snapshot.as_ref().ok_or_else(invalid)?;
                require(
                    gear.status == equipment::Status::Ok
                        && gear.changed == changed
                        && snapshot.revision == self.durable_revision
                        && life == s.life
                        && (!changed || self.data.world.revision > s.revision),
                )?;
                matches_snapshot(&self.data, snapshot)
            }
            Disposition::PersistedRetired => require(
                q.equipment.expected_revision.checked_add(1) == Some(self.durable_revision)
                    && gear.status == equipment::Status::BackendUnavailable
                    && gear.snapshot.is_none()
                    && self
                        .data
                        .personal
                        .equipment
                        .as_ref()
                        .is_some_and(|e| e.revision == q.equipment.expected_revision)
                    && matches!(
                        self.lifecycle.phase,
                        world::Phase::Aborted | world::Phase::Finished
                    ),
            ),
        }
    }
}
fn matches_snapshot(
    data: &payload::UnvalidatedData,
    snapshot: &equipment::Snapshot,
) -> Result<(), app::ApplicationError> {
    snapshot.validate()?;
    let personal = data.personal_view(&data.personal.player, &data.world.identity)?;
    let e = data.personal.equipment.as_ref().ok_or_else(invalid)?;
    let lifetimes = snapshot.lifetimes.as_ref().ok_or_else(invalid)?;
    require(
        personal.source() == Some(snapshot.source.as_ref())
            && e.revision == snapshot.revision
            && e.equipped.0.len() == snapshot.flags.len()
            && e.equipped
                .0
                .iter()
                .zip(&snapshot.flags)
                .all(|(a, b)| a.object as u64 == b.item_id && a.equipped == b.equipped)
            && e.lifetimes.0.len() == lifetimes.len()
            && e.lifetimes.0.iter().zip(lifetimes).all(|(a, b)| {
                a.object as u64 == b.item_id
                    && a.state as u8 == b.state as u8
                    && a.expires_at_millis == b.expires_at_millis.unwrap_or(-1)
            }),
    )
}
pub fn negotiated(ack: &protocol::Ack) -> bool {
    (VERSION..=protocol::MAX_VERSION).contains(&ack.version)
        && super::battle_world::negotiated(ack)
        && ack.capabilities & protocol::CAP_EQUIPMENT != 0
}
pub(crate) fn reply_negotiated(r: &Reply, ack: &protocol::Ack) -> bool {
    reply_negotiated_selected(r, protocol::extensions::Selected::from(*ack))
}
pub(crate) fn reply_negotiated_selected(r: &Reply, peer: protocol::extensions::Selected) -> bool {
    let ack = &peer.base();
    let needed =
        world::MatchDescriptor::feature_capabilities(r.data.world.descriptor.features as u32);
    negotiated(ack)
        && super::battle_read::command_data_negotiated_selected(&r.data, peer)
        && r.data.presentation.is_some()
        && ack.capabilities & needed == needed
        && (r.data.manifest.sectors.0.len() <= 16
            || ack.capabilities & protocol::CAP_LARGE_MAPS != 0)
}
pub fn encode_request(
    id: u32,
    q: &Request,
    version: u16,
) -> Result<Vec<u8>, app::ApplicationError> {
    q.validate()?;
    require((VERSION..=protocol::MAX_VERSION).contains(&version))?;
    let s = q.stamp;
    let mut b = app::header(id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for n in [
        s.world[0], s.world[1], s.world[2], s.entity, s.life, s.revision,
    ] {
        b.extend(n.to_be_bytes());
    }
    b.extend_from_slice(&equipment::encode_request(id, q.equipment)?[16..]);
    Ok(b)
}
fn expected_request(b: &[u8], version: u16) -> Result<Request, app::ApplicationError> {
    require(b.len() == REQUEST_BYTES)?;
    let context = |at| app::SelectionContext {
        key: app::CharacterKey {
            source_id: u32_at(b, at),
            object_id: i64_at(b, at + 4) as u64,
        },
        epoch: i64_at(b, at + 12) as u64,
    };
    require(b[108] == 1 && b[109] <= 1)?;
    let q = Request {
        stamp: Stamp {
            context: context(16),
            channel: u32_at(b, 36).try_into().map_err(|_| invalid())?,
            world: [i64_at(b, 40), i64_at(b, 48), i64_at(b, 56)],
            entity: i64_at(b, 64),
            life: i64_at(b, 72),
            revision: i64_at(b, 80),
        },
        equipment: equipment::Request {
            context: context(88),
            action: equipment::Action::Set,
            desired: b[109] == 1,
            item_id: i64_at(b, 112) as u64,
            expected_revision: i64_at(b, 120),
            expected_view_token: b[128..160].try_into().unwrap(),
        },
    };
    require(encode_request(u32_at(b, 8), &q, version)? == b)?;
    Ok(q)
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
    let mut h = [0; 16];
    reader.read_exact(&mut h).map_err(|_| invalid())?;
    let size = u32_at(&h, 4) as usize;
    require(
        &h[..4] == b"LTM1"
            && h[8..12] == expected[8..12]
            && h[12..14] == RESULT_KIND.to_be_bytes()
            && h[14..16] == [0; 2]
            && (MIN_RESULT_BYTES..=format.limit(MAX_RESULT_BYTES)).contains(&size),
    )?;
    let mut body = vec![0; size - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    require(body[..REQUEST_BYTES - 16] == expected[16..])?;
    let m = &body[REQUEST_BYTES - 16..MIN_RESULT_BYTES - 16];
    let gear_len = u32_at(m, 36) as usize;
    let data_len = u32_at(m, 40) as usize;
    require(
        m[2..4] == [0; 2]
            && gear_len <= equipment::MAX_FRAME_BYTES
            && data_len <= format.maximum()
            && gear_len.checked_add(data_len) == Some(size - MIN_RESULT_BYTES),
    )?;
    let nested = &body[MIN_RESULT_BYTES - 16..MIN_RESULT_BYTES - 16 + gear_len];
    let gear = if nested.is_empty() {
        None
    } else {
        let frame = app::decode_response_version(nested, version)?;
        require(frame.request_id == u32_at(expected, 8))?;
        match frame.response {
            app::Response::Equipment(r) if r.request == q.equipment => Some(r),
            _ => return Err(invalid()),
        }
    };
    let r = Reply {
        echo: q,
        reason: *[
            Reason::None,
            Reason::NotActive,
            Reason::StaleLife,
            Reason::Dead,
            Reason::Busy,
            Reason::Unavailable,
        ]
        .get(m[0] as usize)
        .ok_or_else(invalid)?,
        disposition: *[
            Disposition::NotCommitted,
            Disposition::Unchanged,
            Disposition::Applied,
            Disposition::PersistedRetired,
        ]
        .get(m[1] as usize)
        .ok_or_else(invalid)?,
        durable_revision: i64_at(m, 4),
        lifecycle: Lifecycle::read(&m[12..36])?,
        equipment: gear,
        data: format.decode_command(&body[MIN_RESULT_BYTES - 16 + gear_len..])?,
    };
    r.validate()?;
    Ok(r)
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
fn require(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid canonical equipment receipt",
    )
}

#[cfg(test)]
#[path = "battle_equipment_tests.rs"]
mod tests;
