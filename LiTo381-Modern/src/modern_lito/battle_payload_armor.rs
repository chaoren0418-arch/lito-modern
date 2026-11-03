//! Schema8 complete armor values. Candidate and selected data use the same
//! borrowed proof without stripping armor/status/healing. Parsing and attachment
//! are not authentication, temporal admission or permission to render/command.
use super::{
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::*,
    healing::{self, Status},
    invalid, ApplicationError,
};

pub const SCHEMA: u16 = 8;
pub const FEATURE: i32 = 1 << 25;
pub const MAX_EFFECTS: usize = 1024;
pub const STATE_LIMIT: usize = 6 * 8 + 4 + MAX_EFFECTS * (8 + 8 + 1 + 8);
pub const LIMIT: usize = 4 + 100 * (8 + 1 + STATE_LIMIT);
pub const HEALING_LIMIT: usize = 1 + healing::LIMIT;
pub const MAX_BYTES: usize =
    super::MAX_PRESENTATION_BYTES + 8 + healing::STATUS_LIMIT + 8 + HEALING_LIMIT + 8 + LIMIT;
const SECOND: i64 = 1_000_000_000;
const MAX_TIME: i64 = 1800 * SECOND;

#[path = "battle_payload_armor_scene.rs"]
mod scene;
pub use scene::Checked;
#[path = "battle_payload_armor_sequence.rs"]
mod sequence;
pub(super) use sequence::retained_value_events;
pub use sequence::{EffectEvent, EffectId, Follower};

record!(Effect [25] { graphics:i64, graphics_mode:i64, applied_protection:bool, duration_seconds:i64 });
record!(State [STATE_LIMIT] { entity:i64, life:i64, sequence:i64, revision:i64, observed_nanos:i64, until_nanos:i64, effects:List<Effect,MAX_EFFECTS> });
record!(Actor [8 + 1 + STATE_LIMIT] { entity:i64, state:Option<State> });

#[derive(Clone, PartialEq, Eq)]
pub struct Candidate {
    pub data: UnvalidatedData,
    pub status: Status,
    pub healing: Option<List<healing::Actor, 100>>,
    pub actors: List<Actor, 100>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Envelope {
    pub status: Status,
    pub healing: Option<List<healing::Actor, 100>>,
    pub actors: List<Actor, 100>,
    pub presentation: Option<super::armor_presentation::Presentation>,
}
pub(crate) struct View<'a> {
    pub data: &'a UnvalidatedData,
    pub status: &'a Status,
    pub healing: &'a Option<List<healing::Actor, 100>>,
    pub actors: &'a List<Actor, 100>,
    pub presentation: Option<&'a super::armor_presentation::Presentation>,
}
impl Candidate {
    pub(crate) fn view(&self) -> Result<View<'_>, ApplicationError> {
        require(self.data.armor.is_none())?;
        Ok(View {
            data: &self.data,
            status: &self.status,
            healing: &self.healing,
            actors: &self.actors,
            presentation: None,
        })
    }
    pub fn check_armor_values(&self) -> Result<(), ApplicationError> {
        self.view()?.check_armor_values()
    }
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.view()?.check_values(player, world)
    }
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        self.view()?.check_successor(&next.view()?)
    }
    pub fn into_data(self) -> Result<UnvalidatedData, ApplicationError> {
        self.check_armor_values()?;
        let mut data = self.data;
        data.armor = Some(Envelope {
            status: self.status,
            healing: self.healing,
            actors: self.actors,
            presentation: None,
        });
        Ok(data)
    }
}
pub(crate) fn attached(data: &UnvalidatedData) -> Result<View<'_>, ApplicationError> {
    require(
        data.healing.is_none()
            && data.speed.is_none()
            && data.haste.is_none()
            && data.npc_motion.is_none(),
    )?;
    let a = data.armor.as_ref().ok_or_else(invalid)?;
    Ok(View {
        data,
        status: &a.status,
        healing: &a.healing,
        actors: &a.actors,
        presentation: a.presentation.as_ref(),
    })
}
pub(crate) fn follows_attached(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), ApplicationError> {
    attached(a)?.check_successor(&attached(b)?)
}
impl std::fmt::Debug for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnvalidatedArmorBattle[redacted]")
    }
}
pub(super) fn check_base_shape(d: &UnvalidatedData) -> Result<(), ApplicationError> {
    let f = d.world.descriptor.features;
    require(
        f & (FEATURE | 1) == (FEATURE | 1)
            && d.presentation.is_some()
            && d.healing.is_none()
            && d.speed.is_none()
            && d.haste.is_none()
            && d.npc_motion.is_none()
            && (f & super::speed::FEATURE != 0) == d.personal.materials.is_some()
            && (f & healing::FEATURE != 0)
                == d.personal
                    .history
                    .as_ref()
                    .is_some_and(|h| h.0.healing.is_some()),
    )?;
    if f & healing::FEATURE != 0 {
        healing::check_base_shape(d)?;
    }
    for a in &d.actors.0 {
        for (flag, lane) in [
            (1 << 21, Lane::GreaterHaste),
            (1 << 22, Lane::Haste),
            (1 << 23, Lane::Slow),
        ] {
            require(a.spells.histories.0.iter().any(|s| s.lane == lane) == (f & flag != 0))?;
        }
    }
    Ok(())
}
impl State {
    fn shape(&self) -> Result<(), ApplicationError> {
        require(
            self.entity >= 1
                && self.life >= 0
                && self.sequence >= 0
                && self.revision >= 1
                && (0..=MAX_TIME).contains(&self.observed_nanos)
                && self.until_nanos >= 0
                && (self.until_nanos == 0
                    || self.until_nanos > self.observed_nanos
                        && self.until_nanos <= self.observed_nanos + MAX_TIME)
                && self.effects.0.len() <= MAX_EFFECTS
                && (self.sequence != 0
                    || self.observed_nanos == 0
                        && self.until_nanos == 0
                        && self.effects.0.is_empty())
                && (self.until_nanos != 0 || self.effects.0.is_empty()),
        )?;
        let mut grants = 0;
        for e in &self.effects.0 {
            require(
                (0..=65535).contains(&e.graphics)
                    && (0..=2).contains(&e.graphics_mode)
                    && (1..=1800).contains(&e.duration_seconds),
            )?;
            if e.applied_protection {
                grants += 1;
                require(
                    grants == 1
                        && self.until_nanos == self.observed_nanos + e.duration_seconds * SECOND,
                )?;
            }
        }
        Ok(())
    }
}
impl View<'_> {
    /// Local shape and public armor life/clock binding, not full-world admission.
    pub fn check_armor_values(&self) -> Result<(), ApplicationError> {
        check_base_shape(&self.data)?;
        let d = &self.data;
        let f = d.world.descriptor.features;
        require(match &self.status {
            Status::Speed(_) => f & super::speed::FEATURE != 0,
            Status::Haste(_) => f & super::speed::FEATURE == 0 && f & super::haste::FEATURE != 0,
            Status::Motion(m) => {
                f & (super::speed::FEATURE | super::haste::FEATURE) == 0
                    && m.is_some() == d.npcs.is_some()
            }
        })?;
        require(
            self.healing.is_some() == (f & healing::FEATURE != 0)
                && !self.actors.0.is_empty()
                && self.actors.0.len() <= 100
                && self.actors.0.len() == d.actors.0.len()
                && (-1..=1).contains(&d.world.lane)
                && d.room.members.0.len() <= 100
                && d.world.revision >= 1
                && d.world.active_elapsed_nanos >= -1
                && d.world.active_elapsed_nanos <= MAX_TIME,
        )?;
        let visible: Vec<_> = d
            .room
            .members
            .0
            .iter()
            .enumerate()
            .filter(|(_, m)| d.world.lane < 0 || m.team == d.world.lane)
            .collect();
        require(visible.len() == self.actors.0.len())?;
        for ((h, a), (index, member)) in self.actors.0.iter().zip(&d.actors.0).zip(visible) {
            require(
                h.entity == a.entity
                    && h.entity == index as i64 + 1
                    && h.state.is_some() == (member.identity.kind == MemberKind::Human),
            )?;
            let Some(s) = &h.state else { continue };
            s.shape()?;
            let r = a.resources.as_ref().ok_or_else(invalid)?;
            let v = a.vital.as_ref().ok_or_else(invalid)?;
            require(
                s.entity == h.entity
                    && s.life == r.life
                    && s.revision <= d.world.revision
                    && v.values == r.vitals
                    && r.vitals.hp >= 0
                    && s.observed_nanos <= d.world.active_elapsed_nanos.max(0)
                    && (d.world.active_elapsed_nanos >= 0
                        || s.sequence == 0 && s.until_nanos == 0 && s.effects.0.is_empty()),
            )?;
            let participating = d
                .time_attack
                .as_ref()
                .is_none_or(|t| !t.departed.0.contains(&h.entity))
                && d.race.as_ref().is_none_or(|race| {
                    race.runners
                        .0
                        .iter()
                        .any(|r| r.entity == h.entity && r.dnf == Dnf::None && r.placement == 0)
                });
            if s.until_nanos != 0 || !s.effects.0.is_empty() {
                require(
                    d.world.phase == Phase::Active
                        && participating
                        && r.vitals.hp > 0
                        && s.until_nanos > d.world.active_elapsed_nanos,
                )?;
            }
        }
        if let Some(actors) = &self.healing {
            healing::View {
                data: d,
                status: &self.status,
                actors,
            }
            .check_healing_accounts(&d.personal.player, &d.world.identity)?;
        }
        if let Some(presentation) = self.presentation {
            presentation.check_values(self)?;
        }
        Ok(())
    }
}
pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let p = super::inspect_formats(bytes, false, false, false, false, true)?;
    require(p.schema() == SCHEMA)?;
    decode_payload(&p)
}
pub(super) fn decode_payload(
    p: &super::UnvalidatedPayload<'_>,
) -> Result<Candidate, ApplicationError> {
    let data = super::data::decode_payload(&p)?;
    fn read<T: Wire>(bytes: &[u8]) -> Result<T, ApplicationError> {
        let mut r = Reader::new(bytes);
        r.schema = SCHEMA;
        let v = r.read()?;
        r.finish()?;
        Ok(v)
    }
    let c = Candidate {
        data,
        status: read(p.sections[13])?,
        healing: read(p.sections[14])?,
        actors: read(p.sections[15])?,
    };
    c.check_armor_values()?;
    Ok(c)
}
pub fn encode(c: &Candidate) -> Result<Vec<u8>, ApplicationError> {
    encode_view(&c.view()?)
}
pub(crate) fn encode_attached(d: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    if attached(d)?.presentation.is_some() {
        super::armor_presentation::encode(d)
    } else {
        encode_view(&attached(d)?)
    }
}
fn encode_view(c: &View<'_>) -> Result<Vec<u8>, ApplicationError> {
    require(c.presentation.is_none())?;
    encode_common(c)
}
/// Common first sixteen sections. Only the schema9 encoder may append its
/// required typed presentation section; this is not a legacy conversion API.
pub(super) fn encode_common(c: &View<'_>) -> Result<Vec<u8>, ApplicationError> {
    c.check_armor_values()?;
    let mut bytes = super::data::encode_base(&c.data, Some(SCHEMA))?;
    append(&mut bytes, 14, c.status, healing::STATUS_LIMIT)?;
    append(&mut bytes, 15, c.healing, HEALING_LIMIT)?;
    append(&mut bytes, 16, c.actors, LIMIT)?;
    let n = u32::try_from(bytes.len()).map_err(|_| invalid())?;
    bytes[8..12].copy_from_slice(&n.to_be_bytes());
    bytes[12..14].copy_from_slice(&16u16.to_be_bytes());
    Ok(bytes)
}
fn append<T: Wire>(
    bytes: &mut Vec<u8>,
    id: u16,
    value: &T,
    limit: usize,
) -> Result<(), ApplicationError> {
    let mut count = Writer::count();
    count.schema = SCHEMA;
    count.write(value)?;
    require(
        count.pos <= limit
            && count.pos + 8 <= MAX_BYTES
            && bytes.len() <= MAX_BYTES - 8 - count.pos,
    )?;
    let mut out = Writer::allocated(count.pos)?;
    out.schema = SCHEMA;
    out.write(value)?;
    require(out.pos == count.pos)?;
    bytes
        .try_reserve_exact(8 + count.pos)
        .map_err(|_| invalid())?;
    bytes.extend(id.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((count.pos as u32).to_be_bytes());
    bytes.extend(out.bytes.ok_or_else(invalid)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> State {
        State {
            entity: 1,
            life: 0,
            sequence: 1,
            revision: 2,
            observed_nanos: 0,
            until_nanos: 3 * SECOND,
            effects: List(vec![Effect {
                graphics: 11101,
                graphics_mode: 1,
                applied_protection: true,
                duration_seconds: 3,
            }]),
        }
    }
    #[test]
    fn bounds_pin_every_effect_without_truncation_or_graphics_deduplication() {
        let mut s = state();
        let mut later = s.effects.0[0].clone();
        later.applied_protection = false;
        s.effects
            .0
            .extend(std::iter::repeat_n(later.clone(), MAX_EFFECTS - 1));
        s.shape().unwrap();
        let mut w = Writer::allocated(STATE_LIMIT).unwrap();
        w.write(&s).unwrap();
        assert_eq!(w.pos, STATE_LIMIT);
        let bytes = w.bytes.unwrap();
        let mut r = Reader::new(&bytes);
        let decoded: State = r.read().unwrap();
        r.finish().unwrap();
        assert_eq!(s, decoded);
        assert_eq!(decoded.effects.0.len(), 1024);
        assert_eq!(LIMIT, 2566104);
        assert_eq!(MAX_BYTES, healing::MAX_BYTES + 1 + 8 + LIMIT);
        s.effects.0.push(later);
        assert!(s.shape().is_err());
        assert!(Writer::count().write(&s).is_err());
        for count in [-1, 1025, i32::MAX] {
            let mut b = bytes.clone();
            b[48..52].copy_from_slice(&count.to_be_bytes());
            assert!(Reader::new(&b).read::<State>().is_err());
        }
    }
    #[test]
    fn state_rejects_forged_grants_initial_life_overflows_and_invalid_effects() {
        state().shape().unwrap();
        for change in 0..11 {
            let mut s = state();
            match change {
                0 => s.life = -1,
                1 => s.entity = 0,
                2 => s.sequence = 0,
                3 => s.revision = 0,
                4 => s.observed_nanos = i64::MAX,
                5 => s.until_nanos = i64::MAX,
                6 => s.effects.0[0].duration_seconds = 1801,
                7 => s.effects.0[0].graphics = -1,
                8 => s.effects.0[0].graphics_mode = -1,
                9 => s.effects.0.push(s.effects.0[0].clone()),
                _ => s.until_nanos = 0,
            };
            assert!(s.shape().is_err(), "{change}");
        }
        let mut s = state();
        s.effects.0.clear();
        s.shape().unwrap(); // A nonproc hit may retain the lease.
        s.until_nanos = 0;
        s.shape().unwrap(); // Expiry retains its accepted sequence.
    }
    #[test]
    fn explicit_candidate_format_does_not_extend_the_healing_offer() {
        use crate::modern_lito::protocol::extensions as ext;
        assert_eq!(ext::HEALING_MASK, 3);
        assert_eq!(ext::HEALING.supported, 3);
        assert_eq!(ext::HEALING.required, 3);
        assert_eq!(ext::HEALING_ARMOR.required, 7);
        assert_eq!(ext::HEALING.supported & ext::ARMOR_PROTECTION_DATA, 0);
    }
}
