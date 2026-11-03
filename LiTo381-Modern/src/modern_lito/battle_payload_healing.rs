//! Complete schema7 values, shared by candidate and selected-format admission.
//! All ordinary sections remain present. Attached observations still require
//! explicit session extensions; values alone grant no commands or rendering.
use super::{
    codec::{record, Reader, Wire, Writer},
    combat::require,
    data::*,
    invalid, ApplicationError,
};
use crate::modern_lito::{
    participant, player_healing as heal, player_healing_history as history, skill_spending,
    target_history as target, use_history, vital_balance,
};

pub const SCHEMA: u16 = 7;
pub const FEATURE: i32 = 1 << 24;
pub const LIMIT: usize = 4 + 100 * 1024;
pub const STATUS_LIMIT: usize = 1 + super::speed::LIMIT;
pub const MAX_BYTES: usize = super::MAX_PRESENTATION_BYTES + 8 + STATUS_LIMIT + 8 + LIMIT;
#[path = "battle_payload_healing_scene.rs"]
mod scene;
pub use scene::Checked;
#[path = "battle_payload_healing_sequence.rs"]
mod sequence;
pub use super::haste::Participant;
pub use sequence::Follower;
record!(Impact {
    world: WorldId,
    caster: Participant,
    usage: Cast,
    target_effect_revision: i64,
    effects: u8,
    base_amount: i64,
    adjusted_amount: i64,
    target: Effect
});
record!(Incoming {world:WorldId, recipient:Participant, received:i64, restored:i64, last:Option<Impact>});
record!(Receipt {
    impact: Impact,
    casts: i64,
    spent: i64,
    loading_elapsed_nanos: i64
});
record!(Outgoing {ledger:Ledger, receipt:Option<Receipt>});
record!(Actor [1024] {entity:i64, incoming:Incoming, outgoing:Option<Outgoing>});

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Motion(Option<super::npc_motion::State>),
    Haste(super::haste::State),
    Speed(super::speed::State),
}
impl Wire for Status {
    const LIMIT: usize = STATUS_LIMIT;
    fn read(r: &mut Reader<'_>) -> Result<Self, ApplicationError> {
        let schema = r.schema;
        let value = match r.read::<u8>()? {
            0 => Self::Motion(r.read()?),
            1 => {
                r.schema = super::haste::SCHEMA;
                Self::Haste(r.read()?)
            }
            2 => {
                r.schema = super::speed::SCHEMA;
                Self::Speed(r.read()?)
            }
            _ => return Err(invalid()),
        };
        r.schema = schema;
        Ok(value)
    }
    fn write(&self, w: &mut Writer) -> Result<(), ApplicationError> {
        let schema = w.schema;
        match self {
            Self::Motion(s) => {
                w.write(&0u8)?;
                w.write(s)?;
            }
            Self::Haste(s) => {
                w.write(&1u8)?;
                w.schema = super::haste::SCHEMA;
                w.write(s)?;
            }
            Self::Speed(s) => {
                w.write(&2u8)?;
                w.schema = super::speed::SCHEMA;
                w.write(s)?;
            }
        };
        w.schema = schema;
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Candidate {
    pub data: UnvalidatedData,
    pub status: Status,
    pub actors: List<Actor, 100>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub status: Status,
    pub actors: List<Actor, 100>,
}
/// Borrow the complete received data without cloning or deleting any field.
pub(crate) struct View<'a> {
    pub data: &'a UnvalidatedData,
    pub status: &'a Status,
    pub actors: &'a List<Actor, 100>,
}
impl Candidate {
    pub(crate) fn view(&self) -> Result<View<'_>, ApplicationError> {
        require(self.data.healing.is_none() && self.data.armor.is_none())?;
        Ok(View {
            data: &self.data,
            status: &self.status,
            actors: &self.actors,
        })
    }
    pub fn check_healing_accounts(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.view()?.check_healing_accounts(player, world)
    }
    fn shape(&self) -> Result<(), ApplicationError> {
        self.view()?.shape()
    }
    pub fn into_data(self) -> Result<UnvalidatedData, ApplicationError> {
        self.shape()?;
        let mut data = self.data;
        data.healing = Some(Envelope {
            status: self.status,
            actors: self.actors,
        });
        Ok(data)
    }
}
pub(crate) fn attached(data: &UnvalidatedData) -> Result<View<'_>, ApplicationError> {
    require(data.npc_motion.is_none() && data.haste.is_none() && data.speed.is_none())?;
    if let Some(a) = &data.armor {
        require(data.healing.is_none())?;
        return Ok(View {
            data,
            status: &a.status,
            actors: a.healing.as_ref().ok_or_else(invalid)?,
        });
    }
    let h = data.healing.as_ref().ok_or_else(invalid)?;
    Ok(View {
        data,
        status: &h.status,
        actors: &h.actors,
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
        f.write_str("UnvalidatedHealingBattle[redacted]")
    }
}
pub(super) fn check_base_shape(d: &UnvalidatedData) -> Result<(), ApplicationError> {
    let f = d.world.descriptor.features;
    require(
        f & FEATURE != 0
            && f & (1 | 2 | 8 | 16) == (1 | 2 | 8 | 16)
            && d.presentation.is_some()
            && d.personal
                .history
                .as_ref()
                .is_some_and(|h| h.0.healing.is_some())
            && (f & super::speed::FEATURE != 0) == d.personal.materials.is_some()
            && d.npc_motion.is_none()
            && d.haste.is_none()
            && d.speed.is_none(),
    )?;
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
impl View<'_> {
    fn shape(&self) -> Result<(), ApplicationError> {
        check_base_shape(&self.data)?;
        let f = self.data.world.descriptor.features;
        require(match &self.status {
            Status::Speed(_) => f & super::speed::FEATURE != 0,
            Status::Haste(_) => f & super::speed::FEATURE == 0 && f & super::haste::FEATURE != 0,
            Status::Motion(m) => {
                f & (super::speed::FEATURE | super::haste::FEATURE) == 0
                    && m.is_some() == self.data.npcs.is_some()
            }
        })?;
        require(!self.actors.0.is_empty() && self.actors.0.len() == self.data.actors.0.len())?;
        require(self.actors.0.windows(2).all(|p| p[0].entity < p[1].entity))?;
        for (h, a) in self.actors.0.iter().zip(&self.data.actors.0) {
            require(h.entity == a.entity)?;
        }
        Ok(())
    }
    /// Exact public/private healing and resource accounts, not full-world admission.
    pub fn check_healing_accounts(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        self.shape()?;
        let d = &self.data;
        require(&d.personal.player == player && &d.world.identity == world)?;
        require(
            (-1..=1).contains(&d.world.lane)
                && self.actors.0.len()
                    == d.room
                        .members
                        .0
                        .iter()
                        .filter(|m| d.world.lane < 0 || m.team == d.world.lane)
                        .count(),
        )?;
        let owner = &d.personal.history.as_ref().ok_or_else(invalid)?.0;
        require(
            owner.valid()
                && owner.world_revision == d.world.revision
                && owner.identity.world == world_value(world)?
                && i64::try_from(owner.identity.actor.entity_id).ok() == Some(d.personal.entity)
                && i32::try_from(owner.identity.source.key.source_id).ok() == Some(player.source)
                && i64::try_from(owner.identity.source.key.object_id).ok() == Some(player.object)
                && owner.identity.source.epoch == player.epoch as u64
                && i32::from(owner.identity.channel_id) == player.channel,
        )?;
        let visible = |p: target::Participant| {
            self.actors
                .0
                .iter()
                .any(|a| participant_value(&a.incoming.recipient).ok() == Some(p))
        };
        for (h, a) in self.actors.0.iter().zip(&d.actors.0) {
            let incoming = h.incoming.value()?;
            let member = d
                .room
                .members
                .0
                .get(
                    usize::try_from(h.entity.checked_sub(1).ok_or_else(invalid)?)
                        .map_err(|_| invalid())?,
                )
                .ok_or_else(invalid)?;
            require(
                incoming.world == world_value(world)?
                    && incoming.recipient.entity_id == h.entity as u64
                    && h.incoming.recipient.identity == member.identity
                    && (d.world.lane < 0 || d.world.lane == member.team)
                    && (member.identity.kind == MemberKind::Human) == h.outgoing.is_some(),
            )?;
            let resources: vital_balance::State = a.resources.as_ref().ok_or_else(invalid)?.into();
            require(resources.valid() && resources.revision <= d.world.revision)?;
            if let Some(r) = incoming.last {
                require(
                    visible(r.caster)
                        && visible(r.recipient().ok_or_else(invalid)?)
                        && r.use_receipt.revision <= resources.revision
                        && r.target.before.life
                            <= resources.life.checked_add(1).ok_or_else(invalid)?
                        && r.target.before.equipment_revision <= resources.equipment_revision
                        && (r.target.before.equipment_revision != resources.equipment_revision
                            || r.target.before.maximum_hp == resources.vitals.maximum_hp),
                )?;
            }
            let mut spent = 0i64;
            for s in &a.spells.histories.0 {
                require(use_history::Spell::from(s).valid())?;
                spent = spent.checked_add(s.ledger.spent).ok_or_else(invalid)?;
            }
            let mut sent = None;
            if let Some(o) = &h.outgoing {
                let spending = o.spell();
                require(
                    spending.valid()
                        && spending.current.cost <= 4
                        && bounded(spending.current.spent, spending.current.casts, 4)
                        && bounded(
                            spending.current.prior_spent,
                            spending.current.prior_casts,
                            4,
                        )
                        && spending.current.equipment_revision == resources.equipment_revision
                        && spending.current.profile_revision <= resources.revision,
                )?;
                spent = spent
                    .checked_add(spending.current.spent)
                    .ok_or_else(invalid)?;
                if let Some(r) = &o.receipt {
                    let impact = r.impact.value()?;
                    sent = Some(impact);
                    let anchor = impact.use_receipt.before;
                    require(
                        impact.world == incoming.world
                            && impact.caster == incoming.recipient
                            && visible(impact.recipient().ok_or_else(invalid)?)
                            && r.casts == spending.current.casts
                            && r.spent == spending.current.spent
                            && r.loading_elapsed_nanos >= 0
                            && r.loading_elapsed_nanos <= d.world.loading_elapsed_nanos
                            && impact.use_receipt.sequence <= a.spells.sequence
                            && impact.use_receipt.revision <= a.spells.request_revision
                            && impact.use_receipt.revision <= resources.revision
                            && anchor.life <= resources.life.checked_add(1).ok_or_else(invalid)?
                            && anchor.equipment_revision <= resources.equipment_revision
                            && (anchor.equipment_revision != resources.equipment_revision
                                || anchor.vitals.maximum_hp == resources.vitals.maximum_hp
                                    && anchor.vitals.maximum_mp == resources.vitals.maximum_mp),
                    )?;
                    let mut casts = r.casts;
                    for s in &a.spells.histories.0 {
                        if let Some(old) = &s.receipt {
                            casts = casts.checked_add(old.casts).ok_or_else(invalid)?;
                            let u = &old.impact.usage;
                            require(
                                u.sequence != impact.use_receipt.sequence
                                    && u.revision != impact.use_receipt.revision
                                    && (u.sequence < impact.use_receipt.sequence)
                                        == (u.revision < impact.use_receipt.revision)
                                    && if u.sequence < impact.use_receipt.sequence {
                                        old.loading_elapsed_nanos <= r.loading_elapsed_nanos
                                    } else {
                                        old.loading_elapsed_nanos >= r.loading_elapsed_nanos
                                    },
                            )?;
                        }
                    }
                    require(casts <= a.spells.sequence)?;
                }
            }
            if let Some(r) = incoming.last.filter(|r| r.is_self()) {
                let s = sent.ok_or_else(invalid)?;
                require(
                    s.use_receipt.revision >= r.use_receipt.revision
                        && if s.use_receipt.revision == r.use_receipt.revision {
                            s == r
                        } else {
                            s.use_receipt.sequence > r.use_receipt.sequence
                        },
                )?;
            }
            if let Some(s) = sent.filter(|r| r.is_self()) {
                let r = incoming.last.ok_or_else(invalid)?;
                require(
                    r.use_receipt.revision >= s.use_receipt.revision
                        && (r.use_receipt.revision != s.use_receipt.revision || r == s),
                )?;
            }
            require(
                spent == resources.flows.spent && incoming.restored <= resources.flows.restored,
            )?;
            if let Some(regen) = &a.regeneration {
                require(
                    regen.item_gain.checked_add(incoming.restored)
                        == Some(resources.flows.restored),
                )?;
            }
            if h.entity == d.personal.entity {
                let private = owner.healing.ok_or_else(invalid)?;
                require(
                    owner.resources == resources
                        && owner.identity.actor == incoming.recipient
                        && private.incoming == incoming,
                )?;
                match (private.outgoing, &h.outgoing) {
                    (Some(p), Some(o)) => require(
                        p.spending == o.spell()
                            && p.last.map(|r| r.impact())
                                == o.receipt.as_ref().map(|r| r.impact.value()).transpose()?,
                    )?,
                    (None, None) => {}
                    _ => return Err(invalid()),
                }
            }
        }
        require(
            self.actors
                .0
                .iter()
                .filter(|a| a.entity == d.personal.entity)
                .count()
                == 1,
        )
    }
}
fn bounded(total: i64, count: i64, max: i64) -> bool {
    total >= 0 && count >= 0 && (total / max < count || total / max == count && total % max == 0)
}
fn world_value(w: &WorldId) -> Result<target::World, ApplicationError> {
    let v = target::World {
        room_id: w.room.try_into().map_err(|_| invalid())?,
        match_id: w.match_id.try_into().map_err(|_| invalid())?,
        generation: w.generation.try_into().map_err(|_| invalid())?,
    };
    require(v.valid())?;
    Ok(v)
}
fn participant_value(p: &Participant) -> Result<target::Participant, ApplicationError> {
    let v = target::Participant {
        entity_id: p.entity.try_into().map_err(|_| invalid())?,
        identity: participant::ParticipantId {
            kind: match p.identity.kind {
                MemberKind::Human => participant::MemberKind::Human,
                MemberKind::ServerBot => participant::MemberKind::ServerBot,
            },
            source_id: p.identity.source.try_into().map_err(|_| invalid())?,
            id: p.identity.id.try_into().map_err(|_| invalid())?,
        },
    };
    require(v.valid())?;
    Ok(v)
}
impl Impact {
    fn value(&self) -> Result<heal::Impact, ApplicationError> {
        require(self.effects <= 15)?;
        let v = heal::Impact {
            world: world_value(&self.world)?,
            caster: participant_value(&self.caster)?,
            use_receipt: (&self.usage).into(),
            target_effect_revision: self.target_effect_revision,
            effects: heal::Effects {
                water_life: self.effects & 1 != 0,
                pollute_water: self.effects & 2 != 0,
                dirty_current: self.effects & 4 != 0,
                healing_erosion: self.effects & 8 != 0,
            },
            base_amount: self.base_amount,
            adjusted_amount: self.adjusted_amount,
            target: super::values::effect(&self.target).ok_or_else(invalid)?,
        };
        require(v.valid())?;
        Ok(v)
    }
}
impl Incoming {
    fn value(&self) -> Result<history::Incoming, ApplicationError> {
        let v = history::Incoming {
            world: world_value(&self.world)?,
            recipient: participant_value(&self.recipient)?,
            received: self.received,
            restored: self.restored,
            last: self.last.as_ref().map(Impact::value).transpose()?,
        };
        require(v.valid())?;
        Ok(v)
    }
}
impl Outgoing {
    fn spell(&self) -> use_history::Spell {
        let l = &self.ledger;
        use_history::Spell {
            current: skill_spending::Ledger {
                equipment_revision: l.equipment_revision,
                profile_revision: l.profile_revision,
                cost: l.cost,
                prior_casts: l.prior_casts,
                prior_spent: l.prior_spent,
                casts: l.casts,
                spent: l.spent,
                last_cast_revision: l.last_cast_revision,
                last_cast_sequence: l.last_cast_sequence,
            },
            last: self.receipt.as_ref().map(|r| (&r.impact.usage).into()),
        }
    }
}
pub fn decode(bytes: &[u8]) -> Result<Candidate, ApplicationError> {
    let p = super::inspect_all(bytes, false, false, false, true)?;
    require(p.schema() == SCHEMA)?;
    let data = super::data::decode_payload(&p)?;
    let mut r = Reader::new(p.sections[13]);
    r.schema = SCHEMA;
    let status = r.read()?;
    r.finish()?;
    let mut r = Reader::new(p.sections[14]);
    r.schema = SCHEMA;
    let actors = r.read()?;
    r.finish()?;
    let c = Candidate {
        data,
        status,
        actors,
    };
    c.shape()?;
    Ok(c)
}
pub fn encode(c: &Candidate) -> Result<Vec<u8>, ApplicationError> {
    encode_view(c.view()?)
}
pub(super) fn encode_attached(data: &UnvalidatedData) -> Result<Vec<u8>, ApplicationError> {
    encode_view(attached(data)?)
}
fn encode_view(c: View<'_>) -> Result<Vec<u8>, ApplicationError> {
    c.shape()?;
    let mut bytes = super::data::encode_base(&c.data, Some(SCHEMA))?;
    append(&mut bytes, 14, c.status, STATUS_LIMIT)?;
    append(&mut bytes, 15, c.actors, LIMIT)?;
    let len = bytes.len() as u32;
    bytes[8..12].copy_from_slice(&len.to_be_bytes());
    bytes[12..14].copy_from_slice(&15u16.to_be_bytes());
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
    require(count.pos <= limit)?;
    let mut out = Writer::allocated(count.pos)?;
    out.schema = SCHEMA;
    out.write(value)?;
    require(out.pos == count.pos && bytes.len() <= MAX_BYTES - 8 - count.pos)?;
    bytes
        .try_reserve_exact(8 + count.pos)
        .map_err(|_| invalid())?;
    bytes.extend(id.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend((count.pos as u32).to_be_bytes());
    bytes.extend(out.bytes.ok_or_else(invalid)?);
    Ok(())
}
