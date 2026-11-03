//! Complete schema7 value proof without changing/stripping any received field.
//! Shared-resource qualification and authenticated command admission are later
//! gates; this module never constructs a legacy or live Observation.
use super::super::{data, personal::PersonalView};
use super::*;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub struct Checked {
    candidate: Arc<Candidate>,
    personal: PersonalView,
}
impl std::fmt::Debug for Checked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CheckedHealingBattle[redacted]")
    }
}
impl Checked {
    pub fn candidate(&self) -> &Candidate {
        &self.candidate
    }
    pub fn personal(&self) -> &PersonalView {
        &self.personal
    }
}
impl Candidate {
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.view()?.check_values(player, world)
    }
    pub fn checked(self, player: &Player, world: &WorldId) -> Result<Checked, ApplicationError> {
        self.check_values(player, world)?;
        let personal = self.data.personal_facts(player, world)?;
        Ok(Checked {
            candidate: Arc::new(self),
            personal,
        })
    }
}
impl View<'_> {
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        if let Some(a) = &self.data.armor {
            require(self.status == &a.status && a.healing.as_ref() == Some(self.actors))?;
            return super::super::armor::attached(self.data)?.check_values(player, world);
        }
        self.shape()?;
        let d = &self.data;
        d.check_healing_context(player, world)?;
        self.check_healing_accounts(player, world)?;
        d.check_combat_with_healing(Some(&self.actors.0))?;
        d.personal_facts(player, world)?;
        d.check_scene_body()?;
        crate::modern_lito::battle_follow::clocks(d)?;
        match &self.status {
            Status::Motion(m) => {
                super::super::npc_motion::MotionView { data: d, motion: m }.check_motion()?
            }
            Status::Haste(s) => super::super::haste::View {
                data: d,
                effects: s,
            }
            .check_effects()?,
            Status::Speed(s) => {
                let v = super::super::speed::View {
                    data: d,
                    effects: s,
                };
                v.check_components()?;
                v.check_effects()?;
            }
        }
        self.check_healing_scene()
    }
    pub(super) fn healing_actor(&self, entity: i64) -> Result<&Actor, ApplicationError> {
        self.actors
            .0
            .iter()
            .find(|h| h.entity == entity)
            .ok_or_else(invalid)
    }
    pub(super) fn healing_events(&self) -> Result<BTreeMap<i64, &Impact>, ApplicationError> {
        let mut events = BTreeMap::new();
        for h in &self.actors.0 {
            for impact in h.incoming.last.iter().chain(
                h.outgoing
                    .as_ref()
                    .and_then(|o| o.receipt.as_ref())
                    .map(|r| &r.impact),
            ) {
                if let Some(old) = events.insert(impact.usage.revision, impact) {
                    require(old == impact)?;
                }
            }
        }
        Ok(events)
    }
    pub(in crate::modern_lito::battle_payload) fn check_healing_scene(
        &self,
    ) -> Result<(), ApplicationError> {
        let d = &self.data;
        let events = self.healing_events()?;
        // No healing crosses an isolated audience. Count conservation therefore
        // holds for shared rooms and for each independently visible lane, even
        // after individual latest receipts have been superseded.
        let casts: i128 = self
            .actors
            .0
            .iter()
            .filter_map(|h| h.outgoing.as_ref())
            .map(|o| i128::from(o.ledger.casts))
            .sum();
        let received: i128 = self
            .actors
            .0
            .iter()
            .map(|h| i128::from(h.incoming.received))
            .sum();
        require(casts == received && casts <= i128::from(d.world.revision))?;
        for h in &self.actors.0 {
            let sent = events
                .values()
                .filter(|e| e.caster.entity == h.entity)
                .count() as i128;
            let received = events
                .values()
                .filter(|e| {
                    matches!(e.target.target,
                data::Target::Participant { entity, .. } if entity == h.entity)
                })
                .count() as i128;
            require(
                sent <= h
                    .outgoing
                    .as_ref()
                    .map_or(0, |o| i128::from(o.ledger.casts))
                    && received <= i128::from(h.incoming.received),
            )?;
        }
        for a in &d.actors.0 {
            for r in a
                .spells
                .histories
                .0
                .iter()
                .filter_map(|s| s.receipt.as_ref())
            {
                require(!events.contains_key(&r.impact.usage.revision))?;
            }
        }
        if super::super::scene::initial(d.world.phase) {
            require(events.is_empty())?;
        }
        for (revision, impact) in events {
            // Runtime currently supports Effects.NONE only. Arithmetic support
            // for other flags is not evidence of an authoritative effect lease.
            require(impact.effects == 0 && impact.target_effect_revision == 0)?;
            let u = &impact.usage;
            let caster = d.scene_actor(impact.caster.entity)?;
            let sent = self
                .healing_actor(caster.entity)?
                .outgoing
                .as_ref()
                .ok_or_else(invalid)?;
            let last = &sent.receipt.as_ref().ok_or_else(invalid)?.impact;
            require(
                d.scene_member(caster)?.identity == impact.caster.identity
                    && u.sequence <= caster.spells.sequence
                    && revision <= caster.spells.request_revision
                    && last.usage.sequence >= u.sequence
                    && last.usage.revision >= revision
                    && (last.usage.sequence != u.sequence || last == impact)
                    && (last.usage.revision != revision || last == impact),
            )?;
            d.scene_usage(caster, u, false)?;
            let (entity, identity) = match &impact.target.target {
                data::Target::Participant { entity, identity } => (*entity, identity),
                _ => return Err(invalid()),
            };
            let target = d.scene_actor(entity)?;
            let received = &self.healing_actor(entity)?.incoming;
            let latest = received.last.as_ref().ok_or_else(invalid)?;
            let resources = target.resources.as_ref().ok_or_else(invalid)?;
            require(
                d.scene_member(target)?.identity == *identity
                    && latest.usage.revision >= revision
                    && (latest.usage.revision != revision || latest == impact)
                    && impact.target.life <= resources.life.checked_add(1).ok_or_else(invalid)?
                    && impact.target.equipment_revision <= resources.equipment_revision
                    && (impact.target.equipment_revision != resources.equipment_revision
                        || impact.target.maximum_hp == resources.vitals.maximum_hp)
                    && impact.target.applied <= received.restored,
            )?;
            d.scene_life(target, impact.target.life, revision)?;
            let cr = caster.resources.as_ref().ok_or_else(invalid)?;
            require(
                u.before.life <= cr.life.checked_add(1).ok_or_else(invalid)?
                    && u.before.equipment_revision <= cr.equipment_revision
                    && revision <= cr.revision
                    && (u.before.equipment_revision != cr.equipment_revision
                        || u.before.vitals.maximum_hp == cr.vitals.maximum_hp
                            && u.before.vitals.maximum_mp == cr.vitals.maximum_mp)
                    && d.manifest.contains_cell(&u.before.cell)
                    && d.manifest.contains_cell(&impact.target.cell),
            )?;
            let dx = i64::from(impact.target.cell.x) - i64::from(u.before.cell.x);
            let dy = i64::from(impact.target.cell.y) - i64::from(u.before.cell.y);
            let distance = dx.abs() + dy.abs();
            let edge = (dx + 18).abs() + (dy + 18).abs();
            require(distance <= 18 || distance == 19 && (19..=52).contains(&edge))?;
            if revision == d.world.revision {
                require(
                    impact.target.hp_after == resources.vitals.hp
                        && impact.target.cell == target.position
                        && u.mp_after == cr.vitals.mp
                        && resources.revision == revision
                        && cr.revision == revision,
                )?;
            }
        }
        Ok(())
    }
}
