//! Explicit v47 targeted skill29 intent/receipt. Codec/transport support does not
//! authorize a player UI or bypass source/world/pending-intent admission.
use super::{
    application_protocol as app, battle_follow,
    battle_operation::Lifecycle,
    battle_payload::{self as payload, data as d},
    battle_window::Stamp,
};
pub const VERSION: u16 = 47;
pub const REQUEST_KIND: u16 = 76;
pub const RESULT_KIND: u16 = 77;
pub const REQUEST_BYTES: usize = 184;
pub const RESULT_PREFIX_BYTES: usize = 216;
pub const MAX_RESULT_BYTES: usize = RESULT_PREFIX_BYTES + payload::MAX_WIRE_BYTES;
pub fn negotiated(ack: &super::protocol::Ack) -> bool {
    use super::protocol as p;
    let required = p::CAP_SLOW_DATA
        | p::CAP_TIMED
        | p::CAP_BATTLE_DATA
        | p::CAP_ACTION_LIFE
        | p::CAP_MATCH_DESCRIPTOR;
    ack.version >= VERSION && ack.capabilities & required == required
}
pub use super::battle_target_haste::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reason {
    None,
    NotActive,
    Dead,
    Unlearned,
    Unsupported,
    TooFast,
    NoMp,
    TooHeavy,
    StaleState,
    Numeric,
    TargetDead,
    StaleTarget,
    WrongInstance,
    OutOfRange,
    Blocked,
    SameTeam,
    NoMaterial,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Intent {
    pub request_id: u32,
    pub stamp: Stamp,
    pub sequence: i64,
    pub equipment_revision: i64,
    pub x: i32,
    pub y: i32,
    pub buff_revision: i64,
    pub target: Target,
}
impl std::fmt::Debug for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SlowIntent[redacted]")
    }
}
impl Intent {
    pub(crate) fn from_observation(
        o: &battle_follow::Observation,
        target: super::battle_window::Target,
    ) -> Option<Self> {
        if !o.legacy_commands_supported() {
            return None;
        }
        let data = o.data();
        let a = actor(data, data.personal.entity).ok()?;
        if data.speed_state().is_none()
            || data.world.phase != d::Phase::Active
            || !a.loaded
            || !a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        {
            return None;
        }
        let q = Self {
            request_id: 1,
            stamp: Stamp::of(o),
            sequence: a.spells.sequence.checked_add(1)?,
            equipment_revision: a.resources.as_ref()?.equipment_revision,
            x: a.position.x,
            y: a.position.y,
            buff_revision: a.buffs.as_ref()?.revision,
            target: Target::from_data(data, target)?,
        };
        q.validate_before(data).ok()?;
        Some(q)
    }
    pub(crate) fn current(&self, o: &battle_follow::Observation) -> bool {
        // Unrelated world ticks do not replace the painted proof. All scope,
        // life, sequence, equipment, position and target facts must still match.
        self.validate().is_ok()
            && self.stamp.current(o)
            && Self::from_observation(o, self.target.selection()).is_some_and(|q| {
                Self {
                    request_id: self.request_id,
                    stamp: self.stamp,
                    ..q
                } == *self
            })
    }
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        let s = self.stamp;
        require(
            self.request_id != 0
                && app::valid_context(s.context)
                && s.channel <= 3
                && s.world.iter().all(|n| *n > 0)
                && (1..=100).contains(&s.entity)
                && s.life > 0
                && s.revision > 0
                && self.sequence > 0
                && self.equipment_revision >= 0
                && (0..=s.revision).contains(&self.buff_revision)
                && self.target.valid(s.world, s.revision),
        )
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
        let w = self.stamp.world;
        d::WorldId {
            room: w[0],
            match_id: w[1],
            generation: w[2],
        }
    }
    pub(crate) fn validate_before(
        &self,
        before: &payload::UnvalidatedData,
    ) -> Result<(), app::ApplicationError> {
        self.validate()?;
        let old = actor(before, self.stamp.entity)?;
        let resources = old.resources.as_ref().ok_or_else(invalid)?;
        require(
            before.speed_state().is_some()
                && before.personal.player == self.player()
                && before.world.identity == self.world()
                && before.personal.entity == self.stamp.entity
                && before.world.revision >= self.stamp.revision
                && resources.life.checked_add(1) == Some(self.stamp.life)
                && resources.equipment_revision == self.equipment_revision
                && old.position.x == self.x
                && old.position.y == self.y
                && old
                    .buffs
                    .as_ref()
                    .is_some_and(|b| b.revision == self.buff_revision)
                && old.spells.sequence.checked_add(1) == Some(self.sequence)
                && Target::from_data(before, self.target.selection()) == Some(self.target),
        )
    }
}
pub fn encode_intent(q: &Intent, version: u16) -> Result<Vec<u8>, app::ApplicationError> {
    require((VERSION..=super::protocol::MAX_VERSION).contains(&version))?;
    q.validate()?;
    let s = q.stamp;
    let mut b = app::header(q.request_id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for n in s.world.into_iter().chain([
        s.entity,
        s.life,
        s.revision,
        q.sequence,
        q.equipment_revision,
    ]) {
        b.extend(n.to_be_bytes());
    }
    b.extend(q.x.to_be_bytes());
    b.extend(q.y.to_be_bytes());
    b.extend(q.buff_revision.to_be_bytes());
    q.target.write(&mut b);
    Ok(b)
}
fn i64_at(b: &[u8], i: usize) -> i64 {
    i64::from_be_bytes(b[i..i + 8].try_into().unwrap())
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_be_bytes(b[i..i + 4].try_into().unwrap())
}
fn header(b: &[u8], kind: u16) -> Result<(), app::ApplicationError> {
    require(
        b.len() >= 16
            && b[..4] == [0x4c, 0x54, 0x4d, 0x31]
            && u32_at(b, 4) as usize == b.len()
            && u32_at(b, 8) != 0
            && b[12..14] == kind.to_be_bytes()
            && b[14..16] == [0, 0],
    )
}
pub fn decode_intent(b: &[u8], version: u16) -> Result<Intent, app::ApplicationError> {
    require(
        (VERSION..=super::protocol::MAX_VERSION).contains(&version) && b.len() == REQUEST_BYTES,
    )?;
    header(b, REQUEST_KIND)?;
    let q = Intent {
        request_id: u32_at(b, 8),
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: u32_at(b, 16),
                    object_id: i64_at(b, 20) as u64,
                },
                epoch: i64_at(b, 28) as u64,
            },
            channel: u32_at(b, 36).try_into().map_err(|_| invalid())?,
            world: [i64_at(b, 40), i64_at(b, 48), i64_at(b, 56)],
            entity: i64_at(b, 64),
            life: i64_at(b, 72),
            revision: i64_at(b, 80),
        },
        sequence: i64_at(b, 88),
        equipment_revision: i64_at(b, 96),
        x: u32_at(b, 104) as i32,
        y: u32_at(b, 108) as i32,
        buff_revision: i64_at(b, 112),
        target: Target::read(b)?,
    };
    q.validate()?;
    Ok(q)
}
#[derive(Clone, PartialEq, Eq)]
pub struct CandidateReply {
    pub echo: Intent,
    pub reason: Reason,
    pub lifecycle: Lifecycle,
    pub data: payload::UnvalidatedData,
}
impl std::fmt::Debug for CandidateReply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SlowReceipt[redacted]")
    }
}
fn actor(data: &payload::UnvalidatedData, entity: i64) -> Result<&d::Actor, app::ApplicationError> {
    data.actors
        .0
        .iter()
        .find(|a| a.entity == entity)
        .ok_or_else(invalid)
}
fn spell(a: &d::Actor) -> Result<&d::Spell, app::ApplicationError> {
    a.spells
        .histories
        .0
        .iter()
        .find(|s| s.lane == d::Lane::Slow)
        .ok_or_else(invalid)
}
impl CandidateReply {
    pub fn validate(&self) -> Result<(), app::ApplicationError> {
        let q = self.echo;
        q.validate()?;
        let data = &self.data;
        let w = &data.world;
        let s = q.stamp;
        data.check_scene_values(&q.player(), &q.world())?;
        self.lifecycle.validate(data)?;
        require(
            data.speed_state().is_some()
                && data.personal.entity == s.entity
                && s.revision <= w.revision,
        )?;
        let a = actor(data, s.entity)?;
        let r = spell(a)?.receipt.as_ref();
        let materials = data.personal.materials.as_ref().ok_or_else(invalid)?;
        if self.reason == Reason::NotActive {
            return require(
                self.lifecycle.phase != super::world_protocol::Phase::Active
                    || w.phase != d::Phase::Active,
            );
        }
        require(
            w.phase == d::Phase::Active
                && a.spells.sequence == q.sequence
                && a.spells.request_revision == w.revision
                && w.revision > s.revision,
        )?;
        if self.reason == Reason::None {
            let r = r.ok_or_else(invalid)?;
            let c = &r.impact.usage;
            require(
                c.sequence == q.sequence
                    && c.revision == w.revision
                    && c.before.equipment_revision == q.equipment_revision
                    && c.before.life == s.life
                    && c.before.cell.x == q.x
                    && c.before.cell.y == q.y
                    && matches!(&r.detail, d::Detail::Slow(detail) if q.target.matches_receipt(&detail.target))
                    && materials.last_sequence == q.sequence
                    && materials.revision == w.revision
                    && materials.last_life == s.life
                    && materials.last_equipment_revision == q.equipment_revision,
            )?;
        } else {
            require(
                r.is_none_or(|r| r.impact.usage.sequence < q.sequence)
                    && materials.last_sequence < q.sequence,
            )?;
        }
        if self.reason == Reason::Dead {
            require(a.vital.as_ref().is_some_and(|v| v.values.hp == 0))?;
        }
        Ok(())
    }
    /// Bind the immutable painted intent to the last confirmed predecessor.
    /// A newer world revision is allowed only with unchanged proof fields;
    /// validate the whole successor. An inactive world need not consume the
    /// spell sequence, so the snapshot cannot replace the pending intent.
    /// Does not consume the caller's snapshot or install partial state.
    pub fn validate_after(
        &self,
        expected: &Intent,
        before: &payload::UnvalidatedData,
    ) -> Result<(), app::ApplicationError> {
        require(self.echo == *expected)?;
        self.validate()?;
        let q = self.echo;
        q.validate_before(before)?;
        let old = actor(before, q.stamp.entity)?;
        let old_resources = old.resources.as_ref().ok_or_else(invalid)?;
        battle_follow::check_candidate_sequence(before, &self.data)?;
        let new = actor(&self.data, q.stamp.entity)?;
        if self.reason == Reason::NotActive {
            require(
                new.spells.sequence == old.spells.sequence
                    && new.spells.request_revision == old.spells.request_revision
                    || new.spells.sequence == q.sequence
                        && new.spells.request_revision == self.data.world.revision
                        && self.data.world.revision > before.world.revision,
            )?;
        }
        let material_before = before.personal.materials.as_ref().ok_or_else(invalid)?;
        let material_after = self.data.personal.materials.as_ref().ok_or_else(invalid)?;
        if self.reason == Reason::None {
            require(
                material_before.uses.checked_add(1) == Some(material_after.uses)
                    && material_before.stocks.0.len() == 1
                    && material_after.stocks.0.len() == 1,
            )?;
            let a = &material_before.stocks.0[0];
            let b = &material_after.stocks.0[0];
            require(
                a.item == 40318
                    && b.item == a.item
                    && a.unit_weight == b.unit_weight
                    && a.initial == b.initial
                    && a.remaining.checked_sub(1) == Some(b.remaining),
            )?;
        } else {
            require(material_before == material_after)?;
        }
        let previous = spell(old)?;
        let current = spell(new)?;
        require(
            old.spells
                .histories
                .0
                .iter()
                .zip(&new.spells.histories.0)
                .all(|(a, b)| {
                    a.lane == b.lane
                        && (a.lane == d::Lane::Slow
                            || a.ledger.casts == b.ledger.casts
                                && a.ledger.spent == b.ledger.spent
                                && a.last == b.last
                                && a.receipt == b.receipt)
                }),
        )?;
        let spent = if self.reason == Reason::None {
            i64::from(
                current
                    .receipt
                    .as_ref()
                    .ok_or_else(invalid)?
                    .impact
                    .usage
                    .cost,
            )
        } else {
            0
        };
        require(
            old_resources.flows.spent.checked_add(spent)
                == Some(new.resources.as_ref().ok_or_else(invalid)?.flows.spent),
        )?;
        if self.reason == Reason::None {
            let receipt = current.receipt.as_ref().ok_or_else(invalid)?;
            require(
                previous.ledger.casts.checked_add(1) == Some(current.ledger.casts)
                    && receipt.impact.usage.cost == previous.ledger.cost
                    && previous
                        .ledger
                        .spent
                        .checked_add(i64::from(receipt.impact.usage.cost))
                        == Some(current.ledger.spent),
            )?;
        } else {
            require(
                previous.ledger.casts == current.ledger.casts
                    && previous.ledger.spent == current.ledger.spent
                    && previous.last == current.last
                    && previous.receipt == current.receipt,
            )?;
        }
        Ok(())
    }
}
pub fn decode_reply(
    b: &[u8],
    version: u16,
    expected: &Intent,
) -> Result<CandidateReply, app::ApplicationError> {
    decode_reply_format(b, payload::Format::Legacy(version), expected)
}
pub fn decode_reply_selected(
    b: &[u8],
    peer: crate::modern_lito::protocol::extensions::Selected,
    expected: &Intent,
) -> Result<CandidateReply, app::ApplicationError> {
    decode_reply_format(b, payload::Format::Selected(peer), expected)
}
fn decode_reply_format(
    b: &[u8],
    format: payload::Format,
    expected: &Intent,
) -> Result<CandidateReply, app::ApplicationError> {
    let version = format.version();
    require(
        (VERSION..=super::protocol::MAX_VERSION).contains(&version)
            && (RESULT_PREFIX_BYTES..=format.limit(MAX_RESULT_BYTES)).contains(&b.len()),
    )?;
    header(b, RESULT_KIND)?;
    let mut echo = b[..REQUEST_BYTES].to_vec();
    echo[4..8].copy_from_slice(&(REQUEST_BYTES as u32).to_be_bytes());
    echo[12..14].copy_from_slice(&REQUEST_KIND.to_be_bytes());
    let echo = decode_intent(&echo, version)?;
    require(echo == *expected && b[185..188] == [0; 3])?;
    let reason = *[
        Reason::None,
        Reason::NotActive,
        Reason::Dead,
        Reason::Unlearned,
        Reason::Unsupported,
        Reason::TooFast,
        Reason::NoMp,
        Reason::TooHeavy,
        Reason::StaleState,
        Reason::Numeric,
        Reason::TargetDead,
        Reason::StaleTarget,
        Reason::WrongInstance,
        Reason::OutOfRange,
        Reason::Blocked,
        Reason::SameTeam,
        Reason::NoMaterial,
    ]
    .get(b[184] as usize)
    .ok_or_else(invalid)?;
    let lifecycle = Lifecycle::read(&b[188..212])?;
    require(u32_at(b, 212) as usize == b.len() - RESULT_PREFIX_BYTES)?;
    let data = format.decode_command(&b[RESULT_PREFIX_BYTES..])?;
    require(data.speed_state().is_some())?;
    let result = CandidateReply {
        echo,
        reason,
        lifecycle,
        data,
    };
    result.validate()?;
    Ok(result)
}
/// Validate kind/id/length before allocation; leave a coalesced next frame unread.
pub fn read_reply(
    reader: &mut impl std::io::Read,
    expected: &[u8],
    version: u16,
) -> Result<CandidateReply, app::ApplicationError> {
    read_reply_format(reader, expected, payload::Format::Legacy(version))
}
pub fn read_reply_selected(
    reader: &mut impl std::io::Read,
    expected: &[u8],
    peer: crate::modern_lito::protocol::extensions::Selected,
) -> Result<CandidateReply, app::ApplicationError> {
    read_reply_format(reader, expected, payload::Format::Selected(peer))
}
fn read_reply_format(
    reader: &mut impl std::io::Read,
    expected: &[u8],
    format: payload::Format,
) -> Result<CandidateReply, app::ApplicationError> {
    let version = format.version();
    let q = decode_intent(expected, version)?;
    let mut h = [0; 16];
    reader.read_exact(&mut h).map_err(|_| invalid())?;
    let size = u32_at(&h, 4) as usize;
    require(
        &h[..4] == b"LTM1"
            && h[8..12] == expected[8..12]
            && h[12..14] == RESULT_KIND.to_be_bytes()
            && h[14..16] == [0; 2]
            && (RESULT_PREFIX_BYTES..=format.limit(MAX_RESULT_BYTES)).contains(&size),
    )?;
    let mut bytes = vec![0; size];
    bytes[..16].copy_from_slice(&h);
    reader.read_exact(&mut bytes[16..]).map_err(|_| invalid())?;
    decode_reply_format(&bytes, format, &q)
}
fn require(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn invalid() -> app::ApplicationError {
    app::error(app::ErrorKind::MalformedFrame, "invalid slow contract")
}
#[cfg(test)]
#[path = "battle_slow_tests.rs"]
mod tests;
