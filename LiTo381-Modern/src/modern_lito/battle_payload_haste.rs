//! Complete schema5, explicitly selected by v44 with CAP_HASTE.
//! Parsing never installs an Observation, grants commands, or falls back to a
//! legacy world with omitted effects. Explicit value/timeline checks are not
//! session, command or live Observation admission.
use super::{
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::{Buffs, Cell, Identity, List, SpellReceipt, UnvalidatedData, WorldId},
    invalid, ApplicationError,
};

pub const SCHEMA: u16 = 5;
pub const VERSION: u16 = 44;
pub const COOPERATIVE_VERSION: u16 = 45;
/// The same schema gains a new mode combination, not implicit v44 support.
pub(crate) fn minimum_version(features: i32) -> u16 {
    if features & super::super::world_protocol::MatchDescriptor::TIME_ATTACK as i32 != 0 {
        COOPERATIVE_VERSION
    } else {
        VERSION
    }
}
#[path = "battle_payload_haste_sequence.rs"]
mod sequence;
pub const FEATURE: i32 = 1 << 22;
pub const SECTION: usize = 14;
pub const LIMIT: usize = 256 * 1024;
pub const MAX_BYTES: usize = super::MAX_PRESENTATION_BYTES + 8 + LIMIT;
pub(super) fn validate_buff(b: &Buffs) -> Result<(), ApplicationError> {
    values::buff(b)
}
record!(Participant [21] { entity: i64, identity: Identity });
record!(NpcId [32] { world: WorldId, serial: i64 });
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Speed {
    pub revision: i64,
    pub observed_millis: i64,
    pub haste_until_millis: i64,
    pub slow_until_millis: i64,
    pub slow_revision: i64,
}
impl Wire for Speed {
    const LIMIT: usize = 40;
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let mut s = Self {
            revision: r.read()?,
            observed_millis: r.read()?,
            haste_until_millis: r.read()?,
            slow_until_millis: 0,
            slow_revision: 0,
        };
        if super::full_speed(r.schema) {
            s.slow_until_millis = r.read()?;
            s.slow_revision = r.read()?;
        }
        Ok(s)
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        require(
            super::full_speed(w.schema) || (self.slow_until_millis == 0 && self.slow_revision == 0),
        )?;
        w.write(&self.revision)?;
        w.write(&self.observed_millis)?;
        w.write(&self.haste_until_millis)?;
        if super::full_speed(w.schema) {
            w.write(&self.slow_until_millis)?;
            w.write(&self.slow_revision)?;
        }
        Ok(())
    }
}
record!(PlayerStatus [183] { participant: Participant, life: i64, equipment_revision: i64, cell: Cell, before: Buffs, after: Buffs });
record!(NpcStatus [140] { id: NpcId, life: i64, lane: i32, cell: Cell, roster_revision: i64, before: Speed, after: Speed });
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatusTarget {
    Player(PlayerStatus),
    Npc(NpcStatus),
}
impl Wire for StatusTarget {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        match r.read::<u8>()? {
            0 => Ok(Self::Player(r.read()?)),
            1 => Ok(Self::Npc(r.read()?)),
            _ => Err(invalid()),
        }
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        match self {
            Self::Player(p) => {
                w.write(&0u8)?;
                w.write(p)
            }
            Self::Npc(n) => {
                w.write(&1u8)?;
                w.write(n)
            }
        }
    }
}
record!(Detail [192] { active_elapsed_nanos: i64, target: StatusTarget });
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Player(Participant),
    Npc(NpcId),
}
impl Wire for Target {
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        match r.read::<u8>()? {
            0 => Ok(Self::Player(r.read()?)),
            1 => Ok(Self::Npc(r.read()?)),
            _ => Err(invalid()),
        }
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        match self {
            Self::Player(p) => {
                w.write(&0u8)?;
                w.write(p)
            }
            Self::Npc(n) => {
                w.write(&1u8)?;
                w.write(n)
            }
        }
    }
}
record!(Grant [512] { caster: Participant, receipt: SpellReceipt });
record!(Entry [1100] { target: Target, active: Option<Grant>, walk: Option<Grant> });
record!(Motion [185] {
    id: NpcId, life: i64, walk_millis: i32, attack_millis: i32,
    speed: Speed, walk_speed: Speed, last_walk: Option<super::npc_motion::Walk>
});
record!(Npcs [44 + 128 * 185] { world: WorldId, revision: i64, elapsed_millis: i64, actors: List<Motion,128> });
record!(State [LIMIT] { world: WorldId, revision: i64, targets: List<Entry,228>, npcs: Option<Npcs> });
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub data: UnvalidatedData,
    pub effects: State,
}
/// Lossless schema5 attachment. Presence is distinct from schema4 motion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub state: State,
}
pub(super) struct View<'a> {
    pub data: &'a UnvalidatedData,
    pub effects: &'a State,
}
impl Candidate {
    fn view(&self) -> Result<View<'_>, ApplicationError> {
        require(
            self.data.speed.is_none()
                && self.data.healing.is_none()
                && self.data.haste.is_none()
                && self.data.npc_motion.is_none(),
        )?;
        Ok(View {
            data: &self.data,
            effects: &self.effects,
        })
    }
    pub fn check_values(
        &self,
        player: &super::data::Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.view()?.check_values(player, world)
    }
    pub fn check_effects(&self) -> Result<(), ApplicationError> {
        self.view()?.check_effects()
    }
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        crate::modern_lito::battle_follow::check_haste_candidate_sequence(self, next)?;
        self.view()?.check_successor(&next.view()?)
    }
}
pub(super) fn attached(data: &UnvalidatedData) -> Result<View<'_>, ApplicationError> {
    require(data.healing.is_none() && data.speed.is_none() && data.npc_motion.is_none())?;
    Ok(View {
        data,
        effects: &data.haste.as_ref().ok_or_else(invalid)?.state,
    })
}
pub(crate) fn follows_attached(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), ApplicationError> {
    match (&a.haste, &b.haste) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) => attached(a)?.check_successor(&attached(b)?),
        _ => Err(invalid()),
    }
}
#[path = "battle_payload_haste_values.rs"]
mod values;

pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let p = super::inspect_candidate(bytes, false, true)?;
    require(p.schema() == SCHEMA)?;
    let data = super::data::decode_payload(&p)?;
    let mut r = Reader::new(p.sections[SECTION - 1]);
    r.schema = SCHEMA;
    let effects = r.read()?;
    r.finish()?;
    Ok(Candidate { data, effects })
}
pub fn encode(candidate: &Candidate) -> Result<Vec<u8>, ApplicationError> {
    encode_view(candidate.view()?)
}
pub(super) fn encode_attached(data: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    encode_view(attached(data)?)
}
fn encode_view(candidate: View<'_>) -> Result<Vec<u8>, ApplicationError> {
    require(candidate.data.presentation.is_some() && candidate.data.npc_motion.is_none())?;
    let mut bytes = super::data::encode_base(candidate.data, Some(SCHEMA))?;
    let mut count = Writer::count();
    count.schema = SCHEMA;
    count.write(candidate.effects)?;
    require(count.pos <= LIMIT)?;
    let mut section = Writer::allocated(count.pos)?;
    section.schema = SCHEMA;
    section.write(candidate.effects)?;
    let extra = 8 + count.pos;
    require(section.pos == count.pos && bytes.len() <= MAX_BYTES - extra)?;
    bytes.try_reserve_exact(extra).map_err(|_| invalid())?;
    bytes.extend((SECTION as u16).to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((count.pos as u32).to_be_bytes());
    bytes.extend(section.bytes.ok_or_else(invalid)?);
    let n = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&n.to_be_bytes());
    bytes[12..14].copy_from_slice(&(SECTION as u16).to_be_bytes());
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::battle_payload::{self as payload, data};
    // Deliberately syntax-only: this fixture has no combat authority. Merely
    // adding a feature bit must never make it a checked or playable world.
    fn syntax() -> Candidate {
        let mut data = payload::data_tests::fixture();
        data.world.descriptor.features |= FEATURE;
        data.presentation = Some(List(vec![]));
        let effects = State {
            world: data.world.identity.clone(),
            revision: data.world.revision,
            targets: List(vec![]),
            npcs: None,
        };
        Candidate { data, effects }
    }
    #[test]
    fn syntax_only_candidate_cannot_pass_full_values_or_temporal_proof() {
        let p = syntax();
        assert!(p
            .check_values(&p.data.personal.player, &p.data.world.identity)
            .is_err());
        assert!(p.check_successor(&p).is_err());
    }
    #[test]
    fn syntax_roundtrip_has_no_legacy_or_scene_admission() {
        let p = syntax();
        let bytes = encode(&p).unwrap();
        assert_eq!(decode(&bytes).unwrap(), p);
        for v in (1..=43).chain([crate::modern_lito::protocol::MAX_VERSION + 1, u16::MAX]) {
            assert!(payload::decode_versioned(&bytes, v).is_err());
        }
        let decoded = payload::decode_versioned(&bytes, 44).unwrap();
        assert_eq!(payload::decode_versioned(&bytes, 45).unwrap(), decoded);
        assert!(decoded
            .check_scene_values(&decoded.personal.player, &decoded.world.identity)
            .is_err());
        assert!(payload::decode(&bytes).is_err());
        assert!(payload::encode(&p.data).is_err());
        assert!(p
            .data
            .check_scene_values(&p.data.personal.player, &p.data.world.identity)
            .is_err());
    }
    #[test]
    fn every_truncation_reserved_header_and_unknown_section_is_rejected() {
        let bytes = encode(&syntax()).unwrap();
        for n in 0..bytes.len() {
            assert!(decode(&bytes[..n]).is_err());
        }
        for offset in [0, 6, 7, 14, 15, 16, 17, 18, 19] {
            let mut b = bytes.clone();
            b[offset] ^= 128;
            assert!(decode(&b).is_err());
        }
        let mut b = bytes.clone();
        b[20..24].copy_from_slice(&i32::MAX.to_be_bytes());
        assert!(decode(&b).is_err());
        let mut b = bytes.clone();
        b.push(0);
        assert!(decode(&b).is_err());
    }
    #[test]
    fn new_lane_and_origin_are_context_gated_and_existing_tags_do_not_shift() {
        assert_eq!(data::Lane::GreaterHaste as u8, 8);
        assert_eq!(data::Lane::Haste as u8, 9);
        for schema in [1, 2, 3] {
            let mut r = Reader::new(&[9]);
            r.schema = schema;
            assert!(r.read::<data::Lane>().is_err());
            let mut w = Writer::count();
            w.schema = schema;
            assert!(w.write(&data::Lane::Haste).is_err());
        }
        let mut r = Reader::new(&[9]);
        r.schema = SCHEMA;
        assert_eq!(r.read::<data::Lane>().unwrap(), data::Lane::Haste);
        let b = data::Buffs {
            entity: 1,
            revision: 2,
            observed_nanos: 0,
            haste_until_nanos: 1_200_000_000_000,
            brave_until_nanos: 0,
            wisdom_until_nanos: 0,
            brave_kind: 0,
            slow_until_nanos: 0,
            slow_revision: 0,
            haste_origin: data::HasteOrigin::Haste,
        };
        for schema in [1, 2, 3] {
            let mut w = Writer::count();
            w.schema = schema;
            assert!(w.write(&b).is_err());
        }
        let mut w = Writer::allocated(53).unwrap();
        w.schema = SCHEMA;
        w.write(&b).unwrap();
        let bytes = w.bytes.unwrap();
        let mut r = Reader::new(&bytes);
        r.schema = 3;
        assert!(r.read::<data::Buffs>().is_err());
        let mut r = Reader::new(&bytes);
        r.schema = SCHEMA;
        assert_eq!(r.read::<data::Buffs>().unwrap(), b);
    }
    #[test]
    fn world_feature_and_complete_extension_are_not_optional_fallbacks() {
        let mut p = syntax();
        p.data.world.descriptor.features = 0;
        assert!(encode(&p).is_err());
        let mut p = syntax();
        p.data.presentation = None;
        assert!(encode(&p).is_err());
        let mut p = syntax();
        p.data.npc_motion = Some(super::super::npc_motion::Envelope { state: None });
        assert!(encode(&p).is_err());
        let bytes = encode(&syntax()).unwrap();
        let mut missing = bytes[..bytes.len() - 45].to_vec();
        let n = missing.len() as u32;
        missing[8..12].copy_from_slice(&n.to_be_bytes());
        missing[12..14].copy_from_slice(&13u16.to_be_bytes());
        assert!(decode(&missing).is_err());
    }
    #[test]
    fn typed_npc_serial_preserves_full_i64_and_rejects_unknown_target_tags() {
        let id = NpcId {
            world: syntax().effects.world,
            serial: i64::MAX,
        };
        let t = Target::Npc(id);
        let mut w = Writer::allocated(33).unwrap();
        w.schema = SCHEMA;
        w.write(&t).unwrap();
        let bytes = w.bytes.unwrap();
        let mut r = Reader::new(&bytes);
        r.schema = SCHEMA;
        assert_eq!(r.read::<Target>().unwrap(), t);
        r.finish().unwrap();
        for tag in [2, 127, 255] {
            let mut b = bytes.clone();
            b[0] = tag;
            let mut r = Reader::new(&b);
            r.schema = SCHEMA;
            assert!(r.read::<Target>().is_err());
        }
    }
}
