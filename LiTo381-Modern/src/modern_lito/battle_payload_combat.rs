//! Single-observation resource/use integrity, in engine revision space.
//! This is NOT yet complete mode validation or temporal/session admission.
use super::{data::*, invalid, ApplicationError};
use crate::modern_lito::{item_protocol, timed_protocol, use_history, vital_balance};

pub(super) fn require(ok: bool) -> Result<(), ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
impl UnvalidatedData {
    /// Context + combat resource/use checks. Still returns no admitted/UI type:
    /// full mode proofs, inventory qualification and an atomic follower remain.
    pub fn check_combat_values(
        &self,
        player: &Player,
        world: &WorldId,
    ) -> Result<(), ApplicationError> {
        if self.vip.is_some() {
            return super::vip::attached(self)?.check_values(player, world);
        }
        if self.armor.is_some() {
            return super::armor::attached(self)?.check_values(player, world);
        }
        if self.healing.is_some() {
            return super::healing::attached(self)?.check_values(player, world);
        }
        self.check_context(player, world)?;
        self.check_combat_body()
    }
    /// Shared arithmetic after an explicit context gate; never admission alone.
    pub(super) fn check_combat_body(&self) -> Result<(), ApplicationError> {
        self.check_combat_with_healing(None)
    }
    pub(super) fn check_combat_with_healing(
        &self,
        healing: Option<&[super::healing::Actor]>,
    ) -> Result<(), ApplicationError> {
        require(
            (self.world.descriptor.features & super::healing::FEATURE != 0) == healing.is_some(),
        )?;
        if let Some(h) = healing {
            require(h.len() == self.actors.0.len())?;
        }
        for (i, a) in self.actors.0.iter().enumerate() {
            let h = healing.map(|h| &h[i]);
            if let Some(h) = h {
                require(h.entity == a.entity)?;
            }
            self.actor_values(a, h.map_or(0, |h| h.incoming.restored))?;
            match h {
                Some(h) => self.spell_values_with_healing(a, Some(h))?,
                None => self.spell_values(a)?,
            }
        }
        let owner = self
            .actors
            .0
            .iter()
            .find(|a| a.entity == self.personal.entity)
            .ok_or_else(invalid)?;
        if let Some(h) = &self.personal.history {
            let h = &h.0;
            require(
                h.valid()
                    && owner
                        .resources
                        .as_ref()
                        .is_some_and(|r| vital_balance::State::from(r) == h.resources),
            )?;
            let spells = owner
                .spells
                .histories
                .0
                .iter()
                .map(|s| (s.lane.into(), use_history::Spell::from(s)))
                .collect::<std::collections::BTreeMap<_, _>>();
            require(h.spells == spells)?;
            // Compare event-time targets too; matching current HP/MP alone is
            // insufficient after a heal, respawn or equipment epoch change.
            for s in &owner.spells.histories.0 {
                match (&s.receipt, h.targets.get(&s.lane.into())) {
                    (None, None) => {}
                    (Some(r), Some(t)) => {
                        require(impact_matches(&r.impact, &t.last))?;
                        match (&owner.spells.structure, &t.structure) {
                            (Some(v), Some(old)) => require(
                                s.lane == Lane::NpcFireStorm && impact_matches(&v.impact, old),
                            )?,
                            (_, None)
                                if s.lane != Lane::NpcFireStorm
                                    || owner.spells.structure.is_none() => {}
                            _ => return Err(invalid()),
                        }
                    }
                    _ => return Err(invalid()),
                }
            }
            require(
                h.items
                    == self
                        .personal
                        .supplies
                        .as_ref()
                        .map(|s| use_history::Items::from(&s.history)),
            )?;
            let birth = self.personal.spell_birth_revision;
            if let Some(h) = healing.and_then(|h| h.iter().find(|h| h.entity == owner.entity)) {
                require(
                    h.outgoing
                        .as_ref()
                        .and_then(|o| o.receipt.as_ref())
                        .is_none_or(|r| r.impact.usage.revision > birth),
                )?;
            }
            require(
                birth >= 1
                    && birth <= self.world.revision
                    && (owner.spells.sequence == 0
                        || owner.spells.request_revision > birth
                            && owner.spells.sequence <= owner.spells.request_revision - birth)
                    && owner
                        .spells
                        .histories
                        .0
                        .iter()
                        .all(|s| s.last.as_ref().is_none_or(|l| l.revision > birth)),
            )?;
            require(
                owner
                    .spells
                    .structure
                    .as_ref()
                    .is_none_or(|s| s.impact.usage.revision > birth),
            )?;
            let e = self.personal.equipment.as_ref().ok_or_else(invalid)?;
            require(
                e.effective.maximum_hp == h.resources.vitals.maximum_hp
                    && e.effective.maximum_mp == h.resources.vitals.maximum_mp
                    && owner.vital.as_ref().is_some_and(|v| v.ac == e.effective.ac)
                    && (0..=225).contains(&e.saved.food225),
            )?;
        } else {
            require(self.personal.spell_birth_revision == 0)?;
        }
        if let Some(s) = &self.personal.supplies {
            self.supply_values(s)?;
        }
        Ok(())
    }
    fn actor_values(&self, a: &Actor, received_healing: i64) -> Result<(), ApplicationError> {
        let revision = self.world.revision;
        if let Some(v) = &a.vital {
            let r = a.resources.as_ref().ok_or_else(invalid)?;
            require(
                vital_balance::State::from(r).valid()
                    && v.values == r.vitals
                    && r.revision <= revision
                    && (0..=18).contains(&v.weapon_type)
                    && v.attack_sequence >= 0
                    && [v.attack_revision, v.hurt_revision, v.death_revision]
                        .iter()
                        .all(|r| (0..=revision).contains(r))
                    && (v.values.hp != 0 || v.death_revision > 0)
                    && v.target >= 0
                    && (v.attack_revision == 0) == (v.target == 0)
                    && (1_000_000..=10_000_000_000).contains(&v.attack_interval_nanos),
            )?;
            if let Some(g) = &a.regeneration {
                for axis in [&g.hp, &g.mp] {
                    require(
                        [axis.gain, axis.loss, axis.before, axis.after]
                            .iter()
                            .all(|n| *n >= 0)
                            && (0..=revision).contains(&axis.revision),
                    )?;
                }
                let f = &r.flows;
                require(
                    g.hp.gain == f.hp_gain
                        && g.hp.loss == f.hp_loss
                        && g.mp.gain == f.mp_gain
                        && g.mp.loss == f.mp_loss
                        && g.item_gain.checked_add(received_healing) == Some(f.restored)
                        && g.damage_loss == f.damaged,
                )?;
            }
        }
        if let Some(b) = &a.buffs {
            let speed = self.world.descriptor.features & super::speed::FEATURE != 0;
            if speed {
                super::speed::validate_buff(b)?;
            }
            require(
                b.entity == a.entity
                    && b.revision <= revision
                    && (speed || b.slow_until_nanos == 0 && b.slow_revision == 0),
            )?;
            let state = timed_protocol::State {
                entity_id: b.entity.try_into().map_err(|_| invalid())?,
                revision: b.revision,
                observed_nanos: b.observed_nanos,
                haste_until_nanos: b.haste_until_nanos,
                brave_until_nanos: b.brave_until_nanos,
                wisdom_until_nanos: b.wisdom_until_nanos,
                brave_kind: b.brave_kind.try_into().map_err(|_| invalid())?,
            };
            require((b.haste_origin == HasteOrigin::None) == (b.haste_until_nanos == 0))?;
            if b.haste_origin == HasteOrigin::GreaterHaste {
                state.validate_greater_haste()?;
                let receipt = a
                    .spells
                    .histories
                    .0
                    .iter()
                    .find(|s| s.lane == Lane::GreaterHaste)
                    .and_then(|s| s.receipt.as_ref())
                    .ok_or_else(invalid)?;
                let Detail::GreaterHaste {
                    active_elapsed_nanos,
                    haste_until_nanos,
                } = receipt.detail
                else {
                    return Err(invalid());
                };
                let resources = a.resources.as_ref().ok_or_else(invalid)?;
                require(
                    self.greater_haste()
                        && self.world.phase == Phase::Active
                        && resources.vitals.hp > 0
                        && haste_until_nanos == b.haste_until_nanos
                        && active_elapsed_nanos <= b.observed_nanos
                        && b.observed_nanos <= self.world.active_elapsed_nanos
                        && b.haste_until_nanos > self.world.active_elapsed_nanos
                        && receipt.impact.usage.revision <= b.revision
                        && receipt.impact.usage.before.life == resources.life + 1
                        && receipt
                            .loading_elapsed_nanos
                            .checked_sub(active_elapsed_nanos)
                            == Some(
                                self.world.loading_elapsed_nanos - self.world.active_elapsed_nanos,
                            ),
                )?;
            } else if b.haste_origin == HasteOrigin::Haste {
                require(self.world.descriptor.features & super::haste::FEATURE != 0)?;
                // The complete target/caster proof lives in the schema5
                // extension. Do not invent a self-cast for a foreign buff.
                if !speed {
                    super::haste::validate_buff(b)?;
                }
            } else {
                state.validate()?;
            }
        }
        Ok(())
    }
    fn supply_values(&self, s: &Supplies) -> Result<(), ApplicationError> {
        let h = use_history::Items::from(&s.history);
        require(
            h.valid()
                && s.request_sequence >= 0
                && s.request_revision >= 0
                && (s.request_sequence == 0) == (s.request_revision == 0)
                && (s.request_sequence == 0 || s.request_sequence < s.request_revision)
                && s.request_revision <= self.world.revision,
        )?;
        let mut previous = 0;
        let mut consumed = 0i64;
        for stock in &s.stocks.0 {
            require(
                stock.item > previous
                    && item_protocol::supported(stock.item)
                    && stock.initial >= 0
                    && stock.remaining >= 0
                    && stock.remaining <= stock.initial,
            )?;
            consumed = consumed
                .checked_add(stock.initial - stock.remaining)
                .ok_or_else(invalid)?;
            previous = stock.item;
        }
        require(consumed == h.uses && consumed <= s.request_sequence)?;
        let (effect, delay) = match h.last {
            None => (0, 0),
            Some(l) => {
                require(self.manifest.contains_cell(&Cell {
                    x: l.before.x,
                    y: l.before.y,
                }))?;
                require(
                    l.sequence <= s.request_sequence
                        && l.revision <= s.request_revision
                        && (l.sequence == s.request_sequence) == (l.revision == s.request_revision)
                        && s.stocks
                            .0
                            .iter()
                            .any(|s| s.item == l.item_id && s.remaining < s.initial),
                )?;
                if let Some(k) = item_protocol::healing(l.item_id) {
                    (k.effect, k.delay_millis)
                } else {
                    (
                        timed_protocol::Kind::from_item(l.item_id)
                            .ok_or_else(invalid)?
                            .effect(),
                        0,
                    )
                }
            }
        };
        require(s.effect == effect && s.delay_millis == delay)
    }
}
fn impact_matches(i: &Impact, b: &crate::modern_lito::target_history::Batch) -> bool {
    crate::modern_lito::target_history::Lane::from(i.lane) == b.lane
        && use_history::Cast::from(&i.usage) == b.use_receipt
        && i.targets.0.len() == b.targets.len()
        && i.targets
            .0
            .iter()
            .zip(&b.targets)
            .all(|(a, b)| super::values::effect(a).as_ref() == Some(b))
}
