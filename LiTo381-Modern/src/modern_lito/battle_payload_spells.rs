//! Visible spell receipts omit private source epochs. Validate their public
//! arithmetic directly instead of fabricating a foreign owner's legacy frame.
use super::{combat::require, data::*, invalid, values, ApplicationError};
use crate::modern_lito::{use_history, world_protocol::MatchDescriptor as Features};

impl UnvalidatedData {
    pub(super) fn spell_values(&self, a: &Actor) -> Result<(), ApplicationError> {
        require(self.world.descriptor.features & super::healing::FEATURE == 0)?;
        self.spell_values_with_healing(a, None)
    }
    pub(super) fn spell_values_with_healing(
        &self,
        a: &Actor,
        healing: Option<&super::healing::Actor>,
    ) -> Result<(), ApplicationError> {
        let v = &a.spells;
        require(
            v.sequence >= 0
                && v.request_revision >= 0
                && v.request_revision <= self.world.revision
                && (v.sequence == 0) == (v.request_revision == 0)
                && (v.sequence == 0 || v.sequence < v.request_revision),
        )?;
        let enabled = |f: u32| (self.world.descriptor.features as u32) & f == f;
        let expected = [
            (Lane::Teleport, Features::SPELLS),
            (Lane::LightArrow, Features::LIGHT_ARROW),
            (Lane::Healing, Features::HONOR),
            (Lane::FireStorm, Features::FIRE_STORM),
            (Lane::Disintegrate, Features::DISINTEGRATE),
            (Lane::NpcLightArrow, Features::NPC_LIGHT_ARROW),
            (Lane::NpcFireStorm, Features::NPC_FIRE_STORM),
            (Lane::NpcDisintegrate, Features::NPC_DISINTEGRATE),
            (Lane::GreaterHaste, Features::GREATER_HASTE),
            (Lane::Haste, Features::HASTE_CANDIDATE),
            (Lane::Slow, Features::SLOW_CANDIDATE),
        ]
        .into_iter()
        .filter(|(_, f)| enabled(*f))
        .map(|(l, _)| l)
        .collect::<Vec<_>>();
        require(
            expected.len() == v.histories.0.len()
                && expected
                    .iter()
                    .zip(&v.histories.0)
                    .all(|(l, s)| *l == s.lane),
        )?;
        require(if enabled(Features::SPELLS) {
            (10_000_000..=10_000_000_000).contains(&a.cast_interval_nanos)
                && a.cast_interval_nanos % 1_000_000 == 0
        } else {
            a.cast_interval_nanos == 0
        })?;
        let mut count = 0i128;
        let mut spent = 0i128;
        let mut lawful_count = 0i128;
        let mut ordered = Vec::new();
        let mut profile = None;
        for s in &v.histories.0 {
            require(use_history::Spell::from(s).valid())?;
            let resources = a.resources.as_ref().ok_or_else(invalid)?;
            let l = &s.ledger;
            require(
                l.equipment_revision == resources.equipment_revision
                    && l.profile_revision <= resources.revision
                    && profile.is_none_or(|p| p == l.profile_revision)
                    && s.last.is_none() == s.receipt.is_none(),
            )?;
            profile = Some(l.profile_revision);
            count += i128::from(l.casts);
            spent += i128::from(l.spent);
            if let Some(r) = &s.receipt {
                let c = &r.impact.usage;
                require(
                    r.impact.lane == s.lane
                        && s.last.as_ref() == Some(c)
                        && r.casts == l.casts
                        && r.spent == l.spent
                        && r.casts >= 1
                        && c.sequence <= v.sequence
                        && c.revision <= v.request_revision
                        && c.revision <= resources.revision
                        && c.before.life <= resources.life + 1
                        && (c.before.equipment_revision != resources.equipment_revision
                            || c.before.vitals.maximum_hp == resources.vitals.maximum_hp
                                && c.before.vitals.maximum_mp == resources.vitals.maximum_mp)
                        && (0..=self.world.loading_elapsed_nanos)
                            .contains(&r.loading_elapsed_nanos),
                )?;
                impact_values(&r.impact, a.entity)?;
                self.impact_cells(&r.impact)?;
                detail_values(r)?;
                // Bind retained timestamps even after the effect was overwritten
                // or cleared. A valid duration alone does not prove event time.
                if let Detail::GreaterHaste {
                    active_elapsed_nanos,
                    ..
                } = r.detail
                {
                    require(
                        active_elapsed_nanos <= self.world.active_elapsed_nanos
                            && r.loading_elapsed_nanos.checked_sub(active_elapsed_nanos)
                                == self
                                    .world
                                    .loading_elapsed_nanos
                                    .checked_sub(self.world.active_elapsed_nanos),
                    )?;
                }
                if let Detail::GreaterHasteNeutralized(n) = &r.detail {
                    require(
                        n.before.entity == a.entity
                            && n.after.entity == a.entity
                            && n.active_elapsed_nanos <= self.world.active_elapsed_nanos
                            && r.loading_elapsed_nanos.checked_sub(n.active_elapsed_nanos)
                                == self
                                    .world
                                    .loading_elapsed_nanos
                                    .checked_sub(self.world.active_elapsed_nanos),
                    )?;
                    if c.revision == self.world.revision {
                        require(a.buffs.as_ref() == Some(&n.after))?;
                    }
                }
                if let Detail::Teleport { destination } = &r.detail {
                    require(self.manifest.contains_cell(destination))?;
                }
                if let Detail::Disintegrate {
                    lawful_before,
                    lawful_after,
                    ..
                } = r.detail
                {
                    require(lawful_before >= v.initial_lawful && lawful_after <= v.lawful)?;
                    lawful_count += i128::from(r.casts);
                }
                ordered.push(r);
            }
        }
        if let Some(h) = healing.and_then(|h| h.outgoing.as_ref()) {
            count += i128::from(h.ledger.casts);
            spent += i128::from(h.ledger.spent);
            require(profile.is_some_and(|p| p == h.ledger.profile_revision))?;
        }
        require(
            count <= i128::from(v.sequence)
                && i128::from(v.lawful) == i128::from(v.initial_lawful) + lawful_count
                && spent
                    == a.resources
                        .as_ref()
                        .map_or(0, |r| i128::from(r.flows.spent)),
        )?;
        ordered.sort_by_key(|r| r.impact.usage.sequence);
        for pair in ordered.windows(2) {
            require(
                pair[0].impact.usage.sequence < pair[1].impact.usage.sequence
                    && pair[0].impact.usage.revision < pair[1].impact.usage.revision
                    && pair[0].loading_elapsed_nanos <= pair[1].loading_elapsed_nanos,
            )?;
        }
        if let Some(lawful) = ordered.iter().rev().find_map(|r| match r.detail {
            Detail::Disintegrate { lawful_after, .. } => Some(lawful_after),
            _ => None,
        }) {
            require(lawful == v.lawful)?;
        }
        structure_values(v, a.entity)?;
        if let Some(s) = &v.structure {
            self.impact_cells(&s.impact)?;
        }
        Ok(())
    }
    fn impact_cells(&self, i: &Impact) -> Result<(), ApplicationError> {
        require(
            self.manifest.contains_cell(&i.usage.before.cell)
                && i.targets
                    .0
                    .iter()
                    .all(|t| self.manifest.contains_cell(&t.cell)),
        )
    }
}
fn has_structure(i: &Impact) -> bool {
    i.targets
        .0
        .iter()
        .any(|t| matches!(t.target, Target::Quest { .. } | Target::TimeTower(_)))
}
fn structure_kind(i: &Impact, kind: Option<QuestKind>) -> Result<(), ApplicationError> {
    let quest = i.targets.0.iter().find_map(|t| match &t.target {
        Target::Quest { id, .. } => Some(id),
        _ => None,
    });
    require(match quest {
        None => kind.is_none(),
        Some(id) => kind.is_some_and(|k| (k != QuestKind::RoomSymbol) == ((id.slot - 1) % 4 == 3)),
    })
}
fn detail_values(r: &SpellReceipt) -> Result<(), ApplicationError> {
    let i = &r.impact;
    match (&r.detail, i.lane) {
        (Detail::Ordinary, Lane::Healing) => Ok(()),
        (Detail::Slow(_), Lane::Slow)
        | (Detail::GreaterHasteNeutralized(_), Lane::GreaterHaste) => {
            super::speed::validate_receipt(r)
        }
        (Detail::Haste(d), Lane::Haste) => require(
            (0..=crate::modern_lito::timed_protocol::MAX_OBSERVED_NANOS)
                .contains(&d.active_elapsed_nanos)
                && (15..=40).contains(&i.usage.cost)
                && i.targets.0.is_empty(),
        ),
        (
            Detail::GreaterHaste {
                active_elapsed_nanos,
                haste_until_nanos,
            },
            Lane::GreaterHaste,
        ) => require(
            (0..=crate::modern_lito::timed_protocol::MAX_OBSERVED_NANOS)
                .contains(active_elapsed_nanos)
                && active_elapsed_nanos.checked_add(2_400_000_000_000) == Some(*haste_until_nanos)
                && (25..=60).contains(&i.usage.cost)
                && i.targets.0.is_empty(),
        ),
        (Detail::Arrow { .. }, Lane::LightArrow | Lane::NpcLightArrow) => Ok(()),
        (Detail::Teleport { destination }, Lane::Teleport) => {
            require(*destination != i.usage.before.cell)
        }
        (Detail::FireStorm { damage_loss_after }, Lane::FireStorm) => require(
            damage_loss_after.0.len() == i.targets.0.len()
                && damage_loss_after
                    .0
                    .iter()
                    .zip(&i.targets.0)
                    .all(|(loss, t)| *loss >= t.applied),
        ),
        (Detail::NpcFireStorm { quest_kind }, Lane::NpcFireStorm) => structure_kind(i, *quest_kind),
        (
            Detail::Disintegrate {
                lawful_before,
                lawful_after,
                damage_loss_after,
                ..
            },
            lane @ (Lane::Disintegrate | Lane::NpcDisintegrate),
        ) => require(
            *lawful_before >= 500
                && lawful_before.checked_add(1) == Some(*lawful_after)
                && if lane == Lane::Disintegrate {
                    i.targets
                        .0
                        .first()
                        .is_some_and(|t| *damage_loss_after >= t.applied)
                } else {
                    *damage_loss_after == 0
                },
        ),
        _ => Err(invalid()),
    }
}
fn impact_values(i: &Impact, caster: i64) -> Result<(), ApplicationError> {
    require(use_history::Cast::from(&i.usage).valid())?;
    let maximum = match i.lane {
        Lane::Teleport | Lane::GreaterHaste | Lane::Haste | Lane::Slow => 0,
        Lane::FireStorm => 101,
        Lane::NpcFireStorm => 48,
        _ => 1,
    };
    require(i.targets.0.len() <= maximum && (maximum != 1 || i.targets.0.len() == 1))?;
    let mut structures = 0;
    let mut entities = std::collections::BTreeSet::new();
    for (index, t) in i.targets.0.iter().enumerate() {
        require(
            values::effect(t).is_some_and(|e| e.valid())
                && !i.targets.0[..index].iter().any(|p| p.target == t.target)
                && t.kind
                    == if i.lane == Lane::Healing {
                        EffectKind::Heal
                    } else {
                        EffectKind::Damage
                    },
        )?;
        if let Target::Participant { entity, .. } | Target::Honor { entity, .. } = &t.target {
            require(entities.insert(*entity))?;
        }
        require(match (&t.target, i.lane) {
            (
                Target::Participant { entity, .. },
                Lane::LightArrow | Lane::FireStorm | Lane::Disintegrate,
            ) => *entity != caster,
            (Target::Honor { entity, .. }, Lane::LightArrow | Lane::FireStorm | Lane::Healing) => {
                *entity != caster
            }
            (
                Target::Npc { .. },
                Lane::NpcLightArrow | Lane::NpcFireStorm | Lane::NpcDisintegrate,
            ) => true,
            (Target::Quest { born_revision, .. }, Lane::NpcFireStorm) => {
                structures += 1;
                *born_revision < i.usage.revision
            }
            (Target::TimeTower(_), Lane::NpcFireStorm) => {
                structures += 1;
                true
            }
            _ => false,
        })?;
    }
    require(structures <= 1)
}
fn structure_values(v: &Spells, caster: i64) -> Result<(), ApplicationError> {
    let fire = v
        .histories
        .0
        .iter()
        .find(|s| s.lane == Lane::NpcFireStorm)
        .and_then(|s| s.receipt.as_ref());
    if let Some(f) = fire.filter(|f| has_structure(&f.impact)) {
        require(v.structure.as_ref().is_some_and(|s| {
            s.casts == f.casts
                && s.impact == f.impact
                && f.detail
                    == Detail::NpcFireStorm {
                        quest_kind: s.quest_kind,
                    }
        }))?;
    }
    if let Some(s) = &v.structure {
        let f = fire.ok_or_else(invalid)?;
        let before = &s.impact.usage;
        let after = &f.impact.usage;
        require(
            s.impact.lane == Lane::NpcFireStorm
                && has_structure(&s.impact)
                && s.casts >= 1
                && s.casts <= before.sequence
                && s.casts <= f.casts
                && before.revision <= after.revision
                && before.sequence <= after.sequence
                && before.before.life <= after.before.life
                && before.before.equipment_revision <= after.before.equipment_revision,
        )?;
        impact_values(&s.impact, caster)?;
        structure_kind(&s.impact, s.quest_kind)?;
        if s.casts == f.casts {
            require(s.impact == f.impact)?;
        } else {
            require(
                before.revision < after.revision
                    && before.sequence < after.sequence
                    && f.casts - s.casts <= after.revision - before.revision
                    && f.casts - s.casts <= after.sequence - before.sequence,
            )?;
        }
    }
    Ok(())
}
