//! LTB1 container and complete typed syntax codec used by the opt-in read-only
//! channel. `inspect` only checks framing; `decode` returns owned
//! `UnvalidatedData`. The checked personal view is not a complete world proof:
//! mode semantics, ACKs, durable receipts and atomic following remain separate.
//! Reading a payload does not install live world/UI state here.
use super::application_protocol::ApplicationError;

/// Complete typed candidate data. Parsing is not application admission: the
/// negotiated owner/world, semantic proof and atomic follower remain required.
#[path = "battle_payload_codec.rs"]
mod codec;
#[path = "battle_payload_data.rs"]
pub mod data;
pub use data::{decode, encode, UnvalidatedData};
/// One selected payload format shared by all reply families. Version-only
/// callers deliberately remain incapable of decoding the extension container.
#[derive(Clone, Copy)]
pub(crate) enum Format {
    Legacy(u16),
    Selected(super::protocol::extensions::Selected),
}
impl Format {
    pub(crate) fn version(self) -> u16 {
        match self {
            Self::Legacy(v) => v,
            Self::Selected(p) => p.base().version,
        }
    }
    pub(crate) fn maximum(self) -> usize {
        match self {
            Self::Selected(p) if p.allows_vip_appearance() => vip::MAX_APPEARANCE_BYTES,
            Self::Selected(p) if p.allows_vip_data() => vip::MAX_BYTES,
            Self::Selected(p) if p.allows_armor_presentation() => armor_presentation::MAX_BYTES,
            Self::Selected(p) if p.allows_armor() => armor::MAX_BYTES,
            Self::Selected(p) if p.allows_data() => healing::MAX_BYTES,
            _ => MAX_WIRE_BYTES,
        }
    }
    pub(crate) fn limit(self, old: usize) -> usize {
        old + self.maximum() - MAX_WIRE_BYTES
    }
    pub(crate) fn decode(self, bytes: &[u8]) -> Result<UnvalidatedData, ApplicationError> {
        match self {
            Self::Legacy(v) => decode_versioned(bytes, v),
            Self::Selected(p) => decode_selected(bytes, p),
        }
    }
    /// READ support is not permission to use the old mutation receipts.
    pub(crate) fn decode_command(self, bytes: &[u8]) -> Result<UnvalidatedData, ApplicationError> {
        let data = self.decode(bytes)?;
        if data.vip.is_some() || data.world.descriptor.features & vip::FEATURE != 0 {
            return Err(invalid());
        }
        Ok(data)
    }
}
pub(crate) fn decode_selected(
    bytes: &[u8],
    peer: super::protocol::extensions::Selected,
) -> Result<UnvalidatedData, ApplicationError> {
    if [vip::SCHEMA, vip::APPEARANCE_SCHEMA]
        .iter()
        .any(|s| bytes.get(4..6) == Some(s.to_be_bytes().as_slice()))
    {
        if !peer.allows_vip_data() {
            return Err(invalid());
        }
        let data = vip::decode(bytes)?.into_data()?;
        if !super::battle_read::data_negotiated_selected(&data, peer) {
            return Err(invalid());
        }
        return Ok(data);
    }
    let schema = inspect_with_presentation(
        bytes,
        true,
        true,
        true,
        peer.allows_data(),
        peer.allows_armor(),
        peer.allows_armor_presentation(),
    )?
    .schema();
    let data = match schema {
        armor_presentation::SCHEMA if peer.allows_armor_presentation() => {
            armor_presentation::decode(bytes)?
        }
        armor::SCHEMA if peer.allows_armor() => armor::decode(bytes)?.into_data()?,
        healing::SCHEMA if peer.allows_data() => healing::decode(bytes)?.into_data()?,
        _ => decode_versioned(bytes, peer.base().version)?,
    };
    if !super::battle_read::data_negotiated_selected(&data, peer) {
        return Err(invalid());
    }
    Ok(data)
}
/// Existing application versions must not admit the new data capability.
pub(crate) fn decode_legacy(bytes: &[u8]) -> Result<UnvalidatedData, ApplicationError> {
    if inspect(bytes)?.schema() == GREATER_HASTE_SCHEMA {
        return Err(invalid());
    }
    decode(bytes)
}
/// Framing version gate only; capability and owner admission follow parsing.
pub(crate) fn decode_versioned(
    bytes: &[u8],
    version: u16,
) -> Result<UnvalidatedData, ApplicationError> {
    if version > super::protocol::MAX_VERSION {
        Err(invalid())
    } else if version >= speed::VERSION
        && inspect_extended(bytes, true, true, true)?.schema() == speed::SCHEMA
    {
        let c = speed::decode(bytes)?;
        let mut data = c.data;
        data.speed = Some(speed::Envelope { state: c.effects });
        Ok(data)
    } else if version >= haste::VERSION
        && inspect_candidate(bytes, true, true)?.schema() == haste::SCHEMA
    {
        let candidate = haste::decode(bytes)?;
        if version < haste::minimum_version(candidate.data.world.descriptor.features) {
            return Err(invalid());
        }
        let mut data = candidate.data;
        data.haste = Some(haste::Envelope {
            state: candidate.effects,
        });
        Ok(data)
    } else if version >= npc_motion::VERSION {
        let candidate = npc_motion::decode(bytes)?;
        let mut data = candidate.data;
        data.npc_motion = Some(npc_motion::Envelope {
            state: candidate.motion,
        });
        Ok(data)
    } else if version < super::battle_greater_haste::VERSION {
        decode_legacy(bytes)
    } else {
        decode(bytes)
    }
}
#[path = "battle_payload_combat.rs"]
mod combat;
#[cfg(test)]
#[path = "battle_payload_combat_tests.rs"]
mod combat_tests;
#[path = "battle_payload_context.rs"]
mod context;
#[cfg(test)]
#[path = "battle_payload_data_tests.rs"]
pub(crate) mod data_tests;
#[path = "battle_payload_haste.rs"]
pub(crate) mod haste;
#[path = "battle_payload_honor.rs"]
mod honor;
#[path = "battle_payload_life.rs"]
mod life;
#[cfg(test)]
#[path = "battle_payload_mode_tests.rs"]
mod mode_tests;
#[path = "battle_payload_npc_motion.rs"]
pub(crate) mod npc_motion;
#[path = "battle_payload_npcs.rs"]
mod npcs;
#[path = "battle_payload_personal.rs"]
pub mod personal;
#[cfg(test)]
#[path = "battle_payload_personal_tests.rs"]
mod personal_tests;
#[path = "battle_payload_presentation.rs"]
pub(crate) mod presentation;
#[path = "battle_payload_quest.rs"]
mod quest;
#[path = "battle_payload_race.rs"]
mod race;
#[cfg(test)]
#[path = "battle_read_tests.rs"]
mod read_tests;
#[path = "battle_payload_scene.rs"]
mod scene;
#[cfg(test)]
#[path = "battle_payload_scene_tests.rs"]
mod scene_tests;
#[path = "battle_payload_siege.rs"]
mod siege;
#[path = "battle_payload_speed.rs"]
pub(crate) mod speed;
#[path = "battle_payload_spells.rs"]
mod spells;
#[path = "battle_payload_time.rs"]
mod time;
#[path = "battle_payload_values.rs"]
mod values;

pub const MAGIC: u32 = 0x4c54_4231;
pub const SCHEMA: u16 = 1;
pub const PRESENTATION_SCHEMA: u16 = 2;
pub const GREATER_HASTE_SCHEMA: u16 = 3;
pub const PRESENTATION_LIMIT: usize = 4 + 100 * 256;
pub const HEADER_BYTES: usize = 16;
pub const SECTION_COUNT: usize = 12;
pub const ACTOR_LIMIT: usize = 32 * 1024;
pub const SECTION_LIMITS: [usize; SECTION_COUNT] = [
    256,
    64 * 1024,
    8 * 1024,
    4 + 100 * ACTOR_LIMIT,
    32 * 1024,
    32 * 1024,
    16 * 1024,
    32 * 1024,
    8 * 1024,
    128 * 1024,
    512 * 1024,
    32 * 1024,
];
pub const MAX_BYTES: usize = HEADER_BYTES
    + SECTION_COUNT * 8
    + 256
    + 64 * 1024
    + 8 * 1024
    + 4
    + 100 * ACTOR_LIMIT
    + 32 * 1024
    + 32 * 1024
    + 16 * 1024
    + 32 * 1024
    + 8 * 1024
    + 128 * 1024
    + 512 * 1024
    + 32 * 1024;
pub const MAX_PRESENTATION_BYTES: usize = MAX_BYTES + 8 + PRESENTATION_LIMIT;
pub const MAX_WIRE_BYTES: usize = speed::MAX_BYTES;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Section {
    World = 1,
    Room,
    Manifest,
    Actors,
    Honor,
    Quest,
    TimeAttack,
    Siege,
    Race,
    Npcs,
    Personal,
    Result,
    Presentation,
}

/// Contains potentially malicious semantic content. No conversion to a live
/// scene, authoritative world, inventory or selection is implemented here.
pub struct UnvalidatedPayload<'a> {
    bytes: &'a [u8],
    sections: [&'a [u8]; SECTION_COUNT + 7],
    schema: u16,
}
impl<'a> UnvalidatedPayload<'a> {
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
    pub fn schema(&self) -> u16 {
        self.schema
    }
    // Schema9 explicitly preserves the first sixteen schema8 layouts. This
    // selects their field codec; section17 is still mandatory and retained.
    fn layout_schema(&self) -> u16 {
        if matches!(
            self.schema,
            armor_presentation::SCHEMA | vip::SCHEMA | vip::APPEARANCE_SCHEMA
        ) {
            armor::SCHEMA
        } else {
            self.schema
        }
    }
    pub fn section(&self, id: Section) -> &'a [u8] {
        self.sections[id as usize - 1]
    }
}
impl std::fmt::Debug for UnvalidatedPayload<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnvalidatedBattlePayload[redacted]")
    }
}
fn invalid() -> ApplicationError {
    super::application_protocol::error(
        super::application_protocol::ErrorKind::MalformedFrame,
        "invalid battle payload framing",
    )
}
pub fn inspect(bytes: &[u8]) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_inner(bytes, false)
}
// Only the explicit schema4 codec (selected by decode_versioned at v43) can
// inspect that container. Legacy inspect/decode stay schema1-3 only.
fn inspect_inner(
    bytes: &[u8],
    motion_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_candidate(bytes, motion_candidate, false)
}
fn inspect_candidate(
    bytes: &[u8],
    motion_candidate: bool,
    haste_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_extended(bytes, motion_candidate, haste_candidate, false)
}
fn inspect_extended(
    bytes: &[u8],
    motion_candidate: bool,
    haste_candidate: bool,
    speed_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_all(
        bytes,
        motion_candidate,
        haste_candidate,
        speed_candidate,
        false,
    )
}
#[path = "battle_payload_armor.rs"]
pub mod armor;
#[path = "battle_payload_armor_presentation.rs"]
pub mod armor_presentation;
#[path = "battle_payload_healing.rs"]
pub mod healing;
#[path = "battle_payload_vip.rs"]
pub mod vip;
fn full_speed(schema: u16) -> bool {
    matches!(schema, speed::SCHEMA | healing::SCHEMA | armor::SCHEMA)
}
fn inspect_all(
    bytes: &[u8],
    motion_candidate: bool,
    haste_candidate: bool,
    speed_candidate: bool,
    healing_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_formats(
        bytes,
        motion_candidate,
        haste_candidate,
        speed_candidate,
        healing_candidate,
        false,
    )
}
fn inspect_formats(
    bytes: &[u8],
    motion_candidate: bool,
    haste_candidate: bool,
    speed_candidate: bool,
    healing_candidate: bool,
    armor_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    inspect_with_presentation(
        bytes,
        motion_candidate,
        haste_candidate,
        speed_candidate,
        healing_candidate,
        armor_candidate,
        false,
    )
}
fn inspect_with_presentation(
    bytes: &[u8],
    motion_candidate: bool,
    haste_candidate: bool,
    speed_candidate: bool,
    healing_candidate: bool,
    armor_candidate: bool,
    presentation_candidate: bool,
) -> Result<UnvalidatedPayload<'_>, ApplicationError> {
    let maximum = if presentation_candidate {
        armor_presentation::MAX_BYTES
    } else if armor_candidate {
        armor::MAX_BYTES
    } else if healing_candidate {
        healing::MAX_BYTES
    } else if speed_candidate {
        speed::MAX_BYTES
    } else if haste_candidate {
        haste::MAX_BYTES
    } else if motion_candidate {
        npc_motion::MAX_BYTES
    } else {
        MAX_PRESENTATION_BYTES
    };
    if !(HEADER_BYTES + SECTION_COUNT * 8..=maximum).contains(&bytes.len()) {
        return Err(invalid());
    }
    let mut r = Cursor { bytes, position: 0 };
    if r.u32()? != MAGIC {
        return Err(invalid());
    }
    let schema = r.u16()?;
    let count = match schema {
        SCHEMA => SECTION_COUNT,
        PRESENTATION_SCHEMA | GREATER_HASTE_SCHEMA => SECTION_COUNT + 1,
        npc_motion::SCHEMA if motion_candidate => SECTION_COUNT + 2,
        haste::SCHEMA if haste_candidate => SECTION_COUNT + 2,
        speed::SCHEMA if speed_candidate => SECTION_COUNT + 2,
        healing::SCHEMA if healing_candidate => SECTION_COUNT + 3,
        armor::SCHEMA if armor_candidate => SECTION_COUNT + 4,
        armor_presentation::SCHEMA if armor_candidate && presentation_candidate => {
            SECTION_COUNT + 5
        }
        _ => return Err(invalid()),
    };
    if r.u16()? != 0
        || r.u32()? as usize != bytes.len()
        || r.u16()? as usize != count
        || r.u16()? != 0
    {
        return Err(invalid());
    }
    let mut sections = [&[][..]; SECTION_COUNT + 7];
    for (i, section) in sections.iter_mut().enumerate().take(count) {
        if r.u16()? as usize != i + 1 || r.u16()? != 0 {
            return Err(invalid());
        }
        let n = r.u32()? as usize;
        let armor_layout = matches!(schema, armor::SCHEMA | armor_presentation::SCHEMA);
        let limit = if schema == armor_presentation::SCHEMA && i + 1 == 17 {
            armor_presentation::LIMIT
        } else if armor_layout && i + 1 == 16 {
            armor::LIMIT
        } else if armor_layout && i + 1 == 15 {
            armor::HEALING_LIMIT
        } else if armor_layout && i + 1 == 14 {
            healing::STATUS_LIMIT
        } else if schema == healing::SCHEMA && i + 1 == 15 {
            healing::LIMIT
        } else if schema == healing::SCHEMA && i + 1 == 14 {
            healing::STATUS_LIMIT
        } else if schema == speed::SCHEMA && i + 1 == speed::SECTION {
            speed::LIMIT
        } else if schema == haste::SCHEMA && i + 1 == haste::SECTION {
            haste::LIMIT
        } else {
            section_limit(i + 1).ok_or_else(invalid)?
        };
        if n > limit {
            return Err(invalid());
        }
        *section = r.take(n)?;
    }
    if r.position != bytes.len() {
        return Err(invalid());
    }
    Ok(UnvalidatedPayload {
        bytes,
        sections,
        schema,
    })
}
fn section_limit(id: usize) -> Option<usize> {
    if id == npc_motion::SECTION {
        Some(npc_motion::LIMIT)
    } else if id == SECTION_COUNT + 1 {
        Some(PRESENTATION_LIMIT)
    } else {
        id.checked_sub(1)
            .and_then(|i| SECTION_LIMITS.get(i).copied())
    }
}
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ApplicationError> {
        let end = self.position.checked_add(n).ok_or_else(invalid)?;
        let value = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16, ApplicationError> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().map_err(|_| invalid())?,
        ))
    }
    fn u32(&mut self) -> Result<u32, ApplicationError> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().map_err(|_| invalid())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(MAGIC.to_be_bytes());
        b.extend(SCHEMA.to_be_bytes());
        b.extend([0; 6]);
        b.extend((SECTION_COUNT as u16).to_be_bytes());
        b.extend([0; 2]);
        for i in 1..=SECTION_COUNT {
            b.extend((i as u16).to_be_bytes());
            b.extend([0; 2]);
            b.extend(1u32.to_be_bytes());
            b.push(i as u8);
        }
        let length = b.len() as u32;
        b[8..12].copy_from_slice(&length.to_be_bytes());
        b
    }
    #[test]
    fn framing_is_borrowed_and_explicitly_does_not_validate_world_semantics() {
        let b = fixture();
        let p = inspect(&b).unwrap();
        assert_eq!(p.bytes(), b);
        assert_eq!(p.section(Section::Personal), &[11]); // Not a valid personal DTO.
        assert_eq!(format!("{p:?}"), "UnvalidatedBattlePayload[redacted]");
        assert_eq!(
            MAX_BYTES,
            HEADER_BYTES + SECTION_COUNT * 8 + SECTION_LIMITS.iter().sum::<usize>()
        );
        assert_eq!(p.section(Section::World).as_ptr(), b[24..].as_ptr());
    }
    #[test]
    fn every_truncation_and_trailing_byte_is_rejected_without_allocating_sections() {
        let b = fixture();
        for n in 0..b.len() {
            assert!(inspect(&b[..n]).is_err(), "{n}");
        }
        let mut extra = b.clone();
        extra.push(0);
        assert!(inspect(&extra).is_err());
        let n = extra.len() as u32;
        extra[8..12].copy_from_slice(&n.to_be_bytes());
        assert!(inspect(&extra).is_err());
    }
    #[test]
    fn section_order_reserved_bits_and_unsigned_lengths_cannot_be_normalized() {
        let b = fixture();
        for index in [0, 4, 6, 7, 12, 14, 15, 16, 18, 19] {
            let mut bad = b.clone();
            bad[index] ^= 1;
            assert!(inspect(&bad).is_err(), "{index}");
        }
        for value in [u32::MAX, (SECTION_LIMITS[0] + 1) as u32] {
            let mut bad = b.clone();
            bad[20..24].copy_from_slice(&value.to_be_bytes());
            assert!(inspect(&bad).is_err());
        }
        let mut duplicate = b;
        duplicate[25..27].copy_from_slice(&1u16.to_be_bytes());
        assert!(inspect(&duplicate).is_err());
    }
}

#[cfg(all(test, windows))]
#[path = "battle_payload_interop_tests.rs"]
mod interop_tests;
