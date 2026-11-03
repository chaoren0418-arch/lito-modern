//! Lossless schema6, explicitly negotiated by v46 CAP_SLOW_DATA.
//! Parsing never grants commands or rendering; atomic admission is separate.
pub use super::haste::{Entry, Grant, Npcs, Participant, Speed, StatusTarget, Target};
use super::{
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::{Buffs, List, Player, UnvalidatedData, WorldId},
    invalid, ApplicationError,
};
#[path = "battle_payload_speed_effects.rs"]
mod effects;
#[path = "battle_payload_speed_sequence.rs"]
mod sequence;
#[path = "battle_payload_speed_values.rs"]
mod values;
pub const SCHEMA: u16 = 6;
pub const VERSION: u16 = 46;
pub const FEATURE: i32 = 1 << 23;
pub const SECTION: usize = 14;
pub const LIMIT: usize = 512 * 1024;
pub const MAX_BYTES: usize = super::MAX_PRESENTATION_BYTES + 8 + LIMIT;
record!(MissingEffect [8] { npc_template:i32, required_skill:i32 });
record!(SlowDetail [256] { active_elapsed_nanos:i64, probability:i32, roll:i32, equipment_haste_items:i64, missing_effect:Option<MissingEffect>, target:StatusTarget });
record!(Neutralized [146] { active_elapsed_nanos:i64, before:Buffs, after:Buffs });
record!(Stock [32] { item:i64, unit_weight:i64, initial:i64, remaining:i64 });
record!(Materials [256] { world:WorldId, player:Player, caster:Participant, revision:i64, uses:i64, last_sequence:i64, last_life:i64, last_equipment_revision:i64, stocks:List<Stock,1> });
record!(State [LIMIT] { world:WorldId, revision:i64, targets:List<Entry,228>, npcs:Option<Npcs> });
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub data: UnvalidatedData,
    pub effects: State,
}
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
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.view()?.check_values(player, world)
    }
    pub fn check_components(&self) -> Result<(), ApplicationError> {
        self.view()?.check_components()
    }
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        crate::modern_lito::battle_follow::check_speed_candidate_sequence(self, next)?;
        self.view()?.check_successor(&next.view()?)
    }
}
pub(super) fn attached(data: &UnvalidatedData) -> Result<View<'_>, ApplicationError> {
    require(data.healing.is_none() && data.haste.is_none() && data.npc_motion.is_none())?;
    Ok(View {
        data,
        effects: &data.speed.as_ref().ok_or_else(invalid)?.state,
    })
}
pub(crate) fn follows_attached(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), ApplicationError> {
    match (&a.speed, &b.speed) {
        (None, None) => Ok(()),
        (Some(_), Some(_)) => attached(a)?.check_successor(&attached(b)?),
        _ => Err(invalid()),
    }
}
pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let p = super::inspect_extended(bytes, false, false, true)?;
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
    require(
        candidate.data.presentation.is_some()
            && candidate.data.npc_motion.is_none()
            && candidate.data.haste.is_none(),
    )?;
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

pub(super) fn validate_buff(b: &Buffs) -> Result<(), ApplicationError> {
    values::buff(b)
}
pub(super) fn validate_receipt(r: &super::data::SpellReceipt) -> Result<(), ApplicationError> {
    values::receipt(r)
}
