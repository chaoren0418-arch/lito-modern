//! Schema9: all sixteen armor sections plus one owner-only typed section.
//! Parsing/semantic proof is not asset qualification, rendering or LOAD.
use super::{
    armor,
    codec::{record, tags, Reader, Wire, Writer},
    combat::require,
    data::*,
    invalid, ApplicationError,
};

pub const SCHEMA: u16 = 9;
pub const GRAPHICS_LIMIT: usize = armor::MAX_EFFECTS + 1;
pub const MAX_VISUALS: usize = 2 * armor::MAX_EFFECTS + 1;
pub const BATCH_LIMIT: usize = 3 * 8 + 8 + 2 * 4 + 4 + MAX_VISUALS * 9;
pub const LIMIT: usize = 7 * 8 + 4 + GRAPHICS_LIMIT * 4 + 4 + 4 + 1 + BATCH_LIMIT;
pub const MAX_BYTES: usize = armor::MAX_BYTES + 8 + LIMIT;
pub const STATUS_ICON: i32 = 1562;
pub const SKILL_GRAPHIC: i32 = 228;

tags!(Role { ProcTarget=0, ProcActor=1, SkillCast=2 });
record!(Anchor [16] { move_sequence:i64, cell:Cell });
record!(Visual [9] { ordinal:i32, graphic:i32, role:Role });
record!(Batch [BATCH_LIMIT] { sequence:i64, revision:i64, observed_nanos:i64, anchor:Anchor, visuals:List<Visual,MAX_VISUALS> });
record!(Requirements [4+GRAPHICS_LIMIT*4+4+4] { graphics:List<i32,GRAPHICS_LIMIT>, icons:List<i32,1> });
record!(Presentation [LIMIT] { world:WorldId, world_revision:i64, entity:i64, life:i64, equipment_revision:i64, required:Requirements, batch:Option<Batch> });

impl Requirements {
    fn check(&self) -> Result<(), ApplicationError> {
        let g = &self.graphics.0;
        let icons = &self.icons.0;
        require(
            g.len() <= GRAPHICS_LIMIT
                && g.iter().all(|id| (1..=65535).contains(id))
                && g.windows(2).all(|v| v[0] < v[1])
                && (icons.is_empty() || icons.as_slice() == [STATUS_ICON])
                && g.is_empty() == icons.is_empty()
                && (g.is_empty() || g.contains(&SKILL_GRAPHIC)),
        )
    }
}
impl Presentation {
    /// Bound to the SAME complete armor envelope, not the last actor position
    /// or a separately fetched equipment/DB row. The source promises resource
    /// completeness; a client still has to load and qualify every requirement.
    pub(super) fn check_values(&self, view: &armor::View<'_>) -> Result<(), ApplicationError> {
        let d = view.data;
        let actor = d.scene_actor(d.personal.entity)?;
        let resources = actor.resources.as_ref().ok_or_else(invalid)?;
        let equipment = d.personal.equipment.as_ref().ok_or_else(invalid)?;
        let state = view
            .actors
            .0
            .iter()
            .find(|a| a.entity == d.personal.entity)
            .and_then(|a| a.state.as_ref())
            .ok_or_else(invalid)?;
        self.required.check()?;
        require(
            self.world == d.world.identity
                && self.world_revision == d.world.revision
                && self.entity == d.personal.entity
                && self.life == state.life
                && self.life == resources.life
                && self.equipment_revision == resources.equipment_revision
                && self.equipment_revision == equipment.revision
                && self.batch.is_some() == !state.effects.0.is_empty()
                && (state.until_nanos == 0 || self.required.icons.0.as_slice() == [STATUS_ICON]),
        )?;
        let Some(batch) = &self.batch else {
            return Ok(());
        };
        require(
            batch.sequence == state.sequence
                && batch.revision == state.revision
                && batch.observed_nanos == state.observed_nanos
                && batch.anchor.move_sequence >= 0
                && batch.anchor.move_sequence <= actor.last_move_sequence
                && d.manifest.contains_cell(&batch.anchor.cell)
                && (batch.revision != d.world.revision || batch.anchor.cell == actor.position)
                && batch.visuals.0.len() <= MAX_VISUALS,
        )?;
        // Packet multiplicity matters: mode2 sends twice, and siblings with
        // the same GFX remain separate. Only required asset IDs are deduped.
        let mut expected = Vec::new();
        expected
            .try_reserve_exact(MAX_VISUALS)
            .map_err(|_| invalid())?;
        for effect in &state.effects.0 {
            let graphic = i32::try_from(effect.graphics).map_err(|_| invalid())?;
            if graphic > 0 {
                if effect.graphics_mode != 1 {
                    expected.push((graphic, Role::ProcTarget));
                }
                if effect.graphics_mode != 0 {
                    expected.push((graphic, Role::ProcActor));
                }
            }
            if effect.applied_protection {
                expected.push((SKILL_GRAPHIC, Role::SkillCast));
            }
        }
        require(expected.len() == batch.visuals.0.len())?;
        for (index, (visual, (graphic, role))) in batch.visuals.0.iter().zip(expected).enumerate() {
            require(
                visual.ordinal == index as i32
                    && visual.graphic == graphic
                    && visual.role == role
                    && self.required.graphics.0.contains(&graphic),
            )?;
        }
        Ok(())
    }
}

pub(super) fn check_successor(
    a: &armor::View<'_>,
    b: &armor::View<'_>,
) -> Result<(), ApplicationError> {
    match (a.presentation, b.presentation) {
        (None, None) => Ok(()),
        (Some(old), Some(new)) => {
            // The admitted gear set/catalog is frozen for this match. A later
            // SET cannot smuggle new assets past initial resource qualification.
            require(old.required == new.required)?;
            if a.data.world.revision == b.data.world.revision
                || matches!(a.data.world.phase, Phase::Aborted | Phase::Finished)
            {
                return require(old == new);
            }
            if let Some(next) = &new.batch {
                if let Some(prior) = &old.batch {
                    if old.life == new.life
                        && (prior.revision == next.revision || prior.sequence == next.sequence)
                    {
                        return require(prior == next);
                    }
                }
                require(
                    next.revision > a.data.world.revision
                        && next.anchor.move_sequence
                            >= a.data.scene_actor(old.entity)?.last_move_sequence,
                )?;
            }
            Ok(())
        }
        _ => Err(invalid()), // Negotiated envelopes cannot appear/disappear mid-match.
    }
}

pub fn decode(bytes: &[u8]) -> Result<UnvalidatedData, ApplicationError> {
    let p = super::inspect_with_presentation(bytes, false, false, false, false, true, true)?;
    require(p.schema() == SCHEMA)?;
    let base = armor::decode_payload(&p)?;
    let mut reader = Reader::new(p.sections[16]);
    reader.schema = SCHEMA;
    let presentation = reader.read::<Presentation>()?;
    reader.finish()?;
    let mut data = base.into_data()?;
    data.armor.as_mut().ok_or_else(invalid)?.presentation = Some(presentation);
    armor::attached(&data)?.check_armor_values()?;
    Ok(data)
}
pub fn encode(data: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    let view = armor::attached(data)?;
    let presentation = view.presentation.ok_or_else(invalid)?;
    view.check_armor_values()?;
    let mut count = Writer::count();
    count.schema = SCHEMA;
    count.write(presentation)?;
    require(count.pos <= LIMIT)?;
    let mut bytes = armor::encode_common(&view)?;
    require(bytes.len() <= MAX_BYTES - 8 - count.pos)?;
    let mut out = Writer::allocated(count.pos)?;
    out.schema = SCHEMA;
    out.write(presentation)?;
    require(out.pos == count.pos)?;
    bytes
        .try_reserve_exact(8 + count.pos)
        .map_err(|_| invalid())?;
    bytes.extend(17u16.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((count.pos as u32).to_be_bytes());
    bytes.extend(out.bytes.ok_or_else(invalid)?);
    let size = u32::try_from(bytes.len()).map_err(|_| invalid())?;
    bytes[4..6].copy_from_slice(&SCHEMA.to_be_bytes());
    bytes[8..12].copy_from_slice(&size.to_be_bytes());
    bytes[12..14].copy_from_slice(&17u16.to_be_bytes());
    Ok(bytes)
}
