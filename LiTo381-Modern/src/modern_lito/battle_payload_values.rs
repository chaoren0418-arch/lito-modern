//! Exact value conversions into independent accounting validators. No legacy
//! World/Combat frame, synthetic revision, foreign epoch or UI state is made.
use super::data::*;
use crate::modern_lito::{
    equipment_transition as gear, npc_protocol as npc, participant, quest_protocol, skill_spending,
    target_history as target, time_attack_protocol, use_history as usage, vital_balance,
    world_map::WorldCell,
};

impl Manifest {
    pub(super) fn contains_cell(&self, cell: &Cell) -> bool {
        self.sectors.0.iter().any(|s| {
            // Match the shared Lineage sector origin without narrowing or
            // adding to untrusted i32 coordinates.
            let x = 32704 + (i64::from(s.x) - 0x7fff) * 64;
            let y = 32704 + (i64::from(s.y) - 0x7fff) * 64;
            (x..x + 64).contains(&i64::from(cell.x)) && (y..y + 64).contains(&i64::from(cell.y))
        })
    }
}

impl From<&Vitals> for gear::Vitals {
    fn from(v: &Vitals) -> Self {
        Self {
            hp: v.hp,
            maximum_hp: v.maximum_hp,
            mp: v.mp,
            maximum_mp: v.maximum_mp,
        }
    }
}
impl From<&Resources> for vital_balance::State {
    fn from(v: &Resources) -> Self {
        let f = &v.flows;
        Self {
            revision: v.revision,
            equipment_revision: v.equipment_revision,
            life: v.life,
            birth_hp: v.birth_hp,
            birth_mp: v.birth_mp,
            vitals: (&v.vitals).into(),
            flows: vital_balance::Flows {
                hp_gain: f.hp_gain,
                hp_loss: f.hp_loss,
                restored: f.restored,
                damaged: f.damaged,
                hp_granted: f.hp_granted,
                mp_gain: f.mp_gain,
                mp_loss: f.mp_loss,
                spent: f.spent,
                mp_granted: f.mp_granted,
                hp_clipped: f.hp_clipped,
                mp_clipped: f.mp_clipped,
            },
        }
    }
}
impl From<&Anchor> for usage::Anchor {
    fn from(v: &Anchor) -> Self {
        Self {
            equipment_revision: v.equipment_revision,
            life: v.life,
            vitals: (&v.vitals).into(),
            x: v.cell.x,
            y: v.cell.y,
        }
    }
}
impl From<&Cast> for usage::Cast {
    fn from(v: &Cast) -> Self {
        Self {
            revision: v.revision,
            sequence: v.sequence,
            before: (&v.before).into(),
            cost: v.cost,
            mp_after: v.mp_after,
        }
    }
}
impl From<&Spell> for usage::Spell {
    fn from(v: &Spell) -> Self {
        let l = &v.ledger;
        Self {
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
            last: v.last.as_ref().map(Into::into),
        }
    }
}
impl From<&Items> for usage::Items {
    fn from(v: &Items) -> Self {
        Self {
            equipment_revision: v.equipment_revision,
            profile_revision: v.profile_revision,
            fixed_recovery: v.fixed_recovery,
            uses: v.uses,
            restored: v.restored,
            last: v.last.as_ref().map(|l| usage::Use {
                revision: l.revision,
                sequence: l.sequence,
                item_id: l.item,
                before: (&l.before).into(),
                fixed_recovery: l.fixed_recovery,
                rolled_base: l.rolled_base,
                restored: l.restored,
                hp_after: l.hp_after,
            }),
        }
    }
}
impl From<Lane> for target::Lane {
    fn from(v: Lane) -> Self {
        match v {
            Lane::Teleport => Self::Teleport,
            Lane::LightArrow => Self::LightArrow,
            Lane::Healing => Self::Healing,
            Lane::FireStorm => Self::FireStorm,
            Lane::Disintegrate => Self::Disintegrate,
            Lane::NpcLightArrow => Self::NpcLightArrow,
            Lane::NpcFireStorm => Self::NpcFireStorm,
            Lane::NpcDisintegrate => Self::NpcDisintegrate,
            Lane::GreaterHaste => Self::GreaterHaste,
            Lane::Haste => Self::Haste,
            Lane::Slow => Self::Slow,
        }
    }
}
fn scope(v: &WorldId) -> Option<npc::Scope> {
    Some(npc::Scope {
        room_id: v.room.try_into().ok()?,
        match_id: v.match_id.try_into().ok()?,
        generation: v.generation.try_into().ok()?,
    })
}
pub(super) fn effect(v: &Effect) -> Option<target::Effect> {
    let target = match &v.target {
        Target::Participant { entity, identity } => {
            target::Target::Participant(target::Participant {
                entity_id: (*entity).try_into().ok()?,
                identity: participant::ParticipantId {
                    kind: match identity.kind {
                        MemberKind::Human => participant::MemberKind::Human,
                        MemberKind::ServerBot => participant::MemberKind::ServerBot,
                    },
                    source_id: identity.source.try_into().ok()?,
                    id: identity.id.try_into().ok()?,
                },
            })
        }
        Target::Npc { scope: s, serial } => target::Target::Npc(npc::Id {
            scope: scope(s)?,
            serial: (*serial).try_into().ok()?,
        }),
        Target::Honor { entity, team } => target::Target::Honor {
            entity_id: (*entity).try_into().ok()?,
            team: (*team).try_into().ok()?,
        },
        Target::Quest { id, born_revision } => target::Target::Quest {
            id: quest_protocol::Id {
                scope: scope(&id.scope)?,
                slot: id.slot.try_into().ok()?,
            },
            born_revision: *born_revision,
        },
        Target::TimeTower(s) => {
            target::Target::TimeTower(time_attack_protocol::TowerId { scope: scope(s)? })
        }
    };
    Some(target::Effect {
        kind: match v.kind {
            EffectKind::Damage => target::Kind::Damage,
            EffectKind::Heal => target::Kind::Heal,
        },
        before: target::Health {
            target,
            life: v.life,
            equipment_revision: v.equipment_revision,
            cell: WorldCell {
                x: v.cell.x,
                y: v.cell.y,
            },
            hp: v.hp,
            maximum_hp: v.maximum_hp,
        },
        raw: v.raw,
        applied: v.applied,
        hp_after: v.hp_after,
    })
}
