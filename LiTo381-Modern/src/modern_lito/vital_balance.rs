//! Applied resource accounting across equipment/life changes. Not prediction,
//! authority, or a new wire capability; protocol <=34 remains unchanged.
use super::equipment_transition::Vitals;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flows {
    pub hp_gain: i64,
    pub hp_loss: i64,
    pub restored: i64,
    pub damaged: i64,
    pub hp_granted: i64,
    pub mp_gain: i64,
    pub mp_loss: i64,
    pub spent: i64,
    pub mp_granted: i64,
    pub hp_clipped: i64,
    pub mp_clipped: i64,
}
impl Flows {
    pub fn valid(self) -> bool {
        self.fields().iter().all(|&n| n >= 0)
    }
    fn fields(self) -> [i64; 11] {
        [
            self.hp_gain,
            self.hp_loss,
            self.restored,
            self.damaged,
            self.hp_granted,
            self.mp_gain,
            self.mp_loss,
            self.spent,
            self.mp_granted,
            self.hp_clipped,
            self.mp_clipped,
        ]
    }
    fn plus(self, d: Self) -> Option<Self> {
        if !self.valid() || !d.valid() {
            return None;
        }
        Some(Self {
            hp_gain: self.hp_gain.checked_add(d.hp_gain)?,
            hp_loss: self.hp_loss.checked_add(d.hp_loss)?,
            restored: self.restored.checked_add(d.restored)?,
            damaged: self.damaged.checked_add(d.damaged)?,
            hp_granted: self.hp_granted.checked_add(d.hp_granted)?,
            mp_gain: self.mp_gain.checked_add(d.mp_gain)?,
            mp_loss: self.mp_loss.checked_add(d.mp_loss)?,
            spent: self.spent.checked_add(d.spent)?,
            mp_granted: self.mp_granted.checked_add(d.mp_granted)?,
            hp_clipped: self.hp_clipped.checked_add(d.hp_clipped)?,
            mp_clipped: self.mp_clipped.checked_add(d.mp_clipped)?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub revision: i64,
    pub equipment_revision: i64,
    pub life: i64,
    pub birth_hp: i64,
    pub birth_mp: i64,
    pub vitals: Vitals,
    pub flows: Flows,
}
impl State {
    pub fn valid(self) -> bool {
        let f = self.flows;
        self.revision > 0
            && self.equipment_revision >= 0
            && self.life >= 0
            && self.life < self.revision
            && self.birth_hp > 0
            && self.birth_mp >= 0
            && self.vitals.valid()
            && f.valid()
            && (self.life != 0 || f.hp_granted == 0 && f.mp_granted == 0)
            && self.life <= f.hp_granted
            && self.birth_hp as i128 + f.hp_gain as i128 + f.restored as i128 + f.hp_granted as i128
                - f.hp_loss as i128
                - f.damaged as i128
                - f.hp_clipped as i128
                == self.vitals.hp as i128
            && self.birth_mp as i128 + f.mp_gain as i128 + f.mp_granted as i128
                - f.mp_loss as i128
                - f.spent as i128
                - f.mp_clipped as i128
                == self.vitals.mp as i128
    }
    pub fn initial(revision: i64, equipment_revision: i64, vitals: Vitals) -> Option<Self> {
        let value = Self {
            revision,
            equipment_revision,
            life: 0,
            birth_hp: vitals.hp,
            birth_mp: vitals.mp,
            vitals,
            flows: Flows::default(),
        };
        value.valid().then_some(value)
    }
    fn values(self, hp: i64, mp: i64) -> Vitals {
        Vitals {
            hp,
            mp,
            ..self.vitals
        }
    }
    fn step(
        self,
        revision: i64,
        equipment_revision: i64,
        life: i64,
        vitals: Vitals,
        delta: Flows,
    ) -> Option<Self> {
        if !self.valid() || revision <= self.revision {
            return None;
        }
        let value = Self {
            revision,
            equipment_revision,
            life,
            vitals,
            flows: self.flows.plus(delta)?,
            ..self
        };
        value.valid().then_some(value)
    }
    pub fn damage(self, next: i64, applied: i64) -> Option<Self> {
        if !self.valid() || applied < 0 || applied > self.vitals.hp {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(self.vitals.hp - applied, self.vitals.mp),
            Flows {
                damaged: applied,
                ..Flows::default()
            },
        )
    }
    pub fn restore(self, next: i64, applied: i64) -> Option<Self> {
        if !self.valid()
            || self.vitals.hp == 0
            || applied < 0
            || applied > self.vitals.maximum_hp - self.vitals.hp
        {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(self.vitals.hp + applied, self.vitals.mp),
            Flows {
                restored: applied,
                ..Flows::default()
            },
        )
    }
    /// A single self-healing event. This is accounting, not a negotiated cast
    /// or permission to accept a healing packet on an older connection.
    pub fn restore_and_spend(self, next: i64, restored: i64, spent: i64) -> Option<Self> {
        if !self.valid()
            || self.vitals.hp == 0
            || restored < 0
            || spent < 0
            || restored > self.vitals.maximum_hp - self.vitals.hp
            || spent > self.vitals.mp
        {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(self.vitals.hp + restored, self.vitals.mp - spent),
            Flows {
                restored,
                spent,
                ..Flows::default()
            },
        )
    }
    pub fn spend(self, next: i64, applied: i64) -> Option<Self> {
        if !self.valid() || self.vitals.hp == 0 || applied < 0 || applied > self.vitals.mp {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(self.vitals.hp, self.vitals.mp - applied),
            Flows {
                spent: applied,
                ..Flows::default()
            },
        )
    }
    pub fn regenerate(
        self,
        next: i64,
        hp_gain: i64,
        hp_loss: i64,
        mp_gain: i64,
        mp_loss: i64,
    ) -> Option<Self> {
        let v = self.vitals;
        if !self.valid()
            || v.hp == 0
            || hp_gain < 0
            || hp_loss < 0
            || mp_gain < 0
            || mp_loss < 0
            || hp_gain > 0 && hp_loss > 0
            || mp_gain > 0 && mp_loss > 0
            || hp_gain > v.maximum_hp - v.hp
            || hp_loss >= v.hp
            || mp_gain > v.maximum_mp - v.mp
            || mp_loss > v.mp
        {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(v.hp + hp_gain - hp_loss, v.mp + mp_gain - mp_loss),
            Flows {
                hp_gain,
                hp_loss,
                mp_gain,
                mp_loss,
                ..Flows::default()
            },
        )
    }
    /// Accounting for applied catch-up ticks, which can change sign. The
    /// caller must separately authenticate the actual ticks; net balance is
    /// not permission for a client to invent recovery.
    pub fn regenerate_batch(
        self,
        next: i64,
        hp_gain: i64,
        hp_loss: i64,
        mp_gain: i64,
        mp_loss: i64,
        hp_after: i64,
        mp_after: i64,
    ) -> Option<Self> {
        if !self.valid()
            || self.vitals.hp == 0
            || hp_after < 1
            || hp_gain < 0
            || hp_loss < 0
            || mp_gain < 0
            || mp_loss < 0
            || self.vitals.hp as i128 + hp_gain as i128 - hp_loss as i128 != hp_after as i128
            || self.vitals.mp as i128 + mp_gain as i128 - mp_loss as i128 != mp_after as i128
        {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life,
            self.values(hp_after, mp_after),
            Flows {
                hp_gain,
                hp_loss,
                mp_gain,
                mp_loss,
                ..Flows::default()
            },
        )
    }
    pub fn respawn(self, next: i64, hp_granted: i64, mp_granted: i64) -> Option<Self> {
        let v = self.vitals;
        if !self.valid()
            || v.hp != 0
            || hp_granted < 1
            || hp_granted > v.maximum_hp
            || mp_granted < 0
            || mp_granted > v.maximum_mp - v.mp
        {
            return None;
        }
        self.step(
            next,
            self.equipment_revision,
            self.life.checked_add(1)?,
            self.values(hp_granted, v.mp + mp_granted),
            Flows {
                hp_granted,
                mp_granted,
                ..Flows::default()
            },
        )
    }
    pub fn equip(self, next: i64, gear: i64, maximum_hp: i64, maximum_mp: i64) -> Option<Self> {
        if !self.valid() || gear != self.equipment_revision.checked_add(1)? {
            return None;
        }
        let retained = self.vitals.retain(maximum_hp, maximum_mp)?;
        self.step(
            next,
            gear,
            self.life,
            retained,
            Flows {
                hp_clipped: self.vitals.hp - retained.hp,
                mp_clipped: self.vitals.mp - retained.mp,
                ..Flows::default()
            },
        )
    }
}

/// Exactly one equipment event; an eventual envelope must also bind actor,
/// source/selection/world/life and validate any intervening resource events.
pub fn follows_equipment(before: State, after: State) -> bool {
    before.equip(
        after.revision,
        after.equipment_revision,
        after.vitals.maximum_hp,
        after.vitals.maximum_mp,
    ) == Some(after)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catch_up_keeps_both_gross_flows_and_checks_final_values() {
        let a = State::initial(
            1,
            0,
            Vitals {
                hp: 500,
                maximum_hp: 1000,
                mp: 200,
                maximum_mp: 250,
            },
        )
        .unwrap();
        let b = a.regenerate_batch(2, 4, 4, 0, 2, 500, 198).unwrap();
        assert_eq!(
            (4, 4, 0, 2),
            (
                b.flows.hp_gain,
                b.flows.hp_loss,
                b.flows.damaged,
                b.flows.mp_loss
            )
        );
        assert_eq!((500, 198), (b.vitals.hp, b.vitals.mp));
        assert!(a.regenerate(2, 4, 4, 0, 2).is_none());
        assert!(a.regenerate_batch(2, 4, 4, 0, 2, 499, 198).is_none());
        assert!(a.regenerate_batch(2, 4, 4, 0, 2, 500, 199).is_none());
    }
    #[test]
    fn batch_wide_arithmetic_never_wraps_counters_or_revives_a_dead_life() {
        let a = start();
        let n = i64::MAX;
        let b = a.regenerate_batch(2, n, n, n, n, 1000, 250).unwrap();
        assert_eq!((n, n), (b.flows.hp_gain, b.flows.hp_loss));
        assert!(b.regenerate_batch(3, 1, 1, 0, 0, 1000, 250).is_none());
        assert!(a.regenerate_batch(2, 0, 1000, 0, 0, 0, 250).is_none());
        assert!(a
            .damage(2, 1000)
            .unwrap()
            .regenerate_batch(3, 1, 0, 0, 0, 1, 250)
            .is_none());
        assert!(a.regenerate_batch(2, -1, 0, 0, 0, 999, 250).is_none());
        assert!(a.regenerate_batch(1, 0, 0, 0, 0, 1000, 250).is_none());
        let mut bad = a;
        bad.birth_hp -= 1;
        assert!(bad.regenerate_batch(2, 0, 0, 0, 0, 1000, 250).is_none());
    }
    #[test]
    fn self_healing_commits_both_resources_in_one_revision() {
        let before = start().damage(2, 100).unwrap().spend(3, 70).unwrap();
        let after = before.restore_and_spend(4, 8, 4).unwrap();
        assert_eq!((908, 176), (after.vitals.hp, after.vitals.mp));
        assert_eq!(
            (8, 74, 100),
            (after.flows.restored, after.flows.spent, after.flows.damaged)
        );
        assert_eq!(
            (4, before.life, before.equipment_revision),
            (after.revision, after.life, after.equipment_revision)
        );
        assert!(before.spend(4, 4).unwrap().restore(4, 8).is_none());
        assert!(before.restore(4, 8).unwrap().spend(4, 4).is_none());
        assert_eq!((900, 180), (before.vitals.hp, before.vitals.mp));
    }
    #[test]
    fn self_healing_rejects_invalid_and_overflowing_inputs_without_partial_change() {
        let before = start().damage(2, 100).unwrap();
        for (revision, restored, spent) in
            [(2, 8, 4), (3, -1, 4), (3, 8, -1), (3, 101, 4), (3, 8, 251)]
        {
            assert!(before
                .restore_and_spend(revision, restored, spent)
                .is_none());
        }
        assert!(before
            .damage(3, 900)
            .unwrap()
            .restore_and_spend(4, 1, 0)
            .is_none());
        let limit = State {
            revision: 2,
            equipment_revision: 0,
            life: 0,
            birth_hp: 1000,
            birth_mp: i64::MAX,
            vitals: Vitals {
                hp: 1000,
                maximum_hp: 1001,
                mp: 1,
                maximum_mp: i64::MAX,
            },
            flows: Flows {
                restored: i64::MAX,
                damaged: i64::MAX,
                mp_gain: 1,
                spent: i64::MAX,
                ..Flows::default()
            },
        };
        assert!(limit.valid());
        assert!(limit.restore_and_spend(3, 1, 0).is_none());
        assert!(limit.restore_and_spend(3, 0, 1).is_none());
        let mut invalid = before;
        invalid.birth_hp -= 1;
        assert!(invalid.restore_and_spend(3, 8, 4).is_none());
        assert_eq!(0, start().restore_and_spend(2, 0, 0).unwrap().flows.spent);
    }
    #[test]
    fn wide_self_healing_corpus_matches_java() {
        let maximum = i64::MAX;
        let (mut hp, mut mp, mut restored, mut spent) = (maximum - 10000, maximum, 0, 0);
        let mut state = State::initial(
            1,
            0,
            Vitals {
                hp,
                maximum_hp: maximum,
                mp,
                maximum_mp: maximum,
            },
        )
        .unwrap();
        for i in 1..=256i64 {
            let gain = (maximum - hp).min(i * 37 % 101);
            let cost = i * 13 % 5;
            hp += gain;
            mp -= cost;
            restored += gain;
            spent += cost;
            state = state.restore_and_spend(i + 1, gain, cost).unwrap();
            assert_eq!((hp, mp), (state.vitals.hp, state.vitals.mp));
            assert_eq!((restored, spent), (state.flows.restored, state.flows.spent));
            assert_eq!((0, 0), (state.flows.hp_gain, state.flows.damaged));
        }
        assert_eq!((10000, 513), (restored, spent));
        println!("self-healing-balance-v1:{hp}:{mp}:{restored}:{spent}:256");
    }
    fn start() -> State {
        State::initial(
            1,
            0,
            Vitals {
                hp: 1000,
                maximum_hp: 1000,
                mp: 250,
                maximum_mp: 250,
            },
        )
        .unwrap()
    }
    #[test]
    fn capacity_is_not_birth_and_clipping_is_not_damage_or_spending() {
        let before = start().damage(2, 100).unwrap().spend(3, 70).unwrap();
        let down = before.equip(4, 1, 100, 50).unwrap();
        assert_eq!((1000, 250), (down.birth_hp, down.birth_mp));
        assert_eq!((100, 70), (down.flows.damaged, down.flows.spent));
        assert_eq!((800, 130), (down.flows.hp_clipped, down.flows.mp_clipped));
        let up = down.equip(5, 2, 2000, 900).unwrap();
        assert_eq!((100, 50), (up.vitals.hp, up.vitals.mp));
        assert_eq!(down.flows, up.flows);
        assert!(follows_equipment(before, down));
        assert!(follows_equipment(down, up));
        assert!(!follows_equipment(before, up));
    }
    #[test]
    fn recovery_spending_respawn_and_changing_life_capacities_conserve_resources() {
        let a = start()
            .damage(2, 100)
            .unwrap()
            .spend(3, 70)
            .unwrap()
            .regenerate(4, 10, 0, 0, 5)
            .unwrap()
            .restore(5, 20)
            .unwrap();
        let b = a
            .equip(6, 1, 100, 50)
            .unwrap()
            .equip(7, 2, 1000, 250)
            .unwrap()
            .damage(8, 100)
            .unwrap();
        let dead = b.equip(9, 3, 200, 100).unwrap();
        assert_eq!(0, dead.vitals.hp);
        let born = dead.respawn(10, 120, 25).unwrap();
        assert_eq!((120, 75, 1), (born.vitals.hp, born.vitals.mp, born.life));
        assert_eq!((830, 125), (born.flows.hp_clipped, born.flows.mp_clipped));
        assert_eq!(
            (10, 20, 200, 70),
            (
                born.flows.hp_gain,
                born.flows.restored,
                born.flows.damaged,
                born.flows.spent
            )
        );
        let again = born
            .damage(11, 120)
            .unwrap()
            .equip(12, 4, 400, 100)
            .unwrap()
            .respawn(13, 400, 0)
            .unwrap();
        assert_eq!(
            (520, 25, 2),
            (again.flows.hp_granted, again.flows.mp_granted, again.life)
        );
        assert_eq!(75, again.spend(14, 0).unwrap().vitals.mp);
    }
    #[test]
    fn unapplied_overheal_negative_costs_dead_healing_and_stale_epochs_are_rejected() {
        let a = start();
        let dead = a.damage(2, 1000).unwrap();
        assert!(a.restore(2, 1).is_none());
        assert!(a.damage(2, 1001).is_none());
        assert!(a.spend(2, -1).is_none());
        assert!(a.spend(2, 251).is_none());
        assert!(a.damage(1, 1).is_none());
        assert!(a.equip(2, 2, 100, 50).is_none());
        assert!(a.equip(2, 1, 0, 50).is_none());
        assert!(a.respawn(2, 1, 0).is_none());
        assert!(dead.restore(3, 1).is_none());
        assert!(dead.spend(3, 0).is_none());
        assert!(dead.regenerate(3, 1, 0, 0, 0).is_none());
        assert!(dead.respawn(3, 1001, 0).is_none());
        assert!(a.regenerate(2, 0, 1000, 0, 0).is_none());
        assert!(a.damage(2, 2).unwrap().regenerate(3, 1, 1, 0, 0).is_none());
        assert_eq!(Flows::default(), a.flows);
    }
    #[test]
    fn balanced_forgeries_cannot_replace_birth_or_combine_equipment_events() {
        let before = start().damage(2, 800).unwrap();
        let after = before.equip(3, 1, 100, 50).unwrap();
        let forged = State::initial(3, 1, after.vitals).unwrap();
        assert!(!follows_equipment(before, forged));
        assert!(!follows_equipment(before, after.damage(4, 1).unwrap()));
        assert!(!follows_equipment(before, before));
        let bad = State {
            birth_hp: 999,
            ..after
        };
        assert!(!bad.valid());
        assert!(bad.damage(5, 1).is_none());
        assert!(bad.equip(5, 2, 1, 0).is_none());
        assert!(!State {
            revision: 3,
            life: 1,
            ..start()
        }
        .valid());
        assert!(!Flows {
            hp_gain: -1,
            ..Flows::default()
        }
        .valid());
        assert!(!State {
            vitals: Vitals {
                hp: i64::MIN,
                ..before.vitals
            },
            ..before
        }
        .valid());
    }
    #[test]
    fn wide_conservation_is_safe_but_individual_counters_cannot_wrap() {
        let n = i64::MAX;
        let full = State {
            revision: 2,
            equipment_revision: 0,
            life: 0,
            birth_hp: n,
            birth_mp: n,
            vitals: Vitals {
                hp: n,
                maximum_hp: n,
                mp: n,
                maximum_mp: n,
            },
            flows: Flows {
                hp_gain: n,
                hp_loss: n,
                restored: n,
                damaged: n,
                mp_gain: n,
                mp_loss: n,
                ..Flows::default()
            },
        };
        assert!(full.valid());
        assert!(full.damage(3, 1).is_none());
        assert_eq!(n, full.vitals.hp);
        let clipped = State {
            vitals: Vitals {
                hp: 2,
                mp: 2,
                ..full.vitals
            },
            flows: Flows {
                hp_gain: n,
                damaged: n - 2,
                mp_gain: n,
                spent: n - 2,
                hp_clipped: n,
                mp_clipped: n,
                ..Flows::default()
            },
            ..full
        };
        assert!(clipped.valid());
        assert!(clipped.equip(3, 1, 1, 1).is_none());
        let wide = State::initial(1, 0, full.vitals)
            .unwrap()
            .damage(2, 9007199254740993)
            .unwrap();
        assert_eq!(n - 9007199254740993, wide.vitals.hp);
        assert!(State::initial(1, n, start().vitals)
            .unwrap()
            .equip(2, 0, 1, 0)
            .is_none());
    }
    #[test]
    fn partial_birth_and_zero_mp_do_not_gain_clipped_resources_after_reequip() {
        let a = State::initial(
            1,
            8,
            Vitals {
                hp: 7,
                maximum_hp: 100,
                mp: 0,
                maximum_mp: 0,
            },
        )
        .unwrap();
        let b = a.equip(2, 9, 3, 0).unwrap().equip(3, 10, 100, 100).unwrap();
        assert_eq!(
            (7, 3, 0, 4),
            (b.birth_hp, b.vitals.hp, b.vitals.mp, b.flows.hp_clipped)
        );
        assert_eq!(0, b.spend(4, 0).unwrap().flows.spent);
    }
    #[test]
    fn deterministic_mixed_event_corpus_matches_java() {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        let mut a = start();
        let mut rng = 0x123456789abcdef0u64;
        for i in 1..=512i64 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let next = i + 1;
            let v = a.vitals;
            a = if i % 17 == 0 {
                a.damage(next, v.hp)
            } else {
                match i % 7 {
                    0 => a.equip(
                        next,
                        a.equipment_revision + 1,
                        1 + (rng % 2000) as i64,
                        ((rng >> 16) % 500) as i64,
                    ),
                    1 => a.damage(next, v.hp.min((rng % 91) as i64)),
                    2 if v.hp == 0 => a.respawn(next, v.maximum_hp, 0),
                    2 => a.restore(next, (v.maximum_hp - v.hp).min((rng % 61) as i64)),
                    3 if v.hp == 0 => a.damage(next, 0),
                    3 => a.spend(next, v.mp.min((rng % 71) as i64)),
                    4 if v.hp == 0 => a.damage(next, 0),
                    4 => a.regenerate(
                        next,
                        (v.maximum_hp - v.hp).min((rng % 31) as i64),
                        0,
                        (v.maximum_mp - v.mp).min((rng % 19) as i64),
                        0,
                    ),
                    5 if v.hp == 0 => a.damage(next, 0),
                    5 => a.regenerate(
                        next,
                        0,
                        (v.hp - 1).min((rng % 21) as i64),
                        0,
                        v.mp.min((rng % 13) as i64),
                    ),
                    6 if v.hp == 0 => a.respawn(next, v.maximum_hp, 0),
                    6 => a.spend(next, 0),
                    _ => unreachable!(),
                }
            }
            .unwrap();
            let v = a.vitals;
            for n in [
                a.revision,
                a.equipment_revision,
                a.life,
                a.birth_hp,
                a.birth_mp,
                v.hp,
                v.maximum_hp,
                v.mp,
                v.maximum_mp,
            ]
            .into_iter()
            .chain(a.flows.fields())
            {
                hash.update(n.to_be_bytes());
            }
        }
        let digest = format!("{:x}", hash.finalize());
        assert_eq!(
            "b7e60dc748a940071930911b77ce84155a6e658652062464a4f7fbdaea9c8f38",
            digest
        );
        println!("vital-balance-v1:{digest}:512");
    }
}
