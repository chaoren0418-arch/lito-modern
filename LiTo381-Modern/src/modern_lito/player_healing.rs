//! Candidate participant-healing receipt validation. No command, wire decoder,
//! negotiated capability, optimistic HP, RNG, UI enablement or Honor change.
//! A future follower must ALSO bind this to its exact world/selection, accepted
//! command, authoritative spending/incoming-heal ledger and effect lifecycle.
use super::{
    equipment_history::Identity,
    target_history::{Effect, Kind, Participant, Target, World},
    use_history::Cast,
    vital_balance::State,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    pub water_life: bool,
    pub pollute_water: bool,
    pub dirty_current: bool,
    pub healing_erosion: bool,
}
impl Effects {
    pub fn adjusted(self, base: i64) -> Option<i64> {
        if base < 0 {
            return None;
        }
        let mut damage = if self.water_life {
            base.checked_mul(2)?
        } else {
            base
        }
        .checked_neg()?;
        if self.pollute_water {
            damage >>= 1;
        }
        if self.dirty_current {
            damage >>= 1;
        }
        if self.healing_erosion {
            damage = damage.checked_neg()?;
        }
        damage.checked_neg()
    }
    pub fn consumed(self) -> Self {
        Self {
            water_life: false,
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub identity: Identity,
    pub use_receipt: Cast,
    pub target_effect_revision: i64,
    pub effects: Effects,
    pub base_amount: i64,
    pub adjusted_amount: i64,
    pub target: Effect,
}
impl Receipt {
    pub fn valid(self) -> bool {
        self.identity.valid() && self.impact().valid()
    }
    pub fn impact(self) -> Impact {
        Impact {
            world: self.identity.world,
            caster: self.identity.actor,
            use_receipt: self.use_receipt,
            target_effect_revision: self.target_effect_revision,
            effects: self.effects,
            base_amount: self.base_amount,
            adjusted_amount: self.adjusted_amount,
            target: self.target,
        }
    }
    /// Exact immediate transition, not a skipped-frame follower. In particular,
    /// a same-objid OTHER source is not self and cannot debit the wrong actor.
    pub fn matches_balances(
        self,
        caster_before: State,
        target_before: State,
        caster_after: State,
        target_after: State,
    ) -> bool {
        if !self.valid()
            || !caster_before.valid()
            || !target_before.valid()
            || !caster_after.valid()
            || !target_after.valid()
        {
            return false;
        }
        let u = self.use_receipt;
        let t = self.target.before;
        if caster_before.equipment_revision != u.before.equipment_revision
            || caster_before.life.checked_add(1) != Some(u.before.life)
            || caster_before.vitals != u.before.vitals
            || target_before.equipment_revision != t.equipment_revision
            || target_before.life.checked_add(1) != Some(t.life)
            || target_before.vitals.hp != t.hp
            || target_before.vitals.maximum_hp != t.maximum_hp
        {
            return false;
        }
        if self.impact().is_self() {
            caster_before == target_before
                && caster_after == target_after
                && caster_before.restore_and_spend(
                    u.revision,
                    self.target.applied,
                    i64::from(u.cost),
                ) == Some(caster_after)
        } else {
            caster_before.spend(u.revision, i64::from(u.cost)) == Some(caster_after)
                && target_before.restore(u.revision, self.target.applied) == Some(target_after)
        }
    }
}

/// Public/recipient view. The private caster selection epoch is NOT present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Impact {
    pub world: World,
    pub caster: Participant,
    pub use_receipt: Cast,
    pub target_effect_revision: i64,
    pub effects: Effects,
    pub base_amount: i64,
    pub adjusted_amount: i64,
    pub target: Effect,
}
impl Impact {
    pub fn recipient(self) -> Option<Participant> {
        match self.target.before.target {
            Target::Participant(p) => Some(p),
            _ => None,
        }
    }
    pub fn is_self(self) -> bool {
        self.recipient() == Some(self.caster)
    }
    pub fn valid(self) -> bool {
        let u = self.use_receipt;
        if !self.world.valid()
            || !self.caster.valid()
            || self.caster.identity.kind != super::participant::MemberKind::Human
            || !u.valid()
            || u.cost > 4
            || u.sequence >= u.revision
            || self.target_effect_revision < 0
            || self.target_effect_revision >= u.revision
            || !(0..=32767).contains(&self.base_amount)
            || self.effects.adjusted(self.base_amount) != Some(self.adjusted_amount)
            || !self.target.valid()
            || self.target.kind != Kind::Heal
            || self.target.raw != self.adjusted_amount.max(0)
        {
            return false;
        }
        let Target::Participant(p) = self.target.before.target else {
            return false;
        };
        let caster = self.caster;
        let same_entity = caster.entity_id == p.entity_id;
        let same_identity = caster.identity == p.identity;
        if same_entity != same_identity {
            return false;
        }
        let t = self.target.before;
        !same_entity
            || (u.before.life == t.life
                && u.before.equipment_revision == t.equipment_revision
                && u.before.x == t.cell.x
                && u.before.y == t.cell.y
                && u.before.vitals.hp == t.hp
                && u.before.vitals.maximum_hp == t.maximum_hp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        application_protocol::{CharacterKey, SelectionContext},
        equipment_transition::Vitals,
        participant::{MemberKind, ParticipantId},
        target_history::{Health, Participant, World},
        use_history::Anchor,
        world_map::WorldCell,
    };
    fn sample(self_target: bool) -> (Receipt, State, State) {
        let key = CharacterKey {
            source_id: 7,
            object_id: 3333,
        };
        let actor = Participant {
            entity_id: 1,
            identity: ParticipantId::new(MemberKind::Human, key),
        };
        let caster = State::initial(
            1,
            0,
            Vitals {
                hp: 40,
                maximum_hp: 100,
                mp: 100,
                maximum_mp: 100,
            },
        )
        .unwrap();
        let target = caster;
        let participant = if self_target {
            actor
        } else {
            Participant {
                entity_id: 2,
                identity: ParticipantId::new(
                    MemberKind::Human,
                    CharacterKey {
                        source_id: 42,
                        ..key
                    },
                ),
            }
        };
        let receipt = Receipt {
            identity: Identity {
                world: World {
                    room_id: 1,
                    match_id: 2,
                    generation: 3,
                },
                source: SelectionContext { key, epoch: 1 },
                channel_id: 0,
                actor,
            },
            use_receipt: Cast {
                revision: 2,
                sequence: 1,
                before: Anchor {
                    equipment_revision: 0,
                    life: 1,
                    vitals: caster.vitals,
                    x: 10,
                    y: 10,
                },
                cost: 4,
                mp_after: 96,
            },
            target_effect_revision: 0,
            effects: Effects::default(),
            base_amount: 8,
            adjusted_amount: 8,
            target: Effect {
                kind: Kind::Heal,
                before: Health {
                    target: Target::Participant(participant),
                    life: 1,
                    equipment_revision: 0,
                    cell: WorldCell { x: 10, y: 10 },
                    hp: 40,
                    maximum_hp: 100,
                },
                raw: 8,
                applied: 8,
                hp_after: 48,
            },
        };
        (receipt, caster, target)
    }
    #[test]
    fn same_source_self_has_one_balance_but_cross_source_same_objid_has_two() {
        let (r, a, b) = sample(true);
        let after = a.restore_and_spend(2, 8, 4).unwrap();
        assert!(r.valid());
        assert!(r.matches_balances(a, b, after, after));
        assert!(!r.matches_balances(a, b, a.spend(2, 4).unwrap(), b.restore(2, 8).unwrap()));
        let (r, a, b) = sample(false);
        assert!(r.valid());
        assert!(r.matches_balances(a, b, a.spend(2, 4).unwrap(), b.restore(2, 8).unwrap()));
        assert!(!r.matches_balances(a, b, after, after));
    }
    #[test]
    fn corrupted_cost_recovery_life_gear_identity_or_kind_cannot_validate() {
        let (r, _, _) = sample(true);
        let mut candidates = vec![];
        let mut bad = r;
        bad.base_amount += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.adjusted_amount += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.target.applied += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.target.hp_after += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.target.before.life += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.target.before.equipment_revision += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.target.before.cell.x += 1;
        candidates.push(bad);
        let mut bad = r;
        bad.use_receipt.cost = 5;
        bad.use_receipt.mp_after = 95;
        candidates.push(bad);
        let mut bad = r;
        bad.target.kind = Kind::Damage;
        bad.target.hp_after = 32;
        candidates.push(bad);
        let mut bad = r;
        bad.target.before.target = Target::Honor {
            entity_id: 3,
            team: 0,
        };
        candidates.push(bad);
        let mut bad = r;
        bad.target_effect_revision = 2;
        candidates.push(bad);
        let mut bad = r;
        bad.use_receipt.sequence = 2;
        candidates.push(bad);
        let mut bad = r;
        bad.identity.source.key.source_id = 42;
        candidates.push(bad);
        for bad in candidates {
            assert!(!bad.valid(), "{bad:?}");
        }
    }
    #[test]
    fn bot_namespace_is_retained_and_entity_identity_aliases_are_rejected() {
        let (mut r, _, _) = sample(false);
        let Target::Participant(mut p) = r.target.before.target else {
            panic!()
        };
        p.identity.source_id = 7;
        p.identity.kind = MemberKind::ServerBot;
        r.target.before.target = Target::Participant(p);
        assert!(r.valid());
        p.identity.kind = MemberKind::Human;
        r.target.before.target = Target::Participant(p);
        assert!(!r.valid());
        p.identity.source_id = 42;
        p.entity_id = 1;
        r.target.before.target = Target::Participant(p);
        assert!(!r.valid());
    }
    #[test]
    fn all_modifier_combinations_keep_native_negative_healing_rounding() {
        for flags in 0..16 {
            for base in 0..=257i64 {
                let e = Effects {
                    water_life: flags & 1 != 0,
                    pollute_water: flags & 2 != 0,
                    dirty_current: flags & 4 != 0,
                    healing_erosion: flags & 8 != 0,
                };
                // Independent positive-domain reference: each half rounds up.
                let mut expected = base * if e.water_life { 2 } else { 1 };
                if e.pollute_water {
                    expected = (expected + 1) / 2;
                }
                if e.dirty_current {
                    expected = (expected + 1) / 2;
                }
                if e.healing_erosion {
                    expected = -expected;
                }
                assert_eq!(Some(expected), e.adjusted(base));
                assert!(!e.consumed().water_life);
            }
        }
        assert!(Effects {
            water_life: true,
            ..Effects::default()
        }
        .adjusted(i64::MAX)
        .is_none());
        assert!(Effects::default().adjusted(-1).is_none());
    }
    #[test]
    fn erosion_is_zero_restoration_not_damage_and_consumes_water_life() {
        let (mut r, a, b) = sample(false);
        r.effects = Effects {
            water_life: true,
            healing_erosion: true,
            ..Effects::default()
        };
        r.adjusted_amount = -16;
        r.target.raw = 0;
        r.target.applied = 0;
        r.target.hp_after = 40;
        assert!(r.valid());
        assert!(r.matches_balances(a, b, a.spend(2, 4).unwrap(), b.restore(2, 0).unwrap()));
        assert!(!r.effects.consumed().water_life);
        assert!(!r.matches_balances(a, b, a.spend(2, 4).unwrap(), b.damage(2, 16).unwrap()));
    }
    #[test]
    fn old_honor_healing_batch_stays_strictly_incompatible_with_player_receipts() {
        let (r, _, _) = sample(false);
        let old = crate::modern_lito::target_history::Batch {
            world: r.identity.world,
            source: r.identity.source,
            channel_id: r.identity.channel_id,
            caster: r.identity.actor,
            lane: crate::modern_lito::target_history::Lane::Healing,
            use_receipt: r.use_receipt,
            targets: vec![r.target],
        };
        assert!(r.valid());
        assert!(!old.valid());
    }
    #[test]
    fn wide_self_healing_receipt_retains_exact_capacity_and_zero_cost_cast() {
        let (mut r, _, _) = sample(true);
        let before = State::initial(
            1,
            0,
            Vitals {
                hp: i64::MAX - 1,
                maximum_hp: i64::MAX,
                mp: 0,
                maximum_mp: 0,
            },
        )
        .unwrap();
        r.use_receipt.before.vitals = before.vitals;
        r.use_receipt.cost = 0;
        r.use_receipt.mp_after = 0;
        r.target.before.hp = i64::MAX - 1;
        r.target.before.maximum_hp = i64::MAX;
        r.target.applied = 1;
        r.target.hp_after = i64::MAX;
        let after = before.restore_and_spend(2, 1, 0).unwrap();
        assert!(r.valid());
        assert!(r.matches_balances(before, before, after, after));
        let mut forged = after;
        forged.vitals.hp -= 1;
        assert!(!r.matches_balances(before, before, forged, forged));
        assert_eq!(1, r.use_receipt.sequence);
    }
}
