//! Complete schema10/11 data. Common field layouts are shared with schema8,
//! never its capability/admission. The VIP bit and every selected section are
//! retained. Candidate scene/following proofs never grant live observation,
//! authenticated transport, resource admission or commands.
use super::{
    armor, armor_presentation,
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::*,
    healing, invalid, ApplicationError,
};
use std::collections::BTreeSet;

#[path = "battle_payload_vip_scene.rs"]
mod scene;
pub use scene::Checked;
#[path = "battle_payload_vip_sequence.rs"]
mod sequence;
pub use sequence::Follower;

pub const SCHEMA: u16 = 10;
pub const APPEARANCE_SCHEMA: u16 = 11;
pub const APPEARANCE_SECTION: usize = 19;
pub const APPEARANCE_LIMIT: usize = 4 + 100 * (6 * 8 + 4);
pub const FEATURE: i32 = 1 << 26;
pub const SECTION: usize = 18;
pub const FIELD_NAMES: [&str; 31] = [
    "STR",
    "DEX",
    "CON",
    "INT",
    "WIS",
    "CHA",
    "AC",
    "MAX_HP",
    "MAX_MP",
    "HPR",
    "MPR",
    "MR",
    "SP",
    "MELEE_DAMAGE",
    "MELEE_HIT",
    "RANGED_DAMAGE",
    "RANGED_HIT",
    "DAMAGE_REDUCTION",
    "MAGIC_REDUCTION",
    "FIRE",
    "WIND",
    "EARTH",
    "WATER",
    "STUN",
    "STONE",
    "SLEEP",
    "FREEZE",
    "SUSTAIN",
    "BLIND",
    "EXP_RATE",
    "ADENA_RATE",
];
pub const ITEM_LIMIT: usize = 8 + 4 * 8 + 31 * 8 + 8 + 1 + 8;
pub const LIMIT: usize = 3 * 8
    + 8
    + 24
    + 8
    + 8
    + 3 * 8
    + 4
    + 512 * ITEM_LIMIT
    + 31 * 8
    + 4
    + 512 * 8
    + 1
    + 8
    + 3
    + 31 * 8;
pub const PRESENTATION_LIMIT: usize = 1 + armor_presentation::LIMIT;
pub const MAX_BYTES: usize = armor::MAX_BYTES + 8 + PRESENTATION_LIMIT + 8 + LIMIT;
pub const MAX_APPEARANCE_BYTES: usize = MAX_BYTES + 8 + APPEARANCE_LIMIT;

#[derive(Clone, PartialEq, Eq)]
pub struct Abilities(pub [i64; 31]);
impl std::fmt::Debug for Abilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipAbilities[redacted]")
    }
}
impl Wire for Abilities {
    const LIMIT: usize = 31 * 8;
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let mut values = [0; 31];
        for v in &mut values {
            *v = r.read()?;
        }
        Ok(Self(values))
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        for v in &self.0 {
            w.write(v)?;
        }
        Ok(())
    }
}
record!(Template [4*8+31*8] { catalog:i64, exclusive_type:i64, icon:i64, polymorph:i64, abilities:Abilities });
record!(Item [ITEM_LIMIT] { object:i64, applied:Template, lifetime:Lifetime });
record!(Shape [8+3+31*8] { graphics:i64, present:bool, no_death_exp_loss:bool, retains_on_cancellation:bool, abilities:Abilities });
record!(Versions [24] { saved:i64, runtime:i64, lifecycle:i64 });
// graphics=0 is no VIP override, not a sprite-zero load request. Public base
// class/sex and weapon presentation remain authoritative in the common data.
record!(Appearance [52] { entity:i64, source:i32, object:i64, life:i64, runtime:i64, lifecycle:i64, graphics:i64 });
record!(Owner [LIMIT] { world:WorldId, world_revision:i64, player:Player, entity:i64, life:i64, versions:Versions,
    items:List<Item,512>, abilities:Abilities, icons:List<i64,512>, shape:Option<Shape> });

#[derive(Clone, PartialEq, Eq)]
pub struct Candidate {
    pub appearances: Option<List<Appearance, 100>>,
    pub data: UnvalidatedData,
    pub status: healing::Status,
    pub healing: Option<List<healing::Actor, 100>>,
    pub armor: List<armor::Actor, 100>,
    pub presentation: Option<armor_presentation::Presentation>,
    pub owner: Owner,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Envelope {
    pub appearances: Option<List<Appearance, 100>>,
    pub status: healing::Status,
    pub healing: Option<List<healing::Actor, 100>>,
    pub armor: List<armor::Actor, 100>,
    pub presentation: Option<armor_presentation::Presentation>,
    pub owner: Owner,
}
impl std::fmt::Debug for Envelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipEnvelope[redacted]")
    }
}
pub(crate) struct View<'a> {
    pub appearances: Option<&'a List<Appearance, 100>>,
    pub data: &'a UnvalidatedData,
    pub status: &'a healing::Status,
    pub healing: &'a Option<List<healing::Actor, 100>>,
    pub armor: &'a List<armor::Actor, 100>,
    pub presentation: Option<&'a armor_presentation::Presentation>,
    pub owner: &'a Owner,
}
impl std::fmt::Debug for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UnvalidatedVipBattle[redacted]")
    }
}

impl Candidate {
    pub(crate) fn view(&self) -> Result<View<'_>, ApplicationError> {
        require(self.data.vip.is_none())?;
        Ok(View {
            appearances: self.appearances.as_ref(),
            data: &self.data,
            status: &self.status,
            healing: &self.healing,
            armor: &self.armor,
            presentation: self.presentation.as_ref(),
            owner: &self.owner,
        })
    }
    pub fn check_binding(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.view()?.check_binding(player, world)
    }
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.view()?.check_values(player, world)
    }
    pub fn check_successor(&self, next: &Self) -> Result<(), ApplicationError> {
        self.view()?.check_successor(&next.view()?)
    }
    pub fn into_data(self) -> Result<UnvalidatedData, ApplicationError> {
        self.check_binding(&self.owner.player, &self.owner.world)?;
        let mut data = self.data;
        data.vip = Some(Envelope {
            appearances: self.appearances,
            status: self.status,
            healing: self.healing,
            armor: self.armor,
            presentation: self.presentation,
            owner: self.owner,
        });
        Ok(data)
    }
}
pub(crate) fn attached(data: &UnvalidatedData) -> Result<View<'_>, ApplicationError> {
    let v = data.vip.as_ref().ok_or_else(invalid)?;
    Ok(View {
        appearances: v.appearances.as_ref(),
        data,
        status: &v.status,
        healing: &v.healing,
        armor: &v.armor,
        presentation: v.presentation.as_ref(),
        owner: &v.owner,
    })
}
pub(crate) fn follows_attached(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), ApplicationError> {
    attached(a)?.check_successor(&attached(b)?)
}
impl View<'_> {
    /// Local version/owner/ledger binding only. NOT full scene semantics,
    /// authentication, temporal following, asset loading or an SQL receipt.
    pub fn check_binding(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        let d = &self.data;
        let o = &self.owner;
        let f = d.world.descriptor.features;
        let allowed = FEATURE
            | 255
            | 512
            | (1 << 18)
            | (1 << 21)
            | (1 << 22)
            | (1 << 23)
            | healing::FEATURE
            | armor::FEATURE;
        require(
            f & (FEATURE | 1) == FEATURE | 1
                && f & !allowed == 0
                && matches!(
                    d.world.descriptor.ruleset,
                    Ruleset::PvpSkirmish | Ruleset::DeathmatchElimination
                )
                && d.room.mode == 1
                && d.world.lane == -1
                && d.presentation.is_some()
                && d.armor.is_none()
                && d.healing.is_none()
                && d.haste.is_none()
                && d.speed.is_none()
                && d.npc_motion.is_none()
                && d.npcs.is_none()
                && d.quest.is_none()
                && d.race.is_none()
                && d.siege.is_none()
                && d.time_attack.is_none()
                && d.honor.objectives.0.is_empty()
                && d.honor.lives.0.is_empty()
                && &o.player == player
                && &d.personal.player == player
                && &o.world == world
                && &d.world.identity == world
                && o.entity == d.personal.entity
                && o.world_revision == d.world.revision
                && o.world_revision >= 1
                && o.versions.saved >= 0
                && o.versions.runtime >= o.versions.saved
                && o.versions.lifecycle >= 0
                && (2..=100).contains(&d.actors.0.len())
                && d.actors.0.len() == d.room.members.0.len()
                && self.armor.0.len() == d.actors.0.len()
                && (f & armor::FEATURE != 0) == self.presentation.is_some()
                && (f & healing::FEATURE != 0) == self.healing.is_some()
                && (f & super::speed::FEATURE != 0) == d.personal.materials.is_some(),
        )?;
        for (flag, needs) in [
            (2, 1),
            (4, 2),
            (8, 1),
            (16, 8),
            (32, 16),
            (64, 32),
            (128, 16),
            (512, 128 | 4),
            (1 << 18, 128 | 4),
            (1 << 21, 32 | 2),
            (1 << 22, 32 | 2),
            (1 << 23, 32 | 2),
            (healing::FEATURE, 2 | 16),
        ] {
            require(f & flag == 0 || f & needs == needs)?;
        }
        require(match &self.status {
            healing::Status::Speed(_) => f & super::speed::FEATURE != 0,
            healing::Status::Haste(_) => {
                f & super::speed::FEATURE == 0 && f & super::haste::FEATURE != 0
            }
            healing::Status::Motion(m) => {
                m.is_none() && f & (super::speed::FEATURE | super::haste::FEATURE) == 0
            }
        })?;
        let e = d.personal.equipment.as_ref().ok_or_else(invalid)?;
        let h = &d.personal.history.as_ref().ok_or_else(invalid)?.0;
        let mut identities = BTreeSet::new();
        for (index, (a, m)) in d.actors.0.iter().zip(&d.room.members.0).enumerate() {
            require(
                a.entity == index as i64 + 1
                    && m.identity.kind == MemberKind::Human
                    && m.identity.source > 0
                    && m.identity.id > 0
                    && identities.insert((m.identity.source, m.identity.id))
                    && self.armor.0[index].entity == a.entity
                    && self.armor.0[index].state.is_some() == (f & armor::FEATURE != 0),
            )?;
            for (flag, lane) in [
                (1 << 21, Lane::GreaterHaste),
                (1 << 22, Lane::Haste),
                (1 << 23, Lane::Slow),
            ] {
                require(a.spells.histories.0.iter().any(|s| s.lane == lane) == (f & flag != 0))?;
            }
        }
        let index =
            usize::try_from(o.entity.checked_sub(1).ok_or_else(invalid)?).map_err(|_| invalid())?;
        let actor = d.actors.0.get(index).ok_or_else(invalid)?;
        let member = d.room.members.0.get(index).ok_or_else(invalid)?;
        if let Some(appearances) = self.appearances {
            require(appearances.0.len() == d.actors.0.len())?;
            for ((p, a), m) in appearances.0.iter().zip(&d.actors.0).zip(&d.room.members.0) {
                let r = a.resources.as_ref().ok_or_else(invalid)?;
                require(
                    p.entity == a.entity
                        && p.source == m.identity.source
                        && p.object == m.identity.id
                        && p.life == r.life
                        && p.runtime == r.equipment_revision
                        && p.life >= 0
                        && p.runtime >= 0
                        && p.lifecycle >= 0
                        && p.graphics >= 0,
                )?;
            }
            let p = &appearances.0[index];
            require(
                p.lifecycle == o.versions.lifecycle
                    && p.graphics == o.shape.as_ref().map_or(0, |s| s.graphics),
            )?;
        }
        let r = actor.resources.as_ref().ok_or_else(invalid)?;
        require(
            member.identity.source == player.source
                && member.identity.id == player.object
                && player.epoch != 0
                && (0..=3).contains(&player.channel)
                && e.source == player.source
                && e.object == player.object
                && e.revision == o.versions.runtime
                && r.equipment_revision == o.versions.runtime
                && r.life == o.life
                && o.life >= 0
                && h.resources.equipment_revision == o.versions.runtime
                && h.resources.life == o.life
                && h.valid()
                && h.world_revision == o.world_revision
                && i64::try_from(h.identity.world.room_id).ok() == Some(world.room)
                && i64::try_from(h.identity.world.match_id).ok() == Some(world.match_id)
                && i64::try_from(h.identity.world.generation).ok() == Some(world.generation)
                && i64::try_from(h.identity.actor.entity_id).ok() == Some(o.entity)
                && i32::try_from(h.identity.source.key.source_id).ok() == Some(player.source)
                && i64::try_from(h.identity.source.key.object_id).ok() == Some(player.object)
                && h.identity.source.epoch == player.epoch as u64
                && i32::from(h.identity.channel_id) == player.channel
                && h.healing.is_some() == self.healing.is_some()
                && e.evaluated_at_epoch_millis >= 0
                && (e.valid_until_epoch_millis == -1
                    || e.valid_until_epoch_millis > e.evaluated_at_epoch_millis),
        )?;
        let mut total = [0i64; 31];
        let mut icons = BTreeSet::new();
        let mut types = BTreeSet::new();
        let mut previous = 0;
        for item in &o.items.0 {
            let t = &item.applied;
            let l = &item.lifetime;
            require(
                item.object > previous
                    && l.object == item.object
                    && t.catalog > 0
                    && t.exclusive_type >= 0
                    && types.insert(t.exclusive_type)
                    && t.icon >= 0
                    && t.polymorph >= 0
                    && e.inventory
                        .0
                        .iter()
                        .any(|i| i.object == item.object && i.catalog == t.catalog)
                    && e.equipped
                        .0
                        .iter()
                        .any(|i| i.object == item.object && i.equipped)
                    && e.lifetimes.0.iter().any(|i| i == l)
                    && match l.state {
                        LifetimeState::Permanent => l.expires_at_millis == -1,
                        LifetimeState::Active => l.expires_at_millis > e.evaluated_at_epoch_millis,
                        _ => false,
                    },
            )?;
            previous = item.object;
            for (sum, value) in total.iter_mut().zip(&t.abilities.0) {
                *sum = sum.checked_add(*value).ok_or_else(invalid)?;
            }
            if t.icon != 0 {
                icons.insert(t.icon);
            }
        }
        if let Some(s) = &o.shape {
            require(
                s.graphics > 0
                    && o.items.0.iter().any(|i| i.applied.polymorph == s.graphics)
                    && (s.present
                        || !s.no_death_exp_loss
                            && !s.retains_on_cancellation
                            && s.abilities.0 == [0; 31]),
            )?;
            for (sum, value) in total.iter_mut().zip(&s.abilities.0) {
                *sum = sum.checked_add(*value).ok_or_else(invalid)?;
            }
        }
        require(total == o.abilities.0 && icons.into_iter().eq(o.icons.0.iter().copied()))?;
        if f & armor::FEATURE != 0 {
            armor::View {
                data: d,
                status: &self.status,
                healing: &self.healing,
                actors: &self.armor,
                presentation: self.presentation,
            }
            .check_armor_values()?;
        } else if let Some(actors) = &self.healing {
            healing::View {
                data: d,
                status: &self.status,
                actors,
            }
            .check_healing_accounts(player, world)?;
        }
        Ok(())
    }
}

fn section_limit(id: usize) -> Result<usize, ApplicationError> {
    Ok(match id {
        14 => healing::STATUS_LIMIT,
        15 => armor::HEALING_LIMIT,
        16 => armor::LIMIT,
        17 => PRESENTATION_LIMIT,
        18 => LIMIT,
        19 => APPEARANCE_LIMIT,
        _ => super::section_limit(id).ok_or_else(invalid)?,
    })
}
fn inspect(bytes: &[u8]) -> Result<super::UnvalidatedPayload<'_>, ApplicationError> {
    let appearance = bytes.get(4..6) == Some(APPEARANCE_SCHEMA.to_be_bytes().as_slice());
    let schema = if appearance {
        APPEARANCE_SCHEMA
    } else {
        SCHEMA
    };
    let count = if appearance {
        APPEARANCE_SECTION
    } else {
        SECTION
    };
    let maximum = if appearance {
        MAX_APPEARANCE_BYTES
    } else {
        MAX_BYTES
    };
    require((super::HEADER_BYTES + count * 8..=maximum).contains(&bytes.len()))?;
    let mut r = super::Cursor { bytes, position: 0 };
    require(
        r.u32()? == super::MAGIC
            && r.u16()? == schema
            && r.u16()? == 0
            && r.u32()? as usize == bytes.len()
            && r.u16()? as usize == count
            && r.u16()? == 0,
    )?;
    let mut sections = [&[][..]; APPEARANCE_SECTION];
    for (i, s) in sections.iter_mut().enumerate().take(count) {
        require(r.u16()? as usize == i + 1 && r.u16()? == 0)?;
        let n = r.u32()? as usize;
        require(n <= section_limit(i + 1)?)?;
        *s = r.take(n)?;
    }
    require(r.position == bytes.len())?;
    Ok(super::UnvalidatedPayload {
        bytes,
        sections,
        schema,
    })
}
fn read<T: Wire>(bytes: &[u8]) -> Result<T, ApplicationError> {
    let mut r = Reader::new(bytes);
    r.schema = armor::SCHEMA;
    let v = r.read()?;
    r.finish()?;
    Ok(v)
}
pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let p = inspect(bytes)?;
    let c = Candidate {
        appearances: if p.schema() == APPEARANCE_SCHEMA {
            Some(read(p.sections[18])?)
        } else {
            None
        },
        data: super::data::decode_sections(&p)?,
        status: read(p.sections[13])?,
        healing: read(p.sections[14])?,
        armor: read(p.sections[15])?,
        presentation: read(p.sections[16])?,
        owner: read(p.sections[17])?,
    };
    c.check_binding(&c.data.personal.player, &c.data.world.identity)?;
    Ok(c)
}
fn append<T: Wire>(out: &mut Vec<u8>, id: usize, v: &T) -> Result<(), ApplicationError> {
    let mut count = Writer::count();
    count.schema = armor::SCHEMA;
    count.write(v)?;
    require(count.pos <= section_limit(id)?)?;
    let total = out
        .len()
        .checked_add(8)
        .and_then(|n| n.checked_add(count.pos))
        .ok_or_else(invalid)?;
    require(total <= MAX_APPEARANCE_BYTES)?;
    let mut w = Writer::allocated(count.pos)?;
    w.schema = armor::SCHEMA;
    w.write(v)?;
    require(w.pos == count.pos)?;
    out.try_reserve_exact(8 + count.pos)
        .map_err(|_| invalid())?;
    out.extend_from_slice(&(id as u16).to_be_bytes());
    out.extend_from_slice(&[0; 2]);
    out.extend_from_slice(&(count.pos as u32).to_be_bytes());
    out.extend_from_slice(&w.bytes.ok_or_else(invalid)?);
    Ok(())
}
pub fn encode(c: &Candidate) -> Result<Vec<u8>, ApplicationError> {
    encode_view(&c.view()?)
}
pub(super) fn encode_attached(data: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    encode_view(&attached(data)?)
}
fn encode_view(c: &View<'_>) -> Result<Vec<u8>, ApplicationError> {
    c.check_binding(&c.data.personal.player, &c.data.world.identity)?;
    fn core(w: &mut Writer, d: &UnvalidatedData, appearance: bool) -> Result<(), ApplicationError> {
        w.schema = armor::SCHEMA;
        w.put(&super::MAGIC.to_be_bytes())?;
        w.put(
            &(if appearance {
                APPEARANCE_SCHEMA
            } else {
                SCHEMA
            })
            .to_be_bytes(),
        )?;
        w.put(&[0; 6])?;
        w.put(
            &((if appearance {
                APPEARANCE_SECTION
            } else {
                SECTION
            }) as u16)
                .to_be_bytes(),
        )?;
        w.put(&[0; 2])?;
        super::data::write_sections(w, d)
    }
    let mut count = Writer::count();
    core(&mut count, &c.data, c.appearances.is_some())?;
    let mut out = Writer::allocated(count.pos)?;
    core(&mut out, &c.data, c.appearances.is_some())?;
    require(count.pos == out.pos)?;
    let mut bytes = out.bytes.ok_or_else(invalid)?;
    append(&mut bytes, 14, c.status)?;
    append(&mut bytes, 15, c.healing)?;
    append(&mut bytes, 16, c.armor)?;
    // Option<&T> has no wire implementation; keep the exact original value.
    append(&mut bytes, 17, &c.presentation.cloned())?;
    append(&mut bytes, 18, c.owner)?;
    if let Some(appearances) = c.appearances {
        append(&mut bytes, APPEARANCE_SECTION, appearances)?;
    }
    let n = u32::try_from(bytes.len()).map_err(|_| invalid())?;
    bytes[8..12].copy_from_slice(&n.to_be_bytes());
    Ok(bytes)
}

#[cfg(test)]
#[path = "battle_payload_vip_tests.rs"]
mod tests;
